use crate::plugin::{BuiltSiteView, Plugin};
use novel_shared::PageType;

pub struct FeedPlugin;

impl Plugin for FeedPlugin {
    fn name(&self) -> &str {
        "feed"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        match generate_feed_xml(site) {
            Some(xml) => vec![("feed.xml".to_string(), xml.into_bytes())],
            None => vec![],
        }
    }
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

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
