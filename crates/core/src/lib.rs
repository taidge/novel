// Public extension surface — things external crates reasonably want to
// import to build plugins, custom templates, or serve pages.
pub mod assets;
pub mod data;
pub mod error;
pub mod markdown;
pub mod plugin;
pub mod plugins;
#[cfg(feature = "salvo")]
pub mod serve;
pub mod template;

// Internal implementation details — the public API is [`Novel`], [`DirNovel`],
// [`EmbedNovel`], [`BuiltSite`] and anything re-exported from the modules
// above. Downstream crates should not reach into these modules directly.
pub(crate) mod build_summary;
pub(crate) mod builder;
pub(crate) mod content;
pub(crate) mod fs_retry;
pub(crate) mod pagination;
pub(crate) mod post_process;
pub(crate) mod routing;
pub(crate) mod search;
pub(crate) mod sidebar;
pub(crate) mod source;
pub(crate) mod taxonomy;
pub(crate) mod typst_processor;
pub(crate) mod util;

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
use fs_retry::clean_dir_contents;
use post_process::{ListPage, TermsPage, post_process_general};
use search::generate_search_index;
use source::{DirSource, DocsSource, EmbedSource};
use template::TemplateEngine;

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

        let config = SiteConfig {
            root: root_rel,
            ..SiteConfig::default()
        };
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
            // Set of every locale code in the site — used by the link
            // rewriter to know which prefixes are "already locale-scoped"
            // and should be left alone (rather than double-prefixed).
            let all_locales: Vec<String> =
                i18n.locales.iter().map(|l| l.code.clone()).collect();

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
                        page.route.route_path = format!("{}{}", prefix, page.route.route_path);
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

                    // Rewrite internal links inside content_html, summary_html
                    // and hero/feature action links so authors don't have to
                    // hard-code `/en/...` / `/zh/...` prefixes in their pages.
                    // (T-UI-4.4 / F2)
                    page.content_html = rewrite_locale_links_in_html(
                        &page.content_html,
                        &locale.code,
                        &all_locales,
                    );
                    if let Some(ref mut summary) = page.summary_html {
                        *summary =
                            rewrite_locale_links_in_html(summary, &locale.code, &all_locales);
                    }
                    if let Some(ref mut hero) = page.frontmatter.hero
                        && let Some(ref mut actions) = hero.actions
                    {
                        for action in actions {
                            action.link = prefix_internal_path(
                                &action.link,
                                &locale.code,
                                &all_locales,
                            );
                        }
                    }
                    if let Some(ref mut features) = page.frontmatter.features {
                        for feature in features {
                            if let Some(ref mut link) = feature.link {
                                *link = prefix_internal_path(link, &locale.code, &all_locales);
                            }
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

        let engine = TemplateEngine::new(Some(&self.project_root), &self.plugins, &self.config)?;

        // Take plugins out of self to move into BuiltSite
        let plugins = std::mem::take(&mut self.plugins);

        // General-SSG post-processing: discover collections, filter, build
        // list/term pages. A malformed `_collection.toml` is now a hard
        // error (was silently replaced with defaults pre-F11).
        let collections = content::discover_collections(&docs_root)?;
        let (mut all_pages, list_pages, terms_pages) =
            post_process_general(&self.config, all_pages, &collections);

        // Populate the per-page list of alternate-language versions.
        // Pages are considered translations of each other when they share a
        // `relative_path` and each has a `route.locale` set. Only runs in
        // i18n mode. (T-UI-4)
        populate_translations(&mut all_pages);

        let sidebar_keys = BuiltSite::build_sidebar_index(&all_pages, &merged_sidebar);

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: Some(self.project_root.clone()),
            pages: all_pages,
            nav: merged_nav,
            sidebar: merged_sidebar,
            sidebar_keys,
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
        let (mut pages, list_pages, terms_pages) =
            post_process_general(&self.config, br.pages, &collections);

        // Populate translations — no-op unless pages carry a locale, which
        // EmbedNovel currently never sets. Kept for API symmetry. (T-UI-4)
        populate_translations(&mut pages);

        let sidebar_keys = BuiltSite::build_sidebar_index(&pages, &br.sidebar);

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: None,
            pages,
            nav: br.nav,
            sidebar: br.sidebar,
            sidebar_keys,
            engine,
            source: Box::new(source),
            plugins,
            list_pages,
            terms_pages,
        })
    }
}

