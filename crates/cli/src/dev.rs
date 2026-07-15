use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher};
use novel_core::Novel;
use novel_shared::SiteConfig;
use salvo::prelude::*;
use salvo::serve_static::StaticDir;
use salvo::sse::{SseEvent, SseKeepAlive};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;
use tracing::warn;

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
        .plugin(novel_core::plugins::LlmsTxtPlugin)
        .plugin(novel_core::plugins::MarkdownMirrorPlugin)
        .plugin(novel_core::plugins::PwaPlugin)
        .plugin(novel_core::plugins::RobotsPlugin)
        .plugin(novel_core::plugins::RedirectsPlugin)
        .plugin(LiveReloadPlugin)
        .build()
}

/// Run the development server with file watching and live reload
pub async fn run_dev_server(project_root: &Path, host: &str, port: u16) -> Result<()> {
    let project_root = project_root.to_path_buf();
    warn_if_public_bind(host);

    // Initial build
    let site = build_site(&project_root)?;
    site.write_to_default_output()?;

    let config = SiteConfig::load(&project_root)?;
    let output_dir = config.output_dir_checked(&project_root)?;
    let docs_root = config.docs_root_checked(&project_root)?;

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
            if should_rebuild_path(&path) {
                let _ = event_tx.send(path);
            }
        }
    })?;

    for (path, mode) in collect_watch_paths(&project_root, &config, &docs_root) {
        if path.exists() {
            watcher.watch(&path, mode)?;
            info!("Watching {}", path.display());
        } else {
            warn!("Watch path does not exist yet: {}", path.display());
        }
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
    let static_root = static_dir_source(&output_dir)?;
    let router = Router::new()
        .push(Router::with_path("__livereload").get(livereload_sse))
        .push(Router::with_path("__livereload.js").get(livereload_js))
        .push(
            Router::with_path("<**path>").get(
                StaticDir::new([static_root])
                    .defaults("index.html")
                    .auto_list(false),
            ),
        );

    info!("Dev server running at {}", server_url(host, port));

    let acceptor = TcpListener::new(bind_address(host, port)).bind().await;
    // `serve` blocks until the process is killed; `watcher` is owned by
    // this scope and only dropped on shutdown, which is what keeps it
    // alive throughout the server's lifetime.
    Server::new(acceptor).serve(router).await;
    Ok(())
}

fn should_rebuild_path(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    matches!(
        ext.as_str(),
        "md" | "mdx"
            | "typ"
            | "json"
            | "toml"
            | "yaml"
            | "yml"
            | "html"
            | "tera"
            | "hbs"
            | "handlebars"
            | "css"
            | "scss"
            | "sass"
            | "js"
            | "mjs"
            | "ts"
            | "tsx"
            | "jsx"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "svg"
            | "webp"
            | "avif"
            | "ico"
            | "pdf"
            | "woff"
            | "woff2"
            | "ttf"
            | "otf"
    )
}

fn collect_watch_paths(
    project_root: &Path,
    config: &SiteConfig,
    docs_root: &Path,
) -> Vec<(PathBuf, RecursiveMode)> {
    let mut out = Vec::new();
    push_watch_path(&mut out, docs_root.to_path_buf(), RecursiveMode::Recursive);

    if let Some(config_path) = SiteConfig::config_path(project_root) {
        push_watch_path(&mut out, config_path, RecursiveMode::NonRecursive);
    }

    push_watch_path(
        &mut out,
        project_root.join("templates"),
        RecursiveMode::Recursive,
    );

    if let Some(path) = project_relative_path(project_root, config.theme.pack.as_deref()) {
        push_watch_path(&mut out, path, RecursiveMode::Recursive);
    }

    if let Some(path) = project_relative_path(project_root, config.theme.custom_css.as_deref()) {
        push_watch_path(&mut out, path, RecursiveMode::NonRecursive);
    }

    for entry in &config.sass.entries {
        if let Some(input) = entry.first()
            && let Some(path) = project_relative_path(project_root, Some(input.as_str()))
        {
            push_watch_path(&mut out, path, RecursiveMode::NonRecursive);
        }
    }

    for load_path in &config.sass.load_paths {
        if let Some(path) = project_relative_path(project_root, Some(load_path.as_str())) {
            push_watch_path(&mut out, path, RecursiveMode::Recursive);
        }
    }

    out
}

fn push_watch_path(out: &mut Vec<(PathBuf, RecursiveMode)>, path: PathBuf, mode: RecursiveMode) {
    if out.iter().any(|(existing, _)| existing == &path) {
        return;
    }
    out.push((path, mode));
}

