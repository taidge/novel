use crate::plugin::{BuiltSiteView, Plugin};
use crate::plugins::llms_txt::{is_public_page, join_base_path, page_source_markdown};
use crate::util::strip_html_tags;
use novel_shared::PageData;
use std::path::Path;

pub struct MarkdownMirrorPlugin;

impl Plugin for MarkdownMirrorPlugin {
    fn name(&self) -> &str {
        "markdown_mirror"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        if !site.config.markdown_mirror.enabled {
            return vec![];
        }

        site.pages
            .iter()
            .filter(|page| is_public_page(page))
            .map(|page| {
                let body = markdown_for_page(site, page);
                (
                    markdown_output_path(&page.route.route_path),
                    body.into_bytes(),
                )
            })
            .collect()
    }
}

pub fn markdown_url_for_route(site: &BuiltSiteView, route_path: &str) -> String {
    let output = markdown_output_path(route_path);
    let path = format!("/{}", output.replace('\\', "/"));
    if let Some(site_url) = site.config.site_url.as_deref() {
        format!("{}{}", site_url.trim_end_matches('/'), path)
    } else {
        join_base_path(&site.config.base, &path)
    }
}

fn markdown_for_page(site: &BuiltSiteView, page: &PageData) -> String {
    if let Some(source) = page_source_markdown(site, page) {
        if site.config.markdown_mirror.strip_frontmatter {
            return source;
        }
        if let Some(path) = crate::plugins::llms_txt::page_source_path(site, page)
            && let Ok(raw) = std::fs::read_to_string(path)
        {
            return raw;
        }
    }

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", page.title.trim()));
    let plain = strip_html_tags(&page.content_html);
    if !plain.trim().is_empty() {
        out.push_str(plain.trim());
        out.push('\n');
    }
    out
}

fn markdown_output_path(route_path: &str) -> String {
    if route_path == "/" {
        return "index.md".to_string();
    }

    let trimmed = route_path.trim_matches('/');
    if route_path.ends_with('/') {
        Path::new(trimmed)
            .join("index.md")
            .to_string_lossy()
            .replace('\\', "/")
    } else {
        format!("{trimmed}.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::BuiltSiteView;
    use novel_shared::config::SiteConfig;

    #[test]
    fn maps_routes_to_markdown_paths() {
        assert_eq!(markdown_output_path("/"), "index.md");
        assert_eq!(markdown_output_path("/guide/"), "guide/index.md");
        assert_eq!(markdown_output_path("/guide/intro"), "guide/intro.md");
        assert_eq!(markdown_output_path("/v1/guide/intro"), "v1/guide/intro.md");
    }

    #[test]
    fn markdown_url_respects_base_path() {
        let config = SiteConfig {
            base: "/docs/".to_string(),
            ..SiteConfig::default()
        };
        let nav = Vec::new();
        let sidebar = std::collections::HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &[],
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        assert_eq!(
            markdown_url_for_route(&site, "/guide/intro"),
            "/docs/guide/intro.md"
        );
    }

    #[test]
    fn frontmatter_stripping_helper_is_used_for_mirrors() {
        let raw = "---\ntitle: Test\n---\n\n# Test\n";
        assert_eq!(
            crate::plugins::llms_txt::markdown_without_frontmatter(raw),
            "# Test"
        );
    }
}
