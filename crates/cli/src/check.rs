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
        .chain(site.generated_route_paths())
        .collect();
    let public_assets = collect_public_assets(&site.config().docs_root_checked(project_root)?)?;
    let base = site.config().base.as_str();

    // Check for missing descriptions
    for page in site.pages() {
        for link in collect_internal_links(&page.content_html) {
            let path = normalize_internal_reference(&link, base);
            match missing_internal_reference(path.as_str(), &valid_routes, &public_assets) {
                Some(MissingReference::Route) => {
                    warn!(
                        "Dead link: {} -> {} ({})",
                        page.route.route_path, link, page.route.relative_path
                    );
                    errors += 1;
                }
                Some(MissingReference::Asset) => {
                    warn!(
                        "Missing static asset: {} -> {} ({})",
                        page.route.route_path, link, page.route.relative_path
                    );
                    errors += 1;
                }
                None => {}
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

#[derive(Debug, PartialEq, Eq)]
enum MissingReference {
    Route,
    Asset,
}

fn missing_internal_reference(
    path: &str,
    valid_routes: &HashSet<&str>,
    public_assets: &HashSet<String>,
) -> Option<MissingReference> {
    if path.is_empty() || path == "/" {
        return None;
    }

    if is_static_asset_reference(path) {
        if public_assets.contains(path) {
            None
        } else {
            Some(MissingReference::Asset)
        }
    } else if is_missing_internal_route(path, valid_routes) {
        Some(MissingReference::Route)
    } else {
        None
    }
}

fn is_missing_internal_route(path: &str, valid_routes: &HashSet<&str>) -> bool {
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

fn normalize_internal_reference(link: &str, base: &str) -> String {
    let path = link.split(['#', '?']).next().unwrap_or(link);
    let normalized_base = format!("/{}", base.trim().trim_matches('/'))
        .trim_end_matches('/')
        .to_string();
    if !normalized_base.is_empty()
        && normalized_base != "/"
        && (path == normalized_base || path.starts_with(&format!("{normalized_base}/")))
    {
        let stripped = path.trim_start_matches(&normalized_base);
        if stripped.is_empty() {
            "/".to_string()
        } else {
            stripped.to_string()
        }
    } else {
        path.to_string()
    }
}

fn collect_public_assets(docs_root: &Path) -> Result<HashSet<String>> {
    let mut out = HashSet::new();
    collect_public_assets_from_dir(docs_root, docs_root, &mut out)?;
    Ok(out)
}

fn collect_public_assets_from_dir(
    docs_root: &Path,
    dir: &Path,
    out: &mut HashSet<String>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_public_assets_from_dir(docs_root, &path, out)?;
        } else if file_type.is_file()
            && let Ok(relative) = path.strip_prefix(docs_root)
        {
            let rel = relative.to_string_lossy().replace('\\', "/");
            if is_public_static_asset(&rel) {
                out.insert(format!("/{}", rel.trim_start_matches('/')));
            }
        }
    }
    Ok(())
}

fn is_public_static_asset(file_path: &str) -> bool {
    let path = Path::new(file_path);
    if file_path.ends_with(".md") || file_path.ends_with(".typ") {
        return false;
    }

    if path
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        == Some("data")
    {
        return false;
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if file_name == "_collection.toml" || file_name == "_meta.json" {
        return false;
    }

    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if file_name.starts_with('_') && matches!(ext, "json" | "toml" | "yaml" | "yml") {
        return false;
    }

    true
}

fn is_static_asset_reference(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .and_then(|last| last.rsplit('.').next())
        .map(|ext| {
            matches!(
                ext,
                "css"
                    | "js"
                    | "mjs"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "svg"
                    | "webp"
                    | "avif"
                    | "ico"
                    | "pdf"
                    | "woff"
                    | "woff2"
                    | "ttf"
                    | "otf"
                    | "json"
                    | "xml"
                    | "txt"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{
        MissingReference, is_missing_internal_route, missing_internal_reference,
        normalize_internal_reference,
    };
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
        let assets = HashSet::from(["/images/logo.png".to_string()]);

        assert_eq!(
            missing_internal_reference("/images/logo.png", &routes, &assets),
            None
        );
        assert_eq!(
            missing_internal_reference("/images/missing.png", &routes, &assets),
            Some(MissingReference::Asset)
        );
    }

    #[test]
    fn route_check_reports_missing_routes() {
        let routes = HashSet::from(["/", "/guide/"]);
        let assets = HashSet::new();

        assert_eq!(
            missing_internal_reference("/missing", &routes, &assets),
            Some(MissingReference::Route)
        );
    }

    #[test]
    fn references_are_normalized_against_base_and_fragments() {
        assert_eq!(
            normalize_internal_reference("/docs/images/logo.png?v=1#top", "/docs/"),
            "/images/logo.png"
        );
        assert_eq!(
            normalize_internal_reference("/images/logo.png", "/docs/"),
            "/images/logo.png"
        );
    }
}
