use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use std::path::Path;

use crate::BuiltSite;

impl BuiltSite {
    /// Convert this built site into a Salvo router that serves the documentation.
    ///
    /// The site is first written to a temporary directory, then served as
    /// static files. This is suitable for embedding docs in a Salvo web app:
    ///
    /// ```ignore
    /// let docs = EmbedNovel::<Docs>::new().title("API Docs").build()?;
    /// let router = Router::new()
    ///     .push(Router::with_path("docs/<**path>").get(docs.into_salvo_router()));
    /// ```
    pub fn into_salvo_router(self) -> Router {
        // Write to a temp directory
        let tmp = std::env::temp_dir().join(format!("novel-serve-{}", std::process::id()));
        let _ = self.write_to(&tmp);

        salvo_static_router(&tmp)
    }
}

/// Create a Salvo router that serves static files from a directory.
pub fn salvo_static_router(dir: impl AsRef<Path>) -> Router {
    let dir_str = dir.as_ref().to_string_lossy().to_string();
    Router::with_path("<**path>").get(
        StaticDir::new([dir_str])
            .defaults("index.html")
            .auto_list(false),
    )
}
