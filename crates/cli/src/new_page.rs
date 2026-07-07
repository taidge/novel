use anyhow::Result;
use std::path::{Component, Path, PathBuf};
use tracing::info;

pub fn run_new_page(project_root: &Path, page_path: &str, layout: &str) -> Result<()> {
    let config = novel_shared::SiteConfig::load(project_root)?;
    let docs_root = config.docs_root_checked(project_root)?;
    let layout = validate_layout(layout)?;
    let relative_file_path = page_file_path(page_path)?;
    let file_path = docs_root.join(&relative_file_path);

    if file_path.exists() {
        anyhow::bail!("File already exists: {}", file_path.display());
    }

    // Derive title from path
    let title = page_path
        .rsplit('/')
        .next()
        .unwrap_or(page_path)
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!(
        "---\ntitle: {title_json}\ndescription: \"\"\nlayout: {layout_json}\n---\n\n# {title}\n\nYour content here.\n",
        title_json = serde_json::to_string(&title)?,
        layout_json = serde_json::to_string(layout)?,
        title = title,
    );

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file_path, content)?;

    info!("Created: {}", file_path.display());
    Ok(())
}

fn validate_layout(layout: &str) -> Result<&str> {
    match layout {
        "doc" | "page" | "blog" | "home" => Ok(layout),
        _ => anyhow::bail!("Invalid layout '{layout}'. Expected one of: doc, page, blog, home"),
    }
}

fn page_file_path(page_path: &str) -> Result<PathBuf> {
    if page_path.trim().is_empty()
        || page_path.ends_with('/')
        || page_path.ends_with('\\')
        || page_path.chars().any(|ch| ch.is_control())
    {
        anyhow::bail!(
            "Page path must be a non-empty relative file path without control characters or a trailing slash"
        );
    }

    let candidate = format!("{page_path}.md");
    let mut out = PathBuf::new();
    let mut saw_normal_component = false;
    for component in Path::new(&candidate).components() {
        match component {
            Component::Normal(part) => {
                saw_normal_component = true;
                out.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("Page path must stay inside the docs root: {page_path}");
            }
        }
    }

    if !saw_normal_component {
        anyhow::bail!("Page path must include a file name");
    }
    match out.file_name().and_then(|name| name.to_str()) {
        Some(".md") | None => anyhow::bail!("Page path must include a file name"),
        Some(_) => Ok(out),
    }
}
