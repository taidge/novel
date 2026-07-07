use crate::plugin::{BuiltSiteView, Plugin};

pub struct RobotsPlugin;

impl Plugin for RobotsPlugin {
    fn name(&self) -> &str {
        "robots"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let mut content = String::from("User-agent: *\nAllow: /\n");
        if let Some(ref site_url) = site.config.site_url {
            let sanitized = site_url.replace(['\r', '\n'], "");
            let sitemap_url =
                crate::util::join_site_url(&sanitized, &site.config.base, "/sitemap.xml");
            content.push_str(&format!("Sitemap: {sitemap_url}\n"));
        }
        vec![("robots.txt".to_string(), content.into_bytes())]
    }
}
