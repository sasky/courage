use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram bot token
    pub telegram_token: String,

    /// OpenAI API key for Whisper transcription
    pub openai_api_key: String,

    /// Anthropic API key for Claude
    pub anthropic_api_key: String,

    /// Path to the People directory (markdown files)
    pub people_dir: PathBuf,

    /// Claude model to use (default: claude-3-5-haiku-20241022)
    pub claude_model: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let telegram_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .context("TELEGRAM_BOT_TOKEN must be set")?;

        let openai_api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY must be set")?;

        let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY must be set")?;

        let people_dir = std::env::var("PEOPLE_DIR")
            .unwrap_or_else(|_| "./People".to_string());
        let people_dir = PathBuf::from(people_dir);

        let claude_model = std::env::var("CLAUDE_MODEL")
            .unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string());

        Ok(Self {
            telegram_token,
            openai_api_key,
            anthropic_api_key,
            people_dir,
            claude_model,
        })
    }
}
