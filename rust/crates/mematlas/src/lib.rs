pub mod config;
pub mod graph;
pub mod indexer;
pub mod mematlas;
pub mod query;

pub use config::MemAtlasConfig;
pub use mematlas::{LearnResult, MemAtlas, MemAtlasStatus};
pub use query::SearchResult;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemAtlasError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("Indexing error: {0}")]
    Anyhow(#[from] anyhow::Error),
}
