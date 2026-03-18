use novel_shared::PageData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildCache {
    pub version: u32,
    pub config_hash: u64,
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub content_hash: u64,
    pub page_data: PageData,
}

impl BuildCache {
    pub fn new(config_hash: u64) -> Self {
        Self {
            version: CACHE_VERSION,
            config_hash,
            entries: HashMap::new(),
        }
    }

    /// Load cache from disk, returning None if missing/invalid/version-mismatch.
    pub fn load(cache_path: &Path) -> Option<Self> {
        let data = std::fs::read_to_string(cache_path).ok()?;
        let cache: Self = serde_json::from_str(&data).ok()?;
        if cache.version != CACHE_VERSION {
            return None;
        }
        Some(cache)
    }

    /// Save cache to disk.
    pub fn save(&self, cache_path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(self)?;
        std::fs::write(cache_path, json)?;
        Ok(())
    }

    /// Check if a file is cached and unchanged.
    pub fn get(&self, relative_path: &str, content_hash: u64) -> Option<&PageData> {
        let entry = self.entries.get(relative_path)?;
        if entry.content_hash == content_hash {
            Some(&entry.page_data)
        } else {
            None
        }
    }

    /// Insert or update a cache entry.
    pub fn insert(&mut self, relative_path: String, content_hash: u64, page_data: PageData) {
        self.entries.insert(
            relative_path,
            CacheEntry {
                content_hash,
                page_data,
            },
        );
    }

    /// Remove entries for files that no longer exist.
    pub fn retain_existing(&mut self, existing_paths: &[&str]) {
        let set: std::collections::HashSet<&str> = existing_paths.iter().copied().collect();
        self.entries.retain(|k, _| set.contains(k.as_str()));
    }
}

/// Simple FNV-1a hash for content hashing.
pub fn hash_content(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
