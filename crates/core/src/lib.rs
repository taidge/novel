pub mod builder;
pub mod cache;
pub mod content;
pub mod data;
pub mod error;
pub mod markdown;
pub mod pagination;
pub mod taxonomy;
pub mod plugin;
pub mod plugins;
pub mod routing;
pub mod search;
#[cfg(feature = "salvo")]
pub mod serve;
pub mod sidebar;
pub mod source;
pub mod template;
pub mod typst_processor;

use anyhow::Result;
use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, PageType, SidebarItem};
use plugin::{BuiltSiteView, Plugin};
use rust_embed::Embed;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tracing::info;

use builder::{build_pages, get_git_last_updated, route_to_file_path};
use content::Collection;
use pagination::{PageRef, Paginator};
use search::generate_search_index;
use source::{DirSource, DocsSource, EmbedSource};
use template::{TemplateEngine, TermSummary};

/// Static asset contents
pub const CSS_CONTENT: &str = include_str!("../assets/style.css");
pub const JS_CONTENT: &str = include_str!("../assets/main.js");

// ---------------------------------------------------------------------------
// Novel -- trait
// ---------------------------------------------------------------------------

/// Common interface for Novel documentation site builders.
///
/// Both [`DirNovel`] (filesystem-backed) and [`EmbedNovel`] (rust-embed-backed)
/// implement this trait.
pub trait Novel {
    /// Access the current site configuration.
    fn current_config(&self) -> &SiteConfig;

    /// Build the site: parse all markdown, generate navigation, and prepare
    /// everything needed to render pages.
    fn build(&mut self) -> Result<BuiltSite>;
}

// ---------------------------------------------------------------------------
// DirNovel -- filesystem-backed builder
// ---------------------------------------------------------------------------

/// Builder for constructing a documentation site from a filesystem directory.
///
/// # Usage
///
/// **Standalone (with `novel.toml` or `novel.kdl`)**
/// ```no_run
/// use novel_core::{Novel, DirNovel};
/// let site = DirNovel::load(".")?.build()?;
/// site.write_to("dist")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// **As a library**
/// ```no_run
/// use novel_core::{Novel, DirNovel};
/// let site = DirNovel::new("docs")
///     .title("My API Docs")
///     .build()?;
///
/// if let Some(page) = site.page("/guide/intro") {
///     let html = site.render_page(page)?;
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct DirNovel {
    config: SiteConfig,
    project_root: PathBuf,
    plugins: Vec<Box<dyn Plugin>>,
}

impl DirNovel {
    /// Create from a docs directory with default config.
    ///
    /// `docs_dir` is the directory containing `.md` files.
    /// The project root is set to its parent (or `.` if it has none).
    pub fn new(docs_dir: impl AsRef<Path>) -> Self {
        let docs_dir = docs_dir.as_ref();
        let project_root = docs_dir.parent().unwrap_or(Path::new(".")).to_path_buf();
        let root_rel = docs_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "docs".to_string());

        let mut config = SiteConfig::default();
        config.root = root_rel;
        Self {
            config,
            project_root,
            plugins: Vec::new(),
        }
    }

    /// Load from a project root that contains `novel.toml` or `novel.kdl`.
    pub fn load(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let config = SiteConfig::load(&project_root)?;
        Ok(Self {
            config,
            project_root,
            plugins: Vec::new(),
        })
    }

    // -- builder setters ----------------------------------------------------

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.config.description = desc.into();
        self
    }

    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.config.base = base.into();
        self
    }

    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    pub fn out_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.out_dir = dir.into();
        self
    }

    pub fn site_url(mut self, url: impl Into<String>) -> Self {
        self.config.site_url = Some(url.into());
        self
    }

    /// Replace the entire theme config.
    pub fn theme(mut self, theme: novel_shared::config::ThemeConfig) -> Self {
        self.config.theme = theme;
        self
    }

    /// Mutate the theme config in place via a closure.
    pub fn with_theme(mut self, f: impl FnOnce(&mut novel_shared::config::ThemeConfig)) -> Self {
        f(&mut self.config.theme);
        self
    }

    /// Replace the entire site config.
    pub fn config(mut self, config: SiteConfig) -> Self {
        self.config = config;
        self
    }

    /// Mutable access to the site config (for in-place tweaks before build).
    pub fn config_mut(&mut self) -> &mut SiteConfig {
        &mut self.config
    }

    /// Override the project root (affects file-embed resolution etc.).
    pub fn project_root(mut self, root: impl AsRef<Path>) -> Self {
        self.project_root = root.as_ref().to_path_buf();
        self
    }

    /// Register a plugin.
    pub fn plugin(mut self, p: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(p));
        self
    }
}

