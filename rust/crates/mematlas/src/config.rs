use std::path::{Path, PathBuf};

/// Configuration for a MemAtlas instance.
#[derive(Debug, Clone)]
pub struct MemAtlasConfig {
    /// Directory used to persist the index (one sub-dir per project).
    pub index_dir: PathBuf,
    /// Skip files larger than this byte limit (default: 256 KB).
    pub max_file_size_bytes: u64,
    /// File extensions to index.
    pub extensions: Vec<String>,
    /// Globs to exclude from indexing (in addition to .gitignore).
    pub ignore_patterns: Vec<String>,
}

impl MemAtlasConfig {
    /// Returns sane defaults whose index lives under `~/.local/share/mematlas`.
    #[must_use]
    pub fn default_for(project_root: &Path) -> Self {
        let base = dirs_base(project_root);
        Self {
            index_dir: base,
            max_file_size_bytes: 256 * 1024,
            extensions: default_extensions(),
            ignore_patterns: Vec::new(),
        }
    }

    /// Returns the sled DB path for this configuration.
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.index_dir.join("graph.db")
    }
}

fn dirs_base(project_root: &Path) -> PathBuf {
    use sha2::Digest;
    let canonical = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let hash = format!("{:x}", sha2::Sha256::digest(canonical.to_string_lossy().as_bytes()));
    let short = &hash[..12];

    if let Ok(data) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(data).join("mematlas").join(short);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local/share/mematlas")
            .join(short);
    }
    std::env::temp_dir().join("mematlas").join(short)
}

fn default_extensions() -> Vec<String> {
    [
        "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "kt", "kts", "cs", "cpp", "c", "h",
        "hpp", "rb", "swift", "scala", "clj", "ex", "exs", "hs",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}
