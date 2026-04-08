# Getting Started

Get up and running with Novel in minutes.

## Installation

```bash
cargo install novel-cli
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

Open `http://localhost:3000` in your browser. Changes to `.md`, `.json`, or `.toml` files trigger an automatic rebuild.

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
root = "docs"
out_dir = "dist"
base = "/"
lang = "en"
site_url = "https://example.com"  # enables sitemap & RSS

[markdown]
show_line_numbers = false   # line numbers on code blocks
check_dead_links = false    # validate internal links at build time

[theme]
dark_mode = true
footer = "Built with Novel"
last_updated = true         # show git timestamps
edit_link = "https://github.com/user/repo/edit/main/docs/"
source_link = "https://github.com/user/repo"

# announcement banner
[theme.banner]
text = "Novel v0.2 is out!"
link = "/guide/getting-started"
dismissible = true
```

::: tip
Most fields have sensible defaults. You only need to configure what you want to customize.
:::

## Using as a Library

Novel can also be embedded into your own Rust application — see the [Library API](/en/guide/library) guide.
