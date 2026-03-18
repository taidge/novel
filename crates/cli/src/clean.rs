use anyhow::Result;
use std::path::Path;
use tracing::info;

pub fn run_clean(project_root: &Path) -> Result<()> {
    let config = novel_shared::SiteConfig::load(project_root)?;
    let output_dir = config.output_dir(project_root);

    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir)?;
        info!("Removed output directory: {}", output_dir.display());
    } else {
        info!("Output directory does not exist: {}", output_dir.display());
    }

    // Also remove cache file if present
    let cache_file = output_dir.with_extension("").join(".novel-cache.json");
    if cache_file.exists() {
        std::fs::remove_file(&cache_file)?;
        info!("Removed cache file");
    }

    Ok(())
}
