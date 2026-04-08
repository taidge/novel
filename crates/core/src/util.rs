//! Small shared utilities used across the core crate.
//!
//! Anything here should be dependency-light and widely useful. Before adding,
//! double-check whether the standard library or a more specific module already
//! has what you need.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_handles_all_five() {
        assert_eq!(html_escape(r#"<a href="x">&'"#), "&lt;a href=&quot;x&quot;&gt;&amp;&#39;");
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
}
