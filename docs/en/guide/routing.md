# Routing

Novel uses file-based routing. Every `.md` file in your `docs/` directory automatically becomes a page.

## Convention

| File Path | Route URL |
|-----------|-----------|
| `docs/index.md` | `/` |
| `docs/guide/index.md` | `/guide/` |
| `docs/guide/getting-started.md` | `/guide/getting-started` |
| `docs/api/reference.md` | `/api/reference` |

The mapping follows these rules:

- `index.md` files map to the directory path (with a trailing slash)
- Other `.md` files map to their filename (without extension)
- Nested directories create nested URL paths
- File order is alphabetical by default; use `_meta.json` to customize

## Directory Structure

A typical docs directory looks like:

```
docs/
├── index.md              # → /
├── guide/
│   ├── _meta.json        # sidebar ordering for /guide/
│   ├── index.md          # → /guide/
│   ├── getting-started.md # → /guide/getting-started
│   └── configuration.md   # → /guide/configuration
└── api/
    ├── _meta.json        # sidebar ordering for /api/
    └── reference.md      # → /api/reference
```

## Sidebar Ordering with `_meta.json`

By default, sidebar items are sorted alphabetically. Create a `_meta.json` file in any directory to control the order:

```json title="docs/guide/_meta.json"
["getting-started", "configuration", "markdown", "deploy"]
```

Each entry is the filename without the `.md` extension. Pages listed first appear first in the sidebar.

### Advanced `_meta.json`

You can also use object entries for custom labels and grouping:

```json title="docs/guide/_meta.json"
[
  "getting-started",
  {
    "text": "Writing Content",
    "collapsed": false,
    "items": ["markdown", "file-embed"]
  },
  {
    "text": "Advanced",
    "collapsed": true,
    "items": ["configuration", "deploy"]
  }
]
```

## Navigation

Top-level directories automatically become navigation links in the navbar. For example, a `guide/` and `api/` directory produce "Guide" and "API" nav links.

To customise navigation, see the [Configuration](/guide/configuration) guide.

## Home Page

The root `index.md` file is treated as the home page when it has `page_type: home` in its frontmatter. See [Home Page](/guide/home-page) for details.

## Static Assets

Non-markdown files (images, PDFs, etc.) in the `docs/` directory are copied to the output as-is, preserving their directory structure. See [Static Assets](/guide/static-assets) for more.
