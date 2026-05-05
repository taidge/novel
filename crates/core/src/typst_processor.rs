use anyhow::Result;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use novel_shared::{FrontMatter, PageData, RouteMeta, TocItem};
use regex::Regex;
use slug::slugify;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

// Compile-time regex constants used by the helpers below. Declared here so
// they're built exactly once per process instead of once per .typ file.
static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<(h[1-6])([^>]*)>(.*?)</h[1-6]>").expect("valid regex"));
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("valid regex"));
static ID_ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"id="([^"]*)""#).expect("valid regex"));
static TOC_HEADING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<h([2-4])\s+id="([^"]*)"[^>]*>(.*?)</h[2-4]>"#).expect("valid regex")
});
static FIRST_H1_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<h1[^>]*>(.*?)</h1>").expect("valid regex"));

/// Typst document processor — compiles `.typ` files to HTML via the `typst` CLI.
pub struct TypstProcessor {
    docs_root: std::path::PathBuf,
}

impl TypstProcessor {
    pub fn new(docs_root: &Path) -> Self {
        Self {
            docs_root: docs_root.to_path_buf(),
        }
    }

    /// Check whether the `typst` CLI is available on PATH.
    pub fn is_available() -> bool {
        Command::new("typst")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Process a `.typ` file into [`PageData`].
    pub fn process_file(&self, raw_content: &str, route: RouteMeta) -> Result<PageData> {
        // 1. Extract YAML frontmatter from leading `//` comments
        let (frontmatter, _body) = parse_typst_frontmatter(raw_content);

        // 2. Compile to HTML via the `typst` CLI
        let abs_path = self.docs_root.join(&route.relative_path);
        let raw_html = compile_to_html(&abs_path, &self.docs_root)?;

        // 3. Extract body content, add heading anchors
        let body = extract_body_content(&raw_html);
        let content_html = add_heading_anchors(&body);

        // 4. Build TOC from headings
        let toc = extract_toc(&content_html);
        let first_h1 = extract_first_h1(&content_html);

        // 5. Determine title
        let title = frontmatter
            .title
            .clone()
            .or(first_h1)
            .or_else(|| toc.first().map(|t| t.text.clone()))
            .unwrap_or_else(|| route.page_name.clone());

        let description = frontmatter.description.clone().unwrap_or_default();

        Ok(PageData {
            route,
            title,
            description,
            content_html,
            toc,
            date: frontmatter.date.clone(),
            frontmatter,
            last_updated: None,
            prev_page: None,
            next_page: None,
            reading_time: None,
            word_count: None,
            breadcrumbs: Vec::new(),
            summary_html: None,
            collection: None,
            translations: Vec::new(),
            version_links: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Frontmatter parsing
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter from `//` comment blocks at the top of a `.typ` file.
///
/// ```typst
/// // ---
/// // title: My Page
/// // description: A page written in Typst
/// // ---
///
/// = Heading
/// Some content…
/// ```
fn parse_typst_frontmatter(content: &str) -> (FrontMatter, String) {
    let mut yaml_lines: Vec<&str> = Vec::new();
    let mut in_frontmatter = false;
    let mut found_end = false;
    let mut body_byte_offset: usize = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        // Track byte position (line + newline char)
        let line_len = line.len() + 1; // approximate; handles \n

        if !in_frontmatter {
            if trimmed == "// ---" {
                in_frontmatter = true;
                body_byte_offset += line_len;
                continue;
            } else if trimmed.is_empty() {
                body_byte_offset += line_len;
                continue;
            } else {
                // No frontmatter found — the whole file is body
                break;
            }
        }

        body_byte_offset += line_len;

        if trimmed == "// ---" {
            found_end = true;
            break;
        }

        // Strip the leading `// ` or `//`
        if let Some(rest) = trimmed.strip_prefix("// ") {
            yaml_lines.push(rest);
        } else if let Some(rest) = trimmed.strip_prefix("//") {
            yaml_lines.push(rest);
        }
    }

    if !found_end || yaml_lines.is_empty() {
        return (FrontMatter::default(), content.to_string());
    }

    // Build a synthetic markdown-style frontmatter string so we can reuse
    // gray_matter for parsing.
    let yaml_content = yaml_lines.join("\n");
    let synthetic = format!("---\n{}\n---\n", yaml_content);

    let matter = Matter::<YAML>::new();
    let frontmatter = match matter.parse(&synthetic) {
        Ok(parsed) => parsed
            .data
            .and_then(|d: gray_matter::Pod| d.deserialize().ok())
            .unwrap_or_default(),
        Err(_) => FrontMatter::default(),
    };

    let body = if body_byte_offset < content.len() {
        content[body_byte_offset..].to_string()
    } else {
        String::new()
    };

    (frontmatter, body)
}

// ---------------------------------------------------------------------------
// Compilation
// ---------------------------------------------------------------------------

/// Compile a `.typ` file to HTML using the `typst` CLI.
fn compile_to_html(file_path: &Path, root: &Path) -> Result<String> {
    // Deterministic temp filename derived from the source path
    let hash = fnv1a(file_path.to_string_lossy().as_bytes());
    let temp_output = std::env::temp_dir().join(format!("novel_typst_{:016x}.html", hash));

    let output = Command::new("typst")
        .args(["compile", "--format", "html", "--root"])
        .arg(root)
        .arg(file_path)
        .arg(&temp_output)
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to run `typst` — is it installed and on PATH? ({})",
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&temp_output);
        anyhow::bail!(
            "typst compile failed for {}: {}",
            file_path.display(),
            stderr
        );
    }

    let html = std::fs::read_to_string(&temp_output)?;
    let _ = std::fs::remove_file(&temp_output);
    Ok(html)
}

/// FNV-1a hash for generating unique temp filenames.
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ---------------------------------------------------------------------------
// HTML post-processing
// ---------------------------------------------------------------------------

/// Extract the content between `<body>` and `</body>` tags.
/// Falls back to the full string if no body tags are found.
fn extract_body_content(html: &str) -> String {
    if let Some(body_start) = html.find("<body")
        && let Some(tag_end) = html[body_start..].find('>')
    {
        let content_start = body_start + tag_end + 1;
        if let Some(body_end) = html.find("</body>") {
            return html[content_start..body_end].trim().to_string();
        }
    }
    html.to_string()
}

/// Add anchor IDs and `#` links to headings (h1–h6) in the HTML.
fn add_heading_anchors(html: &str) -> String {
    HEADING_RE
        .replace_all(html, |caps: &regex::Captures| {
            let tag = &caps[1];
            let attrs = &caps[2];
            let inner = &caps[3];

            // Reuse existing id if present, otherwise generate from text
            let id = ID_ATTR_RE
                .captures(attrs)
                .map(|c| c[1].to_string())
                .unwrap_or_else(|| {
                    let text = TAG_RE.replace_all(inner, "");
                    slugify(text.trim())
                });

            format!(
                "<{tag} id=\"{id}\">{inner} <a class=\"header-anchor\" href=\"#{id}\">#</a></{tag}>"
            )
        })
        .to_string()
}

/// Extract a table-of-contents from h2–h4 headings in the HTML.
fn extract_toc(html: &str) -> Vec<TocItem> {
    TOC_HEADING_RE
        .captures_iter(html)
        .map(|caps| {
            // Safe: the [2-4] character class in the regex guarantees a single digit.
            let depth: u32 = caps[1].parse().expect("regex guarantees digit");
            let id = caps[2].to_string();
            let text = TAG_RE.replace_all(&caps[3], "").trim().to_string();
            TocItem { id, text, depth }
        })
        .collect()
}

/// Extract the text of the first `<h1>` in the HTML, if any.
fn extract_first_h1(html: &str) -> Option<String> {
    FIRST_H1_RE
        .captures(html)
        .map(|caps| TAG_RE.replace_all(&caps[1], "").trim().to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typst_frontmatter() {
        let content = "// ---\n// title: Hello\n// description: World\n// ---\n\n= Heading\n";
        let (fm, body) = parse_typst_frontmatter(content);
        assert_eq!(fm.title.as_deref(), Some("Hello"));
        assert_eq!(fm.description.as_deref(), Some("World"));
        assert!(body.contains("= Heading"));
    }

    #[test]
    fn test_parse_typst_frontmatter_missing() {
        let content = "= Just a heading\nSome text.\n";
        let (fm, body) = parse_typst_frontmatter(content);
        assert!(fm.title.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_extract_body_content() {
        let html = r#"<!DOCTYPE html><html><head></head><body><p>Hello</p></body></html>"#;
        assert_eq!(extract_body_content(html), "<p>Hello</p>");
    }

    #[test]
    fn test_add_heading_anchors() {
        let html = "<h2>Getting Started</h2><p>text</p>";
        let result = add_heading_anchors(html);
        assert!(result.contains("id=\"getting-started\""));
        assert!(result.contains("href=\"#getting-started\""));
    }

    #[test]
    fn test_extract_toc() {
        let html = r#"<h1 id="title">Title</h1><h2 id="intro">Intro</h2><h3 id="sub">Sub</h3>"#;
        let toc = extract_toc(html);
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].id, "intro");
        assert_eq!(toc[0].depth, 2);
        assert_eq!(toc[1].id, "sub");
        assert_eq!(toc[1].depth, 3);
    }
}
