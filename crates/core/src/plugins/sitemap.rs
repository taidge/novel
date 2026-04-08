use crate::plugin::{BuiltSiteView, Plugin};
use novel_shared::PageType;

pub struct SitemapPlugin;

impl Plugin for SitemapPlugin {
    fn name(&self) -> &str {
        "sitemap"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        match generate_sitemap_xml(site) {
            Some(xml) => vec![("sitemap.xml".to_string(), xml.into_bytes())],
            None => vec![],
        }
    }
}

pub fn generate_sitemap_xml(site: &BuiltSiteView) -> Option<String> {
    let base_url = site.config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for page in site.pages {
        // Respect per-page noindex — exclude from sitemap entirely.
        if page.frontmatter.noindex {
            continue;
        }
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
