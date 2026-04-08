# Theming

Novel ships with a clean, responsive default theme (dark mode included), and is designed so you can bend it as far as you like — from tweaking a color variable to shipping an entirely custom look as a reusable **theme pack**.

Customization is layered. Pick the highest level that still does what you need:

| Level | What you change | Effort |
|-------|-----------------|--------|
| 1. Config options | Logo, footer, banner, nav, social links | Minutes |
| 2. CSS variables | Brand colors, spacing, fonts via `[theme.colors]` | Minutes |
| 3. Custom CSS | Arbitrary stylesheet loaded after the defaults | Low |
| 4. Template overrides | Replace individual templates in your `templates/` dir | Medium |
| 5. Theme pack | Ship a reusable folder of templates + CSS + assets | Medium |
| 6. Engine swap | Use Tera or Handlebars instead of MiniJinja | Medium |

## 1. Built-in options

Most sites never need anything beyond the options under `[theme]`. All of these are documented in [Configuration](./configuration.md), but the common ones:

```toml title="novel.toml"
[theme]
dark_mode      = true
footer         = "Copyright 2025 My Project | Built with Novel"
edit_link      = "https://github.com/user/repo/edit/main/docs/"
edit_link_text = "Edit this page on GitHub"
last_updated   = true
source_link    = "https://github.com/user/repo"

[theme.banner]
text        = "We just released v2.0!"
link        = "/guide/changelog"
dismissible = true
```

### Dark mode

Dark mode is enabled by default. The preference is saved to `localStorage`; if nothing is stored, Novel follows `prefers-color-scheme`. Set `dark_mode = false` to hide the toggle.

### Responsive layout

The default theme is three-column on desktop (sidebar / content / TOC), two-column on tablet (content / TOC), and single-column on mobile with a hamburger menu.

## 2. Brand colors via CSS variables

The whole default stylesheet is driven by CSS custom properties. You can override **any of them** without touching CSS files — just set `[theme.colors]` in `novel.toml`. Keys are the CSS variable name *without* the leading `--`:

```toml title="novel.toml"
[theme.colors]
accent       = "#e11d48"
accent-hover = "#be123c"
accent-bg    = "#fff1f2"
bg-primary   = "#fffaf5"
text-primary = "#1f1f28"
font-sans    = '"Inter", system-ui, sans-serif'
```

Novel writes these into the page as:

```html
<style>:root, [data-theme="light"] { --accent: #e11d48; /* ... */ }</style>
```

Common variables you'll probably want to override:

| Variable | Default | Purpose |
|----------|---------|---------|
| `--accent` | `#3b82f6` | Primary brand color (links, buttons) |
| `--accent-hover` | `#2563eb` | Hover state for accent |
| `--accent-bg` | `#eff6ff` | Tinted backgrounds (callouts, highlights) |
| `--bg-primary` | `#ffffff` | Page background |
| `--bg-secondary` | `#f6f8fa` | Sidebar, card backgrounds |
| `--text-primary` | `#1a1a2e` | Body text |
| `--text-secondary` | `#4a5568` | Muted text |
| `--border-color` | `#e2e8f0` | Dividers, borders |
| `--code-bg` | `#f1f5f9` | Inline code background |
| `--font-sans` | system UI stack | Body font |
| `--font-mono` | JetBrains Mono stack | Code font |
| `--sidebar-width` | `260px` | Left sidebar width |
| `--toc-width` | `220px` | Right TOC width |
| `--navbar-height` | `60px` | Top nav height |

