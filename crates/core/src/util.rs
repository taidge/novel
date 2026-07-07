//! Small shared utilities used across the core crate.
//!
//! Anything here should be dependency-light and widely useful. Before adding,
//! double-check whether the standard library or a more specific module already
//! has what you need.

use std::path::{Component, Path, PathBuf};

/// Escape the five characters that are unsafe to inline into HTML text *or*
/// attribute contexts: `& < > " '`.
///
/// Callers that only emit into text nodes technically only need `& < >`, but
/// having a single escape function prevents the foot-gun where an identical
/// helper is used for attribute context later.
pub(crate) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Strip HTML tags from a string and normalise runs of whitespace to single
/// spaces. Used for word counts and search indexing.
pub(crate) fn strip_html_tags(html: &str) -> String {
    let mut plain = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => plain.push(ch),
            _ => {}
        }
    }
    // Normalize whitespace
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// FNV-1a 64-bit non-cryptographic hash — used for asset fingerprinting.
pub(crate) fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn is_special_url(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("//")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
}

pub(crate) fn join_base_path(base: &str, path: &str) -> String {
    let path = path.trim();
    if is_special_url(path) {
        return path.to_string();
    }

    let mut normalized_base = base.trim().trim_end_matches('/').to_string();
    if normalized_base == "/" {
        normalized_base.clear();
    }
    if !normalized_base.is_empty() && !normalized_base.starts_with('/') {
        normalized_base.insert(0, '/');
    }

    let normalized_path = if path == "/" {
        "/".to_string()
    } else {
        format!("/{}", path.trim_start_matches('/'))
    };

    if normalized_base.is_empty() {
        normalized_path
    } else if normalized_path == "/" {
        format!("{}/", normalized_base)
    } else {
        format!("{}{}", normalized_base, normalized_path)
    }
}

pub(crate) fn join_site_url(site_url: &str, base: &str, path: &str) -> String {
    let site = site_url.trim().trim_end_matches('/');
    if site.is_empty() {
        return join_base_path(base, path);
    }

    let base_path = join_base_path(base, "/");
    let normalized_base = base_path.trim_end_matches('/');
    if !normalized_base.is_empty() && normalized_base != "/" && site.ends_with(normalized_base) {
        format!("{}{}", site, join_base_path("/", path))
    } else {
        format!("{}{}", site, join_base_path(base, path))
    }
}

/// Join a caller-provided relative path to `base` without allowing it to
/// escape `base`.
pub(crate) fn safe_join_relative(
    base: &Path,
    relative: impl AsRef<Path>,
) -> std::io::Result<PathBuf> {
    let relative = relative.as_ref();
    let mut out = base.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("path escapes base directory: {}", relative.display()),
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn html_escape_handles_all_five() {
        assert_eq!(
            html_escape(r#"<a href="x">&'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;"
        );
    }

    #[test]
    fn strip_html_tags_normalises_whitespace() {
        let html = "<p>hello   <b>world</b></p>\n<p>foo</p>";
        assert_eq!(strip_html_tags(html), "hello world foo");
    }

    #[test]
    fn fnv1a_is_deterministic() {
        assert_eq!(fnv1a(b"hello"), fnv1a(b"hello"));
        assert_ne!(fnv1a(b"hello"), fnv1a(b"world"));
    }

    #[test]
    fn join_base_path_prefixes_internal_urls_only() {
        assert_eq!(
            join_base_path("/docs/", "/guide/intro"),
            "/docs/guide/intro"
        );
        assert_eq!(join_base_path("/docs/", "/"), "/docs/");
        assert_eq!(join_base_path("/", "/guide/intro"), "/guide/intro");
        assert_eq!(
            join_base_path("/docs/", "https://example.com"),
            "https://example.com"
        );
        assert_eq!(join_base_path("/docs/", "#top"), "#top");
    }

    #[test]
    fn join_site_url_includes_base_without_doubling_it() {
        assert_eq!(
            join_site_url("https://example.com", "/docs/", "/guide/intro"),
            "https://example.com/docs/guide/intro"
        );
        assert_eq!(
            join_site_url("https://example.com/docs", "/docs/", "/guide/intro"),
            "https://example.com/docs/guide/intro"
        );
    }

    #[test]
    fn safe_join_allows_plain_relative_paths() {
        let out = safe_join_relative(Path::new("dist"), Path::new("guide/index.html")).unwrap();
        assert_eq!(out, Path::new("dist").join("guide").join("index.html"));
    }

    #[test]
    fn safe_join_rejects_parent_segments() {
        assert!(safe_join_relative(Path::new("dist"), Path::new("../secret")).is_err());
        assert!(safe_join_relative(Path::new("dist"), Path::new("guide/../../secret")).is_err());
    }

    #[test]
    fn safe_join_rejects_absolute_paths() {
        assert!(safe_join_relative(Path::new("dist"), Path::new("/tmp/secret")).is_err());
    }
}
