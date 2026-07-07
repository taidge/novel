use anyhow::Result;
use gray_matter::Matter;
use gray_matter::engine::YAML;
use novel_shared::{FrontMatter, PageData, RouteMeta, TocItem};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};
use regex::Regex;
use slug::slugify;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use super::container::preprocess_containers;
use super::file_embed::{parse_file_embed, read_embedded_file};
use super::highlight::highlight_code;
use crate::plugin::ContainerDirective;
use crate::util::html_escape;

// Compile-time regex constants — built once per process, not per code fence.
static HIGHLIGHTED_LINES_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([0-9,\-\s]+)\}").expect("valid regex"));
static CODE_TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"title="([^"]+)""#).expect("valid regex"));
static INTERNAL_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?:href|src)="(/[^"]*?)""#).expect("valid regex"));

/// Main markdown processing engine
pub struct MarkdownProcessor {
    project_root: Option<std::path::PathBuf>,
    source_root: Option<std::path::PathBuf>,
    show_line_numbers: bool,
    wrap_code: bool,
    summary_separator: String,
    enable_math: bool,
    enable_mermaid: bool,
    syntax_theme: String,
    custom_directives: Vec<Box<dyn ContainerDirective>>,
}

impl MarkdownProcessor {
    pub fn new(project_root: Option<&Path>) -> Self {
        Self {
            project_root: project_root.map(|p| p.to_path_buf()),
            source_root: None,
            show_line_numbers: false,
            wrap_code: false,
            summary_separator: "<!-- more -->".to_string(),
            enable_math: false,
            enable_mermaid: false,
            syntax_theme: "base16-ocean.dark".to_string(),
            custom_directives: Vec::new(),
        }
    }

