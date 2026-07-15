# Novel

[English](./README.md) | [简体中文](./README.zh.md)

**A fast static documentation site generator built with Rust — run standalone or embed as a library.**

Novel turns a folder of Markdown files into a polished documentation website in milliseconds. Use it as a CLI, or drop it into your own Rust application as a library.

## Features

- **Blazing fast** — Built entirely in Rust. Builds complete in milliseconds, not seconds.
- **Embeddable** — Use as a CLI or embed as a library: `DirNovel::new("docs").build()`.
- **Markdown first** — GFM, syntax highlighting, tabs, steps, badges, and container directives out of the box.
- **File embedding** — Embed source code from external files with line-range support so docs stay in sync with code.
- **Beautiful themes** — Dark mode, responsive layout, search, prev/next navigation, and image zoom built in.
- **SEO & AI ready** — Sitemap, RSS/JSON feeds, `llms.txt`, edit links, Git updated timestamps, and custom head tags.
- **Operational docs features** — Versioned docs, per-page Markdown mirrors, optional PWA output, and static page feedback.

## Installation

Novel requires Rust 1.94 or newer. The `novel-cli` name on crates.io belongs
to a different project, so install this CLI from its GitHub repository:

```bash
cargo install --git https://github.com/taidge/novel --package novel-cli --locked
```

## Quick Start

```bash
# create a new project
novel init my-docs
cd my-docs

# start the dev server with live reload
novel dev

# build for production
novel build

# preview the production build
novel preview
```

A new project has the following structure:

```
my-docs/
├── docs/
│   ├── index.md          # Home page
│   └── guide/
│       ├── _meta.json    # Sidebar ordering
│       ├── getting-started.md
│       └── markdown.md
├── novel.toml            # Configuration
└── .gitignore
```

The dev server runs on `http://localhost:3000` and rebuilds automatically when `.md`, `.json`, or `.toml` files change.

## Configuration

Create a `novel.toml` in your project root:

```toml
title = "My Docs"
description = "My documentation site"
docs_dir = "docs"
output_dir = "dist"
base = "/"
lang = "en"
site_url = "https://example.com"  # enables sitemap & RSS

[markdown]
show_line_numbers = false
check_dead_links = false

[theme]
dark_mode = true
footer = "Built with Novel"
show_git_updated_at = true
edit_url = "https://github.com/user/repo/edit/main/docs/"
source_url = "https://github.com/user/repo"
```

Most fields have sensible defaults — you only need to configure what you want to customize.

## Library Usage

Novel can be embedded directly into your own Rust application.

```toml
[dependencies]
novel-core = { path = "path/to/novel/crates/core" }
```

Build a site and write it to disk in three lines:

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs").build()?;
site.write_to("dist")?;
```

Or customize it with the builder API:

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs")
    .title("My API Reference")
    .description("Generated docs for my-crate")
    .base("/docs/")
    .site_url("https://example.com")
    .with_theme(|t| {
        t.dark_mode = true;
        t.footer = Some("Built with Novel".into());
        t.show_git_updated_at = true;
    })
    .build()?;

site.write_to("./output")?;
```

### Serving from a web server

The `BuiltSite` returned by `.build()` can render pages on demand, which makes it easy to serve docs directly from an Axum (or any other) server:

```rust
use novel_core::{DirNovel, Novel};

let site = DirNovel::new("docs").build()?;
let page = site.page("/guide/intro").unwrap();
let html = site.render_page(page)?;
```

See [docs/guide/library.md](./docs/guide/library.md) for the full API reference.

## Workspace Layout

Novel is a Cargo workspace with three crates:

| Crate | Description |
|-------|-------------|
| `crates/shared` | Shared types and configuration |
| `crates/core`   | Site building and rendering engine |
| `crates/cli`    | The `novel` command-line binary |

## Documentation

Full documentation lives in the `docs/` directory and can be built with `novel build`. Topics covered:

- Getting started, configuration, routing
- Markdown features, frontmatter, file embedding
- Theming, home page, static assets
- Search, deploy, library API

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).
