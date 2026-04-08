//! Data file loading. Reads `<docs_root>/data/**/*.{toml,json}` into a nested
//! JSON value keyed by file stem (and subdirectory). Made available to
//! templates via `site_data` in the render context.

use crate::error::{NovelError, NovelResult};
use serde_json::{Map, Value};
use std::path::Path;
use walkdir::WalkDir;

/// Load all data files under `<docs_root>/data/`. Returns a JSON object
/// where keys are file stems (subdirectories nest deeper).
///
/// Missing `data/` directory is not an error — returns an empty object.
pub fn load_data(docs_root: &Path) -> NovelResult<Value> {
    let data_dir = docs_root.join("data");
    if !data_dir.is_dir() {
        return Ok(Value::Object(Map::new()));
    }

    let mut root = Map::new();
    for entry in WalkDir::new(&data_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let value: Value = match ext {
            "toml" => {
                let s = std::fs::read_to_string(path)?;
                let v: toml::Value = toml::from_str(&s).map_err(|e| NovelError::Data {
                    file: path.display().to_string(),
                    message: e.to_string(),
                })?;
                serde_json::to_value(v).map_err(|e| NovelError::Data {
                    file: path.display().to_string(),
                    message: e.to_string(),
                })?
            }
            "json" => {
                let s = std::fs::read_to_string(path)?;
                serde_json::from_str(&s).map_err(|e| NovelError::Data {
                    file: path.display().to_string(),
                    message: e.to_string(),
                })?
            }
            _ => continue,
        };

        // Build nested key path from path relative to data_dir
        let rel = path.strip_prefix(&data_dir).unwrap_or(path);
        let mut segments: Vec<String> = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(String::from))
            .collect();
        if let Some(last) = segments.last_mut() {
            // strip extension
            if let Some(stem) = Path::new(last).file_stem().and_then(|s| s.to_str()) {
                *last = stem.to_string();
            }
        }
        insert_nested(&mut root, &segments, value);
    }
    Ok(Value::Object(root))
}

fn insert_nested(map: &mut Map<String, Value>, keys: &[String], value: Value) {
    if keys.is_empty() {
        return;
    }
    if keys.len() == 1 {
        map.insert(keys[0].clone(), value);
        return;
    }
    let entry = map
        .entry(keys[0].clone())
        .or_insert_with(|| Value::Object(Map::new()));
    if let Value::Object(inner) = entry {
        insert_nested(inner, &keys[1..], value);
    }
}
