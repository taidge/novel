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

### Content / blog fields

For blogs and content collections, the following extra fields are recognised. See [General SSG Mode](./general-ssg) for the full picture.

| Field | Type | Description |
|---|---|---|
| `date` | `YYYY-MM-DD` | Publish date — used for sorting, archives, feeds, OG `article:published_time` |
| `updated` | `YYYY-MM-DD` | Last-updated date — OG `article:modified_time` |
| `draft` | bool | Excluded from build unless `--drafts` |
| `weight` | int | Sort key for `sort_by = "weight"` |
| `summary` | string | Manual summary (overrides the `<!-- more -->` separator) |
| `tags` | list | Taxonomy entries (`/tags/<term>/`) |
| `categories` | list | Taxonomy entries (`/categories/<term>/`) |
| `series` | string | Series id — generates `/series/<slug>/` |
| `authors` | list | Author names — OG `article:author` |
| `expiry_date` | `YYYY-MM-DD` | Excluded after this date unless `--future` |

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
