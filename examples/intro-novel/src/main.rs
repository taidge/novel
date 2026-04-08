//! # Intro to Novel — library example
//!
//! This example shows how to use `novel-core` as a library to build a
//! documentation site from an existing `docs/` directory. As a meta twist,
//! the site it builds *is* the Novel project's own documentation.
//!
//! Run it from the workspace root:
//!
//! ```bash
//! cargo run -p intro-novel
//! ```
//!
//! The generated site will be written to `examples/intro-novel/dist/`.

use std::path::PathBuf;

use anyhow::Result;
use novel_core::plugins::{
    FeedPlugin, RedirectsPlugin, RobotsPlugin, SearchIndexPlugin, SitemapPlugin,
};
use novel_core::{DirNovel, Novel};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Resolve paths relative to the workspace root so the example works no
    // matter where `cargo run` is invoked from.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let docs_dir = workspace_root.join("docs");
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dist");

    tracing::info!("Building Novel's own docs from {}", docs_dir.display());

    // Use the builder API to construct a site. `DirNovel::new` takes the
    // path to the directory containing the `.md` files; additional options
    // are layered on top with fluent setters.
    //
    // We register the same default plugins the `novel` CLI ships with. In
    // particular, `SearchIndexPlugin` emits `assets/search-index.json` —
    // without it the client-side search box in the generated pages has
    // nothing to fetch and silently returns no results.
    let site = DirNovel::new(&docs_dir)
        .title("Novel")
        .description("A fast static documentation site generator built with Rust")
        .base("/")
        .lang("en")
        .site_url("https://novel.rs")
        .with_theme(|t| {
            t.dark_mode = true;
            t.footer = Some("Built with Novel — library example".into());
            t.last_updated = true;
        })
        .plugin(SitemapPlugin)
        .plugin(FeedPlugin)
        .plugin(SearchIndexPlugin)
        .plugin(RobotsPlugin)
        .plugin(RedirectsPlugin)
        .build()?;

    // Write the rendered site to disk.
    site.write_to(&out_dir)?;
    tracing::info!("Site written to {}", out_dir.display());

    // Demonstrate on-demand page rendering. A `BuiltSite` keeps all pages in
    // memory, so you can render individual routes to HTML strings — handy
    // when embedding Novel inside a web server instead of writing files.
    if let Some(home) = site.page("/") {
        let html = site.render_page(home)?;
        tracing::info!(
            "Rendered home page in memory: {} bytes, title = {:?}",
            html.len(),
            home.title
        );
    }

    // A quick summary of what was built.
    tracing::info!("Built {} pages total", site.pages().len());

    Ok(())
}
