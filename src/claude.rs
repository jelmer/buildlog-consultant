//! Module for using Claude to analyze build logs.
//!
//! This module provides functionality to use the Anthropic Claude API to identify
//! the most likely error line in a build log.

use crate::llm::{self, AnalysisResult};
use anthropic::client::ClientBuilder;
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};

/// Uses Claude to analyze log lines and identify the most likely error.
///
/// # Arguments
/// * `api_key` - API key for Anthropic
/// * `lines` - Vector of log lines to analyze
///
/// # Returns
/// A Result containing an optional [`AnalysisResult`] with the identified error line
/// and problem details, or an error if the Claude API call fails.
pub async fn analyze(
    api_key: String,
    lines: Vec<&str>,
) -> std::result::Result<Option<AnalysisResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = ClientBuilder::default().api_key(api_key).build()?;

    let (offset, selected) = llm::truncate_lines(&lines);
    let prompt = llm::format_prompt(&selected, offset);

    let messages = vec![Message {
        role: Role::User,
        content: vec![ContentBlock::Text { text: prompt }],
    }];

    let request = MessagesRequestBuilder::default()
        .model("claude-3-5-sonnet-20241022".to_string())
        .system(llm::system_prompt())
        .messages(messages)
        .max_tokens(256_usize)
        .build()?;

    let response = client.messages(request).await?;

    let text = response
        .content
        .iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .unwrap_or("");

    match llm::parse_response(text, &lines, "claude") {
        Some(result) => Ok(Some(result)),
        None => {
            log::debug!("Unable to parse claude response: {:?}", text);
            Ok(None)
        }
    }
}
