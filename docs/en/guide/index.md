---
title: Guide
description: Everything you need to build a documentation site with Novel — from your first page to advanced features.
---

# Guide

Welcome to the Novel guide. Novel is a fast static documentation site generator built in Rust — usable as a standalone CLI or embedded as a library in your own Rust application.

## Start here

New to Novel? These are the pages to read in order:

- [**Getting Started**](./getting-started.md) — install the CLI, create a project, run the dev server
- [**Configuration**](./configuration.md) — the `novel.toml` reference (title, base, theme, markdown options, …)
- [**Routing**](./routing.md) — how files on disk become URLs
- [**Markdown**](./markdown.md) — GFM, syntax highlighting, tabs, steps, badges, containers
- [**Frontmatter**](./frontmatter.md) — per-page metadata: title, layout, date, tags, canonical, noindex, …

## Writing content

- [**Home Page**](./home-page.md) — build a landing page with hero and features
- [**File Embedding**](./file-embed.md) — embed source code from external files so docs stay in sync with code
- [**Static Assets**](./static-assets.md) — images, downloads, and other public files

## Beyond docs — general SSG mode

Novel isn't just a documentation generator. It has first-class support for general static sites:

- [**General SSG**](./general-ssg.md) — collections, taxonomies, pagination, data files, and everything else that makes Novel a viable Hugo / Zola alternative

## Customization

- [**Theming**](./theming.md) — colors, dark mode, footer, edit links, banners
- [**Search**](./search.md) — the built-in client-side search index

## Shipping

- [**Deploy**](./deploy.md) — publish your site to GitHub Pages, Netlify, Vercel, Cloudflare Pages, or any static host
- [**Library API**](./library.md) — embed Novel inside your own Rust application or web server

::: tip
Most pages have sensible defaults — you can get a working site with just a few lines of `novel.toml` and a `docs/` folder of Markdown files. Start with [Getting Started](./getting-started.md) and come back here when you need to customize something specific.
:::