impl Novel for DirNovel {
    fn current_config(&self) -> &SiteConfig {
        &self.config
    }

    fn build(&mut self) -> Result<BuiltSite> {
        // Let plugins mutate config
        for p in &self.plugins {
            p.on_config(&mut self.config);
        }

        // Plugin: configure from [plugins.<name>] config
        for p in self.plugins.iter_mut() {
            let val = self.config.plugins.get(p.name());
            p.configure(val);
        }

        let docs_root = self.config.docs_root(&self.project_root);
        if !docs_root.exists() {
            anyhow::bail!(
                "Docs root directory does not exist: {}",
                docs_root.display()
            );
        }

        let mut all_pages = Vec::new();
        let mut merged_nav = Vec::new();
        let mut merged_sidebar = HashMap::new();

        if let Some(ref i18n) = self.config.i18n {
            // i18n multi-locale build
            for locale in &i18n.locales {
                let locale_docs = docs_root.join(&locale.dir);
                if !locale_docs.exists() {
                    tracing::warn!(
                        "Locale docs directory does not exist: {}",
                        locale_docs.display()
                    );
                    continue;
                }

                let source = DirSource::new(locale_docs.clone());
                let mut locale_config = self.config.clone();

                // Apply locale-specific overrides
                if let Some(ref title) = locale.title {
                    locale_config.title = title.clone();
                }
                if let Some(ref desc) = locale.description {
                    locale_config.description = desc.clone();
                }
                if let Some(ref theme_overrides) = locale.theme {
                    if let Some(ref nav) = theme_overrides.nav {
                        locale_config.theme.nav = nav.clone();
                    }
                    if let Some(ref sidebar) = theme_overrides.sidebar {
                        locale_config.theme.sidebar = sidebar.clone();
                    }
                    if let Some(ref footer) = theme_overrides.footer {
                        locale_config.theme.footer = Some(footer.clone());
                    }
                    if let Some(ref text) = theme_overrides.edit_link_text {
                        locale_config.theme.edit_link_text = Some(text.clone());
                    }
                    if let Some(ref text) = theme_overrides.last_updated_text {
                        locale_config.theme.last_updated_text = Some(text.clone());
                    }
                }

                let mut br = build_pages(
                    &source,
                    &locale_config,
                    Some(&self.project_root),
                    &self.plugins,
                )?;

                // Prefix all routes with /<locale.code>/
                let prefix = format!("/{}", locale.code);
                for page in &mut br.pages {
                    if page.route.route_path == "/" {
                        page.route.route_path = format!("{}/", prefix);
                    } else {
                        page.route.route_path =
                            format!("{}{}", prefix, page.route.route_path);
                    }
                    page.route.locale = Some(locale.code.clone());

                    // Update prev/next links
                    if let Some(ref mut prev) = page.prev_page {
                        if prev.link == "/" {
                            prev.link = format!("{}/", prefix);
                        } else {
                            prev.link = format!("{}{}", prefix, prev.link);
                        }
                    }
                    if let Some(ref mut next) = page.next_page {
                        if next.link == "/" {
                            next.link = format!("{}/", prefix);
                        } else {
                            next.link = format!("{}{}", prefix, next.link);
                        }
                    }

                    // Update breadcrumb links
                    for crumb in &mut page.breadcrumbs {
                        if crumb.link == "/" {
                            crumb.link = format!("{}/", prefix);
                        } else {
                            crumb.link = format!("{}{}", prefix, crumb.link);
                        }
                    }
                }

                // Git timestamps
                if self.config.theme.last_updated {
                    for page in &mut br.pages {
                        let file_path = locale_docs.join(&page.route.relative_path);
                        page.last_updated = get_git_last_updated(&file_path);
                    }
                }

                // Prefix sidebar keys
                let locale_sidebar: HashMap<String, Vec<SidebarItem>> = br
                    .sidebar
                    .into_iter()
                    .map(|(k, v)| (format!("{}{}", prefix, k), v))
                    .collect();

                all_pages.extend(br.pages);
                if locale.code == i18n.default_locale {
                    merged_nav = br.nav;
                }
                merged_sidebar.extend(locale_sidebar);
            }
        } else {
            // Standard single-locale build
            let source = DirSource::new(docs_root.clone());
            let mut br = build_pages(
                &source,
                &self.config,
                Some(&self.project_root),
                &self.plugins,
            )?;

            // Git timestamps (DirNovel-specific)
            if self.config.theme.last_updated {
                for page in &mut br.pages {
                    let file_path = docs_root.join(&page.route.relative_path);
                    page.last_updated = get_git_last_updated(&file_path);
                }
            }

            all_pages = br.pages;
            merged_nav = br.nav;
            merged_sidebar = br.sidebar;
        }

        let source = DirSource::new(docs_root.clone());

        let engine = TemplateEngine::new(
            Some(&self.project_root),
            &self.plugins,
            &self.config,
        )?;

        // Take plugins out of self to move into BuiltSite
        let plugins = std::mem::take(&mut self.plugins);

        // General-SSG post-processing: discover collections, filter, build
        // list/term pages.
        let collections = content::discover_collections(&docs_root).unwrap_or_default();
        let (all_pages, list_pages, terms_pages) =
            post_process_general(&self.config, all_pages, &collections);

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: Some(self.project_root.clone()),
            pages: all_pages,
            nav: merged_nav,
            sidebar: merged_sidebar,
            engine,
            source: Box::new(source),
            plugins,
            list_pages,
            terms_pages,
        })
    }
}

