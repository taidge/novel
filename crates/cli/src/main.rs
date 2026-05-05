use anyhow::Result;
use clap::{Parser, Subcommand};
use novel_core::Novel;
use std::path::PathBuf;
use tracing::info;

mod check;
mod clean;
mod dev;
mod init;
mod new_page;

#[derive(Parser)]
#[command(
    name = "novel",
    version,
    about = "A fast static documentation site generator"
)]
struct Cli {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a development server with live reload
    Dev {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Build the static site for production
    Build {
        /// Force full rebuild (bypass cache)
        #[arg(long)]
        force: bool,
        /// Include draft pages
        #[arg(long)]
        drafts: bool,
        /// Include pages with future dates
        #[arg(long)]
        future: bool,
    },
    /// Preview the built site locally
    Preview {
        /// Port to listen on
        #[arg(short, long, default_value = "4000")]
        port: u16,
    },
    /// Initialize a new documentation project
    Init {
        /// Project name
        #[arg(default_value = "my-docs")]
        name: String,
    },
    /// Check the site for issues (dead links, missing descriptions, orphan pages)
    Check,
    /// Delete the output directory
    Clean,
    /// Create a new documentation page
    New {
        /// Page path relative to docs root (without .md extension)
        path: String,
        /// Page layout (doc, page, blog)
        #[arg(long, default_value = "doc")]
        layout: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let project_root = cli.root.canonicalize().unwrap_or(cli.root);

    match cli.command {
        Commands::Dev { port } => {
            dev::run_dev_server(&project_root, port).await?;
        }
        Commands::Build {
            force: _,
            drafts,
            future,
        } => {
            info!("Building site...");
            let mut novel = novel_core::DirNovel::load(&project_root)?;
            {
                // Apply CLI overrides for content config
                let cfg = novel.config_mut();
                if drafts {
                    cfg.content.drafts = true;
                }
                if future {
                    cfg.content.future = true;
                }
            }
            let site = novel
                .plugin(novel_core::plugins::SitemapPlugin)
                .plugin(novel_core::plugins::FeedPlugin)
                .plugin(novel_core::plugins::SearchIndexPlugin)
                .plugin(novel_core::plugins::LlmsTxtPlugin)
                .plugin(novel_core::plugins::MarkdownMirrorPlugin)
                .plugin(novel_core::plugins::PwaPlugin)
                .plugin(novel_core::plugins::RobotsPlugin)
                .plugin(novel_core::plugins::RedirectsPlugin)
                .build()?;
            site.write_to_default_output()?;
        }
        Commands::Preview { port } => {
            info!("Previewing built site on http://localhost:{}", port);
            let config = novel_shared::SiteConfig::load(&project_root)?;
            let output_dir = config.output_dir(&project_root);
            dev::serve_static(&output_dir, port).await?;
        }
        Commands::Init { name } => {
            init::create_project(&project_root, &name)?;
        }
        Commands::Check => {
            check::run_check(&project_root)?;
        }
        Commands::Clean => {
            clean::run_clean(&project_root)?;
        }
        Commands::New { path, layout } => {
            new_page::run_new_page(&project_root, &path, &layout)?;
        }
    }

    Ok(())
}
