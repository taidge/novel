use crate::plugin::{ContainerDirective, Plugin};
use crate::util::html_escape;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};

static TITLE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"title="([^"]+)""#).expect("valid regex"));

/// Monotonic group counter so each rendered `::: code-group` gets a unique
/// id prefix for its ARIA relationships. Monotonic across the whole
/// process is fine: the ids only need to be unique within a single page,
/// and even if two pages happen to use the same id the pages are never
/// loaded together in the same DOM. (T-UI-7)
static CODE_GROUP_COUNTER: AtomicUsize = AtomicUsize::new(0);

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
                        .map(|c| c.get(1).expect("regex has 1 group").as_str().to_string())
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

        let group = CODE_GROUP_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut html =
            String::from("<div class=\"code-group\">\n<div class=\"tabs-header\" role=\"tablist\">\n");
        for (i, (label, _, _)) in tabs.iter().enumerate() {
            let is_active = i == 0;
            let active_cls = if is_active { " active" } else { "" };
            let selected = if is_active { "true" } else { "false" };
            let tabindex = if is_active { "0" } else { "-1" };
            html.push_str(&format!(
                "<button class=\"tab-btn{active_cls}\" role=\"tab\" \
                 id=\"cg-tab-{group}-{i}\" data-tab=\"{i}\" \
                 aria-selected=\"{selected}\" aria-controls=\"cg-panel-{group}-{i}\" \
                 tabindex=\"{tabindex}\">{label}</button>\n"
            ));
        }
        html.push_str("</div>\n");

        for (i, (_, lang, code)) in tabs.iter().enumerate() {
            let is_active = i == 0;
            let active_cls = if is_active { " active" } else { "" };
            let hidden_attr = if is_active { "" } else { " hidden" };
            let lang_attr = if lang.is_empty() {
                String::new()
            } else {
                format!(" class=\"language-{}\"", lang)
            };
            html.push_str(&format!(
                "<div class=\"tab-panel{active_cls}\" role=\"tabpanel\" \
                 id=\"cg-panel-{group}-{i}\" data-tab=\"{i}\" \
                 aria-labelledby=\"cg-tab-{group}-{i}\"{hidden_attr}>\n\
                 <pre><code{lang_attr}>{code}</code></pre>\n</div>\n",
                code = html_escape(code.trim_end())
            ));
        }

        html.push_str("</div>");
        html
    }
}
