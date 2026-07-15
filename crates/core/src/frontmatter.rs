use anyhow::Result;
use novel_shared::{FrontMatter, HeadTag};

use crate::dates::validate_frontmatter_dates;

pub(crate) fn validate_frontmatter(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    validate_frontmatter_dates(frontmatter, source)?;
    validate_structured_urls(frontmatter, source)?;
    validate_head_tags(frontmatter, source)
}

fn validate_structured_urls(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    for (field, value) in [
        ("canonical", frontmatter.canonical.as_deref()),
        ("og_image", frontmatter.og_image.as_deref()),
        ("redirect", frontmatter.redirect.as_deref()),
    ] {
        if let Some(value) = value {
            validate_safe_url(field, value, source)?;
        }
    }

    if let Some(hero) = &frontmatter.hero {
        if let Some(image) = &hero.image {
            validate_safe_url("hero.image.src", &image.src, source)?;
        }
        if let Some(actions) = &hero.actions {
            for action in actions {
                validate_safe_url("hero.actions[].url", &action.url, source)?;
            }
        }
    }
    if let Some(features) = &frontmatter.features {
        for feature in features {
            if let Some(url) = &feature.url {
                validate_safe_url("features[].url", url, source)?;
            }
        }
    }

    Ok(())
}

fn validate_head_tags(frontmatter: &FrontMatter, source: &str) -> Result<()> {
    let Some(tags) = frontmatter.head.as_ref() else {
        return Ok(());
    };

    for tag in tags {
        validate_head_tag(tag, source)?;
    }

    Ok(())
}

fn validate_head_tag(tag: &HeadTag, source: &str) -> Result<()> {
    let tag_name = tag.tag.to_ascii_lowercase();
    let allowed_attrs: &[&str] = match tag_name.as_str() {
        "meta" => &["charset", "content", "itemprop", "name", "property"],
        "link" => &[
            "as",
            "color",
            "crossorigin",
            "fetchpriority",
            "href",
            "hreflang",
            "imagesizes",
            "imagesrcset",
            "integrity",
            "media",
            "referrerpolicy",
            "rel",
            "sizes",
            "type",
        ],
        "title" => &[],
        _ => {
            anyhow::bail!(
                "Unsafe frontmatter head tag in {}: `{}`. Allowed tags: meta, link, title",
                source,
                tag.tag
            );
        }
    };

    if matches!(tag_name.as_str(), "meta" | "link") && tag.content.is_some() {
        anyhow::bail!(
            "Frontmatter head tag `{}` in {} must not have text content",
            tag.tag,
            source
        );
    }

    for (attr, value) in &tag.attrs {
        let attr_name = attr.to_ascii_lowercase();
        if !allowed_attrs.contains(&attr_name.as_str()) {
            anyhow::bail!(
                "Unsafe attribute `{}` on frontmatter head tag `{}` in {}",
                attr,
                tag.tag,
                source
            );
        }
        if tag_name == "link" && attr_name == "href" {
            validate_safe_url("head[link].href", value, source)?;
        }
    }

    Ok(())
}

fn validate_safe_url(field: &str, value: &str, source: &str) -> Result<()> {
    if is_safe_url(value) {
        Ok(())
    } else {
        anyhow::bail!("Unsafe URL in frontmatter field `{field}` in {source}: `{value}`")
    }
}

fn is_safe_url(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }

    let normalized: String = value
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace() && !ch.is_ascii_control())
        .collect();
    let Some(colon) = normalized.find(':') else {
        return true;
    };
    let first_delimiter = normalized.find(['/', '?', '#']).unwrap_or(usize::MAX);
    if colon > first_delimiter {
        return true;
    }

    matches!(
        normalized[..colon].to_ascii_lowercase().as_str(),
        "http" | "https" | "mailto" | "tel"
    )
}

#[cfg(test)]
mod tests {
    use super::validate_frontmatter;
    use novel_shared::{FrontMatter, HeadTag};
    use std::collections::HashMap;

    fn tag(name: &str, attrs: &[(&str, &str)], content: Option<&str>) -> HeadTag {
        HeadTag {
            tag: name.to_string(),
            attrs: attrs
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
            content: content.map(str::to_string),
        }
    }

    fn frontmatter_with(tag: HeadTag) -> FrontMatter {
        FrontMatter {
            head: Some(vec![tag]),
            ..FrontMatter::default()
        }
    }

    #[test]
    fn accepts_safe_metadata_resource_hints_and_titles() {
        for head_tag in [
            tag(
                "meta",
                &[("property", "og:title"), ("content", "My page")],
                None,
            ),
            tag(
                "link",
                &[("rel", "canonical"), ("href", "https://example.com/page")],
                None,
            ),
            tag("title", &[], Some("A custom title")),
        ] {
            validate_frontmatter(&frontmatter_with(head_tag), "test.md").unwrap();
        }
    }

    #[test]
    fn rejects_executable_or_navigation_head_tags() {
        for head_tag in [
            tag("script", &[("src", "https://example.com/app.js")], None),
            tag("style", &[], Some("body { display: none }")),
            tag(
                "meta",
                &[("http-equiv", "refresh"), ("content", "0;url=/login")],
                None,
            ),
        ] {
            assert!(validate_frontmatter(&frontmatter_with(head_tag), "test.md").is_err());
        }
    }

    #[test]
    fn rejects_event_attrs_dangerous_urls_and_void_content() {
        for head_tag in [
            tag("link", &[("onload", "alert(1)")], None),
            tag("link", &[("href", "javascript:alert(1)")], None),
            tag("meta", &[], Some("unexpected")),
        ] {
            assert!(validate_frontmatter(&frontmatter_with(head_tag), "test.md").is_err());
        }
    }

    #[test]
    fn rejects_attrs_not_allowed_for_the_specific_tag() {
        let mut attrs = HashMap::new();
        attrs.insert("href".to_string(), "https://example.com".to_string());
        let frontmatter = FrontMatter {
            head: Some(vec![HeadTag {
                tag: "meta".to_string(),
                attrs,
                content: None,
            }]),
            ..FrontMatter::default()
        };

        assert!(validate_frontmatter(&frontmatter, "test.md").is_err());
    }

    #[test]
    fn rejects_dangerous_urls_in_structured_frontmatter() {
        let mut frontmatter = FrontMatter {
            canonical: Some("java\nscript:alert(1)".to_string()),
            ..FrontMatter::default()
        };
        assert!(validate_frontmatter(&frontmatter, "test.md").is_err());

        frontmatter.canonical = None;
        frontmatter.redirect = Some("data:text/html,<script>alert(1)</script>".to_string());
        assert!(validate_frontmatter(&frontmatter, "test.md").is_err());
    }
}
