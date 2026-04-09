use crate::graph::CodeGraph;
use std::path::Path;

pub struct SearchResult {
    pub path: String,
    pub score: f64,
    pub summary: String,
}

pub struct QueryEngine<'a> {
    graph: &'a CodeGraph,
}

impl<'a> QueryEngine<'a> {
    pub fn new(graph: &'a CodeGraph) -> Self {
        Self { graph }
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        let query = query.to_lowercase();
        let mut hits: Vec<SearchResult> = self
            .graph
            .nodes()
            .filter_map(|node| {
                let path_str = node.file_path.to_string_lossy();
                let mut score = 0.0;
                if path_str.to_lowercase().contains(&query) {
                    score += 10.0;
                }
                if node.name.to_lowercase().contains(&query) {
                    score += 8.0;
                }
                if node.summary.to_lowercase().contains(&query) {
                    score += 5.0;
                }

                if score > 0.0 {
                    Some(SearchResult {
                        path: path_str.to_string(),
                        score,
                        summary: format!("{} | {}", node.name, node.summary),
                    })
                } else {
                    None
                }
            })
            .collect();

        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        hits.into_iter().take(top_k).collect()
    }

    pub fn context_slice_for(&self, path_str: &str, hops: u8) -> String {
        let path = Path::new(path_str);
        let entities = self.graph.get_entities_in_file(path);

        let mut output = String::new();
        output.push_str(&format!("### Context slice for {}\n\n", path_str));

        let mut visited = std::collections::HashSet::new();

        for entity_idx in entities {
            let neighborhood = self.graph.neighbourhood(entity_idx, hops);
            for idx in neighborhood {
                if !visited.contains(&idx) {
                    visited.insert(idx);
                    if let Some(node) = self.graph.node(idx) {
                        output.push_str(&format!(
                            "- **{}:** {} ({})\n",
                            node.file_path.display(),
                            node.name,
                            node.language
                        ));
                        output.push_str(&format!("  Summary: {}\n", node.summary));
                    }
                }
            }
        }

        output
    }

    pub fn export(&self) -> String {
        let mut output = String::new();
        output.push_str("# MemAtlas Knowledge Graph Export\n\n");
        output.push_str(&format!("- **Total Entities:** {}\n", self.graph.node_count()));
        output.push_str(&format!(
            "- **Total Edges:** {}\n\n",
            self.graph.edge_count()
        ));

        output.push_str("## Entities\n\n");
        for node in self.graph.nodes() {
            output.push_str(&format!("### {} in {}\n", node.name, node.file_path.display()));
            output.push_str(&format!("- Language: {}\n", node.language));
            output.push_str(&format!("- Summary: {}\n\n", node.summary));
        }

        output
    }
}
