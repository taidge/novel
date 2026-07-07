use anyhow::Result;
use novel_core::Novel;
use novel_core::markdown::collect_internal_links;
use std::collections::HashSet;
use std::path::Path;
use tracing::{info, warn};

pub fn run_check(project_root: &Path) -> Result<()> {
    info!("Checking site...");

    let site = novel_core::DirNovel::load(project_root)?
        .plugin(novel_core::plugins::SitemapPlugin)
        .plugin(novel_core::plugins::FeedPlugin)
        .plugin(novel_core::plugins::SearchIndexPlugin)
        .plugin(novel_core::plugins::LlmsTxtPlugin)
        .plugin(novel_core::plugins::MarkdownMirrorPlugin)
        .plugin(novel_core::plugins::PwaPlugin)
        .plugin(novel_core::plugins::RobotsPlugin)
        .plugin(novel_core::plugins::RedirectsPlugin)
        .build()?;

    let mut errors = 0;
    let valid_routes: HashSet<&str> = site
        .pages()
        .iter()
        .map(|page| page.route.route_path.as_str())
        .collect();

    // Check for missing descriptions
    for page in site.pages() {
        for link in collect_internal_links(&page.content_html) {
            let path = link.split('#').next().unwrap_or(&link);
            if is_missing_internal_route(path, &valid_routes) {
                warn!(
                    "Dead link: {} -> {} ({})",
                    page.route.route_path, link, page.route.relative_path
                );
                errors += 1;
            }
        }

        if page.description.is_empty() {
            warn!(
                "Missing description: {} ({})",
                page.route.route_path, page.route.relative_path
            );
            errors += 1;
        }
    }

    // Check for orphan pages (not in any sidebar)
    let sidebar = site.sidebar();
    let sidebar_links: std::collections::HashSet<&str> = sidebar
        .values()
        .flat_map(|items| collect_sidebar_links(items))
        .collect();

    for page in site.pages() {
        if matches!(
            page.frontmatter.layout.as_deref(),
            Some("home") | Some("404")
        ) {
            continue;
        }
        if page.route.route_path.ends_with('/') {
            continue;
        }
        if !sidebar_links.contains(page.route.route_path.as_str()) {
            warn!(
                "Orphan page (not in sidebar): {} ({})",
                page.route.route_path, page.route.relative_path
            );
            errors += 1;
        }
    }

    if errors > 0 {
        anyhow::bail!("{} issue(s) found", errors);
    }

    info!("All checks passed!");
    Ok(())
}

fn collect_sidebar_links(items: &[novel_shared::SidebarItem]) -> Vec<&str> {
    let mut links = Vec::new();
    for item in items {
        match item {
            novel_shared::SidebarItem::Link { url, .. } => {
                links.push(url.as_str());
            }
            novel_shared::SidebarItem::Group { items, .. } => {
                links.extend(collect_sidebar_links(items));
            }
            novel_shared::SidebarItem::Divider => {}
        }
    }
    links
}

fn is_missing_internal_route(path: &str, valid_routes: &HashSet<&str>) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }

    if path
        .rsplit('/')
        .next()
        .map(|last| last.contains('.'))
        .unwrap_or(false)
    {
        return false;
    }

    if valid_routes.contains(path) {
        return false;
    }

    let alt = if path.ends_with('/') {
        path.trim_end_matches('/').to_string()
    } else {
        format!("{}/", path)
    };
    !valid_routes.contains(alt.as_str())
}

#[cfg(test)]
mod tests {
    use super::is_missing_internal_route;
    use std::collections::HashSet;

    #[test]
    fn route_check_accepts_known_routes_and_slash_variants() {
        let routes = HashSet::from(["/", "/guide/intro", "/guide/"]);
        assert!(!is_missing_internal_route("/guide/intro", &routes));
        assert!(!is_missing_internal_route("/guide", &routes));
        assert!(is_missing_internal_route("/missing", &routes));
    }

    #[test]
    fn route_check_ignores_static_asset_paths() {
        let routes = HashSet::from(["/"]);
        assert!(!is_missing_internal_route("/images/logo.png", &routes));
    }
}