// ---------------------------------------------------------------------------
// EmbedNovel -- rust-embed-backed builder
// ---------------------------------------------------------------------------

/// Builder for constructing a documentation site from embedded assets.
///
/// # Usage
///
/// ```ignore
/// use novel_core::{Novel, EmbedNovel};
/// use rust_embed::Embed;
///
/// #[derive(Embed)]
/// #[folder = "docs/"]
/// struct Docs;
///
/// let site = EmbedNovel::<Docs>::new()
///     .title("My Embedded Docs")
///     .build()?;
/// ```
pub struct EmbedNovel<E: Embed> {
    config: SiteConfig,
    plugins: Vec<Box<dyn Plugin>>,
    _marker: PhantomData<E>,
}

impl<E: Embed + Send + Sync + 'static> EmbedNovel<E> {
    pub fn new() -> Self {
        Self {
            config: SiteConfig::default(),
            plugins: Vec::new(),
            _marker: PhantomData,
        }
    }

    // -- builder setters ----------------------------------------------------

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.config.description = desc.into();
        self
    }

    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.config.base = base.into();
        self
    }

    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    pub fn out_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.out_dir = dir.into();
        self
    }

    pub fn site_url(mut self, url: impl Into<String>) -> Self {
        self.config.site_url = Some(url.into());
        self
    }

    /// Replace the entire theme config.
    pub fn theme(mut self, theme: novel_shared::config::ThemeConfig) -> Self {
        self.config.theme = theme;
        self
    }

    /// Mutate the theme config in place via a closure.
    pub fn with_theme(mut self, f: impl FnOnce(&mut novel_shared::config::ThemeConfig)) -> Self {
        f(&mut self.config.theme);
        self
    }

    /// Replace the entire site config.
    pub fn config(mut self, config: SiteConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a plugin.
    pub fn plugin(mut self, p: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(p));
        self
    }
}

