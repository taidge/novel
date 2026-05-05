use crate::plugin::{BuiltSiteView, Plugin};
use crate::util::strip_html_tags;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use novel_shared::{PageData, PageType};
use std::path::{Path, PathBuf};

pub struct LlmsTxtPlugin;

impl Plugin for LlmsTxtPlugin {
    fn name(&self) -> &str {
        "llms_txt"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let llms_txt = generate_llms_txt(site).into_bytes();
        let llms_full_txt = generate_llms_full_txt(site).into_bytes();

        vec![
            ("llms.txt".to_string(), llms_txt.clone()),
            ("llms-full.txt".to_string(), llms_full_txt.clone()),
            (".well-known/llms.txt".to_string(), llms_txt),
            (".well-known/llms-full.txt".to_string(), llms_full_txt),
        ]
    }
}

/// Generate a compact, Markdown-formatted map of public documentation pages.
///
/// The format follows the emerging `llms.txt` convention: a site title, a
/// short quoted description, then structured Markdown links. We deliberately
/// exclude pages marked `noindex`, redirects, and the generated 404 page so
/// AI tools receive the same public surface as search engines and sitemaps.
pub fn generate_llms_txt(site: &BuiltSiteView) -> String {
    let pages = public_pages(site);
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", site.config.title.trim()));
    let site_desc = one_line(&site.config.description);
    if !site_desc.is_empty() {
        out.push_str(&format!("> {}\n\n", site_desc));
    }

    out.push_str("## Documentation\n\n");
    if pages.is_empty() {
        out.push_str("No public documentation pages were found.\n");
    } else {
        for page in pages {
            let title = escape_markdown_link_text(&page_title(site, page));
            let url = page_url(site, &page.route.route_path);
            let desc = page_description(page);
            if desc.is_empty() {
                out.push_str(&format!("- [{}]({})\n", title, url));
            } else {
                out.push_str(&format!("- [{}]({}): {}\n", title, url, desc));
            }
        }
    }

    out.push_str("\n## Full Context\n\n");
    out.push_str(&format!(
        "- [llms-full.txt]({}): Complete documentation text.\n",
        site_resource_url(site, "/llms-full.txt")
    ));

    out
}

/// Generate a single-file Markdown/plain-text context dump for AI tools.
///
/// For filesystem-backed builds this prefers the source Markdown with
/// frontmatter removed, preserving author-written code fences and prose. For
/// embedded or virtual pages where source Markdown is unavailable, it falls
/// back to stripped rendered HTML.
pub fn generate_llms_full_txt(site: &BuiltSiteView) -> String {
    let pages = public_pages(site);
    let mut out = String::new();

    out.push_str(&format!(
        "# Full documentation for {}\n\n",
        site.config.title.trim()
    ));
    let site_desc = one_line(&site.config.description);
    if !site_desc.is_empty() {
        out.push_str(&format!("> {}\n\n", site_desc));
    }
    out.push_str("This file contains the public documentation pages for AI tools.\n");

    for page in pages {
        out.push_str("\n---\n\n");
        out.push_str(&format!("## {}\n\n", page_title(site, page)));
        out.push_str(&format!(
            "URL: {}\n",
            page_url(site, &page.route.route_path)
        ));

        let desc = page_description(page);
        if !desc.is_empty() {
            out.push_str(&format!("Description: {}\n", desc));
        }
        if let Some(date) = page.date.as_ref().or(page.last_updated.as_ref()) {
            out.push_str(&format!("Updated: {}\n", one_line(date)));
        }

        let body = page_source_markdown(site, page)
            .unwrap_or_else(|| strip_html_tags(&page.content_html))
            .trim()
            .to_string();
        if !body.is_empty() {
            out.push('\n');
            out.push_str(&body);
            out.push('\n');
        }
    }

    out
}

fn public_pages<'a>(site: &'a BuiltSiteView<'a>) -> Vec<&'a PageData> {
    let mut pages: Vec<&PageData> = site
        .pages
        .iter()
        .filter(|page| is_public_page(page))
        .collect();
    pages.sort_by(|a, b| a.route.route_path.cmp(&b.route.route_path));
    pages
}

pub(crate) fn is_public_page(page: &PageData) -> bool {
    if page.frontmatter.noindex || page.frontmatter.redirect.is_some() {
        return false;
    }
    !matches!(page.frontmatter.page_type, Some(PageType::NotFound))
}

