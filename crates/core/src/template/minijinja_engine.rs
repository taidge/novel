use anyhow::Result;
use minijinja::{Environment, Error as MiniJinjaError, ErrorKind};
use rust_embed::Embed;
use std::path::{Component, Path, PathBuf};

use super::{RenderContext, TemplateRenderer};
use crate::plugin::Plugin;

const REQUIRED_TEMPLATES: [&str; 8] = [
    "base.html",
    "doc.html",
    "home.html",
    "page.html",
    "blog.html",
    "404.html",
    "list.html",
    "terms.html",
];

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

fn template_path(template_dir: &Path, name: &str) -> Option<PathBuf> {
    let relative = Path::new(name);
    if relative.is_absolute() {
        return None;
    }
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(template_dir.join(relative))
}

fn load_template_from_dir(
    template_dir: &Path,
    name: &str,
) -> Result<Option<String>, MiniJinjaError> {
    let Some(path) = template_path(template_dir, name) else {
        return Ok(None);
    };
    if !path.is_file() {
        return Ok(None);
    }
    std::fs::read_to_string(&path).map(Some).map_err(|err| {
        MiniJinjaError::new(
            ErrorKind::InvalidOperation,
            format!("failed to read template {}", path.display()),
        )
        .with_source(err)
    })
}

fn load_embedded_template(name: &str) -> Result<Option<String>, MiniJinjaError> {
    let Some(file) = Templates::get(name) else {
        return Ok(None);
    };
    std::str::from_utf8(file.data.as_ref())
        .map(|template| Some(template.to_owned()))
        .map_err(|err| {
            MiniJinjaError::new(
                ErrorKind::InvalidOperation,
                format!("embedded template {name} is not valid UTF-8"),
            )
            .with_source(err)
        })
}

/// MiniJinja (Jinja2-compatible) template renderer — the default engine.
pub struct MiniJinjaRenderer {
    env: Environment<'static>,
}

impl MiniJinjaRenderer {
    pub fn new(project_root: Option<&Path>, plugins: &[Box<dyn Plugin>]) -> Result<Self> {
        let mut env = Environment::new();
        let template_dir = project_root.map(|p| p.join("templates"));

        env.set_loader(move |name| {
            if let Some(ref dir) = template_dir {
                if let Some(template) = load_template_from_dir(dir, name)? {
                    return Ok(Some(template));
                }
            }
            load_embedded_template(name)
        });

        for name in REQUIRED_TEMPLATES {
            env.get_template(name)?;
        }

        for plugin in plugins {
            plugin.register_template_helpers(&mut env);
        }

        Ok(Self { env })
    }
}

impl TemplateRenderer for MiniJinjaRenderer {
    fn render(&self, template_name: &str, ctx: &RenderContext) -> Result<String> {
        let tmpl = self.env.get_template(template_name)?;
        let value = minijinja::Value::from_serialize(ctx);
        Ok(tmpl.render(value)?)
    }
}
