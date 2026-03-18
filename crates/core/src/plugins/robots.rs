use crate::plugin::{BuiltSiteView, Plugin};

pub struct RobotsPlugin;

impl Plugin for RobotsPlugin {
    fn name(&self) -> &str {
        "robots"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let mut content = String::from("User-agent: *\nAllow: /\n");
        if let Some(ref site_url) = site.config.site_url {
            let base = site_url.trim_end_matches('/');
            content.push_str(&format!("Sitemap: {}/sitemap.xml\n", base));
        }
        vec![("robots.txt".to_string(), content.into_bytes())]
    }
}
