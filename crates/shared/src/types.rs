use serde::{Deserialize, Serialize};

/// Route metadata for a documentation page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMeta {
    /// URL path, e.g. "/guide/intro"
    pub route_path: String,
    /// Absolute file path
    pub absolute_path: String,
    /// Relative file path from docs root
    pub relative_path: String,
    /// Page name (file stem)
    pub page_name: String,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub id: String,
    pub text: String,
    pub depth: u32,
}

/// Frontmatter metadata parsed from YAML header
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub page_type: Option<PageType>,
    #[serde(default)]
    pub sidebar: Option<bool>,
    #[serde(default)]
    pub navbar: Option<bool>,
    #[serde(default)]
    pub outline: Option<bool>,
    #[serde(default)]
    pub hero: Option<Hero>,
    #[serde(default)]
    pub features: Option<Vec<Feature>>,
    /// Custom head tags for this page
    #[serde(default)]
    pub head: Option<Vec<HeadTag>>,
}

/// Custom HTML head tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadTag {
    pub tag: String,
    #[serde(default)]
    pub attrs: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageType {
    Home,
    Doc,
    Custom,
    #[serde(rename = "404")]
    NotFound,
}

/// Hero section for home page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hero {
    pub name: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub actions: Option<Vec<HeroAction>>,
    #[serde(default)]
    pub image: Option<HeroImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroAction {
    pub text: String,
    pub link: String,
    #[serde(default)]
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroImage {
    pub src: String,
    #[serde(default)]
    pub alt: Option<String>,
}

/// Feature item for home page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub title: String,
    pub details: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
}

/// Link to a page (for prev/next navigation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageLink {
    pub title: String,
    pub link: String,
}

/// Processed page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub route: RouteMeta,
    pub title: String,
    pub description: String,
    pub content_html: String,
    pub toc: Vec<TocItem>,
    pub frontmatter: FrontMatter,
    /// Git last updated timestamp
    #[serde(default)]
    pub last_updated: Option<String>,
    /// Previous page in navigation order
    #[serde(default)]
    pub prev_page: Option<PageLink>,
    /// Next page in navigation order
    #[serde(default)]
    pub next_page: Option<PageLink>,
}

/// Sidebar item - can be a link, group, or divider
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SidebarItem {
    Link {
        text: String,
        link: String,
    },
    Group {
        text: String,
        #[serde(default)]
        collapsed: bool,
        items: Vec<SidebarItem>,
    },
    Divider,
}

/// Navigation bar item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    pub text: String,
    pub link: String,
    #[serde(default)]
    pub active_match: Option<String>,
}

/// Social link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub icon: String,
    pub link: String,
}

/// Search index entry for client-side search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexEntry {
    pub route_path: String,
    pub title: String,
    pub description: String,
    pub headers: Vec<String>,
    pub content: String,
}

/// Banner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerConfig {
    pub text: String,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default = "default_true")]
    pub dismissible: bool,
}

fn default_true() -> bool {
    true
}
