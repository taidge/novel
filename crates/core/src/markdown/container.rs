use crate::plugin::ContainerDirective;
use regex::Regex;

/// Container directive types
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    Tip,
    Warning,
    Danger,
    Info,
    Note,
    Details,
    Tabs,
    Steps,
}

impl ContainerType {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Tip => "tip",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Info => "info",
            Self::Note => "note",
            Self::Details => "details",
            Self::Tabs => "tabs",
            Self::Steps => "steps",
        }
    }

    pub fn default_title(&self) -> &'static str {
        match self {
            Self::Tip => "TIP",
            Self::Warning => "WARNING",
            Self::Danger => "DANGER",
            Self::Info => "INFO",
            Self::Note => "NOTE",
            Self::Details => "Details",
            Self::Tabs => "",
            Self::Steps => "",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tip" => Some(Self::Tip),
            "warning" | "caution" => Some(Self::Warning),
            "danger" => Some(Self::Danger),
            "info" => Some(Self::Info),
            "note" => Some(Self::Note),
            "details" => Some(Self::Details),
            "tabs" => Some(Self::Tabs),
            "steps" => Some(Self::Steps),
            _ => None,
        }
    }
}

/// Pre-process container directives in markdown before parsing.
///
/// Transforms:
/// ```text
/// ::: tip Custom Title
/// Content here
/// :::
/// ```
///
/// Into HTML div blocks that pulldown-cmark will pass through:
/// ```html
/// <div class="container tip">
/// <div class="container-title">Custom Title</div>
///
/// Content here
///
/// </div>
/// ```
///
/// Also supports:
/// - `::: tabs` with `== Tab Title` separators
/// - `::: steps` wrapping numbered content
/// - Custom directives provided by plugins via `custom_directives`
pub fn preprocess_containers(
    input: &str,
    custom_directives: &[Box<dyn ContainerDirective>],
) -> String {
    // Build the regex dynamically to include custom directive names
    let mut type_names: Vec<&str> = vec![
        "tip", "warning", "caution", "danger", "info", "note", "details", "tabs", "steps",
    ];
    for d in custom_directives {
        type_names.push(d.name());
    }
    let types_pattern = type_names.join("|");

    let open_re = Regex::new(&format!(r"^:::\s*({})(.*)$", types_pattern)).expect("valid regex");
    let close_re = Regex::new(r"^:::$").expect("valid regex");
    let tab_header_re = Regex::new(r"^==\s+(.+)$").expect("valid regex");

    let mut output = String::with_capacity(input.len());
    let mut in_container = false;
    let mut container_type: Option<ContainerType> = None;
    let mut is_details = false;

    // Custom directive state
    let mut active_custom_directive: Option<&dyn ContainerDirective> = None;
    let mut custom_title = String::new();
    let mut custom_body = String::new();

    // Tabs state
    let mut _tab_index: usize = 0;
    let mut in_tab_panel = false;
    let mut tab_headers: Vec<String> = Vec::new();
    let mut tab_panels: Vec<String> = Vec::new();
    let mut current_panel = String::new();

    for line in input.lines() {
        if !in_container {
            if let Some(caps) = open_re.captures(line) {
                let type_str = caps.get(1).unwrap().as_str();
                let title_part = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                // Try built-in first
                if let Some(ct) = ContainerType::from_str(type_str) {
                    is_details = ct == ContainerType::Details;

                    match ct {
                        ContainerType::Tabs => {
                            _tab_index = 0;
                            in_tab_panel = false;
                            tab_headers.clear();
                            tab_panels.clear();
                            current_panel.clear();
                            container_type = Some(ct);
                            in_container = true;
                        }
                        ContainerType::Steps => {
                            output.push_str("<div class=\"steps-container\">\n\n");
                            container_type = Some(ct);
                            in_container = true;
                        }
                        ContainerType::Details => {
                            let title = if title_part.is_empty() {
                                ct.default_title().to_string()
                            } else {
                                title_part.to_string()
                            };
                            output.push_str(&format!(
                                "<details class=\"container {}\">\n<summary>{}</summary>\n\n",
                                ct.css_class(),
                                title
                            ));
                            container_type = Some(ct);
                            in_container = true;
                        }
                        _ => {
                            let title = if title_part.is_empty() {
                                ct.default_title().to_string()
                            } else {
                                title_part.to_string()
                            };
                            output.push_str(&format!(
                                "<div class=\"container {}\">\n<div class=\"container-title\">{}</div>\n\n",
                                ct.css_class(),
                                title
                            ));
                            container_type = Some(ct);
                            in_container = true;
                        }
                    }
                } else {
                    // Try custom directives
                    let directive = custom_directives
                        .iter()
                        .find(|d| d.name().eq_ignore_ascii_case(type_str));
                    if let Some(d) = directive {
                        active_custom_directive = Some(d.as_ref());
                        custom_title = title_part.to_string();
                        custom_body.clear();
                        in_container = true;
                    }
                }
                continue;
            }
        } else if close_re.is_match(line) {
            // Closing a custom directive?
            if let Some(directive) = active_custom_directive.take() {
                let rendered = directive.render(&custom_title, &custom_body);
                output.push_str(&rendered);
                output.push('\n');
                custom_title.clear();
                custom_body.clear();
                in_container = false;
                continue;
            }

            match &container_type {
                Some(ContainerType::Tabs) => {
                    // Close last tab panel
                    if in_tab_panel {
                        tab_panels.push(current_panel.clone());
                        current_panel.clear();
                    }
                    // Build tabs HTML
                    output
                        .push_str("<div class=\"tabs-container\">\n<div class=\"tabs-header\">\n");
                    for (i, header) in tab_headers.iter().enumerate() {
                        let active = if i == 0 { " active" } else { "" };
                        output.push_str(&format!(
                            "<button class=\"tab-btn{}\" data-tab=\"{}\">{}</button>\n",
                            active, i, header
                        ));
                    }
                    output.push_str("</div>\n");
                    for (i, panel) in tab_panels.iter().enumerate() {
                        let active = if i == 0 { " active" } else { "" };
                        output.push_str(&format!(
                            "<div class=\"tab-panel{}\" data-tab=\"{}\">\n\n{}\n</div>\n",
                            active,
                            i,
                            panel.trim()
                        ));
                    }
                    output.push_str("</div>\n");
                    tab_headers.clear();
                    tab_panels.clear();
                    in_tab_panel = false;
                }
                Some(ContainerType::Steps) => {
                    output.push_str("\n</div>\n");
                }
                _ => {
                    if is_details {
                        output.push_str("\n</details>\n");
                    } else {
                        output.push_str("\n</div>\n");
                    }
                }
            }
            in_container = false;
            container_type = None;
            is_details = false;
            continue;
        }

        // Accumulating body for a custom directive
        if active_custom_directive.is_some() {
            custom_body.push_str(line);
            custom_body.push('\n');
            continue;
        }

        // Handle tab content
        if matches!(&container_type, Some(ContainerType::Tabs)) {
            if let Some(caps) = tab_header_re.captures(line) {
                let title = caps.get(1).unwrap().as_str().trim().to_string();
                if in_tab_panel {
                    tab_panels.push(current_panel.clone());
                    current_panel.clear();
                }
                tab_headers.push(title);
                _tab_index += 1;
                in_tab_panel = true;
                continue;
            }
            if in_tab_panel {
                current_panel.push_str(line);
                current_panel.push('\n');
                continue;
            }
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    // If container was never closed, close it
    if in_container {
        if let Some(directive) = active_custom_directive.take() {
            let rendered = directive.render(&custom_title, &custom_body);
            output.push_str(&rendered);
            output.push('\n');
        } else {
            match &container_type {
                Some(ContainerType::Tabs) => {
                    if in_tab_panel {
                        tab_panels.push(current_panel.clone());
                    }
                    output
                        .push_str("<div class=\"tabs-container\">\n<div class=\"tabs-header\">\n");
                    for (i, header) in tab_headers.iter().enumerate() {
                        let active = if i == 0 { " active" } else { "" };
                        output.push_str(&format!(
                            "<button class=\"tab-btn{}\" data-tab=\"{}\">{}</button>\n",
                            active, i, header
                        ));
                    }
                    output.push_str("</div>\n");
                    for (i, panel) in tab_panels.iter().enumerate() {
                        let active = if i == 0 { " active" } else { "" };
                        output.push_str(&format!(
                            "<div class=\"tab-panel{}\" data-tab=\"{}\">\n\n{}\n</div>\n",
                            active,
                            i,
                            panel.trim()
                        ));
                    }
                    output.push_str("</div>\n");
                }
                Some(ContainerType::Steps) => {
                    output.push_str("\n</div>\n");
                }
                _ => {
                    if is_details {
                        output.push_str("\n</details>\n");
                    } else {
                        output.push_str("\n</div>\n");
                    }
                }
            }
        }
    }

    // Process inline badges: {badge:TYPE|TEXT} -> <span class="badge TYPE">TEXT</span>
    let badge_re =
        Regex::new(r"\{badge:(tip|info|warning|danger|note)\|([^}]+)\}").expect("valid regex");
    let result = badge_re.replace_all(&output, |caps: &regex::Captures| {
        let badge_type = caps.get(1).unwrap().as_str();
        let badge_text = caps.get(2).unwrap().as_str();
        format!(
            "<span class=\"badge badge-{}\">{}</span>",
            badge_type, badge_text
        )
    });

    result.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_tip() {
        let input = "::: tip\nSome advice\n:::\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains(r#"class="container tip""#));
        assert!(output.contains("TIP"));
        assert!(output.contains("Some advice"));
    }

    #[test]
    fn test_container_custom_title() {
        let input = "::: warning Be careful!\nDon't do this.\n:::\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains("Be careful!"));
    }

    #[test]
    fn test_container_details() {
        let input = "::: details Click to expand\nHidden content\n:::\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains("<details"));
        assert!(output.contains("<summary>Click to expand</summary>"));
    }

    #[test]
    fn test_tabs() {
        let input =
            "::: tabs\n== npm\n```bash\nnpm install\n```\n== yarn\n```bash\nyarn add\n```\n:::\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains("tabs-container"));
        assert!(output.contains("tab-btn"));
        assert!(output.contains("npm"));
        assert!(output.contains("yarn"));
    }

    #[test]
    fn test_steps() {
        let input = "::: steps\n### Step 1\nDo this\n### Step 2\nDo that\n:::\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains("steps-container"));
    }

    #[test]
    fn test_badge_inline() {
        let input = "This is {badge:tip|New} feature\n";
        let output = preprocess_containers(input, &[]);
        assert!(output.contains(r#"<span class="badge badge-tip">New</span>"#));
    }

    #[test]
    fn test_custom_directive() {
        struct TestDirective;
        impl ContainerDirective for TestDirective {
            fn name(&self) -> &str {
                "mybox"
            }
            fn render(&self, title: &str, body: &str) -> String {
                format!("<div class=\"mybox\"><h3>{}</h3>{}</div>", title, body.trim())
            }
        }

        let directives: Vec<Box<dyn ContainerDirective>> = vec![Box::new(TestDirective)];
        let input = "::: mybox Hello\nSome content\n:::\n";
        let output = preprocess_containers(input, &directives);
        assert!(output.contains(r#"class="mybox""#));
        assert!(output.contains("Hello"));
        assert!(output.contains("Some content"));
    }
}