/// Decide whether `path` is an internal site path that should get a
/// locale prefix, and return the rewritten form. Leaves alone:
///
/// - external links (`http://`, `https://`, `mailto:`, `tel:`, `//`)
/// - fragment-only and query-only links (`#foo`, `?x`)
/// - relative links (anything not starting with `/`)
/// - paths that already start with `/<known_locale>/` (cross-locale link
///   or already-prefixed)
///
/// Internal paths get `/<current_locale>` prepended. Path `"/"` becomes
/// `"/<current_locale>/"`.
fn prefix_internal_path(path: &str, current_locale: &str, all_locales: &[String]) -> String {
    // External / non-path
    if path.is_empty()
        || path.starts_with("http://")
        || path.starts_with("https://")
        || path.starts_with("//")
        || path.starts_with("mailto:")
        || path.starts_with("tel:")
        || path.starts_with('#')
        || path.starts_with('?')
    {
        return path.to_string();
    }
    // Relative — leave alone (resolved by the browser against the page URL)
    if !path.starts_with('/') {
        return path.to_string();
    }
    // Already has a known locale prefix? leave alone.
    for code in all_locales {
        let p1 = format!("/{}/", code);
        let p2 = format!("/{}", code);
        if path == p2 || path.starts_with(&p1) {
            return path.to_string();
        }
    }
    // Plain absolute internal path: prefix.
    if path == "/" {
        format!("/{}/", current_locale)
    } else {
        format!("/{}{}", current_locale, path)
    }
}

