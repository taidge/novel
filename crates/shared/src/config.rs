use crate::types::{BannerConfig, NavItem, SidebarItem, SocialLink};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

/// Top-level site configuration, parsed from `novel.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteConfig {
    /// Site title
    pub title: String,
    /// Site description
    pub description: String,
    /// Documentation source directory (relative to project root)
    pub docs_dir: String,
    /// Path to logo image
    pub logo: Option<String>,
    /// Path to favicon
    pub icon: Option<String>,
    /// Default language
    pub lang: String,
    /// Output directory
    pub output_dir: String,
    /// Base URL path (e.g. "/" or "/docs/")
    pub base: String,
    /// Site URL for sitemap/RSS (e.g. "https://example.com")
    pub site_url: Option<String>,
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Markdown configuration
    #[serde(default)]
    pub markdown: MarkdownConfig,
    /// Per-plugin configuration (keyed by plugin name)
    #[serde(default)]
    pub plugins: HashMap<String, serde_json::Value>,
    /// Global redirects: old path -> new path
    #[serde(default)]
    pub redirects: HashMap<String, String>,
    /// Enable asset fingerprinting (content hash in CSS/JS filenames)
    #[serde(default)]
    pub asset_fingerprint: bool,
    /// Template engine: "minijinja" (default), "tera", "handlebars"
    #[serde(default = "default_template_engine")]
    pub template_engine: String,
    /// Internationalization configuration
    #[serde(default)]
    pub i18n: Option<I18nConfig>,
    /// Documentation versioning configuration
    #[serde(default)]
    pub versions: Option<VersioningConfig>,
    /// Per-page Markdown mirror output
    #[serde(default)]
    pub markdown_mirror: MarkdownMirrorConfig,
    /// Progressive Web App / offline support
    #[serde(default)]
    pub pwa: PwaConfig,
    /// Static page feedback widget
    #[serde(default)]
    pub feedback: FeedbackConfig,
    /// General content / collection behaviour
    #[serde(default)]
    pub content: ContentConfig,
    /// Taxonomies (e.g. tags, categories)
    #[serde(default)]
    pub taxonomies: HashMap<String, TaxonomyConfig>,
    /// Pagination defaults
    #[serde(default)]
    pub pagination: PaginationConfig,
    /// Sass / SCSS compilation (requires `sass` feature)
    #[serde(default)]
    pub sass: SassConfig,
    /// Image processing (requires `images` feature)
    #[serde(default)]
    pub images: ImagesConfig,
}

/// Sass entry/output mapping. Paths are relative to the project root.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SassConfig {
    /// List of (input, output) pairs, e.g. `[["assets/scss/main.scss", "assets/css/main.css"]]`
    pub entries: Vec<Vec<String>>,
    /// Additional load paths (project-relative).
    pub load_paths: Vec<String>,
}

/// Image processing config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImagesConfig {
    /// Resize widths to generate, e.g. `[400, 800, 1600]`. Empty = disabled.
    pub sizes: Vec<u32>,
    /// JPEG/WebP quality (0-100). Default 82.
    pub quality: u8,
}

impl Default for ImagesConfig {
    fn default() -> Self {
        Self {
            sizes: Vec::new(),
            quality: 82,
        }
    }
}

/// General content config (drafts, future, summary separator).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContentConfig {
    pub drafts: bool,
    pub future: bool,
    pub summary_separator: String,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            drafts: false,
            future: false,
            summary_separator: "<!-- more -->".to_string(),
        }
    }
}

/// Taxonomy configuration (e.g. tags, categories).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TaxonomyConfig {
    /// Permalink template, supports `{slug}`. Default `/<key>/{slug}/`.
    pub permalink: Option<String>,
    pub per_page: Option<usize>,
}

/// Pagination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaginationConfig {
    /// Path segment used for paginated subpages, e.g. "page" -> /posts/page/2/
    pub page_path: String,
    /// Whether the first page is at the collection root (rather than /page/1/).
    pub first_page_in_root: bool,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            page_path: "page".to_string(),
            first_page_in_root: true,
        }
    }
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Novel".to_string(),
            description: "A static documentation site generator".to_string(),
            docs_dir: "docs".to_string(),
            logo: None,
            icon: None,
            lang: "en".to_string(),
            output_dir: "dist".to_string(),
            base: "/".to_string(),
            site_url: None,
            theme: ThemeConfig::default(),
            markdown: MarkdownConfig::default(),
            plugins: HashMap::new(),
            redirects: HashMap::new(),
            asset_fingerprint: false,
            template_engine: default_template_engine(),
            i18n: None,
            versions: None,
            markdown_mirror: MarkdownMirrorConfig::default(),
            pwa: PwaConfig::default(),
            feedback: FeedbackConfig::default(),
            content: ContentConfig::default(),
            taxonomies: HashMap::new(),
            pagination: PaginationConfig::default(),
            sass: SassConfig::default(),
            images: ImagesConfig::default(),
        }
    }
}

