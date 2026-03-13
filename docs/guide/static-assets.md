# Static Assets

Sapid automatically handles static assets (images, fonts, PDFs, etc.) in your documentation directory.

## How It Works

Any non-Markdown file in your `docs/` directory is copied to the output as-is during `sapid build`. The directory structure is preserved.

```
docs/
├── index.md
├── logo.png              # → dist/logo.png
├── guide/
│   ├── getting-started.md
│   └── screenshot.png    # → dist/guide/screenshot.png
└── assets/
    ├── diagram.svg       # → dist/assets/diagram.svg
    └── sample.pdf        # → dist/assets/sample.pdf
```

## Referencing Assets

### From Markdown

Use standard Markdown image syntax with relative paths:

```markdown
![Screenshot](./screenshot.png)
```

Or reference from the docs root:

```markdown
![Logo](/logo.png)
```

### Images

Images in Markdown are automatically:
- **Lazy-loaded** with `loading="lazy"` for performance
- **Zoomable** — click to view full-size in an overlay

### Favicon and Logo

Set the favicon and logo in `sapid.toml`:

```toml title="sapid.toml"
icon = "/favicon.ico"
logo = "/logo.svg"
```

Place these files in your `docs/` directory.

## Built-in Assets

Sapid generates the following assets automatically in `dist/assets/`:

| File | Description |
|------|-------------|
| `style.css` | Site stylesheet |
| `main.js` | Client-side JavaScript (search, theme toggle, etc.) |
| `search-index.json` | Search index for client-side search |

## Excluded Files

The following files are excluded from copying:
- `.md` files (processed as pages instead)
- `_meta.json` files (used for sidebar ordering, not published)
