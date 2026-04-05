use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeKind {
    Import,
    Calls,
    Depends,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: PathBuf,
    pub language: String,
    pub summary: String,
    pub symbol_count: usize,
    pub hash: String,
}

pub struct CodeGraph {
    graph: DiGraph<FileNode, EdgeKind>,
    path_to_index: HashMap<PathBuf, NodeIndex>,
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_index: HashMap::new(),
        }
    }

    pub fn upsert_file(&mut self, node: FileNode) -> NodeIndex {
        if let Some(&idx) = self.path_to_index.get(&node.path) {
            self.graph[idx] = node;
            idx
        } else {
            let path = node.path.clone();
            let idx = self.graph.add_node(node);
            self.path_to_index.insert(path, idx);
            idx
        }
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, kind: EdgeKind) {
        self.graph.add_edge(from, to, kind);
    }

    pub fn get_node_index(&self, path: &Path) -> Option<NodeIndex> {
        self.path_to_index.get(path).copied()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &FileNode> {
        self.graph.node_weights()
    }

    pub fn neighbourhood(&self, path: &Path, hops: u8) -> Vec<NodeIndex> {
        let mut result = Vec::new();
        if let Some(&start_idx) = self.path_to_index.get(path) {
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((start_idx, 0));
            visited.insert(start_idx);

            while let Some((idx, h)) = queue.pop_front() {
                result.push(idx);
                if h < hops {
                    for edge in self.graph.edges(idx) {
                        let neighbor = edge.target();
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back((neighbor, h + 1));
                        }
                    }
                    // Also consider incoming edges for "depends on" / "is used by"
                    for edge in self
                        .graph
                        .edges_directed(idx, petgraph::Direction::Incoming)
                    {
                        let neighbor = edge.source();
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back((neighbor, h + 1));
                        }
                    }
                }
            }
        }
        result
    }

    pub fn top_k_by_centrality(&self, k: usize) -> Vec<NodeIndex> {
        // Simple centrality: degree (in + out)
        let mut degrees: Vec<(NodeIndex, usize)> = self
            .graph
            .node_indices()
            .map(|idx| {
                let degree = self.graph.edges(idx).count()
                    + self
                        .graph
                        .edges_directed(idx, petgraph::Direction::Incoming)
                        .count();
                (idx, degree)
            })
            .collect();

        degrees.sort_by(|a, b| b.1.cmp(&a.1));
        degrees.into_iter().take(k).map(|(idx, _)| idx).collect()
    }

    pub fn node(&self, idx: NodeIndex) -> Option<&FileNode> {
        self.graph.node_weight(idx)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        let serializable = (&self.graph, &self.path_to_index);
        bincode::serialize(&serializable)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let (graph, path_to_index): (DiGraph<FileNode, EdgeKind>, HashMap<PathBuf, NodeIndex>) =
            bincode::deserialize(bytes)?;
        Ok(Self {
            graph,
            path_to_index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_upsert_and_neighbourhood() {
        let mut graph = CodeGraph::new();
        let path1 = PathBuf::from("src/main.rs");
        let path2 = PathBuf::from("src/lib.rs");

        let node1 = FileNode {
            path: path1.clone(),
            language: "rust".into(),
            summary: "main".into(),
            symbol_count: 1,
            hash: "h1".into(),
        };
        let idx1 = graph.upsert_file(node1);

        let node2 = FileNode {
            path: path2.clone(),
            language: "rust".into(),
            summary: "lib".into(),
            symbol_count: 1,
            hash: "h2".into(),
        };
        let idx2 = graph.upsert_file(node2);

        graph.add_edge(idx1, idx2, EdgeKind::Import);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        let nb = graph.neighbourhood(&path1, 1);
        assert_eq!(nb.len(), 2);
        assert!(nb.contains(&idx1));
        assert!(nb.contains(&idx2));
    }
}
