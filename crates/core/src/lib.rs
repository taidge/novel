pub mod builder;
pub mod markdown;
pub mod routing;
pub mod search;
pub mod sidebar;
pub mod source;
pub mod template;

use anyhow::Result;
use novel_shared::config::{SiteConfig, ThemeConfig};
use novel_shared::{NavItem, PageData, PageType, SidebarItem};
use rust_embed::Embed;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tracing::info;

use builder::{build_pages, get_git_last_updated, route_to_file_path};
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
    fn build(&self) -> Result<BuiltSite>;
}

// ---------------------------------------------------------------------------
// DirNovel -- filesystem-backed builder
// ---------------------------------------------------------------------------

/// Builder for constructing a documentation site from a filesystem directory.
///
/// # Usage
///
/// **Standalone (with `novel.toml`)**
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
        }
    }

    /// Load from a project root that contains `novel.toml`.
    pub fn load(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let config = SiteConfig::load(&project_root)?;
        Ok(Self {
            config,
            project_root,
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
    pub fn theme(mut self, theme: ThemeConfig) -> Self {
        self.config.theme = theme;
        self
    }

    /// Mutate the theme config in place via a closure.
    pub fn with_theme(mut self, f: impl FnOnce(&mut ThemeConfig)) -> Self {
        f(&mut self.config.theme);
        self
    }

    /// Replace the entire site config.
    pub fn config(mut self, config: SiteConfig) -> Self {
        self.config = config;
        self
    }

    /// Override the project root (affects file-embed resolution etc.).
    pub fn project_root(mut self, root: impl AsRef<Path>) -> Self {
        self.project_root = root.as_ref().to_path_buf();
        self
    }
}

impl Novel for DirNovel {
    fn current_config(&self) -> &SiteConfig {
        &self.config
    }

    fn build(&self) -> Result<BuiltSite> {
        let docs_root = self.config.docs_root(&self.project_root);
        if !docs_root.exists() {
            anyhow::bail!(
                "Docs root directory does not exist: {}",
                docs_root.display()
            );
        }

        let source = DirSource::new(docs_root.clone());
        let mut br = build_pages(&source, &self.config, Some(&self.project_root))?;

        // Git timestamps (DirNovel-specific)
        if self.config.theme.last_updated {
            for page in &mut br.pages {
                let file_path = docs_root.join(&page.route.relative_path);
                page.last_updated = get_git_last_updated(&file_path);
            }
        }

        let engine = TemplateEngine::new(Some(&self.project_root))?;

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: Some(self.project_root.clone()),
            pages: br.pages,
            nav: br.nav,
            sidebar: br.sidebar,
            engine,
            source: Box::new(source),
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
    _marker: PhantomData<E>,
}

impl<E: Embed + Send + Sync + 'static> EmbedNovel<E> {
    pub fn new() -> Self {
        Self {
            config: SiteConfig::default(),
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
    pub fn theme(mut self, theme: ThemeConfig) -> Self {
        self.config.theme = theme;
        self
    }

    /// Mutate the theme config in place via a closure.
    pub fn with_theme(mut self, f: impl FnOnce(&mut ThemeConfig)) -> Self {
        f(&mut self.config.theme);
        self
    }

    /// Replace the entire site config.
    pub fn config(mut self, config: SiteConfig) -> Self {
        self.config = config;
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

    fn build(&self) -> Result<BuiltSite> {
        let source = EmbedSource::<E>::new();
        let br = build_pages(&source, &self.config, None)?;
        let engine = TemplateEngine::new(None)?;

        Ok(BuiltSite {
            config: self.config.clone(),
            project_root: None,
            pages: br.pages,
            nav: br.nav,
            sidebar: br.sidebar,
            engine,
            source: Box::new(source),
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
pub struct BuiltSite {
    config: SiteConfig,
    project_root: Option<PathBuf>,
    pages: Vec<PageData>,
    nav: Vec<NavItem>,
    sidebar: HashMap<String, Vec<SidebarItem>>,
    engine: TemplateEngine,
    source: Box<dyn DocsSource>,
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
    pub fn render_page(&self, page: &PageData) -> Result<String> {
        let is_home = matches!(page.frontmatter.page_type, Some(PageType::Home));

        if is_home {
            self.engine.render_home(page, &self.config, &self.nav)
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
        builder::generate_sitemap_xml(&self.config, &self.pages)
    }

    /// Atom/RSS feed XML (returns `None` when `site_url` is not configured).
    pub fn feed_xml(&self) -> Option<String> {
        builder::generate_feed_xml(&self.config, &self.pages)
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

        // Assets
        let assets_dir = output_dir.join("assets");
        std::fs::create_dir_all(&assets_dir)?;
        std::fs::write(assets_dir.join("style.css"), CSS_CONTENT)?;
        std::fs::write(assets_dir.join("main.js"), JS_CONTENT)?;

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

        // 404
        let html_404 = self.render_404()?;
        std::fs::write(output_dir.join("404.html"), html_404)?;

        // Search index
        let search_json = self.search_index_json()?;
        std::fs::write(assets_dir.join("search-index.json"), search_json)?;

        // Sitemap
        if let Some(xml) = self.sitemap_xml() {
            std::fs::write(output_dir.join("sitemap.xml"), xml)?;
            info!("Generated sitemap.xml");
        }

        // Feed
        if let Some(xml) = self.feed_xml() {
            std::fs::write(output_dir.join("feed.xml"), xml)?;
            info!("Generated feed.xml");
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

    /// Copy non-markdown, non-meta static assets from the docs source.
    fn copy_static_assets(&self, output_dir: &Path) -> Result<()> {
        for file_path in self.source.list_files() {
            if file_path.ends_with(".md") {
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