impl SiteConfig {
    /// Load configuration from `novel.toml`.
    ///
    /// Falls back to defaults when neither file is present.
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let toml_path = project_root.join("novel.toml");

        if toml_path.exists() {
            let content = std::fs::read_to_string(&toml_path)?;
            Self::from_toml(&content)
        } else {
            Ok(Self::default())
        }
    }

    /// Parse from a TOML string.
    pub fn from_toml(content: &str) -> anyhow::Result<Self> {
        let config: SiteConfig = toml::from_str(content)?;
        Ok(config)
    }

    /// Returns the config file path that exists in the project root, if any.
    pub fn config_path(project_root: &Path) -> Option<PathBuf> {
        let toml = project_root.join("novel.toml");
        if toml.exists() {
            return Some(toml);
        }
        None
    }

    /// Get the absolute path to the docs root
    pub fn docs_root(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.docs_dir)
    }

    /// Get the checked absolute path to the docs root.
    pub fn docs_root_checked(&self, project_root: &Path) -> anyhow::Result<PathBuf> {
        resolve_project_relative_path(project_root, &self.docs_dir, "docs_dir")
    }

    /// Get the absolute path to the output directory
    pub fn output_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.output_dir)
    }

    /// Get the checked absolute path to the output directory.
    pub fn output_dir_checked(&self, project_root: &Path) -> anyhow::Result<PathBuf> {
        resolve_project_relative_path(project_root, &self.output_dir, "output_dir")
    }
}

fn resolve_project_relative_path(
    project_root: &Path,
    configured: &str,
    field: &str,
) -> anyhow::Result<PathBuf> {
    if configured.trim().is_empty() {
        anyhow::bail!("{field} must be a non-empty project-relative path");
    }

    let relative = Path::new(configured);
    let mut out = project_root.to_path_buf();
    let mut saw_normal_component = false;

    for component in relative.components() {
        match component {
            Component::Normal(part) => {
                saw_normal_component = true;
                out.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!(
                    "{field} must be a project-relative path that stays inside the project root: {configured}"
                );
            }
        }
    }

    if !saw_normal_component {
        anyhow::bail!("{field} must not point at the project root");
    }

    Ok(out)
}

/// Markdown processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkdownConfig {
    /// Show line numbers on code blocks by default
    pub show_line_numbers: bool,
    /// Wrap long code lines by default
    pub default_wrap_code: bool,
    /// Check for dead internal links during build
    pub check_dead_links: bool,
    /// Enable math rendering (KaTeX)
    #[serde(default)]
    pub math: bool,
    /// Enable mermaid diagrams
    #[serde(default)]
    pub mermaid: bool,
    /// Syntax highlighting theme name (default: "base16-ocean.dark")
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
}

fn default_template_engine() -> String {
    "minijinja".to_string()
}

fn default_syntax_theme() -> String {
    "base16-ocean.dark".to_string()
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            default_wrap_code: false,
            check_dead_links: false,
            math: false,
            mermaid: false,
            syntax_theme: default_syntax_theme(),
        }
    }
}

