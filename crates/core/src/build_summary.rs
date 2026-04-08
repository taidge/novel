//! Per-build summary: counts, sizes, and a diff vs the previous build.
//!
//! After [`crate::BuiltSite::write_to`] finishes, it walks the output
//! directory to count files and sum byte sizes, then writes a small
//! `.novel-build.json` to disk so the *next* build can show what changed.
//!
//! The summary is informational only — failing to read or write the
//! metadata file is logged at debug level and never aborts the build.
//!
//! (F3)

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

const META_FILE: &str = ".novel-build.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct BuildSummary {
    /// Total number of files in `dist/` (excluding the meta file itself).
    pub total_files: usize,
    /// Total bytes across all files in `dist/`.
    pub total_bytes: u64,
    /// Per-top-level-directory file counts (e.g. `en` → 23, `zh` → 23,
    /// `assets` → 3, `posts` → 5). Useful for spotting which section grew.
    pub by_section: BTreeMap<String, SectionStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct SectionStats {
    pub files: usize,
    pub bytes: u64,
}

impl BuildSummary {
    /// Walk `output_dir` and tally everything below it. Skips the meta
    /// file itself so the previous run's stats aren't counted.
    pub fn collect(output_dir: &Path) -> Self {
        let mut summary = Self::default();
        let meta_path = output_dir.join(META_FILE);
        walk(output_dir, output_dir, &meta_path, &mut summary);
        summary
    }

    /// Read a previously-saved summary from `dist/.novel-build.json`.
    /// Returns `None` if the file doesn't exist or fails to parse.
    pub fn read_previous(output_dir: &Path) -> Option<Self> {
        let path = output_dir.join(META_FILE);
        let raw = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    /// Persist this summary so the next build can compute a diff.
    pub fn write(&self, output_dir: &Path) -> std::io::Result<()> {
        let path = output_dir.join(META_FILE);
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Render a multi-line summary block with optional diff against
    /// `previous`. Designed for `tracing::info!` consumption — caller
    /// chooses how to emit it.
    pub fn render(&self, previous: Option<&Self>) -> String {
        let mut out = String::new();
        out.push_str("Build summary:\n");
        out.push_str(&format!(
            "  Files: {} ({})\n",
            self.total_files,
            humanize_bytes(self.total_bytes)
        ));
        if let Some(prev) = previous {
            let file_delta = self.total_files as i64 - prev.total_files as i64;
            let byte_delta = self.total_bytes as i64 - prev.total_bytes as i64;
            out.push_str(&format!(
                "  Δ vs last build: {:+} files, {}\n",
                file_delta,
                humanize_signed_bytes(byte_delta),
            ));
        }
        if !self.by_section.is_empty() {
            out.push_str("  By section:\n");
            for (name, stats) in &self.by_section {
                let delta = previous
                    .and_then(|p| p.by_section.get(name))
                    .map(|prev| {
                        let f = stats.files as i64 - prev.files as i64;
                        let b = stats.bytes as i64 - prev.bytes as i64;
                        format!(" ({:+} files, {})", f, humanize_signed_bytes(b))
                    })
                    .unwrap_or_default();
                out.push_str(&format!(
                    "    {}: {} files, {}{}\n",
                    name,
                    stats.files,
                    humanize_bytes(stats.bytes),
                    delta
                ));
            }
            // Sections that vanished
            if let Some(prev) = previous {
                for name in prev.by_section.keys() {
                    if !self.by_section.contains_key(name) {
                        out.push_str(&format!("    {}: removed\n", name));
                    }
                }
            }
        }
        out
    }
}

fn walk(root: &Path, dir: &Path, skip: &Path, summary: &mut BuildSummary) {
    let entries = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == *skip {
            continue;
        }
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(root, &path, skip, summary);
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
        summary.total_files += 1;
        summary.total_bytes += len;

        // First component of the path relative to `root` is the section.
        if let Some(section) = first_component(root, &path) {
            let stats = summary.by_section.entry(section).or_default();
            stats.files += 1;
            stats.bytes += len;
        }
    }
}

fn first_component(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let mut comps = rel.components();
    let first = comps.next()?;
    let rest = comps.next();
    match (first, rest) {
        // file directly under root → label by file name
        (c, None) => Some(c.as_os_str().to_string_lossy().into_owned()),
        // file under a subdirectory → label by that subdirectory
        (c, Some(_)) => Some(c.as_os_str().to_string_lossy().into_owned()),
    }
}

fn humanize_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if b >= GB {
        format!("{:.2} GB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.2} MB", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.1} KB", b as f64 / KB as f64)
    } else {
        format!("{} B", b)
    }
}

fn humanize_signed_bytes(b: i64) -> String {
    let sign = if b >= 0 { "+" } else { "-" };
    format!("{}{}", sign, humanize_bytes(b.unsigned_abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_basic() {
        assert_eq!(humanize_bytes(0), "0 B");
        assert_eq!(humanize_bytes(512), "512 B");
        assert_eq!(humanize_bytes(2048), "2.0 KB");
        assert_eq!(humanize_bytes(1024 * 1024 * 3), "3.00 MB");
    }

    #[test]
    fn humanize_signed() {
        assert_eq!(humanize_signed_bytes(0), "+0 B");
        assert_eq!(humanize_signed_bytes(2048), "+2.0 KB");
        assert_eq!(humanize_signed_bytes(-2048), "-2.0 KB");
    }

    #[test]
    fn first_component_picks_top_level() {
        let root = Path::new("/dist");
        assert_eq!(
            first_component(root, Path::new("/dist/en/guide/x.html")),
            Some("en".to_string())
        );
        assert_eq!(
            first_component(root, Path::new("/dist/index.html")),
            Some("index.html".to_string())
        );
    }
}