fn page_title(site: &BuiltSiteView, page: &PageData) -> String {
    if matches!(page.frontmatter.page_type, Some(PageType::Home)) {
        return page
            .frontmatter
            .hero
            .as_ref()
            .map(|hero| hero.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| site.config.title.trim())
            .to_string();
    }
    page.title.trim().to_string()
}

fn page_description(page: &PageData) -> String {
    let mut desc = one_line(&page.description);
    if desc.is_empty() {
        desc = page
            .frontmatter
            .hero
            .as_ref()
            .and_then(|hero| hero.tagline.as_deref())
            .map(one_line)
            .unwrap_or_default();
    }
    truncate(&desc, 300)
}

pub(crate) fn page_source_markdown(site: &BuiltSiteView, page: &PageData) -> Option<String> {
    let source_path = page_source_path(site, page)?;
    if source_path.extension().and_then(|e| e.to_str()) != Some("md") {
        return None;
    }

    let raw = std::fs::read_to_string(source_path).ok()?;
    Some(markdown_without_frontmatter(&raw).trim().to_string())
}

pub(crate) fn page_source_path(site: &BuiltSiteView, page: &PageData) -> Option<PathBuf> {
    if page.route.relative_path.is_empty() {
        return None;
    }

    let project_root = site.project_root?;
    let mut source_root = project_root.join(&site.config.root);

    if let (Some(versions), Some(version_code)) =
        (&site.config.versions, page.route.version.as_deref())
        && let Some(version) = versions
            .items
            .iter()
            .find(|version| version.code == version_code)
    {
        let dir = if version.dir.trim().is_empty() {
            &version.code
        } else {
            &version.dir
        };
        source_root.push(dir);
    }

    if let (Some(i18n), Some(locale_code)) = (&site.config.i18n, page.route.locale.as_deref()) {
        let locale = i18n
            .locales
            .iter()
            .find(|locale| locale.code == locale_code)?;
        source_root.push(&locale.dir);
    }

    Some(source_root.join(Path::new(&page.route.relative_path)))
}

pub(crate) fn markdown_without_frontmatter(raw: &str) -> String {
    let matter = Matter::<YAML>::new();
    matter
        .parse::<serde_json::Value>(raw)
        .map(|parsed| parsed.content)
        .unwrap_or_else(|_| raw.to_string())
}

pub(crate) fn page_url(site: &BuiltSiteView, route_path: &str) -> String {
    if let Some(site_url) = site.config.site_url.as_deref() {
        let base = site_url.trim_end_matches('/');
        if route_path == "/" {
            format!("{}/", base)
        } else {
            format!("{}{}", base, route_path)
        }
    } else {
        join_base_path(&site.config.base, route_path)
    }
}

fn site_resource_url(site: &BuiltSiteView, resource_path: &str) -> String {
    if let Some(site_url) = site.config.site_url.as_deref() {
        format!("{}{}", site_url.trim_end_matches('/'), resource_path)
    } else {
        join_base_path(&site.config.base, resource_path)
    }
}

pub(crate) fn join_base_path(base: &str, path: &str) -> String {
    let mut normalized_base = base.trim().trim_end_matches('/').to_string();
    if normalized_base.is_empty() || normalized_base == "/" {
        return path.to_string();
    }
    if !normalized_base.starts_with('/') {
        normalized_base.insert(0, '/');
    }
    if path == "/" {
        format!("{}/", normalized_base)
    } else {
        format!("{}{}", normalized_base, path)
    }
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let mut out: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        out.push_str("...");
    }
    out
}

