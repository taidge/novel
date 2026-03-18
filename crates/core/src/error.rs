/// Structured error type for Novel operations.
///
/// CLI code can continue using `anyhow` — `NovelError` auto-converts via
/// `std::error::Error`.
#[derive(Debug, thiserror::Error)]
pub enum NovelError {
    #[error("Config: {0}")]
    Config(String),

    #[error("Template: {source}")]
    Template {
        #[from]
        source: minijinja::Error,
    },

    #[error("Markdown error in {file}: {message}")]
    Markdown { file: String, message: String },

    #[error("I/O: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Plugin '{plugin}': {message}")]
    Plugin { plugin: String, message: String },

    #[error("Build: {0}")]
    Build(String),
}

pub type NovelResult<T> = Result<T, NovelError>;
