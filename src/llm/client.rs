use anyhow::{Context, Result};
use reqwest::Client;

use super::prompts::build_system_prompt;
use super::types::{LlmResponse, OpenAIMessage, OpenAIRequest, OpenAIResponse};
use crate::utils::http::{handle_api_response, parse_json_response};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    /// Process a transcript and get structured actions
    pub async fn process_transcript(
        &self,
        transcript: &str,
        people_list: &str,
    ) -> Result<LlmResponse> {
        let system_prompt = build_system_prompt(people_list);

        let user_message = format!(
            r#"## TRANSCRIPT TO PROCESS
"{transcript}"

## RESPOND WITH VALID JSON ONLY - NO OTHER TEXT"#
        );

        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: user_message,
            },
        ];

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(4096),
            response_format: Some(serde_json::json!({"type": "json_object"})),
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response = handle_api_response(response, "OpenAI").await?;
        let api_response: OpenAIResponse = parse_json_response(response, "OpenAI response").await?;

        let text = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Parse the JSON response from OpenAI
        let llm_response: LlmResponse =
            serde_json::from_str(&text).context("Failed to parse LLM response as JSON")?;

        Ok(llm_response)
    }

    /// Continue a conversation with clarification
    pub async fn continue_with_clarification(
        &self,
        original_transcript: &str,
        clarification: &str,
        people_list: &str,
    ) -> Result<LlmResponse> {
        let system_prompt = build_system_prompt(people_list);

        let user_message = format!(
            r#"## ORIGINAL TRANSCRIPT
"{original_transcript}"

## USER CLARIFICATION
"{clarification}"

Based on the user's clarification, now process the original transcript and respond with valid JSON."#
        );

        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: user_message,
            },
        ];

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(4096),
            response_format: Some(serde_json::json!({"type": "json_object"})),
        };

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response = handle_api_response(response, "OpenAI").await?;
        let api_response: OpenAIResponse = parse_json_response(response, "OpenAI response").await?;

        let text = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let llm_response: LlmResponse =
            serde_json::from_str(&text).context("Failed to parse LLM response as JSON")?;

        Ok(llm_response)
    }
}
