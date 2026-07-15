use rust_embed::Embed;
use std::error::Error as _;
use std::path::Path;
use tera::Tera;

use super::{RenderContext, TemplateRenderer};
use crate::error::{NovelError, NovelResult};

#[derive(Embed)]
#[folder = "templates_tera/"]
struct TeraTemplates;

/// Tera template renderer.
///
/// Tera uses a Jinja2-like syntax very similar to minijinja.
/// User templates in `{project_root}/templates/` override the embedded ones.
pub struct TeraRenderer {
    tera: Tera,
}

impl TeraRenderer {
    pub fn new(project_root: Option<&Path>) -> NovelResult<Self> {
        let mut tera = Tera::default();

        // Parse each template first, then build inheritance after the full batch
        // is present. Registering children one-by-one fails when rust-embed's
        // iteration order yields a child before base.html.
        let mut embedded_templates = Vec::new();
        for name in TeraTemplates::iter() {
            let name_str = name.as_ref();
            if let Some(file) = TeraTemplates::get(name_str) {
                let content = std::str::from_utf8(file.data.as_ref()).map_err(|e| {
                    NovelError::Template(format!(
                        "Embedded tera template {name_str} is not valid UTF-8: {e}"
                    ))
                })?;
                embedded_templates.push((name_str.to_string(), content.to_string()));
            }
        }
        tera.add_raw_templates(embedded_templates)
            .map_err(tera_error)?;

        // Override with user templates from project directory
        if let Some(root) = project_root {
            let template_dir = root.join("templates");
            if template_dir.is_dir() {
                let mut user_templates = Vec::new();
                for entry in std::fs::read_dir(&template_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file()
                        && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    {
                        let content = std::fs::read_to_string(&path)?;
                        user_templates.push((name.to_string(), content));
                    }
                }
                tera.add_raw_templates(user_templates).map_err(tera_error)?;
            }
        }

        Ok(Self { tera })
    }
}

impl TemplateRenderer for TeraRenderer {
    fn render(&self, template_name: &str, ctx: &RenderContext) -> NovelResult<String> {
        let context =
            tera::Context::from_serialize(ctx).map_err(|e| NovelError::Template(e.to_string()))?;
        self.tera
            .render(template_name, &context)
            .map_err(tera_error)
    }
}

fn tera_error(error: tera::Error) -> NovelError {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    NovelError::Template(message)
}
