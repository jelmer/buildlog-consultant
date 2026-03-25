//! Shared utilities for LLM-based build log analysis.

use crate::{Problem, ProblemKindInfo, SingleLineMatch};
use std::borrow::Cow;

/// Maximum number of characters to include in a prompt.
pub const MAX_PROMPT_CHARS: usize = 4096;

/// Base system prompt instructing the LLM how to analyze build logs.
const SYSTEM_PROMPT_PREFIX: &str = "\
You are a build log analyst. You will be given a numbered excerpt from a build log. \
Identify the single line that is the root cause or clearest explanation of the build failure.

Respond with a JSON object (and nothing else) with the following fields:
- \"line\": the line number (integer)
- \"kind\": the problem kind (string), one of the known kinds listed below, or a new descriptive kebab-case kind if none fit
- \"details\": an object with the detail fields for that kind

Known problem kinds:\n";

/// Build the full system prompt, including dynamically-registered problem kinds.
pub fn system_prompt() -> String {
    let mut prompt = SYSTEM_PROMPT_PREFIX.to_string();
    let mut kinds: Vec<&ProblemKindInfo> = inventory::iter::<ProblemKindInfo>().collect();
    kinds.sort_by_key(|k| k.kind);
    for info in &kinds {
        if info.detail_fields.is_empty() {
            prompt.push_str(&format!("- {}: {{}}\n", info.kind));
        } else {
            let fields: Vec<String> = info
                .detail_fields
                .iter()
                .map(|f| format!("\"{f}\": ..."))
                .collect();
            prompt.push_str(&format!("- {}: {{{}}}\n", info.kind, fields.join(", ")));
        }
    }
    prompt
}

/// A problem identified by an LLM, represented as a kind string and JSON details.
#[derive(Clone, Debug)]
pub struct LlmProblem {
    /// The problem kind identifier (e.g. "missing-file", "command-missing").
    pub kind: String,
    /// Structured details about the problem.
    pub details: serde_json::Value,
}

impl Problem for LlmProblem {
    fn kind(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.kind)
    }

    fn json(&self) -> serde_json::Value {
        self.details.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for LlmProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.details)
    }
}

/// Result of LLM-based log analysis.
pub struct AnalysisResult {
    /// The matched line in the log.
    pub r#match: SingleLineMatch,
    /// The identified problem, if any.
    pub problem: Option<Box<dyn Problem>>,
}

/// Format log lines into a numbered prompt for LLM analysis.
///
/// Lines are numbered starting from their actual position in the full log.
pub fn format_prompt(lines: &[&str], offset: usize) -> String {
    let mut prompt = String::new();
    for (i, line) in lines.iter().enumerate() {
        let numbered = format!("{}: {}\n", offset + i + 1, line);
        prompt.push_str(&numbered);
    }
    prompt
}

/// Select a tail portion of log lines that fits within [`MAX_PROMPT_CHARS`].
///
/// Returns `(offset, selected_lines)` where `offset` is the index of the
/// first selected line in the original `lines` slice.
pub fn truncate_lines<'a>(lines: &[&'a str]) -> (usize, Vec<&'a str>) {
    let mut total = 0;
    let mut count = 0;
    for line in lines.iter().rev() {
        let numbered_len = format!("{}: {}\n", lines.len() - count, line).len();
        if total + numbered_len > MAX_PROMPT_CHARS {
            break;
        }
        total += numbered_len;
        count += 1;
    }
    let offset = lines.len() - count;
    let selected: Vec<&str> = lines[offset..].to_vec();
    (offset, selected)
}

