use anyhow::Result;
use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, PageLink, PageType, SidebarItem};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use tracing::info;

use crate::markdown::{MarkdownProcessor, collect_internal_links};
use crate::routing::scan_routes;
use crate::sidebar::{generate_nav, generate_sidebar};
use crate::source::DocsSource;

/// Internal build result
pub(crate) struct BuildResult {
    pub pages: Vec<PageData>,
    pub nav: Vec<NavItem>,
    pub sidebar: HashMap<String, Vec<SidebarItem>>,
}

/// Build all documentation pages (called by `Novel::build()` implementations).
///
/// `project_root` is `Some` for filesystem-backed builds (enables file-embed
/// resolution) and `None` for embed-backed builds.
pub(crate) fn build_pages(
    source: &dyn DocsSource,
    config: &SiteConfig,
    project_root: Option<&Path>,
) -> Result<BuildResult> {
    let processor =
        MarkdownProcessor::new(project_root).with_line_numbers(config.markdown.show_line_numbers);

    info!("Scanning routes...");
    let routes = scan_routes(source)?;
    info!("Found {} routes", routes.len());

    let mut pages = Vec::new();
    for route in routes {
        info!("Processing: {}", route.relative_path);
        let content = match source.read_to_string(&route.relative_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to read {}: {}", route.relative_path, e);
                continue;
            }
        };
        let relative_path = route.relative_path.clone();
        match processor.process_string(&content, Path::new(&relative_path), route) {
            Ok(page) => {
                pages.push(page);
            }
            Err(e) => {
                tracing::warn!("Failed to process {}: {}", relative_path, e);
            }
        }
    }

    set_prev_next_links(&mut pages);

    if config.markdown.check_dead_links {
        check_dead_links(&pages);
    }

    let nav = if config.theme.nav.is_empty() {
        generate_nav(&pages)
    } else {
        config.theme.nav.clone()
    };

    let sidebar = if config.theme.sidebar.is_empty() {
        generate_sidebar(source, &pages)?
    } else {
        config.theme.sidebar.clone()
    };

    Ok(BuildResult {
        pages,
        nav,
        sidebar,
    })
}

// ---------------------------------------------------------------------------
// helpers used by lib.rs (BuiltSite)
// ---------------------------------------------------------------------------

/// Convert a route path to an output file path.
/// All pages use `<route>/index.html` for clean URLs.
pub(crate) fn route_to_file_path(output_dir: &Path, route_path: &str) -> std::path::PathBuf {
    if route_path == "/" {
        output_dir.join("index.html")
    } else {
        let trimmed = route_path.trim_matches('/');
        output_dir.join(trimmed).join("index.html")
    }
}

/// Generate sitemap XML string.
pub(crate) fn generate_sitemap_xml(config: &SiteConfig, pages: &[PageData]) -> Option<String> {
    let base_url = config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for page in pages {
        let route = &page.route.route_path;
        let loc = if route == "/" {
            format!("{}/", base_url)
        } else {
            format!("{}{}", base_url, route)
        };

        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", loc));
        if let Some(ref date) = page.last_updated {
            xml.push_str(&format!("    <lastmod>{}</lastmod>\n", date));
        }
        let priority = if matches!(page.frontmatter.page_type, Some(PageType::Home)) {
            "1.0"
        } else {
            "0.7"
        };
        xml.push_str(&format!("    <priority>{}</priority>\n", priority));
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    Some(xml)
}

/// Generate Atom feed XML string.
pub(crate) fn generate_feed_xml(config: &SiteConfig, pages: &[PageData]) -> Option<String> {
    let base_url = config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&config.title)));
    xml.push_str(&format!(
        "  <subtitle>{}</subtitle>\n",
        xml_escape(&config.description)
    ));
    xml.push_str(&format!(
        "  <link href=\"{}/\" rel=\"alternate\"/>\n",
        base_url
    ));
    xml.push_str(&format!(
        "  <link href=\"{}/feed.xml\" rel=\"self\"/>\n",
        base_url
    ));
    xml.push_str(&format!("  <id>{}/</id>\n", base_url));

    for page in pages {
        if matches!(
            page.frontmatter.page_type,
            Some(PageType::Home) | Some(PageType::NotFound)
        ) {
            continue;
        }
        let url = format!("{}{}", base_url, &page.route.route_path);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&page.title)));
        xml.push_str(&format!("    <link href=\"{}\"/>\n", url));
        xml.push_str(&format!("    <id>{}</id>\n", url));
        if let Some(ref date) = page.last_updated {
            xml.push_str(&format!("    <updated>{}T00:00:00Z</updated>\n", date));
        }
        if !page.description.is_empty() {
            xml.push_str(&format!(
                "    <summary>{}</summary>\n",
                xml_escape(&page.description)
            ));
        }
        xml.push_str("  </entry>\n");
    }

    xml.push_str("</feed>\n");
    Some(xml)
}

// ---------------------------------------------------------------------------
// internal helpers
// ---------------------------------------------------------------------------

fn set_prev_next_links(pages: &mut [PageData]) {
    let doc_indices: Vec<usize> = pages
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            !matches!(
                p.frontmatter.page_type,
                Some(PageType::Home) | Some(PageType::NotFound)
            )
        })
        .map(|(i, _)| i)
        .collect();

    for (pos, &idx) in doc_indices.iter().enumerate() {
        if pos > 0 {
            let prev_idx = doc_indices[pos - 1];
            pages[idx].prev_page = Some(PageLink {
                title: pages[prev_idx].title.clone(),
                link: pages[prev_idx].route.route_path.clone(),
            });
        }
        if pos + 1 < doc_indices.len() {
            let next_idx = doc_indices[pos + 1];
            pages[idx].next_page = Some(PageLink {
                title: pages[next_idx].title.clone(),
                link: pages[next_idx].route.route_path.clone(),
            });
        }
    }
}

fn check_dead_links(pages: &[PageData]) {
    let valid_routes: HashSet<&str> = pages.iter().map(|p| p.route.route_path.as_str()).collect();

    for page in pages {
        let links = collect_internal_links(&page.content_html);
        for link in links {
            let path = link.split('#').next().unwrap_or(&link);
            if path.is_empty() || path == "/" {
                continue;
            }
            if !valid_routes.contains(path) {
                let alt = if path.ends_with('/') {
                    path.trim_end_matches('/').to_string()
                } else {
                    format!("{}/", path)
                };
                if !valid_routes.contains(alt.as_str()) {
                    tracing::warn!(
                        "Dead link in {}: {} (target not found)",
                        page.route.relative_path,
                        link
                    );
                }
            }
        }
    }
}

pub(crate) fn get_git_last_updated(file_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%Y-%m-%d", "--"])
        .arg(file_path)
        .output()
        .ok()?;

    if output.status.success() {
        let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if date.is_empty() { None } else { Some(date) }
    } else {
        None
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
