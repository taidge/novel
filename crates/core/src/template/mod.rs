mod minijinja_engine;
#[cfg(feature = "tera")]
mod tera_engine;
#[cfg(feature = "handlebars")]
mod handlebars_engine;

use anyhow::Result;
use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, SidebarItem, TocItem};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::plugin::Plugin;
use crate::{CSS_CONTENT, JS_CONTENT};

/// Serializable render context passed to all template engines.
#[derive(Serialize)]
pub struct RenderContext<'a> {
    pub site: &'a SiteConfig,
    pub page: Option<&'a PageData>,
    pub nav: &'a [NavItem],
    pub sidebar: &'a [SidebarItem],
    pub toc: &'a [TocItem],
    pub edit_url: Option<String>,
    pub edit_link_text: &'a str,
    pub last_updated_text: &'a str,
    pub theme_css_overrides: Option<String>,
    pub custom_css_content: Option<String>,
    pub asset_css: &'a str,
    pub asset_js: &'a str,
    pub not_found_title: &'a str,
    pub not_found_message: &'a str,
}

/// Trait for pluggable template engine backends.
///
/// Implement this trait to add support for a new template language.
/// The renderer receives a template name and a serializable context,
/// and must return the rendered HTML string.
pub trait TemplateRenderer: Send + Sync {
    fn render(&self, template_name: &str, ctx: &RenderContext) -> Result<String>;
}

/// HTML template engine — public API wrapping a pluggable [`TemplateRenderer`].
pub struct TemplateEngine {
    renderer: Box<dyn TemplateRenderer>,
    project_root: Option<PathBuf>,
    css_filename: String,
    js_filename: String,
}

impl TemplateEngine {
    /// Create a new template engine.
    ///
    /// The `config.template_engine` field selects the backend:
    /// - `"minijinja"` (default) — Jinja2-compatible
    /// - `"tera"` — requires the `tera` feature
    /// - `"handlebars"` — requires the `handlebars` feature
    pub fn new(
        project_root: Option<&Path>,
        plugins: &[Box<dyn Plugin>],
        config: &SiteConfig,
    ) -> Result<Self> {
        let renderer: Box<dyn TemplateRenderer> = match config.template_engine.as_str() {
            "minijinja" | "" => {
                Box::new(minijinja_engine::MiniJinjaRenderer::new(project_root, plugins)?)
            }
            #[cfg(feature = "tera")]
            "tera" => Box::new(tera_engine::TeraRenderer::new(project_root)?),
            #[cfg(feature = "handlebars")]
            "handlebars" => {
                Box::new(handlebars_engine::HandlebarsRenderer::new(project_root)?)
            }
            other => {
                #[allow(unused_mut)]
                let mut supported = vec!["minijinja"];
                #[cfg(feature = "tera")]
                supported.push("tera");
                #[cfg(feature = "handlebars")]
                supported.push("handlebars");
                anyhow::bail!(
                    "Unknown template engine '{}'. Supported: {}",
                    other,
                    supported.join(", ")
                );
            }
        };

        let (css_filename, js_filename) = if config.asset_fingerprint {
            let css_hash = format!("{:08x}", crate::simple_hash(CSS_CONTENT.as_bytes()));
            let js_hash = format!("{:08x}", crate::simple_hash(JS_CONTENT.as_bytes()));
            (
                format!("style.{}.css", css_hash),
                format!("main.{}.js", js_hash),
            )
        } else {
            ("style.css".to_string(), "main.js".to_string())
        };

        Ok(Self {
            renderer,
            project_root: project_root.map(|p| p.to_path_buf()),
            css_filename,
            js_filename,
        })
    }

    // -- context helpers (engine-agnostic) ----------------------------------

    fn css_overrides(config: &SiteConfig) -> Option<String> {
        if config.theme.colors.is_empty() {
            return None;
        }
        let css: String = config
            .theme
            .colors
            .iter()
            .map(|(k, v)| format!("--{}: {};", k, v))
            .collect::<Vec<_>>()
            .join(" ");
        Some(css)
    }

