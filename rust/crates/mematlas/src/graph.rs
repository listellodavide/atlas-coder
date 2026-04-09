use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Class,
    Interface,
    AbstractClass,
    Method,
    Function,
    File, // Useful as a fallback or container
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    Implements,
    Extends,
    Calls,
    InjectedVia,
    Imports,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub name: String,
    pub entity_type: EntityType,
    pub line_range: (usize, usize),
    pub file_path: PathBuf,
    pub language: String,
    pub summary: String,
    pub dependencies: Vec<String>,
}

impl CodeEntity {
    pub fn unique_key(&self) -> String {
        format!("{}:{}:{}", self.file_path.display(), self.line_range.0, self.name)
    }
}

pub struct CodeGraph {
    pub graph: DiGraph<CodeEntity, DependencyType>,
    pub key_to_index: HashMap<String, NodeIndex>,
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
            key_to_index: HashMap::new(),
        }
    }

    pub fn upsert_entity(&mut self, entity: CodeEntity) -> NodeIndex {
        let key = entity.unique_key();
        if let Some(&idx) = self.key_to_index.get(&key) {
            self.graph[idx] = entity;
            idx
        } else {
            let idx = self.graph.add_node(entity);
            self.key_to_index.insert(key, idx);
            idx
        }
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, kind: DependencyType) {
        // avoid duplicating edges
        if self.graph.find_edge(from, to).is_none() {
            self.graph.add_edge(from, to, kind);
        }
    }

    pub fn get_node_index(&self, key: &str) -> Option<NodeIndex> {
        self.key_to_index.get(key).copied()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &CodeEntity> {
        self.graph.node_weights()
    }

    pub fn get_entities_in_file(&self, path: &Path) -> Vec<NodeIndex> {
        self.key_to_index
            .iter()
            .filter_map(|(k, idx)| {
                if k.starts_with(&format!("{}:", path.display())) {
                    Some(*idx)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn neighbourhood(&self, start_idx: NodeIndex, hops: u8) -> Vec<NodeIndex> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_idx, 0));
        visited.insert(start_idx);

        while let Some((idx, h)) = queue.pop_front() {
            result.push(idx);
            if h < hops {
                // Outgoing
                for edge in self.graph.edges(idx) {
                    let neighbor = edge.target();
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back((neighbor, h + 1));
                    }
                }
                // Incoming
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
        result
    }

    pub fn impact_score(&self, idx: NodeIndex) -> usize {
        // I = sum(C_direct + C_transitive)
        // For simplicity, node degree in the subgraph up to 2 hops out
        self.neighbourhood(idx, 2).len().saturating_sub(1)
    }

    pub fn top_k_by_centrality(&self, k: usize) -> Vec<NodeIndex> {
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

    pub fn node(&self, idx: NodeIndex) -> Option<&CodeEntity> {
        self.graph.node_weight(idx)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        let serializable = (&self.graph, &self.key_to_index);
        bincode::serialize(&serializable)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let (graph, key_to_index): (DiGraph<CodeEntity, DependencyType>, HashMap<String, NodeIndex>) =
            bincode::deserialize(bytes)?;
        Ok(Self {
            graph,
            key_to_index,
        })
    }
}
