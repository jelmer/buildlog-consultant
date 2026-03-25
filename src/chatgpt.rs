//! Module for using ChatGPT to analyze build logs.
//!
//! This module provides functionality to use the ChatGPT API to identify
//! the most likely error line in a build log.

use crate::llm::{self, AnalysisResult};
use chatgpt::client::ChatGPT;
use chatgpt::config::{ChatGPTEngine, ModelConfiguration};
use chatgpt::types::CompletionResponse;

/// Fetches the list of available models and picks the cheapest suitable one.
///
/// Prefers the cheapest chat model: nano > mini > base, from the latest generation.
async fn pick_model(
    api_key: &str,
) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?;

    let body: serde_json::Value = resp.json().await?;
    let mut models: Vec<&str> = body["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str())
                .filter(|id| {
                    id.starts_with("gpt-")
                        && !id.contains("instruct")
                        && !id.contains("realtime")
                        && !id.contains("audio")
                        && !id.contains("image")
                        && !id.contains("search")
                        && !id.contains("transcribe")
                        && !id.contains("tts")
                })
                .collect()
        })
        .unwrap_or_default();

    models.sort();

    for tier in &["nano", "mini"] {
        if let Some(model) = models.iter().rev().find(|id| id.contains(tier)) {
            log::debug!("Selected OpenAI model: {}", model);
            return Ok(model.to_string());
        }
    }

    if let Some(model) = models
        .iter()
        .rev()
        .find(|id| !id.contains("mini") && !id.contains("nano") && !id.contains("preview"))
    {
        log::debug!("Selected OpenAI model: {}", model);
        return Ok(model.to_string());
    }

    models
        .last()
        .map(|m| m.to_string())
        .ok_or_else(|| "No OpenAI chat models available".into())
}

/// Uses ChatGPT to analyze log lines and identify the most likely error.
///
/// Automatically selects the cheapest available model via the models API.
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
    let model_name = pick_model(&chatgpt_key).await?;
    let model_str: &'static str = Box::leak(model_name.into_boxed_str());
    let config = ModelConfiguration {
        engine: ChatGPTEngine::Custom(model_str),
        ..Default::default()
    };
    let client = ChatGPT::new_with_config(chatgpt_key, config)?;

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
