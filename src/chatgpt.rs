use chatgpt::prelude::*;
use crate::SingleLineMatch;
use chatgpt::types::CompletionResponse;

pub const MAX_TOKENS: usize = 4096;

pub const INITIAL_PROMPT: &str = "Which line in the log file below is the clearest explanation of a problem:\n\n";

pub async fn analyze(chatgpt_key: String, lines: Vec<&str>) -> Option<SingleLineMatch> {
    let client = ChatGPT::new(chatgpt_key).unwrap();

    // Select lines from the end, but not more than MAX_TOKENS - INITIAL_PROMPT.len()
    // Also, only include full lines
    let mut truncated = lines.iter().rev().take_while(|line| line.len() < MAX_TOKENS - INITIAL_PROMPT.len()).collect::<Vec<_>>();

    // Reverse the lines back
    truncated.reverse();

    let prompt = format!("{}{}", INITIAL_PROMPT, truncated.into_iter().map(|line| *line).collect::<Vec<_>>().join("\n"));

    // Sending a message and getting the completion
    let response: CompletionResponse = client
        .send_message(&prompt)
        .await.unwrap();

    let text = &response.message().content;

    for (i, line) in lines.iter().enumerate().rev() {
        if line.starts_with(text) {
            return Some(SingleLineMatch::from_lines(&lines, i, Some("chatgpt")));
        }
    }

    log::debug!("Unable to find chatgpt answer in lines: {:?}", text);

    None
}
