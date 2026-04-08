use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, SidebarItem};
use std::collections::HashMap;
use std::path::Path;

/// A view into the fully-built site, passed to plugins during `on_build_complete`.
pub struct BuiltSiteView<'a> {
    pub config: &'a SiteConfig,
    pub pages: &'a [PageData],
    pub nav: &'a [NavItem],
    pub sidebar: &'a HashMap<String, Vec<SidebarItem>>,
    pub project_root: Option<&'a Path>,
}

/// Extension point for Novel's build pipeline.
///
/// Implement this trait to hook into configuration, markdown/HTML transforms,
/// post-build file generation, and template helper registration.
pub trait Plugin: Send + Sync {
    /// Human-readable name (used in error messages, logging, etc.).
    fn name(&self) -> &str;

    /// Called before the build starts — mutate the site config if needed.
    fn on_config(&self, _config: &mut SiteConfig) {}

    /// Configure the plugin from the `[plugins.<name>]` section in config.
    fn configure(&mut self, _value: Option<&serde_json::Value>) {}

    /// Called before any pages are processed.
    fn on_pre_build(&self, _config: &SiteConfig) {}

    /// Transform raw markdown before it is parsed.
    fn transform_markdown(&self, markdown: String, _path: &Path) -> String {
        markdown
    }

    /// Transform rendered HTML after markdown parsing.
    fn transform_html(&self, html: String, _page: &PageData) -> String {
        html
    }

    /// Called after each page is built.
    fn on_page_built(&self, _page: &PageData) {}

    /// Transform the navigation items after they are generated.
    fn transform_nav(&self, nav: Vec<NavItem>, _site: &BuiltSiteView) -> Vec<NavItem> {
        nav
    }

    /// Transform the sidebar after it is generated.
    fn transform_sidebar(
        &self,
        sidebar: HashMap<String, Vec<SidebarItem>>,
        _site: &BuiltSiteView,
    ) -> HashMap<String, Vec<SidebarItem>> {
        sidebar
    }

    /// Generate virtual pages that don't correspond to source files.
    fn generate_pages(&self, _site: &BuiltSiteView) -> Vec<PageData> {
        vec![]
    }

    /// Generate additional output files after the build completes.
    ///
    /// Returns a list of `(relative_path, contents)` pairs that will be
    /// written into the output directory.
    fn on_build_complete(&self, _site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        vec![]
    }

    /// Register custom template helpers / filters / globals on the
    /// minijinja environment.
    fn register_template_helpers(&self, _env: &mut minijinja::Environment<'static>) {}

    /// Return custom container directives provided by this plugin.
    fn container_directives(&self) -> Vec<Box<dyn ContainerDirective>> {
        vec![]
    }
}

/// A custom container directive that plugins can provide.
///
/// Container directives are triggered by `::: name` blocks in markdown:
///
/// ```text
/// ::: my-directive Title
/// body content
/// :::
/// ```
pub trait ContainerDirective: Send + Sync {
    /// The keyword after `:::` that triggers this directive.
    fn name(&self) -> &str;

    /// Render the directive into HTML.
    fn render(&self, title: &str, body: &str) -> String;
}
