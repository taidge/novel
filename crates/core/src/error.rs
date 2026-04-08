//! Structured error type for Novel operations.
//!
//! # Status
//!
//! Migration from blanket `anyhow::Result<T>` to a structured error type
//! is **in progress**. The first surface to use `NovelError` is the
//! template engine layer (see `template/mod.rs`); other modules will be
//! migrated incrementally.
//!
//! At the public boundary ([`crate::BuiltSite::render_page`] and friends),
//! `NovelError` is converted back to `anyhow::Error` automatically via
//! anyhow's blanket `impl<E: std::error::Error + Send + Sync + 'static>
//! From<E>` — so callers using `?` see no behavioural change.
//!
//! Do **not** add new variants speculatively — the existing six categories
//! cover what's actually thrown today. If a new category is needed, prefer
//! reusing [`NovelError::Build`] (a generic `String` bucket) over
//! introducing more shapes.

#[derive(Debug, thiserror::Error)]
pub enum NovelError {
    #[error("Config: {0}")]
    Config(String),

    /// Template engine error (engine-agnostic — holds the rendered message
    /// from minijinja / tera / handlebars rather than a typed source).
    #[error("Template: {0}")]
    Template(String),

    #[allow(dead_code)]
    #[error("Markdown error in {file}: {message}")]
    Markdown { file: String, message: String },

    /// User data file failed to parse (e.g. `docs/data/authors.toml`).
    /// Used by [`crate::data::load_data`].
    #[error("Data file {file}: {message}")]
    Data { file: String, message: String },

    #[error("I/O: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[allow(dead_code)]
    #[error("Plugin '{plugin}': {message}")]
    Plugin { plugin: String, message: String },

    #[allow(dead_code)]
    #[error("Build: {0}")]
    Build(String),
}

pub type NovelResult<T> = Result<T, NovelError>;
