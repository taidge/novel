use anyhow::Result;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use novel_shared::{FrontMatter, PageData, RouteMeta, TocItem};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};
use slug::slugify;
use std::path::Path;

use super::container::preprocess_containers;
use super::file_embed::{parse_file_embed, read_embedded_file};
use super::highlight::highlight_code;
use crate::plugin::ContainerDirective;

/// Main markdown processing engine
pub struct MarkdownProcessor {
    project_root: Option<std::path::PathBuf>,
    show_line_numbers: bool,
    custom_directives: Vec<Box<dyn ContainerDirective>>,
}

impl MarkdownProcessor {
    pub fn new(project_root: Option<&Path>) -> Self {
        Self {
            project_root: project_root.map(|p| p.to_path_buf()),
            show_line_numbers: false,
            custom_directives: Vec::new(),
        }
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn with_custom_directives(mut self, directives: Vec<Box<dyn ContainerDirective>>) -> Self {
        self.custom_directives = directives;
        self
    }

    /// Process a markdown file into PageData
    pub fn process_file(&self, file_path: &Path, route: RouteMeta) -> Result<PageData> {
        let raw_content = std::fs::read_to_string(file_path)?;
        self.process_string(&raw_content, file_path, route)
    }

    /// Process a markdown string into PageData
    pub fn process_string(
        &self,
        raw_content: &str,
        file_path: &Path,
        route: RouteMeta,
    ) -> Result<PageData> {
        // 1. Parse frontmatter
        let matter = Matter::<YAML>::new();
        let (frontmatter, markdown_body) = match matter.parse(raw_content) {
            Ok(parsed) => {
                let fm: FrontMatter = parsed
                    .data
                    .and_then(|d: gray_matter::Pod| d.deserialize().ok())
                    .unwrap_or_default();
                (fm, parsed.content)
            }
            Err(_) => (FrontMatter::default(), raw_content.to_string()),
        };

        // 2. Pre-process container directives (including tabs, steps, badges)
        let processed = preprocess_containers(&markdown_body, &self.custom_directives);

        // 3. Parse markdown and collect events
        let options = Options::ENABLE_GFM
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_MATH;

        let parser = Parser::new_ext(&processed, options);
        let file_dir = file_path.parent().unwrap_or(Path::new("."));

        let mut toc: Vec<TocItem> = Vec::new();
        let mut first_h1: Option<String> = None;
        let mut events: Vec<Event> = Vec::new();
        let mut in_heading = false;
        let mut _heading_depth: u32 = 0;
        let mut heading_text = String::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_info = String::new();
        let mut code_content = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    _heading_depth = level as u32;
                    heading_text.clear();
                    events.push(event);
                }
                Event::End(TagEnd::Heading(level)) => {
                    in_heading = false;
                    let id = slugify(&heading_text);
                    if level as u32 == 1 && first_h1.is_none() {
                        first_h1 = Some(heading_text.clone());
                    }
                    if (2..=4).contains(&(level as u32)) {
                        toc.push(TocItem {
                            id: id.clone(),
                            text: heading_text.clone(),
                            depth: level as u32,
                        });
                    }
                    // Replace the start event with one that includes the id
                    let idx = events.len();
                    let mut start_idx = idx;
                    for i in (0..idx).rev() {
                        if matches!(events[i], Event::Start(Tag::Heading { .. })) {
                            start_idx = i;
                            break;
                        }
                    }
                    let inner_html = render_events_to_html(&events[start_idx + 1..]);
                    events.truncate(start_idx);
                    let h_tag = format!("h{}", level as u32);
                    events.push(Event::Html(CowStr::from(format!(
                        "<{} id=\"{}\">{} <a class=\"header-anchor\" href=\"#{}\">#</a></{}>",
                        h_tag, id, inner_html, id, h_tag
                    ))));
                }
                Event::Text(ref text) if in_heading => {
                    heading_text.push_str(text);
                    events.push(event);
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_content.clear();
                    match &kind {
                        CodeBlockKind::Fenced(info) => {
                            let info_str = info.as_ref();
                            code_lang =
                                info_str.split_whitespace().next().unwrap_or("").to_string();
                            code_info = info_str.to_string();
                        }
                        CodeBlockKind::Indented => {
                            code_lang.clear();
                            code_info.clear();
                        }
                    }
                }
                Event::Text(ref text) if in_code_block => {
                    code_content.push_str(text);
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;

                    // Mermaid code blocks: render as <pre class="mermaid">
                    if code_lang == "mermaid" {
                        events.push(Event::Html(CowStr::from(format!(
                            "<pre class=\"mermaid\">{}</pre>",
                            html_escape(&code_content)
                        ))));
                        code_lang.clear();
                        code_info.clear();
                        code_content.clear();
                    } else {

                    // Check for file embed (requires project_root)
                    if let Some(ref project_root) = self.project_root {
                        if let Some(embed) = parse_file_embed(&code_info) {
                            match read_embedded_file(&embed, file_dir, project_root) {
                                Ok(file_content) => {
                                    code_content = file_content;
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to embed file: {}", e);
                                    code_content = format!("Error embedding file: {}", e);
                                }
                            }
                        }
                    }

                    // Parse title from info string
                    let title = parse_code_title(&code_info);

                    // Parse highlighted lines from info string {1,3-5}
                    let highlighted_lines = parse_highlighted_lines(&code_info);

                    // Check if line numbers should be shown
                    let show_ln = self.show_line_numbers || code_info.contains("showLineNumbers");

                    // Check if this is a diff
                    let is_diff = code_lang == "diff" || code_info.contains("diff");

                    // Syntax highlight
                    let effective_lang = if is_diff && code_lang == "diff" {
                        // Try to detect actual language from content
                        ""
                    } else {
                        &code_lang
                    };
                    let highlighted = highlight_code(&code_content, effective_lang);

                    // Build HTML with line features
                    let html_output = build_code_block_html(
                        &highlighted,
                        &code_content,
                        &code_lang,
                        title.as_deref(),
                        &highlighted_lines,
                        show_ln,
                        is_diff,
                    );

                    events.push(Event::Html(CowStr::from(html_output)));
                    }
                }
                // External links: add target="_blank"
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    if dest_url.starts_with("http://") || dest_url.starts_with("https://") {
                        events.push(Event::Html(CowStr::from(format!(
                            "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">",
                            dest_url
                        ))));
                    } else {
                        events.push(Event::Start(Tag::Link {
                            link_type,
                            dest_url,
                            title,
                            id,
                        }));
                    }
                }
                Event::End(TagEnd::Link) => {
                    events.push(Event::Html(CowStr::from("</a>")));
                }
                // Images: add lazy loading and zoom support
                Event::Start(Tag::Image {
                    dest_url, title: _, ..
                }) => {
                    events.push(Event::Html(CowStr::from(format!(
                        "<img src=\"{}\" alt=\"",
                        dest_url
                    ))));
                    // We'll collect alt text and close the tag in End(Image)
                    // Actually, let's handle it differently - push a marker
                }
                Event::End(TagEnd::Image) => {
                    events.push(Event::Html(CowStr::from(
                        "\" loading=\"lazy\" class=\"zoomable\">",
                    )));
                }
                // Math: inline $...$ and display $$...$$
                Event::InlineMath(text) => {
                    events.push(Event::Html(CowStr::from(format!(
                        "<span class=\"math math-inline\">{}</span>",
                        html_escape(&text)
                    ))));
                }
                Event::DisplayMath(text) => {
                    events.push(Event::Html(CowStr::from(format!(
                        "<div class=\"math math-display\">{}</div>",
                        html_escape(&text)
                    ))));
                }
                _ => {
                    events.push(event);
                }
            }
        }

        let content_html = render_events_to_html(&events);

        // Determine title: frontmatter > h1 > first toc entry > filename
        let title = frontmatter
            .title
            .clone()
            .or(first_h1)
            .or_else(|| toc.first().map(|t| t.text.clone()))
            .unwrap_or_else(|| route.page_name.clone());

        let description = frontmatter.description.clone().unwrap_or_default();

        Ok(PageData {
            route,
            title,
            description,
            content_html,
            toc,
            frontmatter,
            last_updated: None,
            prev_page: None,
            next_page: None,
        })
    }
}

