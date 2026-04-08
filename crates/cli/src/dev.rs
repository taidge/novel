use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use novel_core::Novel;
use novel_shared::SiteConfig;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use salvo::sse::{SseEvent, SseKeepAlive};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;

/// Quiet window after the last filesystem event before triggering a rebuild.
/// Real debouncing: each new event resets the timer.
const DEBOUNCE_MS: u64 = 200;

// ---------------------------------------------------------------------------
// Why no incremental rebuild?
//
// Investigated 2026-04-09. On a 40-page bilingual site (en + zh, ~947 KB
// dist), measured rebuild times after a single markdown edit:
//
//     run 1: 52 ms
//     run 2: 3178 ms   ← one-shot outlier
//     run 3: 211 ms
//     run 4: 44 ms
//     run 5: 54 ms
//     run 6: 45 ms
//
// Median ≈ 50 ms. The 3178 ms outlier on run 2 is attributed to Windows
// Defender / FS-cache effects scanning the freshly-rewritten dist/
// contents after `clean_dir_contents` — it does not reproduce in steady
// state and is not addressable from inside Novel.
//
// A single-page fast path would shave maybe 20-30 ms off the 50 ms
// steady state, while adding cache-invalidation logic, stale-state
// desync risk (prev/next links, sitemap, search index, and taxonomies
// are all coupled to page neighbours), and a second code path users
// would need to reason about. Not worth it: the full rebuild is
// already faster than the browser's SSE reconnect.
//
// Revisit if the median ever climbs above ~250 ms on a typical doc site.
// ---------------------------------------------------------------------------

/// Small JS snippet injected by the livereload plugin (dev mode only).
const LIVERELOAD_JS: &str = r#"(function(){
  const es = new EventSource('/__livereload');
  es.onmessage = function() { location.reload(); };
  es.onerror = function() { setTimeout(function(){ location.reload(); }, 1000); };
})();"#;

/// Shared state holding the broadcast sender.
static RELOAD_TX: std::sync::OnceLock<broadcast::Sender<()>> = std::sync::OnceLock::new();

/// SSE endpoint that streams reload events to the browser.
#[handler]
async fn livereload_sse(res: &mut Response) {
    let Some(tx) = RELOAD_TX.get() else {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        return;
    };
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|r| match r {
        Ok(()) => Some(Ok::<_, salvo::Error>(SseEvent::default().text("reload"))),
        Err(_) => None,
    });
    SseKeepAlive::new(stream).stream(res);
}

/// Serves the livereload JS snippet.
#[handler]
async fn livereload_js(res: &mut Response) {
    res.render(Text::Plain(LIVERELOAD_JS));
}

/// Inject `<script src="/__livereload.js"></script>` before `</body>` in HTML.
struct LiveReloadPlugin;

impl novel_core::plugin::Plugin for LiveReloadPlugin {
    fn name(&self) -> &str {
        "livereload"
    }

    fn transform_html(&self, html: String, _page: &novel_shared::PageData) -> String {
        html.replace(
            "</body>",
            "<script src=\"/__livereload.js\"></script>\n</body>",
        )
    }
}

fn build_site(project_root: &Path) -> Result<novel_core::BuiltSite> {
    novel_core::DirNovel::load(project_root)?
        .plugin(novel_core::plugins::SitemapPlugin)
        .plugin(novel_core::plugins::FeedPlugin)
        .plugin(novel_core::plugins::SearchIndexPlugin)
        .plugin(novel_core::plugins::RobotsPlugin)
        .plugin(novel_core::plugins::RedirectsPlugin)
        .plugin(LiveReloadPlugin)
        .build()
}

