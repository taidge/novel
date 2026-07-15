use anyhow::Result;
use std::path::{Component, Path};
use tracing::info;

/// Create a new documentation project with scaffolding
pub fn create_project(parent_dir: &Path, name: &str) -> Result<()> {
    let name = validate_project_name(name)?;
    let project_dir = parent_dir.join(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_dir.display());
    }

    std::fs::create_dir_all(&project_dir)?;

    // novel.toml
    let config = format!(
        r#"title = {name_json}
description = "Documentation powered by Novel"
docs_dir = "docs"
output_dir = "dist"
base = "/"
lang = "en"

[theme]
dark_mode = true
"#,
        name_json = serde_json::to_string(name)?,
    );
    std::fs::write(project_dir.join("novel.toml"), config)?;

    // docs/index.md
    let docs_dir = project_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let index_md = format!(
        r#"---
layout: home
hero:
  name: {name_json}
  text: Fast & Simple Documentation
  tagline: Built with Novel - a Rust-powered static site generator
  actions:
    - text: Get Started
      url: /guide/getting-started
      theme: brand
    - text: GitHub
      url: https://github.com
      theme: alt
features:
  - title: Fast
    icon: "\u26A1"
    details: Built with Rust for blazing fast builds and hot reload.
  - title: Simple
    icon: "\U0001F4DD"
    details: Write in Markdown, get a beautiful documentation site.
  - title: Flexible
    icon: "\U0001F527"
    details: Customizable themes, sidebar, and navigation.
---
"#,
        name_json = serde_json::to_string(name)?,
    );
    std::fs::write(docs_dir.join("index.md"), index_md)?;

    // docs/guide/
    let guide_dir = docs_dir.join("guide");
    std::fs::create_dir_all(&guide_dir)?;

    let getting_started = r#"# Getting Started

Welcome to your new documentation site!

## Installation

```bash
cargo install --git https://github.com/taidge/novel --package novel-cli --locked
```

## Quick Start

1. Create a new project:

```bash
novel init my-docs
cd my-docs
```

2. Start the dev server:

```bash
novel dev
```

3. Build for production:

```bash
novel build
```

## Writing Content

Create `.md` files in the `docs/` directory. They will automatically become pages.

::: tip
Use frontmatter at the top of your files to set page metadata.
:::
"#;
    std::fs::write(guide_dir.join("getting-started.md"), getting_started)?;

    let markdown_guide = r#"# Markdown Features

Novel supports standard Markdown with some extensions.

## Tables

| Feature | Status |
|---------|--------|
| GFM Tables | Supported |
| Task Lists | Supported |
| Strikethrough | Supported |
| Code Highlighting | Supported |

## Task Lists

- [x] Write the documentation
- [x] Add code highlighting
- [ ] Add more features

## Container Directives

::: tip
This is a helpful tip.
:::

::: warning
Be careful with this.
:::

::: danger
This is dangerous!
:::

::: info
Some additional information.
:::

::: details Click to see more
Hidden content that can be expanded.
:::

## Code Blocks

```rust title="hello.rs"
fn main() {
    println!("Hello, Novel!");
}
```
"#;
    std::fs::write(guide_dir.join("markdown.md"), markdown_guide)?;

    // _meta.json for sidebar ordering
    let meta = r#"["getting-started", "markdown"]"#;
    std::fs::write(guide_dir.join("_meta.json"), meta)?;

    // .gitignore
    std::fs::write(project_dir.join(".gitignore"), "dist/\ntarget/\n")?;

    info!("Created new project at: {}", project_dir.display());
    info!("  cd {} && novel dev", name);

    Ok(())
}

fn validate_project_name(name: &str) -> Result<&str> {
    if name.trim().is_empty() || name.chars().any(|ch| ch.is_control()) {
        anyhow::bail!("Project name must be non-empty and must not contain control characters");
    }

    let mut components = Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(name),
        _ => anyhow::bail!("Project name must be a single directory name, not a path: {name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{create_project, validate_project_name};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("novel-init-test-{}-{unique}", std::process::id()))
    }

    #[test]
    fn project_names_cannot_escape_the_parent() {
        for name in ["", ".", "..", "../site", "nested/site"] {
            assert!(validate_project_name(name).is_err(), "{name}");
        }
        assert_eq!(validate_project_name("my-docs").unwrap(), "my-docs");
    }

    #[test]
    fn scaffold_uses_the_project_git_install_command() {
        let root = temp_dir();
        fs::create_dir_all(&root).expect("failed to create test root");
        create_project(&root, "site").expect("project scaffold should succeed");

        let guide = fs::read_to_string(root.join("site/docs/guide/getting-started.md"))
            .expect("generated guide should exist");
        assert!(guide.contains(
            "cargo install --git https://github.com/taidge/novel --package novel-cli --locked"
        ));

        let _ = fs::remove_dir_all(root);
    }
}
