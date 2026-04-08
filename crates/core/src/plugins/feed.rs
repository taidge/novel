use crate::plugin::{BuiltSiteView, Plugin};
use novel_shared::{PageData, PageType};
use std::collections::BTreeSet;

pub struct FeedPlugin;

impl Plugin for FeedPlugin {
    fn name(&self) -> &str {
        "feed"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        if let Some(xml) = generate_feed_xml(site) {
            out.push(("feed.xml".to_string(), xml.into_bytes()));
        }
        if let Some(json) = generate_json_feed(site) {
            out.push(("feed.json".to_string(), json.into_bytes()));
        }

        // Per-collection feeds: one feed.xml per collection that has the
        // `feed = true` flag in its _collection.toml. Discovered by scanning
        // pages for `collection` field.
        let collections: BTreeSet<String> = site
            .pages
            .iter()
            .filter_map(|p| p.collection.clone())
            .collect();
        for col in collections {
            if let Some(xml) = generate_collection_feed_xml(site, &col) {
                out.push((format!("{}/feed.xml", col), xml.into_bytes()));
            }
        }
        out
    }
}

/// Generate a JSON Feed v1.1 (https://www.jsonfeed.org/version/1.1/) for the
/// whole site. Returns `None` when `site_url` is not configured.
pub fn generate_json_feed(site: &BuiltSiteView) -> Option<String> {
    let base_url = site.config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let items: Vec<serde_json::Value> = site
        .pages
        .iter()
        .filter(|p| {
            !matches!(
                p.frontmatter.page_type,
                Some(PageType::Home) | Some(PageType::NotFound)
            )
        })
        .map(|p| {
            let url = format!("{}{}", base_url, &p.route.route_path);
            let mut item = serde_json::json!({
                "id": url,
                "url": url,
                "title": p.title,
            });
            if !p.description.is_empty() {
                item["summary"] = serde_json::Value::String(p.description.clone());
            }
            if let Some(ref s) = p.summary_html {
                item["content_html"] = serde_json::Value::String(s.clone());
            }
            if let Some(ref d) = p.date.as_ref().or(p.last_updated.as_ref()) {
                item["date_published"] =
                    serde_json::Value::String(format!("{}T00:00:00Z", d));
            }
            if !p.frontmatter.tags.is_empty() {
                item["tags"] = serde_json::Value::from(p.frontmatter.tags.clone());
            }
            item
        })
        .collect();

    let feed = serde_json::json!({
        "version": "https://jsonfeed.org/version/1.1",
        "title": site.config.title,
        "description": site.config.description,
        "home_page_url": format!("{}/", base_url),
        "feed_url": format!("{}/feed.json", base_url),
        "items": items,
    });
    serde_json::to_string_pretty(&feed).ok()
}

pub fn generate_feed_xml(site: &BuiltSiteView) -> Option<String> {
    let base_url = site.config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!(
        "  <title>{}</title>\n",
        xml_escape(&site.config.title)
    ));
    xml.push_str(&format!(
        "  <subtitle>{}</subtitle>\n",
        xml_escape(&site.config.description)
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

    for page in site.pages {
        if matches!(
            page.frontmatter.page_type,
            Some(PageType::Home) | Some(PageType::NotFound)
        ) {
            continue;
        }
        let url = format!("{}{}", base_url, &page.route.route_path);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            xml_escape(&page.title)
        ));
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

/// Build an Atom feed scoped to a single collection.
pub fn generate_collection_feed_xml(site: &BuiltSiteView, collection: &str) -> Option<String> {
    let base_url = site.config.site_url.as_deref()?.trim_end_matches('/');
    if base_url.is_empty() {
        return None;
    }

    let entries: Vec<&PageData> = site
        .pages
        .iter()
        .filter(|p| p.collection.as_deref() == Some(collection))
        .collect();
    if entries.is_empty() {
        return None;
    }

    let title = format!("{} — {}", site.config.title, collection);
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&title)));
    xml.push_str(&format!(
        "  <link href=\"{}/{}/\" rel=\"alternate\"/>\n",
        base_url, collection
    ));
    xml.push_str(&format!(
        "  <link href=\"{}/{}/feed.xml\" rel=\"self\"/>\n",
        base_url, collection
    ));
    xml.push_str(&format!("  <id>{}/{}/</id>\n", base_url, collection));

    for page in entries {
        let url = format!("{}{}", base_url, &page.route.route_path);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!(
            "    <title>{}</title>\n",
            xml_escape(&page.title)
        ));
        xml.push_str(&format!("    <link href=\"{}\"/>\n", url));
        xml.push_str(&format!("    <id>{}</id>\n", url));
        if let Some(ref date) = page.date {
            xml.push_str(&format!("    <updated>{}T00:00:00Z</updated>\n", date));
        } else if let Some(ref date) = page.last_updated {
            xml.push_str(&format!("    <updated>{}T00:00:00Z</updated>\n", date));
        }
        if let Some(ref summary) = page.summary_html {
            xml.push_str(&format!(
                "    <summary type=\"html\">{}</summary>\n",
                xml_escape(summary)
            ));
        } else if !page.description.is_empty() {
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
