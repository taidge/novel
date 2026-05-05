# Offline / PWA

Novel can generate a small Progressive Web App shell for offline reading.

## Enable PWA Output

```toml title="novel.toml"
[pwa]
enabled = true
name = "My Docs"
short_name = "Docs"
theme_color = "#3b82f6"
background_color = "#ffffff"
display = "standalone"
cache_search_index = true
```

When enabled, Novel writes:

- `manifest.webmanifest`
- `service-worker.js`
- `offline.html`

The default template registers the service worker automatically and links the manifest in `<head>`.

## Caching

The service worker precaches the site root, public page routes, built CSS/JS, `offline.html`, and the search index when `cache_search_index = true`. HTML navigation uses a network-first strategy and falls back to the cached page or `offline.html`.
