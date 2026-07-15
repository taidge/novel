use anyhow::{Context, Result};
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::BuiltSite;

static NEXT_TEMP_SITE_ID: AtomicU64 = AtomicU64::new(0);

impl BuiltSite {
    /// Convert this built site into a Salvo router that serves the documentation.
    ///
    /// The site is first written to a temporary directory, then served as
    /// static files. This is suitable for embedding docs in a Salvo web app:
    ///
    /// ```ignore
    /// let docs = EmbedNovel::<Docs>::new().title("API Docs").build()?;
    /// let router = Router::new().push(
    ///     Router::with_path("docs").push(docs.into_salvo_router()?)
    /// );
    /// ```
    pub fn into_salvo_router(self) -> Result<Router> {
        let tmp = next_temp_site_dir();
        self.write_to(&tmp)
            .with_context(|| format!("failed to prepare Salvo site in {}", tmp.display()))?;
        salvo_static_router(&tmp)
    }
}

/// Create a Salvo router that serves static files from a directory.
pub fn salvo_static_router(dir: impl AsRef<Path>) -> Result<Router> {
    let dir = dir.as_ref();
    let dir_str = dir.to_str().map(str::to_owned).ok_or_else(|| {
        anyhow::anyhow!(
            "Salvo static directory is not valid UTF-8 and cannot be served safely: {}",
            dir.display()
        )
    })?;
    Ok(Router::with_path("<**path>").get(
        StaticDir::new([dir_str])
            .defaults("index.html")
            .auto_list(false),
    ))
}

fn next_temp_site_dir() -> std::path::PathBuf {
    let id = NEXT_TEMP_SITE_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("novel-serve-{}-{id}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::next_temp_site_dir;

    #[test]
    fn embedded_salvo_sites_use_unique_process_local_directories() {
        assert_ne!(next_temp_site_dir(), next_temp_site_dir());
    }
}
