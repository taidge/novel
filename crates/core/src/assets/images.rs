//! Image processing: resize source images under the docs root into multiple
//! widths and write them to the output directory under `_resized/`.
//!
//! Gated behind the `images` cargo feature.

use anyhow::Result;
use novel_shared::config::ImagesConfig;
use std::path::Path;

#[cfg(feature = "images")]
const SUPPORTED_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp"];

#[cfg(feature = "images")]
pub fn process(cfg: &ImagesConfig, docs_root: &Path, output_dir: &Path) -> Result<()> {
    if cfg.sizes.is_empty() {
        return Ok(());
    }
    let resized_root = output_dir.join("_resized");
    for entry in walkdir::WalkDir::new(docs_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        if !ext.as_deref().map(|e| SUPPORTED_EXTS.contains(&e)).unwrap_or(false) {
            continue;
        }

        let img = match image::open(path) {
            Ok(i) => i,
            Err(e) => {
                tracing::warn!("image open failed for {}: {}", path.display(), e);
                continue;
            }
        };
        let (orig_w, _) = (img.width(), img.height());
        let rel = path.strip_prefix(docs_root).unwrap_or(path);
        let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("img");
        let parent = rel.parent().unwrap_or(Path::new(""));
        let ext_str = ext.as_deref().unwrap_or("jpg");

        for &w in &cfg.sizes {
            if w >= orig_w {
                continue;
            }
            let h = (img.height() as f32 * (w as f32 / orig_w as f32)) as u32;
            let resized = img.resize(w, h, image::imageops::FilterType::Lanczos3);
            let out_dir = resized_root.join(parent);
            std::fs::create_dir_all(&out_dir)?;
            let out_name = format!("{}-{}.{}", stem, w, ext_str);
            let out_path = out_dir.join(out_name);
            if let Err(e) = resized.save(&out_path) {
                tracing::warn!("image save failed for {}: {}", out_path.display(), e);
            }
        }
    }
    tracing::info!("Image processing complete");
    Ok(())
}

#[cfg(not(feature = "images"))]
pub fn process(cfg: &ImagesConfig, _docs_root: &Path, _output_dir: &Path) -> Result<()> {
    if !cfg.sizes.is_empty() {
        tracing::warn!(
            "image sizes configured but novel-core was built without `images` feature"
        );
    }
    Ok(())
}
