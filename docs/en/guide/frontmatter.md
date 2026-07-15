---
title: Frontmatter
description: Control page metadata, layout behavior, and navigation with Markdown frontmatter.
---

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
| `published_at` | `YYYY-MM-DD` | Publish date — used for sorting, archives, feeds, OG `article:published_time` |
| `updated_at` | `YYYY-MM-DD` | Last-updated date — OG `article:modified_time` |
| `draft` | bool | Excluded from build unless `--drafts` |
| `weight` | int | Sort key for `sort_by = "weight"` |
| `summary` | string | Manual summary (overrides the `<!-- more -->` separator) |
| `taxonomies` | map | Taxonomy entries keyed by configured taxonomy name |
| `series` | string | Series id — generates `/series/<slug>/` |
| `authors` | list | Author names — OG `article:author` |
| `expires_at` | `YYYY-MM-DD` | Excluded after this date unless `--future` |

## Page Type

Control the page layout with `layout`:

```yaml
---
layout: home   # or doc, custom, 404
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

Used only with `layout: home`. See [Home Page](/guide/home-page) for full details.

```yaml
---
layout: home
hero:
  name: Project Name
  text: Tagline text
  tagline: Longer description
  actions:
    - text: Get Started
      url: /guide/
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
  - tag: link
    attrs:
      rel: canonical
      href: https://example.com/page
  - tag: title
    content: My custom browser title
---
```

For safe defaults, custom head entries are limited to `meta`, `link`, and
`title` with tag-specific attributes. Executable or navigation-changing
entries such as `script`, `style`, `base`, `meta http-equiv`, event
handlers, and dangerous URL schemes are rejected during the build. Add a
trusted custom template when a site intentionally needs analytics or another
script.

::: warning Trusted input boundary
Raw HTML in Markdown, custom templates, theme packs, and custom CSS are treated
as trusted project code and are not sanitized. Do not build unreviewed content
from untrusted contributors.
:::
