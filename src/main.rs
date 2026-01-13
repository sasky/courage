mod config;
mod llm;
mod people;
mod state;
mod telegram;
mod transcription;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::telegram::CourageBot;
use crate::tui::App;

/// Courage - Personal Relationship Database with Voice Input
#[derive(Parser, Debug)]
#[command(name = "courage")]
#[command(about = "Personal relationship database with voice input")]
#[command(version)]
struct Args {
    /// Run in TUI mode instead of Telegram bot mode
    #[arg(long)]
    tui: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging (quieter for TUI mode)
    let log_filter = if args.tui {
        "courage=warn"
    } else {
        "courage=info,teloxide=warn"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Courage - Personal Relationship Database");

    // Load configuration
    let config = Config::from_env()?;

    tracing::info!("People directory: {:?}", config.people_dir);
    tracing::info!("Claude model: {}", config.claude_model);

    // Ensure people directory exists
    tokio::fs::create_dir_all(&config.people_dir).await?;

    if args.tui {
        // Run in TUI mode
        run_tui(config)?;
    } else {
        // Run in Telegram bot mode
        let bot = CourageBot::new(config);
        bot.run().await?;
    }

    Ok(())
}

/// Run the TUI application
fn run_tui(config: Config) -> Result<()> {
    let mut terminal = tui::init()?;
    let mut app = App::new(config);

    let result = app.run(&mut terminal);

    // Always restore terminal, even on error
    tui::restore()?;

    result
}