/// Rewrite every `href="/..."` and `src="/..."` inside an HTML string,
/// applying [`prefix_internal_path`] to each. Used to retrofit i18n
/// locale prefixes onto pre-rendered Markdown content so authors don't
/// need to hard-code `/en/...` / `/zh/...` in their pages.
fn rewrite_locale_links_in_html(html: &str, current_locale: &str, all_locales: &[String]) -> String {
    use std::sync::LazyLock;
    static HREF_OR_SRC_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"(href|src)="(/[^"]*)""#).expect("valid regex")
    });
    HREF_OR_SRC_RE
        .replace_all(html, |caps: &regex::Captures| {
            let attr = caps.get(1).expect("group 1").as_str();
            let path = caps.get(2).expect("group 2").as_str();
            let new_path = prefix_internal_path(path, current_locale, all_locales);
            format!(r#"{}="{}""#, attr, new_path)
        })
        .into_owned()
}

/// Build a `relative_path -> [(locale, route_path), ...]` map from all
/// pages, then copy the matching list onto each page's `translations`
/// field. Pages without a `locale` (single-locale build) or whose
/// `relative_path` is empty (virtual pages like taxonomy / archive indices)
/// are skipped.
fn populate_translations(pages: &mut [PageData]) {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for page in pages.iter() {
        let Some(locale) = page.route.locale.as_deref() else {
            continue;
        };
        if page.route.relative_path.is_empty() {
            continue;
        }
        groups
            .entry(page.route.relative_path.clone())
            .or_default()
            .push((locale.to_string(), page.route.route_path.clone()));
    }
    // Sort each group by locale for stable output.
    for list in groups.values_mut() {
        list.sort_by(|a, b| a.0.cmp(&b.0));
    }
    for page in pages.iter_mut() {
        if page.route.relative_path.is_empty() {
            continue;
        }
        if let Some(list) = groups.get(&page.route.relative_path) {
            // Only emit if there is actually more than one version.
            if list.len() > 1 {
                page.translations = list.clone();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// BuiltSite -- the built result, usable for rendering / writing / serving
// ---------------------------------------------------------------------------

/// A fully-built documentation site.
///
/// Holds all page data, navigation, sidebar, and the template engine so that
/// pages can be rendered on demand or written to disk.
pub struct BuiltSite {
    config: SiteConfig,
    project_root: Option<PathBuf>,
    pages: Vec<PageData>,
    nav: Vec<NavItem>,
    sidebar: HashMap<String, Vec<SidebarItem>>,
    /// Precomputed sidebar group key per page `route_path`. Built once by
    /// [`BuiltSite::build_sidebar_index`] after `pages` and `sidebar` are
    /// finalised so [`BuiltSite::render_page`] avoids an O(N·M) linear
    /// scan over all sidebar keys on every render. (T-PERF-3)
    sidebar_keys: HashMap<String, String>,
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

        // Each branch returns a NovelResult<String>; `?` converts to anyhow
        // via the blanket From<E: std::error::Error> impl.
        let html = if is_home || layout == Some("home") {
            self.engine.render_home(page, &self.config, &self.nav)?
        } else if layout == Some("page") {
            self.engine
                .render_page_layout(page, &self.config, &self.nav)?
        } else if layout == Some("blog") {
            self.engine.render_blog(page, &self.config, &self.nav)?
        } else {
            let sidebar_items = self
                .sidebar_keys
                .get(&page.route.route_path)
                .and_then(|k| self.sidebar.get(k))
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            self.engine
                .render_doc(page, &self.config, &self.nav, sidebar_items)?
        };
        Ok(html)
    }

    /// Pre-resolve the longest-prefix-match sidebar key for every page
    /// once, instead of doing the linear scan per render call.
    ///
    /// This is correct as long as the page list and sidebar are frozen at
    /// the time of the call — which is always the case inside `BuiltSite`.
    fn build_sidebar_index(
        pages: &[PageData],
        sidebar: &HashMap<String, Vec<SidebarItem>>,
    ) -> HashMap<String, String> {
        // Pre-sort keys by length descending so the first prefix match wins
        // (equivalent to the old longest-prefix search).
        let mut keys: Vec<&String> = sidebar.keys().collect();
        keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
        let mut out = HashMap::with_capacity(pages.len());
        for page in pages {
            if let Some(k) = keys
                .iter()
                .find(|k| page.route.route_path.starts_with(k.as_str()))
            {
                out.insert(page.route.route_path.clone(), (*k).clone());
            }
        }
        out
    }

    /// Render the 404 page.
    pub fn render_404(&self) -> Result<String> {
        Ok(self.engine.render_404(&self.config, &self.nav)?)
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

        // Read the previous build's summary BEFORE clearing the output dir,
        // so we can show what changed at the end. Missing/corrupt is fine
        // — we just won't have a baseline. (F3)
        let prev_summary = if output_dir.exists() {
            build_summary::BuildSummary::read_previous(output_dir)
        } else {
            None
        };

        // Ensure the output directory exists and is empty.
        //
        // We deliberately do NOT try to remove `output_dir` itself. On Windows
        // it's common for a file watcher (VS Code, file explorers, indexers)
        // to hold an exclusive handle on the top-level output directory via
        // `ReadDirectoryChangesW`, which makes `remove_dir_all` fail with
        // `os error 32` for as long as that watcher lives. Clearing the
        // *contents* is functionally equivalent and does not require
        // ownership of the directory handle.
        if output_dir.exists() {
            clean_dir_contents(output_dir)?;
        } else {
            std::fs::create_dir_all(output_dir)?;
        }

        // Assets (with optional fingerprinting)
        let assets_dir = output_dir.join("assets");
        std::fs::create_dir_all(&assets_dir)?;

        let (css_filename, js_filename) = if self.config.asset_fingerprint {
            let css_hash = &format!("{:08x}", util::fnv1a(CSS_CONTENT.as_bytes()));
            let js_hash = &format!("{:08x}", util::fnv1a(JS_CONTENT.as_bytes()));
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
            let html =
                self.engine
                    .render_terms(tp.title.clone(), &tp.terms, &self.config, &self.nav)?;
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

        // Sass + image asset pipelines (no-op when feature/config disabled)
        if let Some(ref root) = self.project_root {
            assets::sass::compile(&self.config.sass, root, output_dir)?;
            let docs_root = self.config.docs_root(root);
            assets::images::process(&self.config.images, &docs_root, output_dir)?;
        }

        // Static assets from docs source
        self.copy_static_assets(output_dir)?;

        // Build summary: walk dist/, count + size everything, print, persist.
        // Failures here are non-fatal — the build itself already succeeded.
        // (F3)
        let summary = build_summary::BuildSummary::collect(output_dir);
        let block = summary.render(prev_summary.as_ref());
        for line in block.lines() {
            info!("{}", line);
        }
        if let Err(e) = summary.write(output_dir) {
            tracing::debug!("Could not write .novel-build.json: {e}");
        }

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

#[cfg(test)]
mod tests {
    use super::{prefix_internal_path, rewrite_locale_links_in_html};

    fn locales() -> Vec<String> {
        vec!["en".to_string(), "zh".to_string()]
    }

    #[test]
    fn prefix_internal_leaves_external_alone() {
        let l = locales();
        assert_eq!(prefix_internal_path("https://x.com", "en", &l), "https://x.com");
        assert_eq!(prefix_internal_path("//cdn.x.com/a", "en", &l), "//cdn.x.com/a");
        assert_eq!(prefix_internal_path("mailto:a@b.c", "en", &l), "mailto:a@b.c");
        assert_eq!(prefix_internal_path("#anchor", "en", &l), "#anchor");
        assert_eq!(prefix_internal_path("./rel", "en", &l), "./rel");
    }

    #[test]
    fn prefix_internal_prefixes_bare_paths() {
        let l = locales();
        assert_eq!(prefix_internal_path("/", "en", &l), "/en/");
        assert_eq!(prefix_internal_path("/guide/foo", "en", &l), "/en/guide/foo");
        assert_eq!(prefix_internal_path("/guide/foo", "zh", &l), "/zh/guide/foo");
    }

    #[test]
    fn prefix_internal_leaves_known_locale_paths_alone() {
        let l = locales();
        // Building the en site, encountering a /zh/ link → keep as cross-lang
        assert_eq!(prefix_internal_path("/zh/guide/x", "en", &l), "/zh/guide/x");
        // Building the en site, encountering a /en/ link → no double-prefix
        assert_eq!(prefix_internal_path("/en/guide/x", "en", &l), "/en/guide/x");
        // Bare /en (no trailing slash) is also a known locale prefix
        assert_eq!(prefix_internal_path("/en", "en", &l), "/en");
    }

    #[test]
    fn rewrite_html_handles_href_and_src() {
        let l = locales();
        let input = r#"<a href="/guide/x">a</a> <img src="/img.png">"#;
        let out = rewrite_locale_links_in_html(input, "en", &l);
        assert_eq!(
            out,
            r#"<a href="/en/guide/x">a</a> <img src="/en/img.png">"#
        );
    }

    #[test]
    fn rewrite_html_skips_external_and_anchor() {
        let l = locales();
        let input = r##"<a href="https://x.com">x</a> <a href="#top">top</a>"##;
        let out = rewrite_locale_links_in_html(input, "en", &l);
        assert_eq!(out, input);
    }
}

