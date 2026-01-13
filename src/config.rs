use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram bot token
    pub telegram_token: String,

    /// OpenAI API key for Whisper transcription and LLM
    pub openai_api_key: String,

    /// Path to the People directory (markdown files)
    pub people_dir: PathBuf,

    /// LLM model to use (default: gpt-4o-mini)
    pub llm_model: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let telegram_token =
            std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN must be set")?;

        let openai_api_key =
            std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY must be set")?;

        let people_dir = std::env::var("PEOPLE_DIR").unwrap_or_else(|_| "./People".to_string());
        let people_dir = PathBuf::from(people_dir);

        let llm_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        Ok(Self {
            telegram_token,
            openai_api_key,
            people_dir,
            llm_model,
        })
    }
}
