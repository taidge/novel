use rust_embed::Embed;
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

        // Load embedded templates first
        for name in TeraTemplates::iter() {
            let name_str = name.as_ref();
            if let Some(file) = TeraTemplates::get(name_str) {
                let content = std::str::from_utf8(file.data.as_ref()).map_err(|e| {
                    NovelError::Template(format!(
                        "Embedded tera template {name_str} is not valid UTF-8: {e}"
                    ))
                })?;
                tera.add_raw_template(name_str, content)
                    .map_err(|e| NovelError::Template(e.to_string()))?;
            }
        }

        // Override with user templates from project directory
        if let Some(root) = project_root {
            let template_dir = root.join("templates");
            if template_dir.is_dir() {
                for entry in std::fs::read_dir(&template_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file()
                        && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    {
                        let content = std::fs::read_to_string(&path)?;
                        tera.add_raw_template(name, &content)
                            .map_err(|e| NovelError::Template(e.to_string()))?;
                    }
                }
            }
        }

        Ok(Self { tera })
    }
}

impl TemplateRenderer for TeraRenderer {
    fn render(&self, template_name: &str, ctx: &RenderContext) -> NovelResult<String> {
        let context = tera::Context::from_serialize(ctx)
            .map_err(|e| NovelError::Template(e.to_string()))?;
        self.tera
            .render(template_name, &context)
            .map_err(|e| NovelError::Template(e.to_string()))
    }
}