fn escape_markdown_link_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use novel_shared::config::SiteConfig;
    use novel_shared::{FrontMatter, PageData, RouteMeta};
    use std::collections::HashMap;

    fn page(route_path: &str, relative_path: &str, title: &str) -> PageData {
        PageData {
            route: RouteMeta {
                route_path: route_path.to_string(),
                absolute_path: relative_path.to_string(),
                relative_path: relative_path.to_string(),
                page_name: title.to_string(),
                locale: None,
                version: None,
            },
            title: title.to_string(),
            description: String::new(),
            content_html: "<p>Rendered fallback</p>".to_string(),
            toc: Vec::new(),
            frontmatter: FrontMatter::default(),
            last_updated: None,
            prev_page: None,
            next_page: None,
            reading_time: None,
            word_count: None,
            breadcrumbs: Vec::new(),
            summary_html: None,
            collection: None,
            date: None,
            translations: Vec::new(),
            version_links: Vec::new(),
        }
    }

    #[test]
    fn llms_txt_lists_public_pages_and_skips_noindex() {
        let mut config = SiteConfig {
            title: "Docs".to_string(),
            description: "Project documentation".to_string(),
            site_url: Some("https://example.com".to_string()),
            ..SiteConfig::default()
        };
        config.base = "/".to_string();

        let mut visible = page("/guide/", "guide/index.md", "Guide");
        visible.description = "Start here".to_string();
        let mut hidden = page("/secret", "secret.md", "Secret");
        hidden.frontmatter.noindex = true;
        let pages = vec![hidden, visible];
        let nav = Vec::new();
        let sidebar = HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        let out = generate_llms_txt(&site);

        assert!(out.contains("# Docs"));
        assert!(out.contains("> Project documentation"));
        assert!(out.contains("- [Guide](https://example.com/guide/): Start here"));
        assert!(out.contains("[llms-full.txt](https://example.com/llms-full.txt)"));
        assert!(!out.contains("Secret"));
    }

    #[test]
    fn llms_txt_uses_base_path_without_site_url() {
        let config = SiteConfig {
            base: "/docs/".to_string(),
            ..SiteConfig::default()
        };
        let pages = vec![page("/guide/intro", "guide/intro.md", "Intro")];
        let nav = Vec::new();
        let sidebar = HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        let out = generate_llms_txt(&site);

        assert!(out.contains("- [Intro](/docs/guide/intro)"));
        assert!(out.contains("[llms-full.txt](/docs/llms-full.txt)"));
    }

    #[test]
    fn home_pages_use_hero_title_and_tagline() {
        let config = SiteConfig::default();
        let mut home = page("/", "index.md", "index");
        home.frontmatter.page_type = Some(PageType::Home);
        home.frontmatter.hero = Some(novel_shared::Hero {
            name: "Novel".to_string(),
            text: None,
            tagline: Some("Fast documentation".to_string()),
            actions: None,
            image: None,
        });
        let pages = vec![home];
        let nav = Vec::new();
        let sidebar = HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        let out = generate_llms_txt(&site);

        assert!(out.contains("- [Novel](/): Fast documentation"));
        assert!(!out.contains("[index]"));
    }

    #[test]
    fn llms_full_prefers_source_markdown_without_frontmatter() {
        let root = unique_temp_dir();
        let source = root.join("docs").join("guide");
        std::fs::create_dir_all(&source).expect("create test docs dir");
        std::fs::write(
            source.join("intro.md"),
            "---\ntitle: Intro\n---\n\n# Intro\n\nSource markdown body.",
        )
        .expect("write test markdown");

        let config = SiteConfig {
            root: "docs".to_string(),
            ..SiteConfig::default()
        };
        let pages = vec![page("/guide/intro", "guide/intro.md", "Intro")];
        let nav = Vec::new();
        let sidebar = HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root: Some(&root),
        };

        let out = generate_llms_full_txt(&site);
        let _ = std::fs::remove_dir_all(&root);

        assert!(out.contains("# Intro\n\nSource markdown body."));
        assert!(!out.contains("title: Intro"));
        assert!(!out.contains("Rendered fallback"));
    }

    #[test]
    fn plugin_writes_root_and_well_known_files() {
        let config = SiteConfig::default();
        let pages = vec![page("/", "index.md", "Home")];
        let nav = Vec::new();
        let sidebar = HashMap::new();
        let site = BuiltSiteView {
            config: &config,
            pages: &pages,
            nav: &nav,
            sidebar: &sidebar,
            project_root: None,
        };

        let outputs = LlmsTxtPlugin.on_build_complete(&site);
        let names: Vec<&str> = outputs.iter().map(|(name, _)| name.as_str()).collect();

        assert_eq!(
            names,
            vec![
                "llms.txt",
                "llms-full.txt",
                ".well-known/llms.txt",
                ".well-known/llms-full.txt"
            ]
        );
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!("novel-llms-test-{}-{nanos}", std::process::id()))
    }
}
