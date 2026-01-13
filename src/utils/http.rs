//! HTTP utility functions for API clients

use anyhow::{Context, Result};
use reqwest::Response;

/// Handle API response errors with consistent formatting
///
/// Checks if the response was successful, and if not, extracts the error
/// message and returns a formatted error.
pub async fn handle_api_response(response: Response, api_name: &str) -> Result<Response> {
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("{} API error {}: {}", api_name, status, error_text);
    }
    Ok(response)
}

/// Parse JSON response with context
pub async fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T> {
    response
        .json()
        .await
        .context(format!("Failed to parse {}", context))
}
