use novel_shared::{PageData, SearchIndexEntry, SearchSection};

/// Generate search index from all pages with section-level content splitting
pub fn generate_search_index(pages: &[PageData]) -> Vec<SearchIndexEntry> {
    pages
        .iter()
        .map(|page| {
            let plain_content = strip_html_tags(&page.content_html);
            let headers: Vec<String> = page.toc.iter().map(|t| t.text.clone()).collect();
            let sections = extract_sections(&page.content_html, &page.toc);

            SearchIndexEntry {
                route_path: page.route.route_path.clone(),
                title: page.title.clone(),
                description: page.description.clone(),
                headers,
                content: plain_content,
                sections,
            }
        })
        .collect()
}

/// Extract content sections split by headings for section-level search
fn extract_sections(
    html: &str,
    toc: &[novel_shared::TocItem],
) -> Vec<SearchSection> {
    if toc.is_empty() {
        return vec![];
    }

    let mut sections = Vec::new();

    for (i, item) in toc.iter().enumerate() {
        // Find the heading's anchor in the HTML
        let anchor_pattern = format!("id=\"{}\"", item.id);
        let Some(start_pos) = html.find(&anchor_pattern) else {
            continue;
        };

        // Find the end: next heading or end of content
        let content_start = html[start_pos..].find("</h").map(|p| {
            // Skip past the closing heading tag
            let after_close = start_pos + p;
            html[after_close..].find('>').map(|q| after_close + q + 1).unwrap_or(after_close)
        }).unwrap_or(start_pos);

        let content_end = if i + 1 < toc.len() {
            let next_pattern = format!("id=\"{}\"", toc[i + 1].id);
            html[content_start..].find(&next_pattern)
                .map(|p| {
                    // Back up to the start of the heading tag
                    let region = &html[..content_start + p];
                    region.rfind('<').unwrap_or(content_start + p)
                })
                .unwrap_or(html.len())
        } else {
            html.len()
        };

        let section_html = &html[content_start..content_end];
        let section_text = strip_html_tags(section_html);

        if !section_text.trim().is_empty() {
            sections.push(SearchSection {
                heading: item.text.clone(),
                anchor: item.id.clone(),
                content: section_text,
            });
        }
    }

    sections
}

/// Simple HTML tag stripper
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Normalize whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
