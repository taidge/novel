use anyhow::Result;
use std::path::Path;
use tracing::info;

pub fn run_new_page(project_root: &Path, page_path: &str, layout: &str) -> Result<()> {
    let config = novel_shared::SiteConfig::load(project_root)?;
    let docs_root = config.docs_root(project_root);
    let file_path = docs_root.join(format!("{}.md", page_path));

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
        "---\ntitle: {title}\ndescription: \"\"\nlayout: {layout}\n---\n\n# {title}\n\nYour content here.\n",
        title = title,
        layout = layout,
    );

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file_path, content)?;

    info!("Created: {}", file_path.display());
    Ok(())
}
