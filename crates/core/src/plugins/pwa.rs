use crate::plugin::{BuiltSiteView, Plugin};
use crate::util::html_escape;
use crate::{CSS_CONTENT, JS_CONTENT};
use novel_shared::PageType;

pub struct PwaPlugin;

impl Plugin for PwaPlugin {
    fn name(&self) -> &str {
        "pwa"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        if !site.config.pwa.enabled {
            return vec![];
        }

        vec![
            (
                "manifest.webmanifest".to_string(),
                generate_manifest(site).into_bytes(),
            ),
            (
                "service-worker.js".to_string(),
                generate_service_worker(site).into_bytes(),
            ),
            (
                "offline.html".to_string(),
                generate_offline_html(site).into_bytes(),
            ),
        ]
    }
}

fn generate_manifest(site: &BuiltSiteView) -> String {
    let name = site
        .config
        .pwa
        .name
        .as_deref()
        .unwrap_or(&site.config.title);
    let short_name = site.config.pwa.short_name.as_deref().unwrap_or(name);
    let manifest = serde_json::json!({
        "name": name,
        "short_name": short_name,
        "description": site.config.description,
        "start_url": site.config.base,
        "scope": site.config.base,
        "display": site.config.pwa.display,
        "theme_color": site.config.pwa.theme_color,
        "background_color": site.config.pwa.background_color,
    });
    serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string())
}

fn generate_service_worker(site: &BuiltSiteView) -> String {
    let cache_name = format!(
        "novel-{}-{}",
        sanitize_cache_key(&site.config.title),
        crate::util::fnv1a(site.config.description.as_bytes())
    );
    let mut urls = vec![
        site.config.base.clone(),
        with_base(site, "offline.html"),
        with_base(site, &format!("assets/{}", css_filename(site))),
        with_base(site, &format!("assets/{}", js_filename(site))),
    ];
    if site.config.pwa.cache_search_index {
        urls.push(with_base(site, "assets/search-index.json"));
    }
    for page in site.pages {
        if page.frontmatter.noindex
            || page.frontmatter.redirect.is_some()
            || matches!(page.frontmatter.page_type, Some(PageType::NotFound))
        {
            continue;
        }
        urls.push(with_base(site, &page.route.route_path));
    }
    urls.sort();
    urls.dedup();
    let urls_json = serde_json::to_string(&urls).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"const CACHE_NAME = {cache_name:?};
const PRECACHE_URLS = {urls_json};

self.addEventListener('install', event => {{
  event.waitUntil(caches.open(CACHE_NAME).then(cache => cache.addAll(PRECACHE_URLS)).then(() => self.skipWaiting()));
}});

self.addEventListener('activate', event => {{
  event.waitUntil(
    caches.keys().then(keys => Promise.all(keys.filter(key => key !== CACHE_NAME).map(key => caches.delete(key)))).then(() => self.clients.claim())
  );
}});

self.addEventListener('fetch', event => {{
  const request = event.request;
  if (request.method !== 'GET') return;
  if (request.mode === 'navigate') {{
    event.respondWith(fetch(request).then(response => {{
      const copy = response.clone();
      caches.open(CACHE_NAME).then(cache => cache.put(request, copy));
      return response;
    }}).catch(() => caches.match(request).then(cached => cached || caches.match('{offline_url}'))));
    return;
  }}
  event.respondWith(caches.match(request).then(cached => cached || fetch(request).then(response => {{
    const copy = response.clone();
    caches.open(CACHE_NAME).then(cache => cache.put(request, copy));
    return response;
  }})));
}});
"#,
        cache_name = cache_name,
        urls_json = urls_json,
        offline_url = with_base(site, "offline.html")
    )
}

fn generate_offline_html(site: &BuiltSiteView) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Offline | {title}</title>
<style>body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;margin:0;min-height:100vh;display:grid;place-items:center;background:#fff;color:#1a1a2e}}main{{max-width:36rem;padding:2rem}}h1{{font-size:1.5rem}}</style>
</head>
<body><main><h1>Offline</h1><p>This page is not available offline yet.</p></main></body>
</html>
"#,
        lang = html_escape(&site.config.lang),
        title = html_escape(&site.config.title)
    )
}

fn with_base(site: &BuiltSiteView, path: &str) -> String {
    let base = site.config.base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    if base.is_empty() {
        format!("/{path}")
    } else {
        format!("{base}/{path}")
    }
}

fn css_filename(site: &BuiltSiteView) -> String {
    if site.config.asset_fingerprint {
        format!(
            "style.{:08x}.css",
            crate::util::fnv1a(CSS_CONTENT.as_bytes())
        )
    } else {
        "style.css".to_string()
    }
}

fn js_filename(site: &BuiltSiteView) -> String {
    if site.config.asset_fingerprint {
        format!("main.{:08x}.js", crate::util::fnv1a(JS_CONTENT.as_bytes()))
    } else {
        "main.js".to_string()
    }
}

fn sanitize_cache_key(s: &str) -> String {
    s.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::BuiltSiteView;
    use novel_shared::config::SiteConfig;

    #[test]
    fn disabled_pwa_generates_no_files() {
        let config = SiteConfig::default();
        let nav = Vec::new();
        let sidebar = std::collections::HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &[],
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        assert!(PwaPlugin.on_build_complete(&site).is_empty());
    }

    #[test]
    fn enabled_pwa_generates_manifest_and_service_worker() {
        let mut config = SiteConfig::default();
        config.pwa.enabled = true;
        let nav = Vec::new();
        let sidebar = std::collections::HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &[],
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        let outputs = PwaPlugin.on_build_complete(&site);
        let names: Vec<&str> = outputs.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(
            names,
            vec!["manifest.webmanifest", "service-worker.js", "offline.html"]
        );
    }
}
