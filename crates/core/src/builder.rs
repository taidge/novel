use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, PageLink, SidebarItem};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use tracing::info;

use crate::error::{NovelError, NovelResult};
use crate::markdown::{MarkdownProcessor, collect_internal_links};
use crate::plugin::{BuiltSiteView, Plugin};
use crate::routing::scan_routes;
use crate::sidebar::{generate_nav, generate_sidebar};
use crate::source::DocsSource;
use crate::typst_processor::TypstProcessor;
use crate::util::safe_join_relative;
use crate::util::strip_html_tags;

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
    source_root: Option<&Path>,
    plugins: &[Box<dyn Plugin>],
) -> NovelResult<BuildResult> {
    // Plugin: on_pre_build
    for p in plugins {
        p.on_pre_build(config);
    }

    // Collect custom container directives from all plugins
    let custom_directives: Vec<_> = plugins
        .iter()
        .flat_map(|p| p.container_directives())
        .collect();

    let syntax_theme = config.markdown.syntax_theme.clone();
    let md_processor = MarkdownProcessor::new(project_root)
        .with_source_root(source_root)
        .with_line_numbers(config.markdown.show_line_numbers)
        .with_wrap_code(config.markdown.default_wrap_code)
        .with_summary_separator(config.content.summary_separator.clone())
        .with_math(config.markdown.math)
        .with_mermaid(config.markdown.mermaid)
        .with_syntax_theme(syntax_theme)
        .with_custom_directives(custom_directives);

    // Typst processor (only for filesystem-backed builds)
    let typst_processor = source_root.map(TypstProcessor::new);

    // Check typst CLI availability once if there are .typ files
    info!("Scanning routes...");
    let routes = scan_routes(source);
    info!("Found {} routes", routes.len());

    let has_typst_routes = routes.iter().any(|r| r.relative_path.ends_with(".typ"));
    if has_typst_routes && typst_processor.is_none() {
        tracing::warn!(
            "Found .typ files but this build has no project root (embedded source) — \
             typst compilation requires a filesystem project root, these files will be skipped."
        );
    } else if has_typst_routes && !TypstProcessor::is_available() {
        tracing::warn!(
            "Found .typ files but `typst` CLI is not on PATH — \
             these files will be skipped. Install typst: https://typst.app"
        );
    }

    // Typst is only "available" if the processor exists (needs project root)
    // AND the CLI is installed. Previously the processor check was missing,
    // so EmbedNovel with .typ files would panic at the .expect() below.
    let typst_available =
        has_typst_routes && typst_processor.is_some() && TypstProcessor::is_available();

    // Read all file contents sequentially (I/O bound)
    let mut route_contents = Vec::new();
    let mut read_failures = Vec::new();
    for route in routes {
        // Skip .typ files when typst is not available
        if route.relative_path.ends_with(".typ") && !typst_available {
            continue;
        }
        match source.read_to_string(&route.relative_path) {
            Ok(content) => route_contents.push((route, content)),
            Err(e) => read_failures.push(format!("{}: {}", route.relative_path, e)),
        }
    }

    if !read_failures.is_empty() {
        return Err(NovelError::Build(format!(
            "Failed to read {} page(s): {}",
            read_failures.len(),
            read_failures.join("; ")
        )));
    }

    info!("Processing {} pages in parallel...", route_contents.len());

    // Process all pages in parallel (CPU bound). `into_par_iter` consumes
    // `route_contents` so each `(route, content)` can be moved into the
    // closure without extra clones (T-PERF-2).
    let processed_pages: Vec<Result<PageData, String>> = route_contents
        .into_par_iter()
        .map(|(route, content)| {
            // `relative_path` is kept as an owned String so it remains
            // usable for error reporting after `route` is moved into the
            // per-branch calls below.
            let relative_path = route.relative_path.clone();
            let is_typst = relative_path.ends_with(".typ");

            let page_result = if is_typst {
                // Typst processing — no markdown plugin transforms
                typst_processor
                    .as_ref()
                    .expect("typst_processor must exist for .typ files")
                    .process_file(&content, route)
            } else {
                // Markdown processing with plugin transforms. `content` is
                // moved into the fold, eliminating the previous clone.
                let file_path = Path::new(&relative_path);
                let transformed = plugins
                    .iter()
                    .fold(content, |md, p| p.transform_markdown(md, file_path));
                md_processor.process_string(&transformed, file_path, route)
            };

            match page_result {
                Ok(mut page) => {
                    // Plugin: transform_html (applies to both markdown and typst)
                    for p in plugins {
                        let html = std::mem::take(&mut page.content_html);
                        page.content_html = p.transform_html(html, &page);
                    }

                    // Compute word count and reading time
                    let plain_text = strip_html_tags(&page.content_html);
                    let wc = plain_text.split_whitespace().count() as u32;
                    page.word_count = Some(wc);
                    page.reading_time = Some((wc / 200).max(1));

                    // Plugin: on_page_built
                    for p in plugins {
                        p.on_page_built(&page);
                    }

                    Ok(page)
                }
                Err(e) => Err(format!("{}: {}", relative_path, e)),
            }
        })
        .collect();

    let mut pages = Vec::with_capacity(processed_pages.len());
    let mut process_failures = Vec::new();
    for page in processed_pages {
        match page {
            Ok(page) => pages.push(page),
            Err(err) => process_failures.push(err),
        }
    }

    if !process_failures.is_empty() {
        return Err(NovelError::Build(format!(
            "Failed to process {} page(s): {}",
            process_failures.len(),
            process_failures.join("; ")
        )));
    }

    // Sort by route path for deterministic ordering
    pages.sort_by(|a, b| a.route.route_path.cmp(&b.route.route_path));

    // Compute breadcrumbs for each page
    compute_breadcrumbs(&mut pages);

    set_prev_next_links(&mut pages);

    if config.markdown.check_dead_links {
        check_dead_links(&pages);
    }

    let mut nav = if config.theme.nav.is_empty() {
        generate_nav(&pages)
    } else {
        config.theme.nav.clone()
    };

    let mut sidebar = if config.theme.sidebar.is_empty() {
        generate_sidebar(source, &pages)?
    } else {
        config.theme.sidebar.clone()
    };

    // Plugin: transform_nav and transform_sidebar
    // We pass cloned nav/sidebar in the view since we need to move the originals
    // through the transform chain.
    for p in plugins {
        let nav_snapshot = nav.clone();
        let sidebar_snapshot = sidebar.clone();
        let view = BuiltSiteView {
            config,
            pages: &pages,
            nav: &nav_snapshot,
            sidebar: &sidebar_snapshot,
            project_root,
        };
        nav = p.transform_nav(nav, &view);
        sidebar = p.transform_sidebar(sidebar, &view);
    }

    // Plugin: generate_pages (virtual pages)
    let mut virtual_pages_collected = Vec::new();
    for p in plugins {
        let view = BuiltSiteView {
            config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root,
        };
        let vp = p.generate_pages(&view);
        virtual_pages_collected.extend(vp);
    }
    pages.extend(virtual_pages_collected);

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
pub(crate) fn route_to_file_path(
    output_dir: &Path,
    route_path: &str,
) -> std::io::Result<std::path::PathBuf> {
    if route_path == "/" {
        safe_join_relative(output_dir, Path::new("index.html"))
    } else {
        let trimmed = route_path.trim_matches('/');
        safe_join_relative(output_dir, Path::new(trimmed).join("index.html"))
    }
}

// ---------------------------------------------------------------------------
// internal helpers
// ---------------------------------------------------------------------------

fn set_prev_next_links(pages: &mut [PageData]) {
    let doc_indices: Vec<usize> = pages
        .iter()
        .enumerate()
        .filter(|(_, p)| !matches!(p.frontmatter.layout.as_deref(), Some("home") | Some("404")))
        .map(|(i, _)| i)
        .collect();

    for (pos, &idx) in doc_indices.iter().enumerate() {
        if pos > 0 {
            let prev_idx = doc_indices[pos - 1];
            pages[idx].prev_page = Some(PageLink {
                title: pages[prev_idx].title.clone(),
                url: pages[prev_idx].route.route_path.clone(),
            });
        }
        if pos + 1 < doc_indices.len() {
            let next_idx = doc_indices[pos + 1];
            pages[idx].next_page = Some(PageLink {
                title: pages[next_idx].title.clone(),
                url: pages[next_idx].route.route_path.clone(),
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

/// Compute breadcrumbs for each page from its route path segments.
fn compute_breadcrumbs(pages: &mut [PageData]) {
    // Build a map of route_path -> title for quick lookup
    let title_map: HashMap<String, String> = pages
        .iter()
        .map(|p| (p.route.route_path.clone(), p.title.clone()))
        .collect();

    for page in pages.iter_mut() {
        let route = &page.route.route_path;
        if route == "/" {
            continue;
        }

        let mut crumbs = vec![PageLink {
            title: "Home".to_string(),
            url: "/".to_string(),
        }];

        let trimmed = route.trim_matches('/');
        let segments: Vec<&str> = trimmed.split('/').collect();
        let mut path_acc = String::new();

        for (i, seg) in segments.iter().enumerate() {
            if i < segments.len() - 1 {
                // Intermediate directory
                path_acc.push('/');
                path_acc.push_str(seg);
                let dir_route = format!("{}/", path_acc);
                let title = title_map
                    .get(&dir_route)
                    .cloned()
                    .unwrap_or_else(|| title_case(seg));
                crumbs.push(PageLink {
                    title,
                    url: dir_route,
                });
            } else {
                // Current page (no link needed, but included for display)
                crumbs.push(PageLink {
                    title: page.title.clone(),
                    url: route.clone(),
                });
            }
        }

        page.breadcrumbs = crumbs;
    }
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

pub(crate) fn get_git_updated_at(file_path: &Path) -> Option<String> {
    // `%ad` = author date; `--date=short` formats it as YYYY-MM-DD.
    // (The previous `%Y-%m-%d` was a strftime format, not a git placeholder —
    // git would pass `%Y`, `%m`, `%d` through literally, producing garbage.)
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ad", "--date=short", "--"])
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

#[cfg(test)]
mod tests {
    use super::route_to_file_path;
    use std::path::Path;

    #[test]
    fn route_to_file_path_rejects_parent_segments() {
        assert!(route_to_file_path(Path::new("dist"), "/../secret").is_err());
        assert!(route_to_file_path(Path::new("dist"), "/guide/../../secret").is_err());
    }

    #[test]
    fn route_to_file_path_keeps_pages_under_output_dir() {
        assert_eq!(
            route_to_file_path(Path::new("dist"), "/guide/intro").unwrap(),
            Path::new("dist")
                .join("guide")
                .join("intro")
                .join("index.html")
        );
    }
}
