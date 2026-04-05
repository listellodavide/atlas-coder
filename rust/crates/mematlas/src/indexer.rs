use crate::config::MemAtlasConfig;
use crate::graph::{CodeGraph, EdgeKind, FileNode};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Indexer {
    config: MemAtlasConfig,
}

struct IndexedFile {
    path: PathBuf,
    language: String,
    symbols: Vec<String>,
    imports: Vec<String>,
    hash: String,
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

        let indexed_files: Vec<IndexedFile> = all_files
            .into_par_iter()
            .filter_map(|p| self.process_file(&p).ok())
            .collect();

        // 1. Upsert all nodes
        let mut path_to_idx = std::collections::HashMap::new();
        for file in &indexed_files {
            let node = FileNode {
                path: file.path.clone(),
                language: file.language.clone(),
                summary: format!("{} symbols: {}", file.language, file.symbols.join(", ")),
                symbol_count: file.symbols.len(),
                hash: file.hash.clone(),
            };
            let idx = graph.upsert_file(node);
            path_to_idx.insert(file.path.clone(), idx);
        }

        // 2. Add edges based on imports (best-effort matching)
        for file in &indexed_files {
            if let Some(&from_idx) = path_to_idx.get(&file.path) {
                for import in &file.imports {
                    // Try to find a file whose path contains or matches the import string
                    for (other_path, &to_idx) in &path_to_idx {
                        if file.path == *other_path {
                            continue;
                        }
                        if other_path.to_string_lossy().contains(import) {
                            graph.add_edge(from_idx, to_idx, EdgeKind::Import);
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

    fn process_file(&self, path: &Path) -> Result<IndexedFile> {
        let content = fs::read_to_string(path)?;
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        let language = self.detect_language(path);
        let (symbols, imports) = self.extract_symbols_and_imports(&content, &language);

        Ok(IndexedFile {
            path: path.to_path_buf(),
            language,
            symbols,
            imports,
            hash,
        })
    }

    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn extract_symbols_and_imports(&self, content: &str, lang: &str) -> (Vec<String>, Vec<String>) {
        let mut symbols = Vec::new();
        let mut imports = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            match lang {
                "rs" => {
                    if (line.starts_with("pub fn ") || line.starts_with("fn ")) && line.contains('(') {
                        if let Some(name) = line.split_whitespace().nth(2).or_else(|| line.split_whitespace().nth(1)) {
                           symbols.push(name.split('(').next().unwrap_or(name).to_string());
                        }
                    } else if line.starts_with("pub struct ") || line.starts_with("struct ") {
                         if let Some(name) = line.split_whitespace().nth(2).or_else(|| line.split_whitespace().nth(1)) {
                            symbols.push(name.to_string());
                        }
                    } else if line.starts_with("use ") {
                        imports.push(line.replace("use ", "").replace(";", ""));
                    }
                }
                "ts" | "js" | "tsx" | "jsx" => {
                    if line.contains("function ") || line.contains("class ") {
                         symbols.push(line.to_string());
                    } else if line.starts_with("import ") {
                        imports.push(line.to_string());
                    }
                }
                "py" => {
                    if line.starts_with("def ") || line.starts_with("class ") {
                        symbols.push(line.to_string());
                    } else if line.starts_with("import ") || line.starts_with("from ") {
                        imports.push(line.to_string());
                    }
                }
                _ => {}
            }
        }

        (symbols, imports)
    }
}
