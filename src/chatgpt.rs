//! Module for using ChatGPT to analyze build logs.
//!
//! This module provides functionality to use the ChatGPT API to identify
//! the most likely error line in a build log.

use crate::SingleLineMatch;
use chatgpt::prelude::*;
use chatgpt::types::CompletionResponse;

/// Maximum number of tokens allowed in a ChatGPT request.
pub const MAX_TOKENS: usize = 4096;

/// Initial prompt used to ask ChatGPT to identify the error in a log file.
pub const INITIAL_PROMPT: &str =
    "Which line in the log file below is the clearest explanation of a problem:\n\n";

/// Uses ChatGPT to analyze log lines and identify the most likely error.
///
/// This function sends a portion of the log file to the ChatGPT API and asks
/// it to identify the line that best explains the problem. It then tries to
/// match the ChatGPT response back to an actual line in the log.
///
/// # Arguments
/// * `chatgpt_key` - API key for ChatGPT
/// * `lines` - Vector of log lines to analyze
///
/// # Returns
/// A Result containing an optional `SingleLineMatch` pointing to the identified error line,
/// or an error if the ChatGPT API call fails
pub async fn analyze(
    chatgpt_key: String,
    lines: Vec<&str>,
) -> std::result::Result<Option<SingleLineMatch>, Box<dyn std::error::Error + Send + Sync>> {
    let client = ChatGPT::new(chatgpt_key)?;

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

    // Sending a message and getting the completion
    let response: CompletionResponse = client.send_message(&prompt).await?;

    let text = &response.message().content;

    for (i, line) in lines.iter().enumerate().rev() {
        if line.starts_with(text) {
            return Ok(Some(SingleLineMatch::from_lines(
                &lines,
                i,
                Some("chatgpt"),
            )));
        }
    }

    log::debug!("Unable to find chatgpt answer in lines: {:?}", text);

    Ok(None)
}
