use novel_shared::{PageData, SearchIndexEntry};

/// Generate search index from all pages
pub fn generate_search_index(pages: &[PageData]) -> Vec<SearchIndexEntry> {
    pages
        .iter()
        .map(|page| {
            // Strip HTML tags from content for plain text search
            let plain_content = strip_html_tags(&page.content_html);
            let headers: Vec<String> = page.toc.iter().map(|t| t.text.clone()).collect();

            SearchIndexEntry {
                route_path: page.route.route_path.clone(),
                title: page.title.clone(),
                description: page.description.clone(),
                headers,
                content: plain_content,
            }
        })
        .collect()
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
