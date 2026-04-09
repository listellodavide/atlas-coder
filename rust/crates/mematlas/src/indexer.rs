use crate::config::MemAtlasConfig;
use crate::graph::{CodeEntity, CodeGraph, DependencyType, EntityType};
use anyhow::Result;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Tree};

pub struct Indexer {
    config: MemAtlasConfig,
}

impl Indexer {
    pub fn new(config: MemAtlasConfig) -> Self {
        Self { config }
    }

    pub fn index(&self, roots: &[PathBuf], graph: &mut CodeGraph) -> Result<()> {
        let mut all_files = Vec::new();

        for root in roots {
            let walker = WalkBuilder::new(root)
                .hidden(false)
                .git_ignore(true)
                .build();

            for entry in walker {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && self.should_index(path) {
                    all_files.push(path.to_path_buf());
                }
            }
        }

        let pb = if all_files.len() > 100 {
            let pb = ProgressBar::new(all_files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                    .unwrap(),
            );
            Some(pb)
        } else {
            None
        };

        let parsed_entities: Vec<Vec<CodeEntity>> = all_files
            .into_par_iter()
            .filter_map(|p| {
                let res = self.process_file(&p).ok();
                if let Some(ref pbar) = pb {
                    pbar.inc(1);
                }
                res
            })
            .collect();

        if let Some(pbar) = pb {
            pbar.finish_with_message("Parsed files");
        }

        let mut all_entities = Vec::new();
        for entities in parsed_entities {
            all_entities.extend(entities);
        }

        // 1. Upsert all nodes
        let mut idxs = Vec::new();
        for entity in &all_entities {
            idxs.push((entity.unique_key(), graph.upsert_entity(entity.clone())));
        }

        // 2. Add edges for Dependencies
        for entity in &all_entities {
            let from_key = entity.unique_key();
            if let Some(from_idx) = graph.get_node_index(&from_key) {
                for dep in &entity.dependencies {
                    // Very simple containment / match heuristics for edges
                    // We attempt to find an entity that matches the dependency name
                    for (other_key, to_idx) in &idxs {
                        if from_key == *other_key {
                            continue;
                        }
                        if other_key.ends_with(&format!(":{}", dep)) {
                            graph.add_edge(from_idx, *to_idx, DependencyType::Calls);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn should_index(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if !self.config.extensions.contains(&ext.to_string()) {
                return false;
            }
        } else {
            return false;
        }

        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > self.config.max_file_size_bytes {
                return false;
            }
        }

        true
    }

    fn process_file(&self, path: &Path) -> Result<Vec<CodeEntity>> {
        let content = fs::read_to_string(path)?;
        let language = self.detect_language(path);
        
        let mut parser = Parser::new();
        
        // Setup grammar based on ext
        let lang = match language.as_str() {
            "rs" => tree_sitter_rust::LANGUAGE.into(),
            "js" => tree_sitter_javascript::LANGUAGE.into(),
            "ts" | "tsx" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "py" => tree_sitter_python::LANGUAGE.into(),
            "java" => tree_sitter_java::LANGUAGE.into(),
            _ => return Ok(vec![self.fallback_file_entity(path, &language, &content)]),
        };
        
        if parser.set_language(&lang).is_err() {
            return Ok(vec![self.fallback_file_entity(path, &language, &content)]);
        }

        let tree = parser.parse(&content, None).ok_or_else(|| anyhow::anyhow!("Failed to parse"))?;
        let root = tree.root_node();
        
        let mut entities = Vec::new();
        self.extract_entities(root, &content, path, &language, &mut entities);
        
        // Always include a file entity at minimum to maintain graph connectivity
        if entities.is_empty() {
            entities.push(self.fallback_file_entity(path, &language, &content));
        }
        
        Ok(entities)
    }

    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn extract_entities(&self, node: Node, content: &str, path: &Path, language: &str, entities: &mut Vec<CodeEntity>) {
        let kind = node.kind();
        
        let mut add_entity = |entity_type: EntityType, name: String| {
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            
            // Generate summary heuristic
            let node_text = node.utf8_text(content.as_bytes()).unwrap_or("");
            let summary = self.generate_summary(node_text);
            
            // Extract dependencies roughly inside the node
            let mut dependencies = Vec::new();
            self.extract_deps(node, content, &mut dependencies);
            
            entities.push(CodeEntity {
                name,
                entity_type,
                line_range: (start_line, end_line),
                file_path: path.to_path_buf(),
                language: language.to_string(),
                summary,
                dependencies,
            });
        };

        match kind {
            "function_item" | "method_declaration" | "function_declaration" => {
                let name = node.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(content.as_bytes()).ok())
                    .unwrap_or("unknown_function");
                add_entity(EntityType::Function, name.to_string());
            }
            "class_declaration" | "struct_item" => {
                let name = node.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(content.as_bytes()).ok())
                    .unwrap_or("unknown_class");
                add_entity(EntityType::Class, name.to_string());
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_entities(child, content, path, language, entities);
        }
    }

    fn extract_deps(&self, node: Node, content: &str, deps: &mut Vec<String>) {
        if node.kind() == "call_expression" || node.kind() == "method_invocation" {
            if let Some(function) = node.child_by_field_name("function") {
               if let Ok(name) = function.utf8_text(content.as_bytes()) {
                   deps.push(name.to_string());
               }
            } else if let Some(function) = node.child_by_field_name("name") {
                if let Ok(name) = function.utf8_text(content.as_bytes()) {
                   deps.push(name.to_string());
               }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_deps(child, content, deps);
        }
    }

    fn generate_summary(&self, text: &str) -> String {
        let first_line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        let s = first_line.chars().take(125).collect::<String>();
        format!("{}...", s)
    }

    fn fallback_file_entity(&self, path: &Path, lang: &str, content: &str) -> CodeEntity {
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        CodeEntity {
            name,
            entity_type: EntityType::File,
            line_range: (1, std::cmp::max(1, content.lines().count())),
            file_path: path.to_path_buf(),
            language: lang.to_string(),
            summary: format!("File {}", path.display()),
            dependencies: vec![],
        }
    }
}