/// Theme configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Enable dark mode toggle
    pub dark_mode: bool,
    /// Navigation items (if empty, auto-generated from directory structure)
    pub nav: Vec<NavItem>,
    /// Sidebar configuration per path prefix
    /// Key is a path prefix like "/guide", value is list of sidebar items
    pub sidebar: HashMap<String, Vec<crate::types::SidebarItem>>,
    /// Social links shown in navbar
    pub social_links: Vec<SocialLink>,
    /// Footer text (HTML allowed)
    pub footer: Option<String>,
    /// "Edit this page" URL prefix, e.g. "https://github.com/user/repo/edit/main/docs/"
    pub edit_url: Option<String>,
    /// Custom text for edit action (default: "Edit this page")
    pub edit_text: Option<String>,
    /// Show Git last-updated time
    pub show_git_updated_at: bool,
    /// Custom text for Git last-updated timestamp (default: "Last updated")
    pub git_updated_text: Option<String>,
    /// Announcement banner
    pub banner: Option<BannerConfig>,
    /// Source code repository URL in navbar
    pub source_url: Option<String>,
    /// CSS variable overrides (key = CSS variable name without --, value = CSS value)
    #[serde(default)]
    pub colors: HashMap<String, String>,
    /// Path to a custom CSS file (relative to project root)
    pub custom_css: Option<String>,
    /// Theme pack: local directory containing override templates (and
    /// optionally assets). Searched after project `templates/` and before
    /// the embedded defaults.
    pub pack: Option<String>,
    /// Custom 404 page title
    pub not_found_title: Option<String>,
    /// Custom 404 page message
    pub not_found_message: Option<String>,
}

/// Documentation versioning configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VersioningConfig {
    /// Version code that should keep the canonical, unprefixed routes unless
    /// that version explicitly sets `path`.
    pub current: String,
    /// Version entries, ordered as they should appear in the selector.
    pub items: Vec<VersionConfig>,
}

impl Default for VersioningConfig {
    fn default() -> Self {
        Self {
            current: "current".to_string(),
            items: Vec::new(),
        }
    }
}

/// Single documentation version.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VersionConfig {
    /// Stable version code, e.g. "v2" or "next".
    pub code: String,
    /// Label shown in the version selector. Defaults to `code`.
    pub label: String,
    /// Directory under the docs root containing this version.
    pub dir: String,
    /// Optional route prefix. Defaults to no prefix for `current`, otherwise
    /// `/<code>`.
    pub path: Option<String>,
}

/// Per-page Markdown mirror output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkdownMirrorConfig {
    pub enabled: bool,
    pub strip_frontmatter: bool,
}

impl Default for MarkdownMirrorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strip_frontmatter: true,
        }
    }
}

/// Progressive Web App / offline support.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PwaConfig {
    pub enabled: bool,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub theme_color: String,
    pub background_color: String,
    pub display: String,
    pub cache_search_index: bool,
}

impl Default for PwaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            name: None,
            short_name: None,
            theme_color: "#3b82f6".to_string(),
            background_color: "#ffffff".to_string(),
            display: "standalone".to_string(),
            cache_search_index: true,
        }
    }
}

/// Static page feedback widget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeedbackConfig {
    pub enabled: bool,
    pub question: String,
    pub positive_text: String,
    pub negative_text: String,
    pub thanks_text: String,
    pub positive_url: Option<String>,
    pub negative_url: Option<String>,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            question: "Was this page helpful?".to_string(),
            positive_text: "Yes".to_string(),
            negative_text: "No".to_string(),
            thanks_text: "Thanks for the feedback.".to_string(),
            positive_url: None,
            negative_url: None,
        }
    }
}

/// Internationalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub default_locale: String,
    pub locales: Vec<LocaleConfig>,
}

/// Configuration for a single locale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    pub code: String,
    pub name: String,
    pub dir: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub theme: Option<LocaleThemeOverrides>,
}

/// Per-locale theme overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LocaleThemeOverrides {
    pub nav: Option<Vec<NavItem>>,
    pub sidebar: Option<HashMap<String, Vec<SidebarItem>>>,
    pub footer: Option<String>,
    pub edit_text: Option<String>,
    pub git_updated_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::SiteConfig;
    use std::path::Path;

    #[test]
    fn checked_project_paths_reject_escapes_and_project_root() {
        let config = SiteConfig {
            docs_dir: "../docs".to_string(),
            ..SiteConfig::default()
        };
        assert!(config.docs_root_checked(Path::new("project")).is_err());

        let config = SiteConfig {
            output_dir: ".".to_string(),
            ..SiteConfig::default()
        };
        assert!(config.output_dir_checked(Path::new("project")).is_err());
    }

    #[test]
    fn checked_project_paths_accept_plain_relative_paths() {
        let config = SiteConfig::default();
        assert_eq!(
            config.docs_root_checked(Path::new("project")).unwrap(),
            Path::new("project").join("docs")
        );
        assert_eq!(
            config.output_dir_checked(Path::new("project")).unwrap(),
            Path::new("project").join("dist")
        );
    }
}