    pub fn with_source_root(mut self, source_root: Option<&Path>) -> Self {
        self.source_root = source_root.map(|p| p.to_path_buf());
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn with_wrap_code(mut self, wrap: bool) -> Self {
        self.wrap_code = wrap;
        self
    }

    pub fn with_summary_separator(mut self, separator: impl Into<String>) -> Self {
        self.summary_separator = separator.into();
        self
    }

    pub fn with_math(mut self, enable: bool) -> Self {
        self.enable_math = enable;
        self
    }

    pub fn with_mermaid(mut self, enable: bool) -> Self {
        self.enable_mermaid = enable;
        self
    }

    pub fn with_syntax_theme(mut self, theme: String) -> Self {
        self.syntax_theme = theme;
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

        // 2a. Extract summary from <!-- more --> separator (if present)
        let (summary_md, body_for_processing) = if !self.summary_separator.is_empty()
            && let Some(idx) = markdown_body.find(&self.summary_separator)
        {
            (
                Some(markdown_body[..idx].to_string()),
                markdown_body.clone(),
            )
        } else {
            (None, markdown_body.clone())
        };

        // 2. Pre-process container directives (including tabs, steps, badges)
        let processed = preprocess_containers(&body_for_processing, &self.custom_directives);

        // 3. Parse markdown and collect events
        let mut options = Options::ENABLE_GFM
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_FOOTNOTES;
        if self.enable_math {
            options |= Options::ENABLE_MATH;
        }

        let parser = Parser::new_ext(&processed, options);
        let source_file_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else if let Some(ref source_root) = self.source_root {
            source_root.join(file_path)
        } else {
            file_path.to_path_buf()
        };
        let file_dir = source_file_path.parent().unwrap_or(Path::new("."));

        let mut toc: Vec<TocItem> = Vec::new();
        let mut heading_ids: HashMap<String, usize> = HashMap::new();
        let mut first_h1: Option<String> = None;
        let mut events: Vec<Event> = Vec::new();
        let mut in_heading = false;
        let mut _heading_depth: u32 = 0;
        let mut heading_text = String::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_info = String::new();
        let mut code_content = String::new();
        let mut in_image = false;
        let mut image_dest = String::new();
        let mut image_alt = String::new();

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
                    let id = unique_heading_id(&heading_text, &mut heading_ids);
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
                Event::Text(ref text) if in_image => {
                    image_alt.push_str(text);
                    if in_heading {
                        heading_text.push_str(text);
                    }
                }
                Event::Code(ref code) if in_image => {
                    image_alt.push_str(code);
                    if in_heading {
                        heading_text.push_str(code);
                    }
                }
                Event::SoftBreak | Event::HardBreak if in_image => {
                    image_alt.push(' ');
                    if in_heading {
                        heading_text.push(' ');
                    }
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
                    if self.enable_mermaid && code_lang == "mermaid" {
                        events.push(Event::Html(CowStr::from(format!(
                            "<pre class=\"mermaid\">{}</pre>",
                            html_escape(&code_content)
                        ))));
                        code_lang.clear();
                        code_info.clear();
                        code_content.clear();
                    } else {
                        // Check for file embed (requires project_root)
                        if let Some(ref project_root) = self.project_root
                            && let Some(embed) = parse_file_embed(&code_info)
                        {
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

                        // Parse title from info string
                        let title = parse_code_title(&code_info);

                        // Parse highlighted lines from info string {1,3-5}
                        let highlighted_lines = parse_highlighted_lines(&code_info);

                        // Check if line numbers should be shown
                        let show_ln =
                            self.show_line_numbers || code_info.contains("showLineNumbers");
                        let wrap_code = self.wrap_code || code_info.contains("wrap");

                        // Check if this is a diff
                        let is_diff = code_lang == "diff" || code_info.contains("diff");

                        // Syntax highlight
                        let effective_lang = if is_diff && code_lang == "diff" {
                            // Try to detect actual language from content
                            ""
                        } else {
                            &code_lang
                        };
                        let highlighted =
                            highlight_code(&code_content, effective_lang, &self.syntax_theme);

                        // Build HTML with line features
                        let html_output = build_code_block_html(CodeBlockRenderOptions {
                            highlighted_html: &highlighted,
                            raw_code: &code_content,
                            lang: &code_lang,
                            title: title.as_deref(),
                            highlighted_lines: &highlighted_lines,
                            show_line_numbers: show_ln,
                            wrap_code,
                            is_diff,
                        });

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
                            html_escape(&dest_url)
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
                Event::Start(Tag::Image { dest_url, .. }) => {
                    in_image = true;
                    image_dest = dest_url.to_string();
                    image_alt.clear();
                }
                Event::End(TagEnd::Image) => {
                    in_image = false;
                    events.push(Event::Html(CowStr::from(format!(
                        "<img src=\"{}\" alt=\"{}\" loading=\"lazy\" class=\"zoomable\">",
                        html_escape(&image_dest),
                        html_escape(&image_alt)
                    ))));
                    image_dest.clear();
                    image_alt.clear();
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

        // Build summary HTML: prefer frontmatter.summary, then <!-- more --> split, else None
        let summary_html = if let Some(s) = frontmatter.summary.clone() {
            Some(render_simple_markdown(&s))
        } else {
            summary_md.map(|md| render_simple_markdown(&md))
        };

        let published_at = frontmatter.published_at.clone();

        Ok(PageData {
            route,
            title,
            description,
            content_html,
            toc,
            frontmatter,
            git_updated_at: None,
            prev_page: None,
            next_page: None,
            reading_time: None,
            word_count: None,
            breadcrumbs: Vec::new(),
            summary_html,
            collection: None,
            published_at,
            translations: Vec::new(),
            version_links: Vec::new(),
        })
    }
}

/// Build the complete code block HTML with line numbers, highlighting, and diff support
struct CodeBlockRenderOptions<'a> {
    highlighted_html: &'a str,
    raw_code: &'a str,
    lang: &'a str,
    title: Option<&'a str>,
    highlighted_lines: &'a [usize],
    show_line_numbers: bool,
    wrap_code: bool,
    is_diff: bool,
}

fn build_code_block_html(opts: CodeBlockRenderOptions<'_>) -> String {
    let lines: Vec<&str> = opts.raw_code.lines().collect();

    let mut html = String::new();

    // Wrapper div
    let mut classes = vec!["code-block".to_string()];
    if opts.show_line_numbers {
        classes.push("with-line-numbers".to_string());
    }
    if opts.wrap_code {
        classes.push("wrap-code".to_string());
    }
    if opts.is_diff {
        classes.push("diff".to_string());
    }
    html.push_str(&format!("<div class=\"{}\">", classes.join(" ")));

    // Title bar
    if let Some(title) = opts.title {
        html.push_str(&format!(
            "<div class=\"code-block-title\">{}</div>",
            html_escape(title)
        ));
    }

    // Header with language label and copy button
    let has_header = !opts.lang.is_empty();
    if has_header {
        html.push_str("<div class=\"code-block-header\">");
        html.push_str(&format!(
            "<span class=\"code-lang-label\">{}</span>",
            html_escape(opts.lang)
        ));
        html.push_str("<button class=\"copy-btn\" onclick=\"navigator.clipboard.writeText(this.closest('.code-block').querySelector('pre').textContent)\">Copy</button>");
        html.push_str("</div>");
    } else {
        html.push_str("<button class=\"copy-btn\" onclick=\"navigator.clipboard.writeText(this.closest('.code-block').querySelector('pre').textContent)\">Copy</button>");
    }

    // If we need line numbers or highlighted lines, wrap in a custom structure
    if opts.show_line_numbers || !opts.highlighted_lines.is_empty() || opts.is_diff {
        html.push_str("<pre><code>");
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let mut line_classes = Vec::new();

            if opts.highlighted_lines.contains(&line_num) {
                line_classes.push("highlighted");
            }

            if opts.is_diff {
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

            if opts.show_line_numbers {
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
        html.push_str(opts.highlighted_html);
    }

    html.push_str("</div>");
    html
}

/// Parse highlighted line numbers from code fence info string
/// Supports: {1,3-5,8} or {1, 3-5, 8}
fn parse_highlighted_lines(info: &str) -> Vec<usize> {
    let Some(caps) = HIGHLIGHTED_LINES_RE.captures(info) else {
        return Vec::new();
    };
    // Safe: the regex has exactly one capture group.
    let spec = caps.get(1).expect("regex has 1 group").as_str();
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

/// Minimal markdown -> HTML for summaries (no plugins / highlight).
fn render_simple_markdown(md: &str) -> String {
    let parser = Parser::new_ext(md, Options::ENABLE_GFM | Options::ENABLE_TABLES);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

/// Render a slice of pulldown-cmark events to an HTML string
fn render_events_to_html(events: &[Event]) -> String {
    let mut html_output = String::new();
    html::push_html(&mut html_output, events.iter().cloned());
    html_output
}

fn unique_heading_id(text: &str, seen: &mut HashMap<String, usize>) -> String {
    let mut base = slugify(text);
    if base.is_empty() {
        base = "section".to_string();
    }
    let count = seen.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        format!("{}-{}", base, count)
    }
}

/// Parse title="..." from code fence info string
fn parse_code_title(info: &str) -> Option<String> {
    CODE_TITLE_RE
        .captures(info)
        .map(|c| c.get(1).expect("regex has 1 group").as_str().to_string())
}

/// Collect all internal links from content HTML for dead link checking
pub fn collect_internal_links(html: &str) -> Vec<String> {
    INTERNAL_LINK_RE
        .captures_iter(html)
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

#[cfg(test)]
mod tests {
    use super::*;
    use novel_shared::RouteMeta;

    fn route() -> RouteMeta {
        RouteMeta {
            route_path: "/test".to_string(),
            absolute_path: "test.md".to_string(),
            relative_path: "test.md".to_string(),
            page_name: "test".to_string(),
            locale: None,
            version: None,
        }
    }

    #[test]
    fn repeated_headings_get_unique_ids() {
        let page = MarkdownProcessor::new(None)
            .process_string(
                "# Page\n\n## Intro\n\n## Intro\n",
                Path::new("test.md"),
                route(),
            )
            .unwrap();

        assert!(page.content_html.contains(r#"<h2 id="intro">"#));
        assert!(page.content_html.contains(r#"<h2 id="intro-2">"#));
        assert_eq!(page.toc[0].id, "intro");
        assert_eq!(page.toc[1].id, "intro-2");
    }

    #[test]
    fn custom_summary_separator_is_used() {
        let page = MarkdownProcessor::new(None)
            .with_summary_separator("<!-- cut -->")
            .process_string(
                "# Page\n\nLead paragraph.\n\n<!-- cut -->\n\nRest.",
                Path::new("test.md"),
                route(),
            )
            .unwrap();

        assert_eq!(
            page.summary_html.as_deref(),
            Some("<h1>Page</h1>\n<p>Lead paragraph.</p>\n")
        );
    }
}
