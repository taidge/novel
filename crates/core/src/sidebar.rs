use crate::error::{NovelError, NovelResult};
use crate::source::DocsSource;
use novel_shared::{NavItem, PageData, SidebarItem};
use serde::Deserialize;
use std::collections::HashMap;

/// Metadata entry in _meta.json files
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetaEntry {
    /// Simple string: just a filename (without extension)
    Simple(String),
    /// Object with text and optional URL
    Object {
        text: String,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        collapsed: Option<bool>,
        #[serde(default)]
        items: Option<Vec<MetaEntry>>,
    },
}

/// Auto-generate sidebar from directory structure and _meta.json files
pub fn generate_sidebar(
    source: &dyn DocsSource,
    pages: &[PageData],
) -> NovelResult<HashMap<String, Vec<SidebarItem>>> {
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
        let meta_path = format!("{}/_meta.json", dir);
        let prefix = format!("/{}", dir);

        let items = if source.exists(&meta_path) {
            let content = source.read_to_string(&meta_path)?;
            load_sidebar_from_content(&content, &meta_path, &prefix, dir_page_list)?
        } else {
            auto_generate_sidebar_items(&prefix, dir_page_list)
        };

        sidebar_map.insert(prefix, items);
    }

    Ok(sidebar_map)
}

/// Load sidebar configuration from _meta.json content
fn load_sidebar_from_content(
    content: &str,
    file: &str,
    prefix: &str,
    pages: &[&PageData],
) -> NovelResult<Vec<SidebarItem>> {
    let entries: Vec<MetaEntry> = serde_json::from_str(content).map_err(|e| NovelError::Data {
        file: file.to_string(),
        message: e.to_string(),
    })?;

    let mut items = Vec::new();
    for entry in entries {
        match entry {
            MetaEntry::Simple(name) => {
                // Find matching page
                let url = format!("{}/{}", prefix, name);
                let text = pages
                    .iter()
                    .find(|p| p.route.route_path == url)
                    .map(|p| p.title.clone())
                    .unwrap_or_else(|| title_case(&name));
                items.push(SidebarItem::Link { text, url });
            }
            MetaEntry::Object {
                text,
                url,
                collapsed,
                items: sub_items,
            } => {
                if let Some(sub_entries) = sub_items {
                    let sub_items: Vec<SidebarItem> = sub_entries
                        .into_iter()
                        .map(|e| match e {
                            MetaEntry::Simple(name) => {
                                let url = format!("{}/{}", prefix, name);
                                SidebarItem::Link {
                                    text: title_case(&name),
                                    url,
                                }
                            }
                            MetaEntry::Object { text, url, .. } => {
                                let url =
                                    url.unwrap_or_else(|| format!("{}/{}", prefix, slugify(&text)));
                                SidebarItem::Link { text, url }
                            }
                        })
                        .collect();
                    items.push(SidebarItem::Group {
                        text,
                        collapsed: collapsed.unwrap_or(false),
                        items: sub_items,
                    });
                } else if let Some(url) = url {
                    items.push(SidebarItem::Link { text, url });
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
            url: p.route.route_path.clone(),
        })
        .collect();

    items.sort_by(|a, b| {
        let a_url = match a {
            SidebarItem::Link { url, .. } => url,
            _ => "",
        };
        let b_url = match b {
            SidebarItem::Link { url, .. } => url,
            _ => "",
        };
        a_url.cmp(b_url)
    });

    items
}

/// Auto-generate navigation from top-level directories
pub fn generate_nav(pages: &[PageData]) -> Vec<NavItem> {
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
            url: format!("/{}/", dir),
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
