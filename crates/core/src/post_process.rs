//! Post-processing stage of the build pipeline.
//!
//! After pages are parsed and their content is ready, this module does the
//! work that only makes sense when all pages are known:
//!
//! - Assign a collection name to each entry whose path sits under a known
//!   `_collection.toml` and inherit that collection's default layout.
//! - Filter out drafts / future / expired pages based on the site config.
//! - Build virtual list pages for each collection, series, and date archive
//!   (year + year/month), each with its own paginator.
//! - Build a taxonomy inverted index and emit per-term list pages plus an
//!   overview ("terms cloud") page per taxonomy.
//!
//! The result is the (possibly filtered) vector of real pages plus two
//! vectors of virtual list / term-overview pages that `BuiltSite` stores and
//! renders at write time.
//!
//! Extracted from `lib.rs` in T-CODE-1.

use novel_shared::config::SiteConfig;
use novel_shared::PageData;
use slug::slugify;
use std::collections::HashMap;

use crate::content::{self, Collection};
use crate::pagination::{self, PageRef, Paginator};
use crate::taxonomy;
use crate::template::TermSummary;
use crate::util;

/// A virtual list page (collection / term / series / archive, paginated).
pub(crate) struct ListPage {
    pub route_path: String,
    pub title: String,
    pub paginator: Paginator,
}

/// A taxonomy overview (terms cloud) page (e.g. `/tags/`).
pub(crate) struct TermsPage {
    pub route_path: String,
    pub title: String,
    pub terms: Vec<TermSummary>,
}

