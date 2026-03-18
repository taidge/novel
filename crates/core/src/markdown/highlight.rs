use once_cell::sync::Lazy;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Highlight a code block and return HTML
pub fn highlight_code(code: &str, lang: &str, theme_name: &str) -> String {
    let ss = &*SYNTAX_SET;
    let ts = &*THEME_SET;

    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| &ts.themes["base16-ocean.dark"]);

    match highlighted_html_for_string(code, ss, syntax, theme) {
        Ok(html) => html,
        Err(_) => format!(
            "<pre><code class=\"language-{}\">{}</code></pre>",
            lang,
            html_escape(code)
        ),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
