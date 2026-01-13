mod config;
mod llm;
mod people;
mod state;
mod telegram;
mod transcription;
mod utils;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::telegram::CourageBot;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "courage=info,teloxide=warn".into()),
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

    // Create and run the bot
    let bot = CourageBot::new(config);
    bot.run().await?;

    Ok(())
}
