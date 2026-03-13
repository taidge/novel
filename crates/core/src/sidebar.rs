use anyhow::Result;
use sapid_shared::{NavItem, PageData, SidebarItem};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Metadata entry in _meta.json files
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetaEntry {
    /// Simple string: just a filename (without extension)
    Simple(String),
    /// Object with text and optional link
    Object {
        text: String,
        #[serde(default)]
        link: Option<String>,
        #[serde(default)]
        collapsed: Option<bool>,
        #[serde(default)]
        items: Option<Vec<MetaEntry>>,
    },
}

/// Auto-generate sidebar from directory structure and _meta.json files
pub fn generate_sidebar(
    docs_root: &Path,
    pages: &[PageData],
) -> Result<HashMap<String, Vec<SidebarItem>>> {
    let mut sidebar_map: HashMap<String, Vec<SidebarItem>> = HashMap::new();

    // Group pages by their top-level directory
    let mut dir_pages: HashMap<String, Vec<&PageData>> = HashMap::new();
    for page in pages {
        let parts: Vec<&str> = page.route.relative_path.split('/').collect();
        if parts.len() > 1 {
            let top_dir = parts[0].to_string();
            dir_pages.entry(top_dir).or_default().push(page);
        }
    }

    for (dir, dir_page_list) in &dir_pages {
        let meta_path = docs_root.join(dir).join("_meta.json");
        let prefix = format!("/{}", dir);

        let items = if meta_path.exists() {
            load_sidebar_from_meta(&meta_path, &prefix, dir_page_list)?
        } else {
            auto_generate_sidebar_items(&prefix, dir_page_list)
        };

        sidebar_map.insert(prefix, items);
    }

    Ok(sidebar_map)
}

/// Load sidebar configuration from _meta.json
fn load_sidebar_from_meta(
    meta_path: &Path,
    prefix: &str,
    pages: &[&PageData],
) -> Result<Vec<SidebarItem>> {
    let content = std::fs::read_to_string(meta_path)?;
    let entries: Vec<MetaEntry> = serde_json::from_str(&content)?;

    let mut items = Vec::new();
    for entry in entries {
        match entry {
            MetaEntry::Simple(name) => {
                // Find matching page
                let link = format!("{}/{}", prefix, name);
                let text = pages
                    .iter()
                    .find(|p| p.route.route_path == link)
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|| title_case(&name));
                items.push(SidebarItem::Link { text, link });
            }
            MetaEntry::Object {
                text,
                link,
                collapsed,
                items: sub_items,
            } => {
                if let Some(sub_entries) = sub_items {
                    let sub_items: Vec<SidebarItem> = sub_entries
                        .into_iter()
                        .filter_map(|e| match e {
                            MetaEntry::Simple(name) => {
                                let link = format!("{}/{}", prefix, name);
                                Some(SidebarItem::Link {
                                    text: title_case(&name),
                                    link,
                                })
                            }
                            MetaEntry::Object { text, link, .. } => {
                                let link = link
                                    .unwrap_or_else(|| format!("{}/{}", prefix, slugify(&text)));
                                Some(SidebarItem::Link { text, link })
                            }
                        })
                        .collect();
                    items.push(SidebarItem::Group {
                        text,
                        collapsed: collapsed.unwrap_or(false),
                        items: sub_items,
                    });
                } else if let Some(link) = link {
                    items.push(SidebarItem::Link { text, link });
                }
            }
        }
    }

    Ok(items)
}

/// Auto-generate sidebar items from page list (alphabetically sorted)
fn auto_generate_sidebar_items(prefix: &str, pages: &[&PageData]) -> Vec<SidebarItem> {
    let mut items: Vec<SidebarItem> = pages
        .iter()
        .filter(|p| p.route.route_path != format!("{}/", prefix))
        .map(|p| SidebarItem::Link {
            text: p.title.clone(),
            link: p.route.route_path.clone(),
        })
        .collect();

    items.sort_by(|a, b| {
        let a_link = match a {
            SidebarItem::Link { link, .. } => link,
            _ => "",
        };
        let b_link = match b {
            SidebarItem::Link { link, .. } => link,
            _ => "",
        };
        a_link.cmp(b_link)
    });

    items
}

/// Auto-generate navigation from top-level directories
pub fn generate_nav(_docs_root: &Path, pages: &[PageData]) -> Vec<NavItem> {
    let mut dirs: Vec<String> = Vec::new();

    for page in pages {
        let parts: Vec<&str> = page.route.relative_path.split('/').collect();
        if parts.len() > 1 {
            let dir = parts[0].to_string();
            if !dirs.contains(&dir) {
                dirs.push(dir);
            }
        }
    }

    dirs.sort();

    dirs.into_iter()
        .map(|dir| NavItem {
            text: title_case(&dir),
            link: format!("/{}/", dir),
            active_match: Some(format!("/{}/", dir)),
        })
        .collect()
}

fn title_case(s: &str) -> String {
    s.replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slugify(s: &str) -> String {
    s.to_lowercase().replace(' ', "-")
}
