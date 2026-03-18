use anyhow::Result;
use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, PageLink, PageType, SidebarItem};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use tracing::info;

use crate::markdown::{MarkdownProcessor, collect_internal_links};
use crate::plugin::Plugin;
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
    plugins: &[Box<dyn Plugin>],
) -> Result<BuildResult> {
    // Collect custom container directives from all plugins
    let custom_directives: Vec<_> = plugins
        .iter()
        .flat_map(|p| p.container_directives())
        .collect();

    let processor = MarkdownProcessor::new(project_root)
        .with_line_numbers(config.markdown.show_line_numbers)
        .with_custom_directives(custom_directives);

    info!("Scanning routes...");
    let routes = scan_routes(source)?;
    info!("Found {} routes", routes.len());

    // Read all file contents sequentially (I/O bound)
    let route_contents: Vec<_> = routes
        .into_iter()
        .filter_map(|route| {
            match source.read_to_string(&route.relative_path) {
                Ok(content) => Some((route, content)),
                Err(e) => {
                    tracing::warn!("Failed to read {}: {}", route.relative_path, e);
                    None
                }
            }
        })
        .collect();

    info!("Processing {} pages in parallel...", route_contents.len());

    // Process all pages in parallel (CPU bound)
    let mut pages: Vec<PageData> = route_contents
        .par_iter()
        .filter_map(|(route, content)| {
            // Plugin: transform_markdown
            let file_path = Path::new(&route.relative_path);
            let content = plugins
                .iter()
                .fold(content.clone(), |md, p| p.transform_markdown(md, file_path));

            let relative_path = route.relative_path.clone();
            match processor.process_string(&content, Path::new(&relative_path), route.clone()) {
                Ok(mut page) => {
                    // Plugin: transform_html
                    for p in plugins {
                        let html = std::mem::take(&mut page.content_html);
                        page.content_html = p.transform_html(html, &page);
                    }
                    Some(page)
                }
                Err(e) => {
                    tracing::warn!("Failed to process {}: {}", relative_path, e);
                    None
                }
            }
        })
        .collect();

    // Sort by route path for deterministic ordering
    pages.sort_by(|a, b| a.route.route_path.cmp(&b.route.route_path));

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