See [`crates/core/assets/style.css`](https://github.com/chrislearn/novel/blob/main/crates/core/assets/style.css) for the full list, including container (tip/warning/danger) colors, banner colors, diff colors, and the dark-mode overrides under `[data-theme="dark"]`.

::: tip
Overrides in `[theme.colors]` only affect the `light` theme block. To restyle dark mode as well, use a custom CSS file (next section) and target `[data-theme="dark"]`.
:::

## 3. Custom CSS file

When one-off variables aren't enough, point `theme.custom_css` at a CSS file (path is relative to the project root). Its contents are inlined into every page **after** the default stylesheet, so it can override anything:

```toml title="novel.toml"
[theme]
custom_css = "assets/custom.css"
```

```css title="assets/custom.css"
/* Restyle the dark theme */
[data-theme="dark"] {
    --accent: #fbbf24;
    --bg-primary: #0a0a0f;
}

/* Custom font face */
@font-face {
    font-family: "My Brand";
    src: url("/fonts/brand.woff2") format("woff2");
}
:root { --font-sans: "My Brand", system-ui, sans-serif; }

/* Tweak a component */
.hero-name {
    background: linear-gradient(90deg, #f97316, #e11d48);
    -webkit-background-clip: text;
    color: transparent;
}
```

::: note
`custom_css` is **inlined**, not linked — it adds bytes to every HTML page. For large stylesheets, drop the file under `docs/` as a [static asset](./static-assets.md) and `<link>` it yourself from a `base.html` override.
:::

## 4. Template overrides

Novel uses [MiniJinja](https://docs.rs/minijinja) (a Jinja2-compatible engine) by default. When rendering, it looks for each template in this order:

1. `<project_root>/templates/<name>`
2. `<project_root>/<theme.pack>/templates/<name>` (if a theme pack is set)
3. The embedded default, baked into the binary

This means you can override **just** the templates you care about. Any template you don't provide still falls back to the built-in version.

### Available templates

| Template | Renders | Extends |
|----------|---------|---------|
| `base.html` | Page shell (`<html>`, navbar, footer, `<head>`) | — |
| `doc.html` | Documentation page (sidebar + content + TOC) | `base.html` |
| `home.html` | Landing page with hero + features | `base.html` |
| `page.html` | Full-width page (no sidebar, no TOC) | `base.html` |
| `blog.html` | Centered post layout with date header | `base.html` |
| `list.html` | Paginated list (collections, taxonomy terms) | `base.html` |
| `terms.html` | Taxonomy overview (e.g. `/tags/`) | `base.html` |
| `404.html` | Not-found page | `base.html` |

Example layout:

```text
my-site/
├── novel.toml
├── docs/
│   └── ...
└── templates/
    ├── base.html        ← optional full override
    ├── doc.html         ← override doc pages only
    └── home.html        ← override home only
```

### Overriding blocks instead of whole files

Because every layout extends `base.html`, you almost never need to rewrite a full template. Define a file in your `templates/` dir that `{% extends %}` from the built-in and override just the blocks you want. Blocks exposed by `base.html`:

| Block | What it is |
|-------|-----------|
| `title` | Inside `<title>` |
| `description` | Inside the `<meta name="description">` tag |
| `head` | Extra tags inside `<head>` (rendered **before** OG/JSON-LD) |
| `head_extra` | Extra `<head>` tags rendered after OG/JSON-LD |
| `og_meta` | Open Graph + Twitter card tags |
| `json_ld` | Structured data script |
| `nav_extra` | Markup appended inside `<nav class="navbar">` |
| `content_before` / `content` / `content_after` | The page body |
| `footer_extra` | Markup appended inside `<footer>` |

```html title="templates/base.html"
{% extends "base.html" %}
{# This would infinite-loop — instead override individual layouts. #}
```

A more useful pattern — add a custom stylesheet + Plausible analytics to every page by overriding `doc.html` (or `base.html` if you really need it for all layouts):

```html title="templates/doc.html"
{% extends "base.html" %}

{% block head_extra %}
    <link rel="stylesheet" href="{{ asset_url('css/extra.css') }}">
    <script defer data-domain="example.com" src="https://plausible.io/js/script.js"></script>
{% endblock %}

{# Reuse the default doc body untouched. #}
{% block content %}{{ super() }}{% endblock %}
```

::: warning
If you override `base.html` directly (rather than a child layout), put a **different** file in `templates/base.html` — not one that `{% extends "base.html" %}`, because that resolves to *your* file and loops. The simplest approach is to copy the upstream [`base.html`](https://github.com/chrislearn/novel/blob/main/crates/core/templates/base.html) and edit it.
:::

### Render context

Every template is handed this context:

| Variable | Type | Description |
|----------|------|-------------|
| `site` | object | The full `SiteConfig` (`site.title`, `site.base`, `site.theme.*`, `site.markdown.*`, …) |
| `page` | object or null | Current page (`page.title`, `page.content_html`, `page.frontmatter`, `page.toc`, `page.last_updated`, `page.prev_page`, `page.next_page`, `page.breadcrumbs`, …) |
| `nav` | array | Top-nav items |
| `sidebar` | array | Sidebar items for the current section |
| `toc` | array | Table of contents items for the current page |
| `edit_url` | string or null | Computed "edit this page" URL |
| `edit_link_text` | string | Label for the edit link |
| `last_updated_text` | string | Label for the last-updated timestamp |
| `theme_css_overrides` | string or null | Rendered `[theme.colors]` as a CSS string |
| `custom_css_content` | string or null | Inlined contents of `theme.custom_css` |
| `asset_css` / `asset_js` | string | Hashed asset filenames when `asset_fingerprint` is on |
| `site_data` | object | Data files loaded from `<docs>/data/` |
| `paginator` | object or null | On list pages: `items`, `current_page`, `total_pages`, … |
| `terms` | array or null | On taxonomy overview pages |
| `list_title` | string or null | Title of list / term pages |

### Built-in template functions

Two helpers are registered on every render:

- `asset_url(path)` — prefixes a path with the site `base`, e.g. `asset_url("css/extra.css")` → `/docs/css/extra.css` when `base = "/docs/"`.
- `image_set(path, sizes)` — builds a responsive `srcset` string for images produced by the image pipeline:
  ```html
  <img src="{{ asset_url('images/hero.jpg') }}"
       srcset="{{ image_set('images/hero.jpg', [400, 800, 1600]) }}"
       sizes="(max-width: 600px) 100vw, 50vw">
  ```

Plugins can register additional helpers — see the `Plugin::register_template_helpers` hook in the [library guide](./library.md).

## 5. Theme packs (reusable themes)

A **theme pack** is just a folder of template + asset overrides that you point `theme.pack` at. Packs are checked *after* `<project_root>/templates/` but *before* the built-ins, so per-site overrides in `templates/` still win over the pack.

```text
themes/
└── midnight/
    └── templates/
        ├── base.html
        ├── doc.html
        └── home.html
```

```toml title="novel.toml"
[theme]
pack       = "./themes/midnight"
custom_css = "./themes/midnight/assets/style.css"
```

You can check a theme pack into its own git repo, publish it, and share it across projects. Consumers of a theme pack still override anything they want by dropping a file of the same name into their local `templates/`.

::: tip
Only `templates/` inside a pack is resolved automatically. If your pack has CSS, images, or JS, reference them explicitly via `theme.custom_css` or by symlinking / copying them under `docs/` so they get picked up as [static assets](./static-assets.md).
:::

## 6. Switching template engines

MiniJinja is the default. If you prefer Tera or Handlebars syntax, enable the matching cargo feature on `novel-core` and set `template_engine`:

```toml title="novel.toml"
template_engine = "tera"        # or "handlebars" or "minijinja" (default)
```

Each engine has its own built-in template set under `crates/core/templates_tera/` and `crates/core/templates_hbs/`. Overrides in your project `templates/` folder must match the syntax of the engine you pick.

## 7. Custom assets (CSS / JS)

The default stylesheet and JS are compiled into the binary as `style.css` and `main.js`, and written to `<out_dir>/assets/` on every build. You have three options for shipping additional assets:

1. **Custom CSS** (simplest, inlined into every page) — use `theme.custom_css`.
2. **Static files** — drop files under `docs/` (or wherever `root` points) and reference them from templates. They're copied verbatim. See [Static Assets](./static-assets.md).
3. **Sass pipeline** — enable the `sass` cargo feature and configure `[sass]`:
   ```toml title="novel.toml"
   [sass]
   entries    = [["assets/scss/main.scss", "assets/css/main.css"]]
   load_paths = ["assets/scss"]
   ```

If `asset_fingerprint = true`, Novel hashes `style.css` / `main.js` into `style.<hash>.css` / `main.<hash>.js` and exposes the names via `asset_css` / `asset_js` in the template context — use them in overrides rather than hard-coding filenames.

## Built-in components

The default theme includes visual components you can use from Markdown out of the box:

| Component | Description |
|-----------|-------------|
| **Container directives** | `::: tip`, `warning`, `danger`, `info`, `note`, `details` |
| **Tabs** | Tabbed content blocks |
| **Steps** | Numbered step-by-step guides |
| **Badges** | Inline status badges |
| **Code blocks** | Syntax highlighting, line numbers, titles, copy button |
| **Image zoom** | Click to zoom on images |
| **Back to top** | Floating button when scrolled down |
| **Table of contents** | Auto-generated from page headings |

See the [Markdown guide](./markdown.md) for the syntax of each.

## Recipe: a minimal custom theme

Putting it all together — a fully re-skinned site:

```text
my-site/
├── novel.toml
├── assets/
│   └── custom.css
└── templates/
    └── base.html
```

```toml title="novel.toml"
title = "Acme Docs"
[theme]
custom_css = "assets/custom.css"

[theme.colors]
accent       = "#f97316"
accent-hover = "#ea580c"
bg-primary   = "#fffaf5"
font-sans    = '"Inter", system-ui, sans-serif'
```

```css title="assets/custom.css"
[data-theme="dark"] {
    --accent: #fb923c;
    --bg-primary: #0b0b0f;
    --bg-secondary: #14141c;
}
.navbar { backdrop-filter: blur(12px); }
```

```html title="templates/base.html"
{# Copied from the upstream base.html and modified — see link in Theming › Overriding blocks #}
<!DOCTYPE html>
<html lang="{{ site.lang }}" data-theme="light">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}{{ site.title }}{% endblock %}</title>
    <link rel="stylesheet" href="{{ site.base }}assets/{{ asset_css }}">
    {% if theme_css_overrides %}<style>:root { {{ theme_css_overrides | safe }} }</style>{% endif %}
    {% if custom_css_content %}<style>{{ custom_css_content | safe }}</style>{% endif %}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet">
    {% block head %}{% endblock %}
</head>
<body>
    <!-- your custom layout -->
    {% block content %}{% endblock %}
</body>
</html>
```

That's it — a completely restyled site with a custom shell, custom colors, and custom fonts, still driven by Novel's routing, Markdown, and search.
