use anyhow::Result;
use novel_core::Novel;
use std::path::Path;
use tracing::{info, warn};

pub fn run_check(project_root: &Path) -> Result<()> {
    info!("Checking site...");

    let site = novel_core::DirNovel::load(project_root)?
        .plugin(novel_core::plugins::SitemapPlugin)
        .plugin(novel_core::plugins::FeedPlugin)
        .plugin(novel_core::plugins::SearchIndexPlugin)
        .build()?;

    let mut errors = 0;

    // Check for missing descriptions
    for page in site.pages() {
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
            page.frontmatter.page_type,
            Some(novel_shared::PageType::Home) | Some(novel_shared::PageType::NotFound)
        ) {
            continue;
        }
        if page.frontmatter.layout.as_deref() == Some("home") {
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
            novel_shared::SidebarItem::Link { link, .. } => {
                links.push(link.as_str());
            }
            novel_shared::SidebarItem::Group { items, .. } => {
                links.extend(collect_sidebar_links(items));
            }
            novel_shared::SidebarItem::Divider => {}
        }
    }
    links
}