/// Build the complete code block HTML with line numbers, highlighting, and diff support
fn build_code_block_html(
    highlighted_html: &str,
    raw_code: &str,
    lang: &str,
    title: Option<&str>,
    highlighted_lines: &[usize],
    show_line_numbers: bool,
    is_diff: bool,
) -> String {
    let lines: Vec<&str> = raw_code.lines().collect();

    let mut html = String::new();

    // Wrapper div
    let mut classes = vec!["code-block".to_string()];
    if show_line_numbers {
        classes.push("with-line-numbers".to_string());
    }
    if is_diff {
        classes.push("diff".to_string());
    }
    html.push_str(&format!("<div class=\"{}\">", classes.join(" ")));

    // Title bar
    if let Some(title) = title {
        html.push_str(&format!("<div class=\"code-block-title\">{}</div>", title));
    }

    // Language label
    if !lang.is_empty() {
        html.push_str(&format!("<span class=\"code-lang-label\">{}</span>", lang));
    }

    // Copy button
    html.push_str("<button class=\"copy-btn\" onclick=\"navigator.clipboard.writeText(this.parentElement.querySelector('pre').textContent)\">Copy</button>");

    // If we need line numbers or highlighted lines, wrap in a custom structure
    if show_line_numbers || !highlighted_lines.is_empty() || is_diff {
        html.push_str("<pre><code>");
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let mut line_classes = Vec::new();

            if highlighted_lines.contains(&line_num) {
                line_classes.push("highlighted");
            }

            if is_diff {
                if line.starts_with('+') {
                    line_classes.push("diff-add");
                } else if line.starts_with('-') {
                    line_classes.push("diff-remove");
                }
            }

            let class_attr = if line_classes.is_empty() {
                String::new()
            } else {
                format!(" class=\"{}\"", line_classes.join(" "))
            };

            if show_line_numbers {
                html.push_str(&format!("<span class=\"code-line\"{}>", class_attr));
                html.push_str(&format!("<span class=\"line-number\">{}</span>", line_num));
                html.push_str(&format!(
                    "<span class=\"line-content\">{}</span>",
                    html_escape(line)
                ));
                html.push_str("</span>\n");
            } else {
                html.push_str(&format!(
                    "<span class=\"code-line\"{}>{}</span>\n",
                    class_attr,
                    html_escape(line)
                ));
            }
        }
        html.push_str("</code></pre>");
    } else {
        // Use syntect highlighted output directly
        html.push_str(highlighted_html);
    }

    html.push_str("</div>");
    html
}

