use novel_shared::RouteMeta;
use std::path::Path;

use crate::source::DocsSource;

/// File extensions recognised as content pages.
const CONTENT_EXTENSIONS: &[&str] = &[".md", ".typ"];

/// Check whether a file path is a supported content file.
fn is_content_file(path: &str) -> bool {
    CONTENT_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

/// Strip the content file extension from a path string.
fn strip_content_extension(path: &str) -> &str {
    for ext in CONTENT_EXTENSIONS {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped;
        }
    }
    path
}

/// Scan the docs source and generate route metadata for all content files.
///
/// This function is purely structural — it walks `source.list_files()` and
/// constructs `RouteMeta` values. Source listing failures are surfaced
/// upstream by the source impl itself, not here, so the return type is a
/// plain `Vec` rather than a `Result`. (F8)
pub(crate) fn scan_routes(source: &dyn DocsSource) -> Vec<RouteMeta> {
    let mut routes = Vec::new();

    for file_path in source.list_files() {
        if !is_content_file(&file_path) {
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
            locale: None,
            version: None,
        });
    }

    // Sort routes for deterministic output
    routes.sort_by(|a, b| a.route_path.cmp(&b.route_path));

    routes
}

/// Convert a file path to a URL route path
///
/// Examples:
/// - `guide/intro.md` -> `/guide/intro`
/// - `guide/intro.typ` -> `/guide/intro`
/// - `guide/index.md` -> `/guide/`
/// - `index.md` -> `/`
fn file_path_to_route(relative_path: &Path) -> String {
    let path_str = relative_path.to_string_lossy().replace('\\', "/");

    let without_ext = strip_content_extension(&path_str);

    if without_ext == "index" {
        "/".to_string()
    } else if let Some(stripped) = without_ext.strip_suffix("/index") {
        format!("/{}/", stripped)
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

    #[test]
    fn test_typst_file_path_to_route() {
        assert_eq!(file_path_to_route(Path::new("index.typ")), "/");
        assert_eq!(file_path_to_route(Path::new("guide/index.typ")), "/guide/");
        assert_eq!(
            file_path_to_route(Path::new("guide/intro.typ")),
            "/guide/intro"
        );
    }

    #[test]
    fn test_is_content_file() {
        assert!(is_content_file("index.md"));
        assert!(is_content_file("guide/intro.typ"));
        assert!(!is_content_file("image.png"));
        assert!(!is_content_file("style.css"));
    }
}
