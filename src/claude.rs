//! Module for using Claude to analyze build logs.
//!
//! This module provides functionality to use the Anthropic Claude API to identify
//! the most likely error line in a build log.

use crate::llm::{self, AnalysisResult};
use anthropic::client::{Client, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};

/// Fetches the list of available models and picks the cheapest suitable one.
///
/// Prefers the cheapest model in this order: haiku, sonnet, opus.
async fn pick_model(client: &Client) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let resp = client
        .http_client
        .get(format!("{}/v1/models", client.api_base))
        .headers(client.headers())
        .send()
        .await?;

    let body: serde_json::Value = resp.json().await?;
    let models: Vec<&str> = body["data"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|m| m["id"].as_str()).collect())
        .unwrap_or_default();

    for tier in &["haiku", "sonnet", "opus"] {
        if let Some(model) = models.iter().find(|id| id.contains(tier)) {
            log::debug!("Selected Claude model: {}", model);
            return Ok(model.to_string());
        }
    }

    models
        .first()
        .map(|m| m.to_string())
        .ok_or_else(|| "No Claude models available".into())
}

/// Returns the text of the first text block in a response.
fn response_text(
    content: &[ContentBlock],
) -> Result<&str, Box<dyn std::error::Error + Send + Sync>> {
    content
        .iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .ok_or_else(|| "Claude response contained no text block".into())
}

/// Uses Claude to analyze log lines and identify the most likely error.
///
/// Automatically selects the cheapest available model via the models API.
///
/// # Arguments
/// * `api_key` - API key for Anthropic
/// * `lines` - Log lines to analyze
///
/// # Returns
/// A Result containing an optional [`AnalysisResult`] with the identified error line
/// and problem details, or an error if the Claude API call fails.
pub async fn analyze(
    api_key: String,
    lines: &[&str],
) -> std::result::Result<Option<AnalysisResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = ClientBuilder::default().api_key(api_key).build()?;
    let model = pick_model(&client).await?;

    let (offset, selected) = llm::truncate_lines(lines);
    let prompt = llm::format_prompt(selected, offset);

    let messages = vec![Message {
        role: Role::User,
        content: vec![ContentBlock::Text { text: prompt }],
    }];

    let request = MessagesRequestBuilder::default()
        .model(model)
        .system(llm::system_prompt())
        .messages(messages)
        .max_tokens(256_usize)
        .build()?;

    let response = client.messages(request).await?;

    let text = response_text(&response.content)?;

    log::debug!("Claude raw response: {:?}", text);

    match llm::parse_response(text, lines, "claude") {
        Some(result) => Ok(Some(result)),
        None => {
            log::debug!("Unable to parse claude response: {:?}", text);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_text() {
        let content = vec![ContentBlock::Text {
            text: "42".to_string(),
        }];
        assert_eq!(response_text(&content).unwrap(), "42");
    }

    #[test]
    fn test_response_text_missing() {
        let content = vec![ContentBlock::Image {
            source: "base64".to_string(),
            media_type: "image/png".to_string(),
            data: String::new(),
        }];
        assert_eq!(
            response_text(&content).unwrap_err().to_string(),
            "Claude response contained no text block"
        );
    }
}
