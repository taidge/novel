use crate::types::{BannerConfig, NavItem, SidebarItem, SocialLink};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level site configuration, parsed from `novel.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SiteConfig {
    /// Site title
    pub title: String,
    /// Site description
    pub description: String,
    /// Documentation root directory (relative to project root)
    pub root: String,
    /// Path to logo image
    pub logo: Option<String>,
    /// Path to favicon
    pub icon: Option<String>,
    /// Default language
    pub lang: String,
    /// Output directory
    pub out_dir: String,
    /// Base URL path (e.g. "/" or "/docs/")
    pub base: String,
    /// Remove .html extensions from URLs
    pub clean_urls: bool,
    /// Site URL for sitemap/RSS (e.g. "https://example.com")
    pub site_url: Option<String>,
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Markdown configuration
    #[serde(default)]
    pub markdown: MarkdownConfig,
    /// Per-plugin configuration (keyed by plugin name)
    #[serde(default)]
    pub plugins: HashMap<String, toml::Value>,
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
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Novel".to_string(),
            description: "A static documentation site generator".to_string(),
            root: "docs".to_string(),
            logo: None,
            icon: None,
            lang: "en".to_string(),
            out_dir: "dist".to_string(),
            base: "/".to_string(),
            clean_urls: false,
            site_url: None,
            theme: ThemeConfig::default(),
            markdown: MarkdownConfig::default(),
            plugins: HashMap::new(),
            redirects: HashMap::new(),
            asset_fingerprint: false,
            template_engine: default_template_engine(),
            i18n: None,
        }
    }
}

impl SiteConfig {
    /// Load configuration from a TOML file
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let config_path = project_root.join("novel.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: SiteConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Get the absolute path to the docs root
    pub fn docs_root(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.root)
    }

    /// Get the absolute path to the output directory
    pub fn output_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.out_dir)
    }
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
    /// "Edit this page" link pattern, e.g. "https://github.com/user/repo/edit/main/docs/"
    pub edit_link: Option<String>,
    /// Custom text for edit link (default: "Edit this page")
    pub edit_link_text: Option<String>,
    /// Show last updated time
    pub last_updated: bool,
    /// Custom text for last updated (default: "Last updated")
    pub last_updated_text: Option<String>,
    /// Announcement banner
    pub banner: Option<BannerConfig>,
    /// Source code repository link in navbar
    pub source_link: Option<String>,
    /// CSS variable overrides (key = CSS variable name without --, value = CSS value)
    #[serde(default)]
    pub colors: HashMap<String, String>,
    /// Path to a custom CSS file (relative to project root)
    pub custom_css: Option<String>,
    /// Custom 404 page title
    pub not_found_title: Option<String>,
    /// Custom 404 page message
    pub not_found_message: Option<String>,
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
    pub edit_link_text: Option<String>,
    pub last_updated_text: Option<String>,
}