/// Parse highlighted line numbers from code fence info string
/// Supports: {1,3-5,8} or {1, 3-5, 8}
fn parse_highlighted_lines(info: &str) -> Vec<usize> {
    let re = regex::Regex::new(r"\{([0-9,\-\s]+)\}").ok();
    let re = match re {
        Some(r) => r,
        None => return Vec::new(),
    };

    let caps = match re.captures(info) {
        Some(c) => c,
        None => return Vec::new(),
    };

    let spec = caps.get(1).unwrap().as_str();
    let mut lines = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if let Some((start_s, end_s)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (
                start_s.trim().parse::<usize>(),
                end_s.trim().parse::<usize>(),
            ) {
                for n in start..=end {
                    lines.push(n);
                }
            }
        } else if let Ok(n) = part.parse::<usize>() {
            lines.push(n);
        }
    }

    lines
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render a slice of pulldown-cmark events to an HTML string
fn render_events_to_html(events: &[Event]) -> String {
    let mut html_output = String::new();
    html::push_html(&mut html_output, events.iter().cloned());
    html_output
}

/// Parse title="..." from code fence info string
fn parse_code_title(info: &str) -> Option<String> {
    let re = regex::Regex::new(r#"title="([^"]+)""#).ok()?;
    re.captures(info)
        .map(|c| c.get(1).unwrap().as_str().to_string())
}

/// Collect all internal links from content HTML for dead link checking
pub fn collect_internal_links(html: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"href="(/[^"]*?)""#).unwrap();
    re.captures_iter(html)
        .filter_map(|caps| {
            let link = caps.get(1)?.as_str().to_string();
            // Skip anchor-only links
            if link.starts_with("/#") {
                None
            } else {
                Some(link)
            }
        })
        .collect()
}
