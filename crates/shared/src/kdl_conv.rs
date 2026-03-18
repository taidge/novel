use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Convert a parsed [`KdlDocument`] into a [`serde_json::Value`] tree
/// so it can be deserialized into any `#[derive(Deserialize)]` struct.
///
/// ## Mapping rules
///
/// | KDL construct | JSON result |
/// |---|---|
/// | `key "val"` (single arg, no children) | `"key": "val"` |
/// | `key 42` | `"key": 42` |
/// | `key true` | `"key": true` |
/// | `key { … }` (children, no args) | `"key": { … }` (recurse) |
/// | `key a=1 b="x"` (properties, no children) | `"key": {"a":1,"b":"x"}` |
/// | `key a=1 { … }` (properties + children) | `"key": { "a":1, …children }` |
/// | repeated `key …` siblings | `"key": [ …values ]` |
/// | `- "val"` (dash-nodes) | collected into surrounding array context |
pub fn kdl_document_to_value(doc: &KdlDocument) -> Value {
    nodes_to_value(doc.nodes())
}

fn nodes_to_value(nodes: &[KdlNode]) -> Value {
    // Group nodes by name, preserving insertion order.
    let mut groups: Vec<(String, Vec<&KdlNode>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for node in nodes {
        let name = node.name().value().to_string();
        if let Some(&idx) = index.get(&name) {
            groups[idx].1.push(node);
        } else {
            index.insert(name.clone(), groups.len());
            groups.push((name, vec![node]));
        }
    }

    let mut map = Map::new();

    for (name, nodes) in groups {
        if name == "-" {
            // Dash-nodes are a KDL idiom for anonymous array items.
            let arr: Vec<Value> = nodes.iter().map(|n| node_to_value(n)).collect();
            // Store under a special key; the parent will hoist this.
            map.insert("-".to_string(), Value::Array(arr));
        } else if nodes.len() == 1 {
            map.insert(name, node_to_value(nodes[0]));
        } else {
            // Multiple sibling nodes with the same name → array.
            let arr: Vec<Value> = nodes.iter().map(|n| node_to_value(n)).collect();
            map.insert(name, Value::Array(arr));
        }
    }

    Value::Object(map)
}

fn node_to_value(node: &KdlNode) -> Value {
    let positional: Vec<&KdlEntry> = node
        .entries()
        .iter()
        .filter(|e| e.name().is_none())
        .collect();
    let properties: Vec<&KdlEntry> = node
        .entries()
        .iter()
        .filter(|e| e.name().is_some())
        .collect();
    let children = node.children();

    let has_children = children.is_some_and(|c| !c.nodes().is_empty());
    let has_properties = !properties.is_empty();

    // Case 1: Simple scalar — single positional arg, no properties, no children.
    if positional.len() == 1 && !has_properties && !has_children {
        return kdl_value_to_json(positional[0].value());
    }

    // Case 2: Multiple positional args, no properties, no children → array.
    if positional.len() > 1 && !has_properties && !has_children {
        return Value::Array(
            positional
                .iter()
                .map(|e| kdl_value_to_json(e.value()))
                .collect(),
        );
    }

    // Case 3: Object — has properties and/or children.
    let mut map = Map::new();

    // Properties become object keys.
    for prop in &properties {
        let key = prop.name().unwrap().value().to_string();
        map.insert(key, kdl_value_to_json(prop.value()));
    }

    // Recurse into children.
    if let Some(child_doc) = children {
        let child_val = nodes_to_value(child_doc.nodes());
        if let Value::Object(child_map) = child_val {
            // If children produced dash-nodes, hoist them as inline array.
            for (k, v) in child_map {
                if k == "-" {
                    // Dash-node items: if no properties on this node, return
                    // the array directly; otherwise merge as items.
                    if map.is_empty() {
                        return v;
                    }
                } else {
                    map.insert(k, v);
                }
            }
        }
    }

    // Case 4: No properties, no children, no args → null.
    if map.is_empty() && positional.is_empty() {
        return Value::Null;
    }

    Value::Object(map)
}

fn kdl_value_to_json(val: &KdlValue) -> Value {
    match val {
        KdlValue::String(s) => Value::String(s.clone()),
        KdlValue::Integer(i) => {
            // serde_json::Number only supports i64/u64/f64.
            if let Ok(n) = i64::try_from(*i) {
                Value::Number(n.into())
            } else if let Ok(n) = u64::try_from(*i) {
                Value::Number(n.into())
            } else {
                // Fallback: store as string for very large integers.
                Value::String(i.to_string())
            }
        }
        KdlValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        KdlValue::Bool(b) => Value::Bool(*b),
        KdlValue::Null => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars() {
        let doc = KdlDocument::parse(r#"title "Hello""#).unwrap();
        let val = kdl_document_to_value(&doc);
        assert_eq!(val["title"], Value::String("Hello".into()));
    }

    #[test]
    fn nested_object() {
        let doc = KdlDocument::parse(
            r#"
            theme {
                dark_mode #true
                footer "Built with Novel"
            }
            "#,
        )
        .unwrap();
        let val = kdl_document_to_value(&doc);
        assert_eq!(val["theme"]["dark_mode"], Value::Bool(true));
        assert_eq!(
            val["theme"]["footer"],
            Value::String("Built with Novel".into())
        );
    }

    #[test]
    fn repeated_nodes_become_array() {
        let doc = KdlDocument::parse(
            r#"
            theme {
                nav text="Guide" link="/guide/"
                nav text="API" link="/api/"
            }
            "#,
        )
        .unwrap();
        let val = kdl_document_to_value(&doc);
        let nav = val["theme"]["nav"].as_array().unwrap();
        assert_eq!(nav.len(), 2);
        assert_eq!(nav[0]["text"], Value::String("Guide".into()));
        assert_eq!(nav[1]["link"], Value::String("/api/".into()));
    }

    #[test]
    fn dash_nodes_become_array() {
        let doc = KdlDocument::parse(
            r#"
            locales {
                - code="en" name="English" dir="en"
                - code="zh" name="中文" dir="zh"
            }
            "#,
        )
        .unwrap();
        let val = kdl_document_to_value(&doc);
        let locales = val["locales"].as_array().unwrap();
        assert_eq!(locales.len(), 2);
        assert_eq!(locales[0]["code"], Value::String("en".into()));
    }

    #[test]
    fn integer_and_bool() {
        let doc = KdlDocument::parse(
            r#"
            port 3000
            enabled #true
            "#,
        )
        .unwrap();
        let val = kdl_document_to_value(&doc);
        assert_eq!(val["port"], serde_json::json!(3000));
        assert_eq!(val["enabled"], Value::Bool(true));
    }

    #[test]
    fn full_site_config() {
        use crate::config::SiteConfig;

        let kdl_input = r#"
title "My Docs"
description "A documentation site"
root "docs"
out_dir "dist"
base "/"
lang "en"
clean_urls #false
asset_fingerprint #false
template_engine "minijinja"

theme {
    dark_mode #true
    footer "Built with Novel"
    last_updated #true
    source_link "https://github.com/user/repo"

    nav {
        - text="Guide" link="/guide/"
        - text="API" link="/api/"
    }

    social_links {
        - icon="github" link="https://github.com/user/repo"
    }
}

markdown {
    math #true
    mermaid #false
    show_line_numbers #false
    syntax_theme "base16-ocean.dark"
}

redirects {
    "/old-page" "/new-page"
    "/another" "/target"
}
        "#;

        let config = SiteConfig::from_kdl(kdl_input).unwrap();
        assert_eq!(config.title, "My Docs");
        assert_eq!(config.description, "A documentation site");
        assert!(config.theme.dark_mode);
        assert_eq!(config.theme.footer, Some("Built with Novel".into()));
        assert_eq!(config.theme.nav.len(), 2);
        assert_eq!(config.theme.nav[0].text, "Guide");
        assert_eq!(config.theme.nav[1].link, "/api/");
        assert!(config.markdown.math);
        assert!(!config.markdown.mermaid);
        assert_eq!(config.redirects.get("/old-page").unwrap(), "/new-page");
    }
}
