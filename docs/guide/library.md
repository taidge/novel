# Library API

Novel can run standalone as a CLI, or be embedded as a library in your own Rust application — a web server, a build tool, or anything else.

## Add the dependency

```toml title="Cargo.toml"
[dependencies]
novel-core = { path = "path/to/novel/crates/novel-core" }
```

## Quick Start

Build a docs site and write it to disk in three lines:

```rust
let site = novel_core::Novel::new("docs").build()?;
site.write_to("dist")?;
```

## Loading from `novel.toml`

If you have a project with a `novel.toml` config file:

```rust
let site = novel_core::Novel::load(".")?.build()?;
site.write_to_default_output()?;
```

## Builder API

Customise the site programmatically with the builder pattern:

```rust
use novel_core::Novel;

let site = Novel::new("docs")
    .title("My API Reference")
    .description("Generated docs for my-crate")
    .base("/docs/")
    .site_url("https://example.com")
    .with_theme(|t| {
        t.dark_mode = true;
        t.footer = Some("Built with Novel".into());
        t.last_updated = true;
    })
    .build()?;

site.write_to("./output")?;
```

Available builder methods:

| Method | Description |
|--------|-------------|
| `title()` | Site title |
| `description()` | Site description |
| `base()` | Base URL path (e.g. `"/docs/"`) |
| `lang()` | Language code (default `"en"`) |
| `out_dir()` | Output directory name |
| `site_url()` | Full URL for sitemap/RSS |
| `theme()` | Replace the theme config |
| `with_theme()` | Mutate theme config via closure |
| `config()` | Replace the entire `SiteConfig` |
| `project_root()` | Override project root for file embeds |

## The `BuiltSite`

Calling `.build()` returns a `BuiltSite` which holds all processed pages and can render HTML on demand.

### Access pages

```rust
let site = Novel::new("docs").build()?;

// iterate all pages
for page in site.pages() {
    println!("{}: {}", page.route.route_path, page.title);
}

// look up a specific page
if let Some(page) = site.page("/guide/intro") {
    println!("Found: {}", page.title);
}
```

### Render individual pages

```rust
let site = Novel::new("docs").build()?;

// render one page to a full HTML string
let page = site.page("/guide/intro").unwrap();
let html = site.render_page(page)?;

// render the 404 page
let not_found_html = site.render_404()?;
```

### Static assets

```rust
let css: &str = site.css();   // the stylesheet
let js: &str  = site.js();    // the client-side script
```

### Generated data

```rust
// search index as JSON
let json: String = site.search_index_json()?;

// sitemap XML (None if site_url is not set)
if let Some(xml) = site.sitemap_xml() {
    std::fs::write("sitemap.xml", xml)?;
}

// Atom/RSS feed (None if site_url is not set)
if let Some(xml) = site.feed_xml() {
    std::fs::write("feed.xml", xml)?;
}
```

## Embedding in a Web Server

Here is a minimal example using Axum:

```rust title="server.rs"
use axum::{Router, routing::get, extract::Path, response::Html};

#[tokio::main]
async fn main() {
    // build once at startup
    let site = novel_core::Novel::new("docs")
        .title("My App Docs")
        .build()
        .expect("failed to build docs");

    // serve pages
    let app = Router::new()
        .route("/docs/*path", get(move |Path(path): Path<String>| {
            let route = format!("/{}", path);
            let html = site.page(&route)
                .and_then(|p| site.render_page(p).ok())
                .unwrap_or_else(|| site.render_404().unwrap());
            async move { Html(html) }
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

::: tip
The `BuiltSite` struct is `Send` but not `Sync` (due to the template engine). Wrap it in an `Arc<BuiltSite>` if you need shared access across handlers, or rebuild per-request for hot-reload scenarios.
:::

## Workflow Comparison

::: tabs
== CLI (standalone)

```bash
novel build
novel dev
novel preview
```

== Library (embedded)

```rust
let site = Novel::new("docs").build()?;
site.write_to("dist")?;

// or serve directly from memory
let html = site.render_page(page)?;
```
:::
