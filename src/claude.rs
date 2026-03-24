//! Module for using Claude to analyze build logs.
//!
//! This module provides functionality to use the Anthropic Claude API to identify
//! the most likely error line in a build log.

use crate::SingleLineMatch;
use anthropic::client::ClientBuilder;
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};

/// Maximum number of tokens allowed in a Claude request.
pub const MAX_TOKENS: usize = 4096;

/// Initial prompt used to ask Claude to identify the error in a log file.
pub const INITIAL_PROMPT: &str =
    "Which line in the log file below is the clearest explanation of a problem:\n\n";

/// Uses Claude to analyze log lines and identify the most likely error.
///
/// This function sends a portion of the log file to the Claude API and asks
/// it to identify the line that best explains the problem. It then tries to
/// match the Claude response back to an actual line in the log.
///
/// # Arguments
/// * `api_key` - API key for Anthropic
/// * `lines` - Vector of log lines to analyze
///
/// # Returns
/// A Result containing an optional `SingleLineMatch` pointing to the identified error line,
/// or an error if the Claude API call fails.
pub async fn analyze(
    api_key: String,
    lines: Vec<&str>,
) -> std::result::Result<Option<SingleLineMatch>, Box<dyn std::error::Error + Send + Sync>> {
    let client = ClientBuilder::default()
        .api_key(api_key)
        .build()?;

    // Select lines from the end, but not more than MAX_TOKENS - INITIAL_PROMPT.len()
    // Also, only include full lines
    let mut truncated: Vec<&str> = lines
        .iter()
        .rev()
        .take_while(|line| line.len() < MAX_TOKENS - INITIAL_PROMPT.len())
        .copied()
        .collect();

    // Reverse the lines back
    truncated.reverse();

    let prompt = format!("{}{}", INITIAL_PROMPT, truncated.join("\n"));

    let messages = vec![Message {
        role: Role::User,
        content: vec![ContentBlock::Text { text: prompt }],
    }];

    let request = MessagesRequestBuilder::default()
        .model("claude-sonnet-4-20250514".to_string())
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

    for (i, line) in lines.iter().enumerate().rev() {
        if line.starts_with(text) {
            return Ok(Some(SingleLineMatch::from_lines(
                &lines,
                i,
                Some("claude"),
            )));
        }
    }

    log::debug!("Unable to find claude answer in lines: {:?}", text);

    Ok(None)
}
