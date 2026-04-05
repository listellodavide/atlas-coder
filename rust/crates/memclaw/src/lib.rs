pub mod config;
pub mod graph;
pub mod indexer;
pub mod memclaw;
pub mod query;

pub use config::MemClawConfig;
pub use memclaw::{LearnResult, MemClaw, MemClawStatus};
pub use query::SearchResult;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemClawError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("Indexing error: {0}")]
    Anyhow(#[from] anyhow::Error),
}
