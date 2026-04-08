use crate::plugin::{BuiltSiteView, Plugin};
use crate::search::generate_search_index;

pub struct SearchIndexPlugin;

impl Plugin for SearchIndexPlugin {
    fn name(&self) -> &str {
        "search-index"
    }

    fn on_build_complete(&self, site: &BuiltSiteView) -> Vec<(String, Vec<u8>)> {
        let idx = generate_search_index(site.pages);
        match serde_json::to_string(&idx) {
            Ok(json) => vec![("assets/search-index.json".to_string(), json.into_bytes())],
            Err(e) => {
                tracing::error!("Failed to serialize search index: {}", e);
                vec![]
            }
        }
    }
}
