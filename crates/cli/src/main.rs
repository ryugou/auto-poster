mod commands;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use auto_poster_shared::config;
use auto_poster_shared::db;
use auto_poster_shared::telemetry;

use commands::Commands;

#[derive(Debug, Parser)]
#[command(name = "auto-poster", about = "X 自動運用システム")]
struct Cli {
    #[arg(long, default_value = "config", global = true)]
    config_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let app_config = config::load_app_config(&cli.config_dir)?;
    telemetry::init(&app_config.log_level, &app_config.log_format);

    let pool = db::create_pool(&app_config.database_url).await?;

    match cli.command {
        Commands::Migrate => {
            commands::migrate::execute(&pool).await?;
        }
        Commands::Seed => {
            commands::seed::execute(&pool, &cli.config_dir).await?;
        }
        Commands::Collect { ref account } => {
            commands::collect::execute(&pool, account.as_deref()).await?;
        }
        Commands::Generate { ref account } => {
            commands::generate::execute(&pool, account.as_deref()).await?;
        }
        Commands::Operate => {
            commands::operate::execute(&pool).await?;
        }
        Commands::Serve { ref addr } => {
            commands::serve::execute(&pool, addr).await?;
        }
    }

    Ok(())
}
