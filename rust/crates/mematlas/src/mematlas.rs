use crate::config::MemAtlasConfig;
use crate::graph::CodeGraph;
use crate::indexer::Indexer;
use crate::query::{QueryEngine, SearchResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

pub struct MemAtlas {
    config: MemAtlasConfig,
    graph: CodeGraph,
    db: sled::Db,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnResult {
    pub files_indexed: usize,
    pub edges_created: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemAtlasStatus {
    pub files_indexed: usize,
    pub edges_created: usize,
    pub last_indexed: Option<DateTime<Utc>>,
}

impl MemAtlas {
    pub fn new(config: MemAtlasConfig) -> Result<Self> {
        let db = sled::open(config.db_path())?;
        let graph = if let Some(bytes) = db.get("graph")? {
            CodeGraph::deserialize(&bytes).unwrap_or_else(|_| CodeGraph::new())
        } else {
            CodeGraph::new()
        };

        Ok(Self { config, graph, db })
    }

    pub fn learn(&mut self, roots: &[PathBuf]) -> Result<LearnResult> {
        let indexer = Indexer::new(self.config.clone());
        indexer.index(roots, &mut self.graph)?;

        let bytes = self.graph.serialize()?;
        self.db.insert("graph", bytes)?;
        self.db.insert("last_indexed", Utc::now().to_rfc3339().as_bytes())?;
        self.db.flush()?;

        Ok(LearnResult {
            files_indexed: self.graph.node_count(),
            edges_created: self.graph.edge_count(),
        })
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let engine = QueryEngine::new(&self.graph);
        Ok(engine.search(query, 10))
    }

    pub fn context_slice(&self, task: &str) -> Result<String> {
        let engine = QueryEngine::new(&self.graph);
        Ok(engine.context_slice_for(task, 2))
    }

    pub fn status(&self) -> MemAtlasStatus {
        let last_indexed = self.db.get("last_indexed").ok().flatten()
            .and_then(|b| String::from_utf8(b.to_vec()).ok())
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        MemAtlasStatus {
            files_indexed: self.graph.node_count(),
            edges_created: self.graph.edge_count(),
            last_indexed,
        }
    }

    pub fn export(&self) -> Result<String> {
        let engine = QueryEngine::new(&self.graph);
        Ok(engine.export())
    }
}
