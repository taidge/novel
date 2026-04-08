use crate::plugin::{BuiltSiteView, Plugin};

pub struct RedirectsPlugin;

impl Plugin for RedirectsPlugin {
    fn name(&self) -> &str {
        "redirects"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let mut files = Vec::new();

        // Collect redirects from frontmatter aliases
        for page in site.pages {
            for alias in &page.frontmatter.aliases {
                let target = &page.route.route_path;
                let html = redirect_html(alias, target, site.config.site_url.as_deref());
                let path = alias_to_file_path(alias);
                files.push((path, html.into_bytes()));
            }

            // Handle redirect frontmatter (page itself redirects elsewhere)
            if let Some(ref redirect_to) = page.frontmatter.redirect {
                let from = &page.route.route_path;
                let html = redirect_html(from, redirect_to, site.config.site_url.as_deref());
                let path = alias_to_file_path(from);
                // This overwrites the page's own output — the redirect takes priority
                files.push((path, html.into_bytes()));
            }
        }

        // Global redirects from config
        for (from, to) in &site.config.redirects {
            let html = redirect_html(from, to, site.config.site_url.as_deref());
            let path = alias_to_file_path(from);
            files.push((path, html.into_bytes()));
        }

        files
    }
}

fn alias_to_file_path(alias: &str) -> String {
    let trimmed = alias.trim_matches('/');
    if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{}/index.html", trimmed)
    }
}

fn redirect_html(_from: &str, to: &str, site_url: Option<&str>) -> String {
    let canonical = match site_url {
        Some(base) => format!("{}{}", base.trim_end_matches('/'), to),
        None => to.to_string(),
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={to}">
    <link rel="canonical" href="{canonical}">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="{to}">{to}</a>...</p>
    <script>window.location.replace("{to}");</script>
</body>
</html>"#,
        to = to,
        canonical = canonical,
    )
}
