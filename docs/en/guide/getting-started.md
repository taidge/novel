---
title: Getting Started
description: Install Novel, create a project, and build your first documentation site.
---

# Getting Started

Get up and running with Novel in minutes.

## Installation

Novel requires Rust 1.94 or newer. The `novel-cli` name on crates.io belongs
to a different project, so install this CLI from its GitHub repository:

```bash
cargo install --git https://github.com/taidge/novel --package novel-cli --locked
```

## Create a New Project

```bash
novel init my-docs
cd my-docs
```

This creates a project with the following structure:

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

## Development

Start the dev server with live reload:

```bash
novel dev
```

Open `http://localhost:3000` in your browser. Changes to content, templates, CSS/Sass sources, config, data files, and common static assets trigger an automatic rebuild.

The server binds to `127.0.0.1` by default. Binding `--host` to a
non-loopback address exposes the generated site to other machines; Novel emits
a security warning so you can verify the output contains no secrets.

## Build for Production

```bash
novel build
```

Output goes to the `dist/` directory. Serve it with any static file server.

## Preview

```bash
novel preview
```

Serves the built output locally on port 4000.

## Configuration

Create `novel.toml` in your project root:

```toml title="novel.toml"
title = "My Docs"
description = "My documentation site"
docs_dir = "docs"
output_dir = "dist"
base = "/"
lang = "en"
site_url = "https://example.com"  # enables sitemap & RSS

[markdown]
show_line_numbers = false   # line numbers on code blocks
check_dead_links = false    # validate internal links at build time

[theme]
dark_mode = true
footer = "Built with Novel"
show_git_updated_at = true         # show git timestamps
edit_url = "https://github.com/user/repo/edit/main/docs/"
source_url = "https://github.com/user/repo"

# announcement banner
[theme.banner]
text = "Novel v0.2 is out!"
url = "/guide/getting-started"
dismissible = true
```

::: tip
Most fields have sensible defaults. You only need to configure what you want to customize.
:::

## Using as a Library

Novel can also be embedded into your own Rust application — see the [Library API](/guide/library) guide.
