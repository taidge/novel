//! Collection discovery & filtering for general SSG mode.
//!
//! A "collection" is a top-level directory under the docs root that contains a
//! `_collection.toml` marker file. Pages inside that directory are grouped,
//! filterable (drafts/future/expiry), sortable, and can be paginated into a
//! list page.

use anyhow::Result;
use novel_shared::PageData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Per-collection configuration loaded from `<collection>/_collection.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionConfig {
    /// Default layout for entries (e.g. "blog", "page")
    pub layout: String,
    /// Layout for the list (index) page
    pub list_layout: String,
    /// Sort key: "date" | "weight" | "title"
    pub sort_by: String,
    /// "asc" | "desc"
    pub order: String,
    /// Items per page; 0 = no pagination
    pub paginate_by: usize,
    /// Whether this collection is published in the build
    pub publish: bool,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            layout: "blog".to_string(),
            list_layout: "list".to_string(),
            sort_by: "date".to_string(),
            order: "desc".to_string(),
            paginate_by: 10,
            publish: true,
        }
    }
}

/// Discovered collection metadata.
#[derive(Debug, Clone)]
pub struct Collection {
    /// Collection name (top-level directory name)
    pub name: String,
    pub config: CollectionConfig,
}

/// Discover collections in the docs root by scanning for `_collection.toml`
/// files at depth 1.
pub fn discover_collections(docs_root: &Path) -> Result<HashMap<String, Collection>> {
    let mut out = HashMap::new();
    if !docs_root.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(docs_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let cfg_path = path.join("_collection.toml");
        if !cfg_path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let raw = std::fs::read_to_string(&cfg_path)?;
        let config: CollectionConfig = toml::from_str(&raw).unwrap_or_default();
        out.insert(name.clone(), Collection { name, config });
    }
    Ok(out)
}

/// Returns the collection name a page belongs to (top-level directory of its
/// relative path), if that directory is a registered collection.
pub fn collection_for_page(
    relative_path: &str,
    collections: &HashMap<String, Collection>,
) -> Option<String> {
    let segments: Vec<&str> = relative_path.split('/').collect();
    if segments.len() < 2 {
        return None;
    }
    let top = segments[0];
    if collections.contains_key(top) {
        Some(top.to_string())
    } else {
        None
    }
}

/// Filter pages by draft / future / expiry rules.
///
/// Today is given as a string `YYYY-MM-DD` for ordering. We compare lexically
/// (ISO dates sort correctly).
pub fn filter_pages(
    pages: Vec<PageData>,
    include_drafts: bool,
    include_future: bool,
    today: &str,
) -> Vec<PageData> {
    pages
        .into_iter()
        .filter(|p| {
            if !include_drafts && p.frontmatter.draft {
                return false;
            }
            if !include_future
                && let Some(d) = p.date.as_deref()
                && d.as_bytes() > today.as_bytes()
            {
                return false;
            }
            if let Some(exp) = p.frontmatter.expiry_date.as_deref()
                && exp.as_bytes() <= today.as_bytes()
            {
                return false;
            }
            true
        })
        .collect()
}

/// Sort pages within a collection according to its config.
pub fn sort_collection_entries(entries: &mut [&PageData], cfg: &CollectionConfig) {
    let order_desc = cfg.order.eq_ignore_ascii_case("desc");
    match cfg.sort_by.as_str() {
        "weight" => entries.sort_by(|a, b| {
            let aw = a.frontmatter.weight.unwrap_or(i64::MAX);
            let bw = b.frontmatter.weight.unwrap_or(i64::MAX);
            if order_desc { bw.cmp(&aw) } else { aw.cmp(&bw) }
        }),
        "title" => entries.sort_by(|a, b| {
            if order_desc {
                b.title.cmp(&a.title)
            } else {
                a.title.cmp(&b.title)
            }
        }),
        _ => entries.sort_by(|a, b| {
            let ad = a.date.as_deref().unwrap_or("");
            let bd = b.date.as_deref().unwrap_or("");
            if order_desc { bd.cmp(ad) } else { ad.cmp(bd) }
        }),
    }
}

/// Today's date as `YYYY-MM-DD`.
pub fn today_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Convert epoch seconds to YYYY-MM-DD (UTC) without external deps.
    let days = secs / 86_400;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(days_since_epoch: i64) -> (i32, u32, u32) {
    // Algorithm from Howard Hinnant's date library (civil_from_days)
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}
