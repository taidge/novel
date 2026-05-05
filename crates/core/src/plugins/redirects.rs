use crate::plugin::{BuiltSiteView, Plugin};
use crate::util::html_escape;

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
    let to_html = html_escape(to);
    let canonical_html = html_escape(&canonical);
    let to_js = js_string_literal(to);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={to_html}">
    <link rel="canonical" href="{canonical_html}">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="{to_html}">{to_html}</a>...</p>
    <script>window.location.replace({to_js});</script>
</body>
</html>"#,
        to_html = to_html,
        canonical_html = canonical_html,
        to_js = to_js,
    )
}

fn js_string_literal(s: &str) -> String {
    serde_json::to_string(s)
        .unwrap_or_else(|_| "\"/\"".to_string())
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use super::redirect_html;

    #[test]
    fn redirect_escapes_html_and_js_contexts() {
        let html = redirect_html(
            "/old",
            r#"/x" autofocus onfocus="alert(1)</script><script>alert(2)</script>"#,
            None,
        );

        assert!(html.contains("&quot; autofocus onfocus=&quot;alert(1)&lt;/script&gt;"));
        assert!(html.contains(r#"window.location.replace("/x\" autofocus"#));
        assert!(html.contains(r#"\u003c/script\u003e"#));
        assert!(!html.contains(r#"<script>alert(2)</script>"#));
    }
}