/// Post-process pages: assign collection names, filter drafts/future/expiry,
/// build per-collection paginated list pages and taxonomy term pages.
pub(crate) fn post_process_general(
    config: &SiteConfig,
    pages: Vec<PageData>,
    collections: &HashMap<String, Collection>,
) -> (Vec<PageData>, Vec<ListPage>, Vec<TermsPage>) {
    // 1. Assign collection name and re-route entries with collection layout.
    let mut pages: Vec<PageData> = pages
        .into_iter()
        .map(|mut p| {
            if let Some(col) = content::collection_for_page(&p.route.relative_path, collections) {
                p.collection = Some(col.clone());
                if p.frontmatter.layout.is_none()
                    && let Some(c) = collections.get(&col)
                {
                    p.frontmatter.layout = Some(c.config.layout.clone());
                }
            }
            p
        })
        .collect();

    // 2. Drop drafts / future / expired (and their would-be index pages).
    let today = content::today_string();
    pages = content::filter_pages(pages, config.content.drafts, config.content.future, &today);

    // 3. Per-collection: build paginated list pages.
    let mut list_pages = Vec::new();
    for (name, coll) in collections {
        if !coll.config.publish {
            continue;
        }
        // Gather entries for this collection (skip the collection's own index page if any)
        let mut entries: Vec<&PageData> = pages
            .iter()
            .filter(|p| p.collection.as_deref() == Some(name.as_str()))
            .filter(|p| {
                // Skip the index page itself (route ends in /<name>/)
                let r = &p.route.route_path;
                r.trim_end_matches('/') != format!("/{}", name).trim_end_matches('/')
            })
            .collect();
        content::sort_collection_entries(&mut entries, &coll.config);

        let items: Vec<PageRef> = entries
            .iter()
            .map(|p| PageRef {
                title: p.title.clone(),
                link: p.route.route_path.clone(),
                date: p.date.clone(),
                summary_html: p.summary_html.clone().or_else(|| {
                    if !p.description.is_empty() {
                        Some(format!("<p>{}</p>", util::html_escape(&p.description)))
                    } else {
                        None
                    }
                }),
            })
            .collect();

        let base_route = format!("/{}/", name);
        let per = if coll.config.paginate_by == 0 {
            items.len().max(1)
        } else {
            coll.config.paginate_by
        };
        let paginators = pagination::paginate(
            &base_route,
            items,
            per,
            &config.pagination.page_path,
            config.pagination.first_page_in_root,
        );
        for paginator in paginators {
            list_pages.push(ListPage {
                route_path: paginator.route_path.clone(),
                title: capitalize(name),
                paginator,
            });
        }
    }

    // 3b. Series: group entries by frontmatter.series and emit list pages.
    {
        use std::collections::BTreeMap;
        let mut series_map: BTreeMap<String, Vec<&PageData>> = BTreeMap::new();
        for p in &pages {
            if let Some(ref s) = p.frontmatter.series {
                series_map.entry(s.clone()).or_default().push(p);
            }
        }
        for (series, mut entries) in series_map {
            // Sort series chronologically (asc by date) so reading order makes sense.
            entries.sort_by(|a, b| {
                let ad = a.date.as_deref().unwrap_or("");
                let bd = b.date.as_deref().unwrap_or("");
                ad.cmp(bd)
            });
            let items: Vec<PageRef> = entries
                .iter()
                .map(|p| PageRef {
                    title: p.title.clone(),
                    link: p.route.route_path.clone(),
                    date: p.date.clone(),
                    summary_html: p.summary_html.clone(),
                })
                .collect();
            let slug = slugify(&series);
            let base_route = format!("/series/{}/", slug);
            let paginators = pagination::paginate(
                &base_route,
                items,
                usize::MAX, // single-page series index
                &config.pagination.page_path,
                config.pagination.first_page_in_root,
            );
            for paginator in paginators {
                list_pages.push(ListPage {
                    route_path: paginator.route_path.clone(),
                    title: format!("Series: {}", series),
                    paginator,
                });
            }
        }
    }

    // 3c. Date archives (year + year/month) for pages with `date`.
    {
        use std::collections::BTreeMap;
        let mut by_year: BTreeMap<String, Vec<&PageData>> = BTreeMap::new();
        let mut by_ym: BTreeMap<String, Vec<&PageData>> = BTreeMap::new();
        for p in &pages {
            if let Some(date) = p.date.as_deref() {
                if date.len() >= 4 {
                    by_year.entry(date[..4].to_string()).or_default().push(p);
                }
                if date.len() >= 7 {
                    by_ym
                        .entry(date[..7].replace('-', "/"))
                        .or_default()
                        .push(p);
                }
            }
        }
        for (year, mut entries) in by_year {
            entries.sort_by(|a, b| {
                b.date
                    .as_deref()
                    .unwrap_or("")
                    .cmp(a.date.as_deref().unwrap_or(""))
            });
            let items: Vec<PageRef> = entries
                .iter()
                .map(|p| PageRef {
                    title: p.title.clone(),
                    link: p.route.route_path.clone(),
                    date: p.date.clone(),
                    summary_html: p.summary_html.clone(),
                })
                .collect();
            let base_route = format!("/archive/{}/", year);
            let paginators = pagination::paginate(
                &base_route,
                items,
                usize::MAX,
                &config.pagination.page_path,
                config.pagination.first_page_in_root,
            );
            for paginator in paginators {
                list_pages.push(ListPage {
                    route_path: paginator.route_path.clone(),
                    title: format!("Archive: {}", year),
                    paginator,
                });
            }
        }
        for (ym, mut entries) in by_ym {
            entries.sort_by(|a, b| {
                b.date
                    .as_deref()
                    .unwrap_or("")
                    .cmp(a.date.as_deref().unwrap_or(""))
            });
            let items: Vec<PageRef> = entries
                .iter()
                .map(|p| PageRef {
                    title: p.title.clone(),
                    link: p.route.route_path.clone(),
                    date: p.date.clone(),
                    summary_html: p.summary_html.clone(),
                })
                .collect();
            let base_route = format!("/archive/{}/", ym);
            let paginators = pagination::paginate(
                &base_route,
                items,
                usize::MAX,
                &config.pagination.page_path,
                config.pagination.first_page_in_root,
            );
            for paginator in paginators {
                list_pages.push(ListPage {
                    route_path: paginator.route_path.clone(),
                    title: format!("Archive: {}", ym.replace('/', "-")),
                    paginator,
                });
            }
        }
    }

    // 4. Taxonomies: build inverted index, term pages, overview pages.
    let mut terms_pages = Vec::new();
    let tax_set = taxonomy::build(&pages, &config.taxonomies);
    for (key, tax_cfg) in &config.taxonomies {
        let Some(idx) = tax_set.by_key.get(key) else {
            continue;
        };

        // Term pages
        for (term, page_indices) in &idx.terms {
            let mut entries: Vec<&PageData> = page_indices.iter().map(|i| &pages[*i]).collect();
            // Sort: date desc by default
            entries.sort_by(|a, b| {
                let ad = a.date.as_deref().unwrap_or("");
                let bd = b.date.as_deref().unwrap_or("");
                bd.cmp(ad)
            });
            let items: Vec<PageRef> = entries
                .iter()
                .map(|p| PageRef {
                    title: p.title.clone(),
                    link: p.route.route_path.clone(),
                    date: p.date.clone(),
                    summary_html: p.summary_html.clone(),
                })
                .collect();
            let base_route = taxonomy::term_route(key, term, tax_cfg);
            let per = tax_cfg.paginate_by.unwrap_or(items.len().max(1));
            let paginators = pagination::paginate(
                &base_route,
                items,
                per,
                &config.pagination.page_path,
                config.pagination.first_page_in_root,
            );
            for paginator in paginators {
                list_pages.push(ListPage {
                    route_path: paginator.route_path.clone(),
                    title: format!("{}: {}", key, term),
                    paginator,
                });
            }
        }

        // Taxonomy overview page (terms cloud)
        let mut summaries: Vec<TermSummary> = idx
            .terms
            .iter()
            .map(|(term, ids)| TermSummary {
                name: term.clone(),
                slug: slugify(term),
                link: taxonomy::term_route(key, term, tax_cfg),
                count: ids.len(),
            })
            .collect();
        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        terms_pages.push(TermsPage {
            route_path: taxonomy::taxonomy_route(key),
            title: capitalize(key),
            terms: summaries,
        });
    }

    (pages, list_pages, terms_pages)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
