use anyhow::Result;
use std::path::Path;
use tracing::info;

/// Create a new documentation project with scaffolding
pub fn create_project(parent_dir: &Path, name: &str) -> Result<()> {
    let project_dir = parent_dir.join(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_dir.display());
    }

    std::fs::create_dir_all(&project_dir)?;

    // novel.toml
    let config = format!(
        r#"title = "{name}"
description = "Documentation powered by Novel"
root = "docs"
out_dir = "dist"
base = "/"
lang = "en"

[theme]
dark_mode = true
"#,
    );
    std::fs::write(project_dir.join("novel.toml"), config)?;

    // docs/index.md
    let docs_dir = project_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let index_md = format!(
        r#"---
page_type: home
hero:
  name: {name}
  text: Fast & Simple Documentation
  tagline: Built with Novel - a Rust-powered static site generator
  actions:
    - text: Get Started
      link: /guide/getting-started
      theme: brand
    - text: GitHub
      link: https://github.com
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
    );
    std::fs::write(docs_dir.join("index.md"), index_md)?;

    // docs/guide/
    let guide_dir = docs_dir.join("guide");
    std::fs::create_dir_all(&guide_dir)?;

    let getting_started = r#"# Getting Started

Welcome to your new documentation site!

## Installation

```bash
cargo install novel-cli
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
