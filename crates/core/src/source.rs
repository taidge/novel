use anyhow::Result;
use rust_embed::Embed;
use std::marker::PhantomData;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Abstraction over documentation file sources.
///
/// Implementations provide access to documentation files regardless of
/// whether they live on the filesystem or are embedded via `rust-embed`.
pub trait DocsSource: Send + Sync {
    /// List all file paths relative to docs root (forward-slash separated).
    fn list_files(&self) -> Vec<String>;

    /// Read a file as a UTF-8 string.
    fn read_to_string(&self, relative_path: &str) -> Result<String>;

    /// Read a file as raw bytes.
    fn read_bytes(&self, relative_path: &str) -> Result<Vec<u8>>;

    /// Check whether a file exists.
    fn exists(&self, relative_path: &str) -> bool;
}

// ---------------------------------------------------------------------------
// DirSource -- filesystem-backed
// ---------------------------------------------------------------------------

pub(crate) struct DirSource {
    docs_root: PathBuf,
}

impl DirSource {
    pub fn new(docs_root: PathBuf) -> Self {
        Self { docs_root }
    }
}

impl DocsSource for DirSource {
    fn list_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        for entry in WalkDir::new(&self.docs_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Ok(relative) = path.strip_prefix(&self.docs_root) {
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
        files
    }

    fn read_to_string(&self, relative_path: &str) -> Result<String> {
        let path = self.docs_root.join(relative_path);
        Ok(std::fs::read_to_string(path)?)
    }

    fn read_bytes(&self, relative_path: &str) -> Result<Vec<u8>> {
        let path = self.docs_root.join(relative_path);
        Ok(std::fs::read(path)?)
    }

    fn exists(&self, relative_path: &str) -> bool {
        self.docs_root.join(relative_path).exists()
    }
}

// ---------------------------------------------------------------------------
// EmbedSource -- rust-embed-backed
// ---------------------------------------------------------------------------

pub(crate) struct EmbedSource<E: Embed> {
    _marker: PhantomData<E>,
}

impl<E: Embed> EmbedSource<E> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E: Embed + Send + Sync> DocsSource for EmbedSource<E> {
    fn list_files(&self) -> Vec<String> {
        E::iter().map(|f| f.to_string()).collect()
    }

    fn read_to_string(&self, relative_path: &str) -> Result<String> {
        let file = E::get(relative_path)
            .ok_or_else(|| anyhow::anyhow!("File not found in embed: {}", relative_path))?;
        Ok(String::from_utf8(file.data.to_vec())?)
    }

    fn read_bytes(&self, relative_path: &str) -> Result<Vec<u8>> {
        let file = E::get(relative_path)
            .ok_or_else(|| anyhow::anyhow!("File not found in embed: {}", relative_path))?;
        Ok(file.data.to_vec())
    }

    fn exists(&self, relative_path: &str) -> bool {
        E::get(relative_path).is_some()
    }
}
