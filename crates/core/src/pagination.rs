//! Generic paginator for collection / taxonomy list pages.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PageRef {
    pub title: String,
    pub link: String,
    pub date: Option<String>,
    pub summary_html: Option<String>,
}

/// One paginated page.
#[derive(Debug, Clone, Serialize)]
pub struct Paginator {
    pub current: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub items: Vec<PageRef>,
    pub prev_url: Option<String>,
    pub next_url: Option<String>,
    /// Route path for THIS paginator instance
    pub route_path: String,
    /// Base route (collection/term root, e.g. "/posts/")
    pub base_route: String,
}

/// Build paginators for a list of items at `base_route`, paginated by `per_page`.
///
/// `page_path` controls the URL segment used for pages 2..N (e.g. "page" -> /posts/page/2/).
/// First page lives at `base_route` itself when `first_page_in_root` is true.
pub fn paginate(
    base_route: &str,
    items: Vec<PageRef>,
    per_page: usize,
    page_path: &str,
    first_page_in_root: bool,
) -> Vec<Paginator> {
    let total_items = items.len();
    let per = per_page.max(1);
    let total_pages = total_items.div_ceil(per).max(1);

    let mut out = Vec::with_capacity(total_pages);
    for n in 1..=total_pages {
        let start = (n - 1) * per;
        let end = (start + per).min(total_items);
        let chunk = items[start..end].to_vec();

        let route = page_url(base_route, n, page_path, first_page_in_root);
        let prev_url = if n > 1 {
            Some(page_url(base_route, n - 1, page_path, first_page_in_root))
        } else {
            None
        };
        let next_url = if n < total_pages {
            Some(page_url(base_route, n + 1, page_path, first_page_in_root))
        } else {
            None
        };

        out.push(Paginator {
            current: n,
            total_pages,
            total_items,
            items: chunk,
            prev_url,
            next_url,
            route_path: route,
            base_route: base_route.to_string(),
        });
    }
    out
}

fn page_url(base: &str, n: usize, page_path: &str, first_in_root: bool) -> String {
    if n == 1 && first_in_root {
        return base.to_string();
    }
    let trimmed = base.trim_end_matches('/');
    format!("{}/{}/{}/", trimmed, page_path, n)
}