/// Run the development server with file watching and live reload
pub async fn run_dev_server(project_root: &Path, port: u16) -> Result<()> {
    let project_root = project_root.to_path_buf();

    // Initial build
    let site = build_site(&project_root)?;
    site.write_to_default_output()?;

    let config = SiteConfig::load(&project_root)?;
    let output_dir = config.output_dir(&project_root);
    let docs_root = config.docs_root(&project_root);

    // Broadcast channel for signaling rebuilds
    let (reload_tx, _) = broadcast::channel::<()>(16);
    let reload_tx_for_watcher = reload_tx.clone();
    let _ = RELOAD_TX.set(reload_tx);

    // File watcher → mpsc channel of triggering paths. The rebuild task
    // does true silence-based debouncing on this channel: each new event
    // resets the timer, so a burst of edits coalesces into one rebuild.
    let (event_tx, event_rx) = mpsc::unbounded_channel::<PathBuf>();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        let Ok(event) = res else {
            return;
        };
        for path in event.paths {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let should_rebuild =
                matches!(ext, "md" | "json" | "toml" | "kdl" | "typ" | "yaml" | "yml");
            if should_rebuild {
                let _ = event_tx.send(path);
            }
        }
    })?;

    watcher.watch(&docs_root, RecursiveMode::Recursive)?;

    // Watch config file (KDL or TOML)
    if let Some(config_path) = SiteConfig::config_path(&project_root) {
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
    }

    // Rebuild task — debounces by waiting for DEBOUNCE_MS of silence
    // after the most recent event, not after the first.
    let project_root_for_rebuild = project_root.clone();
    let docs_root_for_rebuild = docs_root.clone();
    tokio::spawn(async move {
        let mut rx = event_rx;
        loop {
            // Wait for the first event of a burst.
            let Some(first_path) = rx.recv().await else {
                return; // channel closed
            };
            let mut last_path = first_path;
            // Coalesce: keep extending the burst as long as new events
            // arrive within DEBOUNCE_MS of the last one.
            loop {
                match tokio::time::timeout(Duration::from_millis(DEBOUNCE_MS), rx.recv()).await {
                    Ok(Some(p)) => last_path = p,
                    Ok(None) => return, // channel closed
                    Err(_) => break,    // quiet — go rebuild
                }
            }

            let display_path = last_path
                .strip_prefix(&docs_root_for_rebuild)
                .unwrap_or(&last_path)
                .display()
                .to_string();
            info!("File changed: {} → rebuilding…", display_path);

            let started = Instant::now();
            match build_site(&project_root_for_rebuild) {
                Ok(site) => {
                    if let Err(e) = site.write_to_default_output() {
                        tracing::error!("Rebuild failed (write): {}", e);
                    } else {
                        info!("Rebuild complete in {} ms", started.elapsed().as_millis());
                        let _ = reload_tx_for_watcher.send(());
                    }
                }
                Err(e) => tracing::error!("Rebuild failed (build): {}", e),
            }
        }
    });

    // Build router
    let router = Router::new()
        .push(Router::with_path("__livereload").get(livereload_sse))
        .push(Router::with_path("__livereload.js").get(livereload_js))
        .push(
            Router::with_path("<**path>").get(
                StaticDir::new([output_dir.to_str().unwrap_or("dist")])
                    .defaults("index.html")
                    .auto_list(false),
            ),
        );

    info!("Dev server running at http://localhost:{}", port);

    let acceptor = TcpListener::new(format!("0.0.0.0:{port}")).bind().await;
    // `serve` blocks until the process is killed; `watcher` is owned by
    // this scope and only dropped on shutdown, which is what keeps it
    // alive throughout the server's lifetime.
    Server::new(acceptor).serve(router).await;
    Ok(())
}

/// Serve a static directory (for preview)
pub async fn serve_static(dir: &Path, port: u16) -> Result<()> {
    let router = Router::with_path("<**path>").get(
        StaticDir::new([dir.to_str().unwrap_or("dist")])
            .defaults("index.html")
            .auto_list(false),
    );

    info!("Preview server running at http://localhost:{}", port);

    let acceptor = TcpListener::new(format!("0.0.0.0:{port}")).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}
