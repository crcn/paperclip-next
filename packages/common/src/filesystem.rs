use std::path::{Path, PathBuf};

/// File system abstraction for path resolution and testing
pub trait FileSystem {
    /// Check if a file exists
    fn exists(&self, path: &Path) -> bool;

    /// Canonicalize a path (resolve symlinks, make absolute)
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

/// Real file system implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        std::fs::canonicalize(path)
    }
}

/// Mock file system for testing
pub struct MockFileSystem {
    pub existing_files: std::collections::HashSet<PathBuf>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            existing_files: std::collections::HashSet::new(),
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        self.existing_files.insert(path);
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MockFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.existing_files.contains(path)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        // For mock, just return the path as-is
        Ok(path.to_path_buf())
    }
}
