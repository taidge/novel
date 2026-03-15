use anyhow::Result;
use axum::Router;
use notify::{Event, RecursiveMode, Watcher};
use novel_core::Novel;
use novel_shared::SiteConfig;
use std::path::Path;
use tokio::sync::watch;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing::info;

/// Run the development server with file watching and live reload
pub async fn run_dev_server(project_root: &Path, port: u16) -> Result<()> {
    let project_root = project_root.to_path_buf();

    // Initial build
    let site = novel_core::DirNovel::load(&project_root)?.build()?;
    site.write_to_default_output()?;

    let config = SiteConfig::load(&project_root)?;
    let output_dir = config.output_dir(&project_root);
    let docs_root = config.docs_root(&project_root);

    // Channel for signaling rebuilds
    let (rebuild_tx, rebuild_rx) = watch::channel(());
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();

    // File watcher
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

            match novel_core::DirNovel::load(&project_root_for_rebuild) {
                Ok(xp) => match xp.build() {
                    Ok(site) => {
                        if let Err(e) = site.write_to_default_output() {
                            tracing::error!("Rebuild failed: {}", e);
                        } else {
                            info!("Rebuild complete");
                            reloader.reload();
                        }
                    }
                    Err(e) => tracing::error!("Rebuild failed: {}", e),
                },
                Err(e) => tracing::error!("Config reload failed: {}", e),
            }
        }
    });

    // Serve the output directory
    let app = Router::new()
        .fallback_service(ServeDir::new(&output_dir).append_index_html_on_directories(true))
        .layer(livereload);

    let addr = format!("0.0.0.0:{}", port);
    info!("Dev server running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve a static directory (for preview)
pub async fn serve_static(dir: &Path, port: u16) -> Result<()> {
    let app =
        Router::new().fallback_service(ServeDir::new(dir).append_index_html_on_directories(true));

    let addr = format!("0.0.0.0:{}", port);
    info!("Preview server running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