impl<E: Embed + Send + Sync + 'static> Default for EmbedNovel<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Embed + Send + Sync + 'static> Novel for EmbedNovel<E> {
    fn current_config(&self) -> &SiteConfig {
        &self.config
    }

    fn build(&mut self) -> Result<BuiltSite> {
        // Let plugins mutate config
        for p in &self.plugins {
            p.on_config(&mut self.config);
        }

        // Plugin: configure from [plugins.<name>] config
        for p in self.plugins.iter_mut() {
            let val = self.config.plugins.get(p.name());
            p.configure(val);
        }

        let source = EmbedSource::<E>::new();
        let br = build_pages(&source, &self.config, None, &self.plugins)?;
        let engine = TemplateEngine::new(None, &self.plugins, &self.config)?;

        let plugins = std::mem::take(&mut self.plugins);

        // EmbedNovel currently does not support filesystem _collection.toml
        // discovery; collections come up empty so this is a no-op pass that
        // still applies draft/future/expiry filtering.
        let collections = HashMap::new();
        let (pages, list_pages, terms_pages) =
            post_process_general(&self.config, br.pages, &collections);

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: None,
            pages,
            nav: br.nav,
            sidebar: br.sidebar,
            engine,
            source: Box::new(source),
            plugins,
            list_pages,
            terms_pages,
        })
    }
}

// ---------------------------------------------------------------------------
// BuiltSite -- the built result, usable for rendering / writing / serving
// ---------------------------------------------------------------------------

/// A fully-built documentation site.
///
/// Holds all page data, navigation, sidebar, and the template engine so that
/// pages can be rendered on demand or written to disk.
/// A virtual list page (collection/term paginated list) generated by the
/// general-SSG pipeline. Stored alongside content pages on `BuiltSite`.
struct ListPage {
    route_path: String,
    title: String,
    paginator: Paginator,
}

/// A taxonomy overview (terms cloud) page (e.g. /tags/).
struct TermsPage {
    route_path: String,
    title: String,
    terms: Vec<TermSummary>,
}

pub struct BuiltSite {
    config: SiteConfig,
    project_root: Option<PathBuf>,
    pages: Vec<PageData>,
    nav: Vec<NavItem>,
    sidebar: HashMap<String, Vec<SidebarItem>>,
    engine: TemplateEngine,
    source: Box<dyn DocsSource>,
    plugins: Vec<Box<dyn Plugin>>,
    list_pages: Vec<ListPage>,
    terms_pages: Vec<TermsPage>,
}

impl BuiltSite {
    // -- accessors ----------------------------------------------------------

    /// All built pages.
    pub fn pages(&self) -> &[PageData] {
        &self.pages
    }

    /// Find a page by its route path (e.g. `"/guide/intro"`).
    pub fn page(&self, route_path: &str) -> Option<&PageData> {
        self.pages.iter().find(|p| p.route.route_path == route_path)
    }

    /// Navigation items.
    pub fn nav(&self) -> &[NavItem] {
        &self.nav
    }

    /// Sidebar map (path-prefix -> items).
    pub fn sidebar(&self) -> &HashMap<String, Vec<SidebarItem>> {
        &self.sidebar
    }

    /// The resolved site config.
    pub fn config(&self) -> &SiteConfig {
        &self.config
    }

    // -- rendering ----------------------------------------------------------