    fn custom_css_content(&self, config: &SiteConfig) -> Option<String> {
        let css_path = config.theme.custom_css.as_deref()?;
        let full_path = match &self.project_root {
            Some(root) => root.join(css_path),
            None => PathBuf::from(css_path),
        };
        std::fs::read_to_string(full_path).ok()
    }

    fn base_context<'a>(
        &'a self,
        config: &'a SiteConfig,
        nav: &'a [NavItem],
    ) -> RenderContext<'a> {
        RenderContext {
            site: config,
            page: None,
            nav,
            sidebar: &[],
            toc: &[],
            edit_url: None,
            edit_link_text: config
                .theme
                .edit_link_text
                .as_deref()
                .unwrap_or("Edit this page"),
            last_updated_text: config
                .theme
                .last_updated_text
                .as_deref()
                .unwrap_or("Last updated"),
            theme_css_overrides: Self::css_overrides(config),
            custom_css_content: self.custom_css_content(config),
            asset_css: &self.css_filename,
            asset_js: &self.js_filename,
            not_found_title: config
                .theme
                .not_found_title
                .as_deref()
                .unwrap_or("404"),
            not_found_message: config
                .theme
                .not_found_message
                .as_deref()
                .unwrap_or("Page not found"),
        }
    }

    // -- public render methods (unchanged API) ------------------------------

    pub fn render_doc(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
        sidebar: &[SidebarItem],
    ) -> Result<String> {
        let edit_url = config.theme.edit_link.as_ref().map(|pattern| {
            format!(
                "{}{}",
                pattern.trim_end_matches('/'),
                page.route.relative_path
            )
        });

        let mut ctx = self.base_context(config, nav);
        ctx.page = Some(page);
        ctx.sidebar = sidebar;
        ctx.toc = &page.toc;
        ctx.edit_url = edit_url;
        self.renderer.render("doc.html", &ctx)
    }

    pub fn render_home(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
    ) -> Result<String> {
        let mut ctx = self.base_context(config, nav);
        ctx.page = Some(page);
        self.renderer.render("home.html", &ctx)
    }

    pub fn render_page_layout(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
    ) -> Result<String> {
        let mut ctx = self.base_context(config, nav);
        ctx.page = Some(page);
        self.renderer.render("page.html", &ctx)
    }

    pub fn render_blog(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
    ) -> Result<String> {
        let mut ctx = self.base_context(config, nav);
        ctx.page = Some(page);
        self.renderer.render("blog.html", &ctx)
    }

    pub fn render_404(&self, config: &SiteConfig, nav: &[NavItem]) -> Result<String> {
        let ctx = self.base_context(config, nav);
        self.renderer.render("404.html", &ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestProject {
        root: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "novel-template-test-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("failed to create test project root");
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write_template(&self, name: &str, contents: &str) {
            let path = self.root.join("templates").join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("failed to create test template directory");
            }
            fs::write(path, contents).expect("failed to write test template");
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn prefers_templates_from_project_folder() {
        let project = TestProject::new();
        project.write_template("404.html", "custom 404");

        let config = SiteConfig::default();
        let engine = TemplateEngine::new(Some(project.root()), &[], &config)
            .expect("template engine should load");
        let rendered = engine
            .render_404(&config, &[])
            .expect("404 page should render");

        assert_eq!(rendered, "custom 404");
    }

    #[test]
    fn falls_back_to_embedded_templates_when_missing_from_project_folder() {
        let project = TestProject::new();
        project.write_template(
            "base.html",
            "<html><body>CUSTOM_BASE{% block content %}{% endblock %}</body></html>",
        );

        let config = SiteConfig::default();
        let engine = TemplateEngine::new(Some(project.root()), &[], &config)
            .expect("template engine should load");
        let rendered = engine
            .render_404(&config, &[])
            .expect("404 page should render");

        assert!(rendered.contains("CUSTOM_BASE"));
        assert!(rendered.contains("Page not found"));
    }
}
