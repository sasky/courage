use anyhow::{Context, Result};
use reqwest::Client;

use super::prompts::build_system_prompt;
use super::types::{ClaudeApiResponse, ClaudeMessage, ClaudeRequest, LlmResponse};
use crate::utils::http::{handle_api_response, parse_json_response};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
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

        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: format!("{}\n\n{}", system_prompt, user_message),
        }];

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages,
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response = handle_api_response(response, "Anthropic").await?;
        let api_response: ClaudeApiResponse =
            parse_json_response(response, "Anthropic response").await?;

        let text = api_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        // Parse the JSON response from Claude
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

        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: format!("{}\n\n{}", system_prompt, user_message),
        }];

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages,
        };

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response = handle_api_response(response, "Anthropic").await?;
        let api_response: ClaudeApiResponse =
            parse_json_response(response, "Anthropic response").await?;

        let text = api_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let llm_response: LlmResponse =
            serde_json::from_str(&text).context("Failed to parse LLM response as JSON")?;

        Ok(llm_response)
    }
}
