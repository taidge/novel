use anyhow::Result;
use sapid_shared::RouteMeta;
use std::path::Path;
use walkdir::WalkDir;

/// Scan the docs directory and generate route metadata
pub fn scan_routes(docs_root: &Path) -> Result<Vec<RouteMeta>> {
    let mut routes = Vec::new();

    if !docs_root.exists() {
        anyhow::bail!(
            "Docs root directory does not exist: {}",
            docs_root.display()
        );
    }

    for entry in WalkDir::new(docs_root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "md" {
            continue;
        }

        let relative = path.strip_prefix(docs_root).unwrap_or(path).to_path_buf();

        let route_path = file_path_to_route(&relative);
        let page_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("index")
            .to_string();

        routes.push(RouteMeta {
            route_path,
            absolute_path: path.to_string_lossy().to_string(),
            relative_path: relative.to_string_lossy().to_string().replace('\\', "/"),
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
/// - `guide/intro.md` → `/guide/intro`
/// - `guide/index.md` → `/guide/`
/// - `index.md` → `/`
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
