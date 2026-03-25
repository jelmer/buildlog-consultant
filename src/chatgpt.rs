//! Module for using ChatGPT to analyze build logs.
//!
//! This module provides functionality to use the ChatGPT API to identify
//! the most likely error line in a build log.

use crate::llm::{self, AnalysisResult};
use chatgpt::prelude::*;
use chatgpt::types::CompletionResponse;

/// Uses ChatGPT to analyze log lines and identify the most likely error.
///
/// # Arguments
/// * `chatgpt_key` - API key for ChatGPT
/// * `lines` - Vector of log lines to analyze
///
/// # Returns
/// A Result containing an optional [`AnalysisResult`] with the identified error line
/// and problem details, or an error if the ChatGPT API call fails.
pub async fn analyze(
    chatgpt_key: String,
    lines: Vec<&str>,
) -> std::result::Result<Option<AnalysisResult>, Box<dyn std::error::Error + Send + Sync>> {
    let client = ChatGPT::new(chatgpt_key)?;

    let (offset, selected) = llm::truncate_lines(&lines);
    let prompt = llm::format_prompt(&selected, offset);
    let full_prompt = format!("{}\n\n{}", llm::system_prompt(), prompt);

    let response: CompletionResponse = client.send_message(&full_prompt).await?;

    let text = &response.message().content;

    match llm::parse_response(text, &lines, "chatgpt") {
        Some(result) => Ok(Some(result)),
        None => {
            log::debug!("Unable to parse chatgpt response: {:?}", text);
            Ok(None)
        }
    }
}
