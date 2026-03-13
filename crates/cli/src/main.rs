use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod dev;
mod init;

#[derive(Parser)]
#[command(
    name = "sapid",
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
    Build,
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
        Commands::Build => {
            info!("Building site...");
            let site = sapid_core::Sapid::load(&project_root)?.build()?;
            site.write_to_default_output()?;
        }
        Commands::Preview { port } => {
            info!("Previewing built site on http://localhost:{}", port);
            let config = sapid_shared::SiteConfig::load(&project_root)?;
            let output_dir = config.output_dir(&project_root);
            dev::serve_static(&output_dir, port).await?;
        }
        Commands::Init { name } => {
            init::create_project(&project_root, &name)?;
        }
    }

    Ok(())
}
