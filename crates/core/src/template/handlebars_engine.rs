use anyhow::Result;
use handlebars::Handlebars;
use rust_embed::Embed;
use std::path::Path;

use super::{RenderContext, TemplateRenderer};

#[derive(Embed)]
#[folder = "templates_hbs/"]
struct HbsTemplates;

/// Handlebars template renderer.
///
/// Templates use Handlebars syntax with `{{> partial}}` for composition.
/// An `eq` helper is registered for equality comparisons.
/// User templates in `{project_root}/templates/` override the embedded ones.
pub struct HandlebarsRenderer {
    hbs: Handlebars<'static>,
}

impl HandlebarsRenderer {
    pub fn new(project_root: Option<&Path>) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false);

        // Register built-in helpers
        hbs.register_helper("eq", Box::new(eq_helper));

        // Load embedded templates
        for name in HbsTemplates::iter() {
            let name_str = name.as_ref();
            if let Some(file) = HbsTemplates::get(name_str) {
                let content = std::str::from_utf8(file.data.as_ref())
                    .map_err(|e| anyhow::anyhow!("Embedded hbs template {name_str} is not valid UTF-8: {e}"))?;
                let tmpl_name = name_str.trim_end_matches(".html");
                hbs.register_template_string(tmpl_name, content)?;
            }
        }

        // Override with user templates from project directory
        if let Some(root) = project_root {
            let template_dir = root.join("templates");
            if template_dir.is_dir() {
                for entry in std::fs::read_dir(&template_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            let content = std::fs::read_to_string(&path)?;
                            let tmpl_name = file_name.trim_end_matches(".html");
                            hbs.register_template_string(tmpl_name, &content)?;
                        }
                    }
                }
            }
        }

        Ok(Self { hbs })
    }
}

impl TemplateRenderer for HandlebarsRenderer {
    fn render(&self, template_name: &str, ctx: &RenderContext) -> Result<String> {
        // Template name comes in as "doc.html", strip extension for lookup
        let tmpl_name = template_name.trim_end_matches(".html");
        Ok(self.hbs.render(tmpl_name, ctx)?)
    }
}

/// Handlebars helper for equality comparison: `{{#if (eq a b)}}`
fn eq_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let a = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let b = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(if a == b { "true" } else { "" })?;
    Ok(())
}
