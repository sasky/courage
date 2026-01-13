use anyhow::Result;
use reqwest::{multipart, Client};
use serde::Deserialize;

use crate::utils::http::{handle_api_response, parse_json_response};

const OPENAI_TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

pub struct WhisperClient {
    client: Client,
    api_key: String,
}

impl WhisperClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Transcribe audio data using OpenAI Whisper
    pub async fn transcribe(&self, audio_data: Vec<u8>, filename: &str) -> Result<String> {
        // Determine file extension for content type
        let extension = filename
            .rsplit('.')
            .next()
            .unwrap_or("ogg");

        let mime_type = match extension {
            "ogg" | "oga" => "audio/ogg",
            "mp3" => "audio/mpeg",
            "m4a" => "audio/mp4",
            "wav" => "audio/wav",
            "webm" => "audio/webm",
            _ => "audio/ogg", // Default for Telegram voice messages
        };

        let file_part = multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = multipart::Form::new()
            .text("model", "whisper-1")
            .part("file", file_part);

        let response = self
            .client
            .post(OPENAI_TRANSCRIPTION_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let response = handle_api_response(response, "OpenAI").await?;
        let whisper_response: WhisperResponse = parse_json_response(response, "Whisper response").await?;

        Ok(whisper_response.text)
    }
}