fn project_relative_path(project_root: &Path, configured: Option<&str>) -> Option<PathBuf> {
    let configured = configured?.trim();
    if configured.is_empty() {
        return None;
    }

    let relative = Path::new(configured);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        warn!("Ignoring watch path outside project root: {}", configured);
        return None;
    }

    Some(project_root.join(relative))
}

/// Serve a static directory (for preview)
pub async fn serve_static(dir: &Path, host: &str, port: u16) -> Result<()> {
    warn_if_public_bind(host);
    let static_root = static_dir_source(dir)?;
    let router = Router::with_path("<**path>").get(
        StaticDir::new([static_root])
            .defaults("index.html")
            .auto_list(false),
    );

    info!("Preview server running at {}", server_url(host, port));

    let acceptor = TcpListener::new(bind_address(host, port)).bind().await;
    Server::new(acceptor).serve(router).await;

    Ok(())
}

fn static_dir_source(dir: &Path) -> Result<String> {
    dir.to_str().map(str::to_owned).ok_or_else(|| {
        anyhow::anyhow!(
            "Static server directory is not valid UTF-8 and cannot be served safely: {}",
            dir.display()
        )
    })
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.trim();
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let unbracketed = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    unbracketed
        .parse::<IpAddr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}

fn warn_if_public_bind(host: &str) {
    if !host_is_loopback(host) {
        warn!(
            "Security warning: binding to non-loopback host '{}' exposes the generated site to other machines; verify the output contains no secrets",
            host
        );
    }
}

fn bind_address(host: &str, port: u16) -> String {
    let host = host.trim();
    if host.starts_with('[') && host.ends_with(']') {
        format!("{host}:{port}")
    } else if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn server_url(host: &str, port: u16) -> String {
    format!("http://{}", bind_address(host, port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use novel_shared::config::{SassConfig, ThemeConfig};

    #[test]
    fn rebuild_filter_includes_templates_styles_and_static_assets() {
        for path in [
            "docs/index.md",
            "templates/doc.html",
            "templates/doc.tera",
            "templates/doc.hbs",
            "assets/site.css",
            "assets/site.scss",
            "docs/logo.svg",
            "docs/manual.pdf",
        ] {
            assert!(should_rebuild_path(Path::new(path)), "{path}");
        }

        assert!(!should_rebuild_path(Path::new("dist/index.tmp")));
    }

    #[test]
    fn watch_paths_include_templates_theme_css_and_sass_inputs() {
        let project_root = Path::new("project");
        let config = SiteConfig {
            theme: ThemeConfig {
                pack: Some("themes/midnight".to_string()),
                custom_css: Some("assets/site.css".to_string()),
                ..ThemeConfig::default()
            },
            sass: SassConfig {
                entries: vec![vec![
                    "assets/scss/main.scss".to_string(),
                    "assets/css/main.css".to_string(),
                ]],
                load_paths: vec!["assets/scss".to_string()],
            },
            ..SiteConfig::default()
        };

        let paths: Vec<PathBuf> =
            collect_watch_paths(project_root, &config, Path::new("project/docs"))
                .into_iter()
                .map(|(path, _)| path)
                .collect();

        assert!(paths.contains(&PathBuf::from("project/docs")));
        assert!(paths.contains(&PathBuf::from("project/templates")));
        assert!(paths.contains(&PathBuf::from("project/themes/midnight")));
        assert!(paths.contains(&PathBuf::from("project/assets/site.css")));
        assert!(paths.contains(&PathBuf::from("project/assets/scss/main.scss")));
        assert!(paths.contains(&PathBuf::from("project/assets/scss")));
    }

    #[test]
    fn loopback_detection_is_conservative() {
        for host in ["127.0.0.1", "::1", "[::1]", "localhost", "LOCALHOST"] {
            assert!(host_is_loopback(host), "{host}");
        }
        for host in ["0.0.0.0", "::", "192.168.1.10", "docs.example.com"] {
            assert!(!host_is_loopback(host), "{host}");
        }
    }

    #[test]
    fn ipv6_bind_addresses_and_urls_are_bracketed() {
        assert_eq!(bind_address("::1", 3000), "[::1]:3000");
        assert_eq!(bind_address("[::1]", 3000), "[::1]:3000");
        assert_eq!(server_url("::1", 3000), "http://[::1]:3000");
        assert_eq!(bind_address("127.0.0.1", 3000), "127.0.0.1:3000");
    }
}
