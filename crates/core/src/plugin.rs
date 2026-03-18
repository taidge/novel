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

    /// Transform raw markdown before it is parsed.
    fn transform_markdown(&self, markdown: String, _path: &Path) -> String {
        markdown
    }

    /// Transform rendered HTML after markdown parsing.
    fn transform_html(&self, html: String, _page: &PageData) -> String {
        html
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
