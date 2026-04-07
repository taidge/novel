//! Sass/SCSS compilation via the `grass` crate (pure Rust).

use anyhow::Result;
use novel_shared::config::SassConfig;
use std::path::Path;

/// Compile every entry in the config and write the resulting CSS to
/// `output_dir/<output>`. The input path is resolved against `project_root`.
///
/// When the `sass` feature is disabled this is a no-op (with a warning if
/// entries are configured).
#[cfg(feature = "sass")]
pub fn compile(cfg: &SassConfig, project_root: &Path, output_dir: &Path) -> Result<()> {
    if cfg.entries.is_empty() {
        return Ok(());
    }
    let load_paths: Vec<std::path::PathBuf> = cfg
        .load_paths
        .iter()
        .map(|p| project_root.join(p))
        .collect();
    let opts = grass::Options::default().load_paths(&load_paths);

    for entry in &cfg.entries {
        let (input_rel, output_rel) = match entry.as_slice() {
            [i, o] => (i.as_str(), o.as_str()),
            _ => {
                tracing::warn!("sass entry must be [input, output], got: {:?}", entry);
                continue;
            }
        };
        let input_path = project_root.join(input_rel);
        let css = grass::from_path(&input_path, &opts)?;
        let output_path = output_dir.join(output_rel);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, css)?;
        tracing::info!("Compiled SCSS: {} -> {}", input_rel, output_rel);
    }
    Ok(())
}

#[cfg(not(feature = "sass"))]
pub fn compile(cfg: &SassConfig, _project_root: &Path, _output_dir: &Path) -> Result<()> {
    if !cfg.entries.is_empty() {
        tracing::warn!(
            "sass entries configured but novel-core was built without `sass` feature"
        );
    }
    Ok(())
}