    /// Render a single page to a full HTML string.
    ///
    /// Layout dispatch:
    /// 1. `page_type: home` or `layout: "home"` → home template
    /// 2. `layout: "page"` → full-width page template (no sidebar/TOC)
    /// 3. `layout: "blog"` → centered blog template with date header
    /// 4. Everything else → doc template (sidebar + TOC)
    pub fn render_page(&self, page: &PageData) -> Result<String> {
        let is_home = matches!(page.frontmatter.page_type, Some(PageType::Home));
        let layout = page.frontmatter.layout.as_deref();

        if is_home || layout == Some("home") {
            self.engine.render_home(page, &self.config, &self.nav)
        } else if layout == Some("page") {
            self.engine
                .render_page_layout(page, &self.config, &self.nav)
        } else if layout == Some("blog") {
            self.engine.render_blog(page, &self.config, &self.nav)
        } else {
            let sidebar_key = find_sidebar_key(&page.route.route_path, &self.sidebar);
            let sidebar_items = sidebar_key
                .and_then(|k| self.sidebar.get(&k))
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            self.engine
                .render_doc(page, &self.config, &self.nav, sidebar_items)
        }
    }

    /// Render the 404 page.
    pub fn render_404(&self) -> Result<String> {
        self.engine.render_404(&self.config, &self.nav)
    }

    // -- static assets ------------------------------------------------------

