# Frontmatter

Every Markdown file can include YAML frontmatter at the top, delimited by `---`. Frontmatter controls page metadata and layout options.

## Basic Fields

```yaml
---
title: Page Title
description: A short description of this page
---
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | string | First `# heading` | Page title (used in `<title>` tag and sidebar) |
| `description` | string | `""` | Page description (used in `<meta>` tag) |

## Page Type

Control the page layout with `page_type`:

```yaml
---
page_type: home   # or doc, custom, 404
---
```

| Value | Description |
|-------|-------------|
| `home` | Home page with hero section and features grid |
| `doc` | Standard documentation page (default) |
| `custom` | Custom page without sidebar |
| `404` | Not found page |

## Layout Control

```yaml
---
sidebar: false    # hide the sidebar on this page
navbar: false     # hide the navbar on this page
outline: false    # hide the table of contents on this page
---
```

## Hero & Features

Used only with `page_type: home`. See [Home Page](/guide/home-page) for full details.

```yaml
---
page_type: home
hero:
  name: Project Name
  text: Tagline text
  tagline: Longer description
  actions:
    - text: Get Started
      link: /guide/
      theme: brand
features:
  - title: Feature
    icon: "\u26A1"
    details: Description
---
```

## Custom Head Tags

Add custom HTML tags to the `<head>` of a specific page:

```yaml
---
head:
  - tag: meta
    attrs:
      property: og:title
      content: My Page Title
  - tag: script
    attrs:
      src: https://example.com/analytics.js
      async: ""
  - tag: link
    attrs:
      rel: canonical
      href: https://example.com/page
---
```

Each entry in the `head` array requires:
- `tag` — the HTML tag name
- `attrs` — key-value map of attributes
- `content` — optional inner text content
