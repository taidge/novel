//! Taxonomy collection: tags / categories / arbitrary terms.

use novel_shared::PageData;
use novel_shared::config::TaxonomyConfig;
use slug::slugify;
use std::collections::{BTreeMap, HashMap};

/// `term -> Vec<page_index>` for one taxonomy.
#[derive(Debug, Clone, Default)]
pub struct TermIndex {
    /// term display name -> entries (page indices into the global pages vec)
    pub terms: BTreeMap<String, Vec<usize>>,
}

/// All taxonomies built from a page set.
#[derive(Debug, Clone, Default)]
pub struct TaxonomySet {
    /// taxonomy key (e.g. "tags") -> term index
    pub by_key: HashMap<String, TermIndex>,
}

/// Returns the (taxonomy_key, terms) for a page based on enabled taxonomies.
fn page_terms_for(page: &PageData, key: &str) -> Vec<String> {
    match key {
        "tags" => page.frontmatter.tags.clone(),
        "categories" => page.frontmatter.categories.clone(),
        _ => Vec::new(),
    }
}

/// Build inverted index for each configured taxonomy.
pub fn build(pages: &[PageData], taxonomies: &HashMap<String, TaxonomyConfig>) -> TaxonomySet {
    let mut set = TaxonomySet::default();
    for key in taxonomies.keys() {
        let mut idx = TermIndex::default();
        for (i, page) in pages.iter().enumerate() {
            for term in page_terms_for(page, key) {
                idx.terms.entry(term).or_default().push(i);
            }
        }
        set.by_key.insert(key.clone(), idx);
    }
    set
}

/// Build the route for a single term.
pub fn term_route(key: &str, term: &str, cfg: &TaxonomyConfig) -> String {
    let slug = slugify(term);
    if let Some(ref tmpl) = cfg.permalink {
        tmpl.replace("{slug}", &slug)
    } else {
        format!("/{}/{}/", key, slug)
    }
}

/// Build the route for the taxonomy overview (list of all terms).
pub fn taxonomy_route(key: &str) -> String {
    format!("/{}/", key)
}