    /// CSS stylesheet content.
    pub fn css(&self) -> &'static str {
        CSS_CONTENT
    }

    /// JavaScript content.
    pub fn js(&self) -> &'static str {
        JS_CONTENT
    }

    // -- generated data -----------------------------------------------------

    /// Search index serialised to JSON.
    pub fn search_index_json(&self) -> Result<String> {
        let idx = generate_search_index(&self.pages);
        Ok(serde_json::to_string(&idx)?)
    }

    /// Sitemap XML (returns `None` when `site_url` is not configured).
    pub fn sitemap_xml(&self) -> Option<String> {
        let view = self.as_view();
        plugins::sitemap::generate_sitemap_xml(&view)
    }

    /// Atom/RSS feed XML (returns `None` when `site_url` is not configured).
    pub fn feed_xml(&self) -> Option<String> {
        let view = self.as_view();
        plugins::feed::generate_feed_xml(&view)
    }

    /// Create a `BuiltSiteView` for plugin consumption.
    fn as_view(&self) -> BuiltSiteView<'_> {
        BuiltSiteView {
            config: &self.config,
            pages: &self.pages,
            nav: &self.nav,
            sidebar: &self.sidebar,
            project_root: self.project_root.as_deref(),
        }
    }

    // -- write to disk ------------------------------------------------------

    /// Write the complete static site to `dir`.
    ///
    /// Creates `dir` if it does not exist, **removes** existing contents first.
    pub fn write_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        let output_dir = dir.as_ref();

        // Clean & create
        if output_dir.exists() {
            std::fs::remove_dir_all(output_dir)?;
        }
        std::fs::create_dir_all(output_dir)?;

        // Assets (with optional fingerprinting)
        let assets_dir = output_dir.join("assets");
        std::fs::create_dir_all(&assets_dir)?;

        let (css_filename, js_filename) = if self.config.asset_fingerprint {
            let css_hash = &format!("{:08x}", simple_hash(CSS_CONTENT.as_bytes()));
            let js_hash = &format!("{:08x}", simple_hash(JS_CONTENT.as_bytes()));
            let css_name = format!("style.{}.css", css_hash);
            let js_name = format!("main.{}.js", js_hash);
            (css_name, js_name)
        } else {
            ("style.css".to_string(), "main.js".to_string())
        };

        std::fs::write(assets_dir.join(&css_filename), CSS_CONTENT)?;
        std::fs::write(assets_dir.join(&js_filename), JS_CONTENT)?;

        // Pages
        for page in &self.pages {
            let html = self.render_page(page)?;
            let out_path = route_to_file_path(output_dir, &page.route.route_path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, html)?;
            info!("Generated: {}", out_path.display());
        }

        // List pages (collections + taxonomy terms, paginated)
        for lp in &self.list_pages {
            let html = self.engine.render_list(
                lp.title.clone(),
                &lp.paginator,
                &self.config,
                &self.nav,
            )?;
            let out_path = route_to_file_path(output_dir, &lp.route_path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, html)?;
            info!("Generated list: {}", out_path.display());
        }

        // Taxonomy overview pages
        for tp in &self.terms_pages {
            let html = self.engine.render_terms(
                tp.title.clone(),
                &tp.terms,
                &self.config,
                &self.nav,
            )?;
            let out_path = route_to_file_path(output_dir, &tp.route_path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, html)?;
            info!("Generated terms: {}", out_path.display());
        }

        // 404
        let html_404 = self.render_404()?;
        std::fs::write(output_dir.join("404.html"), html_404)?;

        // Run plugin on_build_complete hooks
        let view = self.as_view();
        for plugin in &self.plugins {
            let files = plugin.on_build_complete(&view);
            for (rel_path, contents) in files {
                let dest = output_dir.join(&rel_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, contents)?;
                info!("Plugin '{}' generated: {}", plugin.name(), rel_path);
            }
        }

        // i18n root redirect
        if let Some(ref i18n) = self.config.i18n {
            let default_locale = &i18n.default_locale;
            let redirect_html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<script>
var lang = navigator.language || navigator.userLanguage || '';
var locales = [{}];
var match = locales.find(function(l) {{ return lang.toLowerCase().startsWith(l); }});
window.location.replace('/' + (match || '{}') + '/');
</script>
<meta http-equiv="refresh" content="0; url=/{default_locale}/">
</head>
<body><p>Redirecting...</p></body>
</html>"#,
                i18n.locales
                    .iter()
                    .map(|l| format!("'{}'", l.code))
                    .collect::<Vec<_>>()
                    .join(","),
                default_locale,
            );
            std::fs::write(output_dir.join("index.html"), redirect_html)?;
        }

        // Static assets from docs source
        self.copy_static_assets(output_dir)?;

        info!("Build complete! Output: {}", output_dir.display());
        Ok(())
    }

    /// Convenience: write to the configured `out_dir` relative to `project_root`.
    pub fn write_to_default_output(&self) -> Result<()> {
        let dir = match &self.project_root {
            Some(root) => self.config.output_dir(root),
            None => PathBuf::from(&self.config.out_dir),
        };
        self.write_to(dir)
    }

    /// Copy non-content, non-meta static assets from the docs source.
    fn copy_static_assets(&self, output_dir: &Path) -> Result<()> {
        for file_path in self.source.list_files() {
            if file_path.ends_with(".md") || file_path.ends_with(".typ") {
                continue;
            }
            let file_name = Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if file_name == "_meta.json" {
                continue;
            }

            let data = self.source.read_bytes(&file_path)?;
            let dest = output_dir.join(&file_path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(dest, data)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Post-process pages: assign collection names, filter drafts/future/expiry,
/// build per-collection paginated list pages and taxonomy term pages.
fn post_process_general(
    config: &SiteConfig,
    pages: Vec<PageData>,
    collections: &HashMap<String, Collection>,
) -> (Vec<PageData>, Vec<ListPage>, Vec<TermsPage>) {
    use slug::slugify;

    // 1. Assign collection name and re-route entries with collection layout.
    let mut pages: Vec<PageData> = pages
        .into_iter()
        .map(|mut p| {
            if let Some(col) = content::collection_for_page(&p.route.relative_path, collections)
            {
                p.collection = Some(col.clone());
                if p.frontmatter.layout.is_none() {
                    if let Some(c) = collections.get(&col) {
                        p.frontmatter.layout = Some(c.config.layout.clone());
                    }
                }
            }
            p
        })
        .collect();

    // 2. Drop drafts / future / expired (and their would-be index pages).
    let today = content::today_string();
    pages = content::filter_pages(
        pages,
        config.content.drafts,
        config.content.future,
        &today,
    );

    // 3. Per-collection: build paginated list pages.
    let mut list_pages = Vec::new();
    for (name, coll) in collections {
        if !coll.config.publish {
            continue;
        }
        // Gather entries for this collection (skip the collection's own index page if any)
        let mut entries: Vec<&PageData> = pages
            .iter()
            .filter(|p| p.collection.as_deref() == Some(name.as_str()))
            .filter(|p| {
                // Skip the index page itself (route ends in /<name>/)
                let r = &p.route.route_path;
                r.trim_end_matches('/') != format!("/{}", name).trim_end_matches('/')
            })
            .collect();
        content::sort_collection_entries(&mut entries, &coll.config);

        let items: Vec<PageRef> = entries
            .iter()
            .map(|p| PageRef {
                title: p.title.clone(),
                link: p.route.route_path.clone(),
                date: p.date.clone(),
                summary_html: p.summary_html.clone().or_else(|| {
                    if !p.description.is_empty() {
                        Some(format!("<p>{}</p>", html_escape(&p.description)))
                    } else {
                        None
                    }
                }),
            })
            .collect();

        let base_route = format!("/{}/", name);
        let per = if coll.config.paginate_by == 0 { items.len().max(1) } else { coll.config.paginate_by };
        let paginators = pagination::paginate(
            &base_route,
            items,
            per,
            &config.pagination.page_path,
            config.pagination.first_page_in_root,
        );
        let title = coll.config.layout.clone(); // placeholder; replaced below
        let _ = title;
        for paginator in paginators {
            list_pages.push(ListPage {
                route_path: paginator.route_path.clone(),
                title: capitalize(name),
                paginator,
            });
        }
    }

    // 4. Taxonomies: build inverted index, term pages, overview pages.
    let mut terms_pages = Vec::new();
    let tax_set = taxonomy::build(&pages, &config.taxonomies);
    for (key, tax_cfg) in &config.taxonomies {
        let Some(idx) = tax_set.by_key.get(key) else {
            continue;
        };

        // Term pages
        for (term, page_indices) in &idx.terms {
            let mut entries: Vec<&PageData> =
                page_indices.iter().map(|i| &pages[*i]).collect();
            // Sort: date desc by default
            entries.sort_by(|a, b| {
                let ad = a.date.as_deref().unwrap_or("");
                let bd = b.date.as_deref().unwrap_or("");
                bd.cmp(ad)
            });
            let items: Vec<PageRef> = entries
                .iter()
                .map(|p| PageRef {
                    title: p.title.clone(),
                    link: p.route.route_path.clone(),
                    date: p.date.clone(),
                    summary_html: p.summary_html.clone(),
                })
                .collect();
            let base_route = taxonomy::term_route(key, term, tax_cfg);
            let per = tax_cfg.paginate_by.unwrap_or(items.len().max(1));
            let paginators = pagination::paginate(
                &base_route,
                items,
                per,
                &config.pagination.page_path,
                config.pagination.first_page_in_root,
            );
            for paginator in paginators {
                list_pages.push(ListPage {
                    route_path: paginator.route_path.clone(),
                    title: format!("{}: {}", key, term),
                    paginator,
                });
            }
        }

        // Taxonomy overview page (terms cloud)
        let mut summaries: Vec<TermSummary> = idx
            .terms
            .iter()
            .map(|(term, ids)| TermSummary {
                name: term.clone(),
                slug: slugify(term),
                link: taxonomy::term_route(key, term, tax_cfg),
                count: ids.len(),
            })
            .collect();
        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        terms_pages.push(TermsPage {
            route_path: taxonomy::taxonomy_route(key),
            title: capitalize(key),
            terms: summaries,
        });
    }

    (pages, list_pages, terms_pages)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Simple non-cryptographic hash for asset fingerprinting (FNV-1a).
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn find_sidebar_key(
    route_path: &str,
    sidebar: &HashMap<String, Vec<SidebarItem>>,
) -> Option<String> {
    let mut best: Option<String> = None;
    let mut best_len = 0;
    for key in sidebar.keys() {
        if route_path.starts_with(key) && key.len() > best_len {
            best = Some(key.clone());
            best_len = key.len();
        }
    }
    best
}