/// Parse the LLM response to extract a line number, problem kind, and details.
///
/// The response is expected to be a JSON object with `line`, `kind`, and `details` fields.
/// Falls back to extracting just a line number if JSON parsing fails.
pub fn parse_response(response: &str, lines: &Vec<&str>, origin: &str) -> Option<AnalysisResult> {
    // Strip markdown code fences if present
    let trimmed = response.trim();
    let trimmed = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(trimmed);

    // Try parsing as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(lineno) = json.get("line").and_then(|v| v.as_u64()) {
            let lineno = lineno as usize;
            if lineno == 0 || lineno > lines.len() {
                log::debug!(
                    "LLM returned line number {} which is out of range (1-{})",
                    lineno,
                    lines.len()
                );
                return None;
            }

            let offset = lineno - 1;
            let m = SingleLineMatch::from_lines(lines, offset, Some(origin));

            let problem =
                json.get("kind")
                    .and_then(|v| v.as_str())
                    .map(|kind| -> Box<dyn Problem> {
                        Box::new(LlmProblem {
                            kind: kind.to_string(),
                            details: json
                                .get("details")
                                .cloned()
                                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                        })
                    });

            return Some(AnalysisResult {
                r#match: m,
                problem,
            });
        }
    }

    // Fallback: extract just a line number
    let lineno: usize = trimmed
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())?;

    if lineno == 0 || lineno > lines.len() {
        log::debug!(
            "LLM returned line number {} which is out of range (1-{})",
            lineno,
            lines.len()
        );
        return None;
    }

    let offset = lineno - 1;
    Some(AnalysisResult {
        r#match: SingleLineMatch::from_lines(lines, offset, Some(origin)),
        problem: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Match;

    #[test]
    fn test_format_prompt() {
        let lines = vec!["make: *** [all] Error 2", "dpkg-buildpackage: error"];
        let prompt = format_prompt(&lines, 10);
        assert_eq!(
            prompt,
            "11: make: *** [all] Error 2\n12: dpkg-buildpackage: error\n"
        );
    }

    #[test]
    fn test_format_prompt_zero_offset() {
        let lines = vec!["first line"];
        let prompt = format_prompt(&lines, 0);
        assert_eq!(prompt, "1: first line\n");
    }

    #[test]
    fn test_truncate_lines_short() {
        let lines = vec!["line 1", "line 2", "line 3"];
        let (offset, selected) = truncate_lines(&lines);
        assert_eq!(offset, 0);
        assert_eq!(selected, lines);
    }

    #[test]
    fn test_truncate_lines_long() {
        let long_line = "x".repeat(500);
        let lines: Vec<&str> = std::iter::repeat(long_line.as_str()).take(20).collect();
        let (offset, selected) = truncate_lines(&lines);
        assert!(offset > 0);
        assert!(selected.len() < lines.len());
    }

    #[test]
    fn test_parse_response_json() {
        let lines = vec!["ok", "error: missing gcc", "ok"];
        let response = r#"{"line": 2, "kind": "command-missing", "details": {"command": "gcc"}}"#;
        let result = parse_response(response, &lines, "test").unwrap();
        assert_eq!(result.r#match.offset(), 1);
        assert_eq!(result.r#match.line(), "error: missing gcc");
        let problem = result.problem.unwrap();
        assert_eq!(problem.kind().as_ref(), "command-missing");
        assert_eq!(problem.json(), serde_json::json!({"command": "gcc"}));
    }

    #[test]
    fn test_parse_response_json_no_details() {
        let lines = vec!["ok", "Segmentation fault", "ok"];
        let response = r#"{"line": 2, "kind": "segmentation-fault"}"#;
        let result = parse_response(response, &lines, "test").unwrap();
        assert_eq!(result.r#match.offset(), 1);
        let problem = result.problem.unwrap();
        assert_eq!(problem.kind().as_ref(), "segmentation-fault");
        assert_eq!(problem.json(), serde_json::json!({}));
    }

    #[test]
    fn test_parse_response_json_no_kind() {
        let lines = vec!["ok", "error here", "ok"];
        let response = r#"{"line": 2}"#;
        let result = parse_response(response, &lines, "test").unwrap();
        assert_eq!(result.r#match.offset(), 1);
        assert!(result.problem.is_none());
    }

    #[test]
    fn test_parse_response_json_in_code_fence() {
        let lines = vec!["ok", "error: missing gcc", "ok"];
        let response = "```json\n{\"line\": 2, \"kind\": \"command-missing\", \"details\": {\"command\": \"gcc\"}}\n```";
        let result = parse_response(response, &lines, "test").unwrap();
        assert_eq!(result.r#match.offset(), 1);
        let problem = result.problem.unwrap();
        assert_eq!(problem.kind().as_ref(), "command-missing");
    }

    #[test]
    fn test_parse_response_fallback_plain_number() {
        let lines = vec!["ok", "error here", "ok"];
        let result = parse_response("2", &lines, "test").unwrap();
        assert_eq!(result.r#match.offset(), 1);
        assert!(result.problem.is_none());
    }

    #[test]
    fn test_parse_response_out_of_range() {
        let lines = vec!["ok", "ok"];
        assert!(parse_response(r#"{"line": 99}"#, &lines, "test").is_none());
    }

    #[test]
    fn test_parse_response_zero() {
        let lines = vec!["ok"];
        assert!(parse_response(r#"{"line": 0}"#, &lines, "test").is_none());
    }

    #[test]
    fn test_parse_response_no_number() {
        let lines = vec!["ok"];
        assert!(parse_response("no number here", &lines, "test").is_none());
    }

    #[test]
    fn test_llm_problem_display() {
        let p = LlmProblem {
            kind: "command-missing".to_string(),
            details: serde_json::json!({"command": "gcc"}),
        };
        assert_eq!(format!("{}", p), r#"command-missing: {"command":"gcc"}"#);
    }

    #[test]
    fn test_system_prompt_contains_registered_kinds() {
        let prompt = system_prompt();
        assert!(prompt.contains("command-missing"));
        assert!(prompt.contains("missing-file"));
        assert!(prompt.contains("no-space-on-device"));
    }
}
