use anyhow::Result;
use novel_shared::RouteMeta;
use std::path::Path;

use crate::source::DocsSource;

/// Scan the docs source and generate route metadata for all `.md` files.
pub(crate) fn scan_routes(source: &dyn DocsSource) -> Result<Vec<RouteMeta>> {
    let mut routes = Vec::new();

    for file_path in source.list_files() {
        if !file_path.ends_with(".md") {
            continue;
        }

        let relative = file_path.replace('\\', "/");
        let route_path = file_path_to_route(Path::new(&relative));
        let page_name = Path::new(&relative)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("index")
            .to_string();

        routes.push(RouteMeta {
            route_path,
            absolute_path: relative.clone(),
            relative_path: relative,
            page_name,
        });
    }

    // Sort routes for deterministic output
    routes.sort_by(|a, b| a.route_path.cmp(&b.route_path));

    Ok(routes)
}

/// Convert a file path to a URL route path
///
/// Examples:
/// - `guide/intro.md` -> `/guide/intro`
/// - `guide/index.md` -> `/guide/`
/// - `index.md` -> `/`
fn file_path_to_route(relative_path: &Path) -> String {
    let path_str = relative_path.to_string_lossy().replace('\\', "/");

    let without_ext = path_str.trim_end_matches(".md");

    if without_ext == "index" {
        "/".to_string()
    } else if without_ext.ends_with("/index") {
        format!("/{}/", &without_ext[..without_ext.len() - 6])
    } else {
        format!("/{}", without_ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_to_route() {
        assert_eq!(file_path_to_route(Path::new("index.md")), "/");
        assert_eq!(file_path_to_route(Path::new("guide/index.md")), "/guide/");
        assert_eq!(
            file_path_to_route(Path::new("guide/intro.md")),
            "/guide/intro"
        );
        assert_eq!(
            file_path_to_route(Path::new("guide/getting-started.md")),
            "/guide/getting-started"
        );
    }
}
