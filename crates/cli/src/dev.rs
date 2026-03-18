use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use novel_core::Novel;
use novel_shared::SiteConfig;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use salvo::sse::{SseEvent, SseKeepAlive};
use std::path::Path;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::info;

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

    // File watcher with debounce signaling
    let (rebuild_tx, rebuild_rx) = tokio::sync::watch::channel(());
    let rebuild_tx_clone = rebuild_tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let dominated = event.paths.iter().any(|p| {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                ext == "md" || ext == "json" || ext == "toml"
            });
            if dominated {
                info!("File changed, rebuilding...");
                let _ = rebuild_tx_clone.send(());
            }
        }
    })?;

    watcher.watch(&docs_root, RecursiveMode::Recursive)?;

    // Watch config file too
    let config_path = project_root.join("novel.toml");
    if config_path.exists() {
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
    }

    // Rebuild task
    let project_root_for_rebuild = project_root.clone();
    tokio::spawn(async move {
        let mut rx = rebuild_rx;
        while rx.changed().await.is_ok() {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            match build_site(&project_root_for_rebuild) {
                Ok(site) => {
                    if let Err(e) = site.write_to_default_output() {
                        tracing::error!("Rebuild failed: {}", e);
                    } else {
                        info!("Rebuild complete");
                        let _ = reload_tx_for_watcher.send(());
                    }
                }
                Err(e) => tracing::error!("Rebuild failed: {}", e),
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
    Server::new(acceptor).serve(router).await;

    // Keep watcher alive
    drop(watcher);
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
