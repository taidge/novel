use anyhow::Result;
use minijinja::{Environment, Error as MiniJinjaError, ErrorKind, context};
use rust_embed::Embed;
use novel_shared::config::SiteConfig;
use novel_shared::{NavItem, PageData, SidebarItem};
use std::path::{Component, Path, PathBuf};

const REQUIRED_TEMPLATES: [&str; 4] = ["base.html", "doc.html", "home.html", "404.html"];

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

/// Load an embedded template file as a UTF-8 string.
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

/// HTML template engine using minijinja
pub struct TemplateEngine {
    env: Environment<'static>,
}

impl TemplateEngine {
    pub fn new(project_root: Option<&Path>) -> Result<Self> {
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

        Ok(Self { env })
    }

    /// Render a documentation page
    pub fn render_doc(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
        sidebar: &[SidebarItem],
    ) -> Result<String> {
        let edit_url = config.theme.edit_link.as_ref().map(|pattern| {
            format!(
                "{}{}",
                pattern.trim_end_matches('/'),
                page.route.relative_path
            )
        });
        let edit_link_text = config
            .theme
            .edit_link_text
            .as_deref()
            .unwrap_or("Edit this page");
        let last_updated_text = config
            .theme
            .last_updated_text
            .as_deref()
            .unwrap_or("Last updated");

        let tmpl = self.env.get_template("doc.html")?;
        let html = tmpl.render(context! {
            site => config,
            page => page,
            nav => nav,
            sidebar => sidebar,
            toc => page.toc,
            edit_url => edit_url,
            edit_link_text => edit_link_text,
            last_updated_text => last_updated_text,
        })?;
        Ok(html)
    }

    /// Render the home page
    pub fn render_home(
        &self,
        page: &PageData,
        config: &SiteConfig,
        nav: &[NavItem],
    ) -> Result<String> {
        let tmpl = self.env.get_template("home.html")?;
        let html = tmpl.render(context! {
            site => config,
            page => page,
            nav => nav,
        })?;
        Ok(html)
    }

    /// Render the 404 page
    pub fn render_404(&self, config: &SiteConfig, nav: &[NavItem]) -> Result<String> {
        let tmpl = self.env.get_template("404.html")?;
        let html = tmpl.render(context! {
            site => config,
            nav => nav,
        })?;
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestProject {
        root: PathBuf,
    }

    impl TestProject {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "novel-template-test-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("failed to create test project root");
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write_template(&self, name: &str, contents: &str) {
            let path = self.root.join("templates").join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("failed to create test template directory");
            }
            fs::write(path, contents).expect("failed to write test template");
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn prefers_templates_from_project_folder() {
        let project = TestProject::new();
        project.write_template("404.html", "custom 404");

        let engine = TemplateEngine::new(Some(project.root())).expect("template engine should load");
        let rendered = engine
            .render_404(&SiteConfig::default(), &[])
            .expect("404 page should render");

        assert_eq!(rendered, "custom 404");
    }

    #[test]
    fn falls_back_to_embedded_templates_when_missing_from_project_folder() {
        let project = TestProject::new();
        project.write_template(
            "base.html",
            "<html><body>CUSTOM_BASE{% block content %}{% endblock %}</body></html>",
        );

        let engine = TemplateEngine::new(Some(project.root())).expect("template engine should load");
        let rendered = engine
            .render_404(&SiteConfig::default(), &[])
            .expect("404 page should render");

        assert!(rendered.contains("CUSTOM_BASE"));
        assert!(rendered.contains("Page not found"));
    }
}
