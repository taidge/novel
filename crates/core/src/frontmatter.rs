use anyhow::Result;
use novel_shared::FrontMatter;

pub(crate) fn validate_frontmatter(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    validate_head_tags(frontmatter, source)
}

fn validate_head_tags(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    let Some(tags) = frontmatter.head.as_ref() else {
        return Ok(());
    };

    for tag in tags {
        if !is_valid_html_name(&tag.tag) {
            anyhow::bail!(
                "Invalid frontmatter head tag name in {}: `{}`",
                source,
                tag.tag
            );
        }

        for attr in tag.attrs.keys() {
            if !is_valid_html_attr_name(attr) {
                anyhow::bail!(
                    "Invalid frontmatter head attribute name in {}: `{}`",
                    source,
                    attr
                );
            }
        }
    }

    Ok(())
}

fn is_valid_html_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':'))
}

fn is_valid_html_attr_name(name: &str) -> bool {
    if name.len() >= 2 && name[..2].eq_ignore_ascii_case("on") {
        return false;
    }
    is_valid_html_name(name) || name.starts_with("data-") && is_valid_data_attr_name(name)
}

fn is_valid_data_attr_name(name: &str) -> bool {
    name.strip_prefix("data-")
        .map(|rest| {
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::validate_frontmatter;
    use novel_shared::{FrontMatter, HeadTag};
    use std::collections::HashMap;

    #[test]
    fn accepts_plain_head_tag_names_and_attrs() {
        let mut attrs = HashMap::new();
        attrs.insert("property".to_string(), "og:title".to_string());
        attrs.insert("data-site-id".to_string(), "abc".to_string());
        let frontmatter = FrontMatter {
            head: Some(vec![HeadTag {
                tag: "meta".to_string(),
                attrs,
                content: None,
            }]),
            ..FrontMatter::default()
        };

        validate_frontmatter(&frontmatter, "test.md").unwrap();
    }

    #[test]
    fn rejects_malformed_or_event_head_names() {
        let frontmatter = FrontMatter {
            head: Some(vec![HeadTag {
                tag: "meta onclick=alert(1)".to_string(),
                attrs: HashMap::new(),
                content: None,
            }]),
            ..FrontMatter::default()
        };
        assert!(validate_frontmatter(&frontmatter, "test.md").is_err());

        let mut attrs = HashMap::new();
        attrs.insert("onload".to_string(), "alert(1)".to_string());
        let frontmatter = FrontMatter {
            head: Some(vec![HeadTag {
                tag: "link".to_string(),
                attrs,
                content: None,
            }]),
            ..FrontMatter::default()
        };
        assert!(validate_frontmatter(&frontmatter, "test.md").is_err());
    }
}
