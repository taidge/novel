use crate::plugin::{ContainerDirective, Plugin};
use std::sync::LazyLock;

static TITLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"title="([^"]+)""#).unwrap());

/// Plugin that provides the `::: code-group` container directive.
///
/// Parses consecutive fenced code blocks and renders them as a tabbed
/// code panel, using the language or title as the tab label.
pub struct CodeGroupPlugin;

impl Plugin for CodeGroupPlugin {
    fn name(&self) -> &str {
        "code-group"
    }

    fn container_directives(&self) -> Vec<Box<dyn ContainerDirective>> {
        vec![Box::new(CodeGroupDirective)]
    }
}

struct CodeGroupDirective;

impl ContainerDirective for CodeGroupDirective {
    fn name(&self) -> &str {
        "code-group"
    }

    fn render(&self, _title: &str, body: &str) -> String {
        // Parse code blocks from the body.
        // Expected format:
        //   ```lang [title="Tab Name"]
        //   code
        //   ```
        let mut tabs: Vec<(String, String, String)> = Vec::new(); // (label, lang, code)
        let mut current_lang = String::new();
        let mut current_label = String::new();
        let mut current_code = String::new();
        let mut in_block = false;

        for line in body.lines() {
            if !in_block {
                if let Some(rest) = line.strip_prefix("```") {
                    in_block = true;
                    let info = rest.trim();
                    let lang = info.split_whitespace().next().unwrap_or("").to_string();

                    // Try to extract title="..." as label
                    let label = TITLE_RE
                        .captures(info)
                        .map(|c| c.get(1).unwrap().as_str().to_string())
                        .unwrap_or_else(|| {
                            if lang.is_empty() {
                                "Code".to_string()
                            } else {
                                lang.clone()
                            }
                        });

                    current_lang = lang;
                    current_label = label;
                    current_code.clear();
                }
            } else if line.trim() == "```" {
                in_block = false;
                tabs.push((
                    current_label.clone(),
                    current_lang.clone(),
                    current_code.clone(),
                ));
            } else {
                current_code.push_str(line);
                current_code.push('\n');
            }
        }

        // If no tabs found, return body as-is in a div
        if tabs.is_empty() {
            return format!("<div class=\"code-group\">{}</div>", body);
        }

        let mut html = String::from("<div class=\"code-group\">\n<div class=\"tabs-header\">\n");
        for (i, (label, _, _)) in tabs.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            html.push_str(&format!(
                "<button class=\"tab-btn{}\" data-tab=\"{}\">{}</button>\n",
                active, i, label
            ));
        }
        html.push_str("</div>\n");

        for (i, (_, lang, code)) in tabs.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            let lang_attr = if lang.is_empty() {
                String::new()
            } else {
                format!(" class=\"language-{}\"", lang)
            };
            html.push_str(&format!(
                "<div class=\"tab-panel{}\" data-tab=\"{}\">\n<pre><code{}>{}</code></pre>\n</div>\n",
                active,
                i,
                lang_attr,
                html_escape(code.trim_end())
            ));
        }

        html.push_str("</div>");
        html
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
