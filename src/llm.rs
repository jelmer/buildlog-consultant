//! Shared utilities for LLM-based build log analysis.

use crate::SingleLineMatch;

/// Maximum number of characters to include in a prompt.
pub const MAX_PROMPT_CHARS: usize = 4096;

/// System prompt instructing the LLM how to analyze build logs.
pub const SYSTEM_PROMPT: &str = "\
You are a build log analyst. You will be given a numbered excerpt from a build log. \
Identify the single line that is the root cause or clearest explanation of the build failure. \
Respond with ONLY the line number (e.g. \"42\"). Do not include any other text.";

/// Format log lines into a numbered prompt for LLM analysis.
///
/// Lines are numbered starting from their actual position in the full log.
/// The returned prompt contains only the trailing portion of the log that
/// fits within `MAX_PROMPT_CHARS`.
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

/// Parse the LLM response to extract a line number and match it back to the log.
///
/// The response is expected to contain just a line number. We extract the first
/// number found and convert it to a 0-based offset.
pub fn parse_response(
    response: &str,
    lines: &Vec<&str>,
    origin: &str,
) -> Option<SingleLineMatch> {
    let lineno: usize = response
        .trim()
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
    Some(SingleLineMatch::from_lines(lines, offset, Some(origin)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Match;

    #[test]
    fn test_format_prompt() {
        let lines = vec!["make: *** [all] Error 2", "dpkg-buildpackage: error"];
        let prompt = format_prompt(&lines, 10);
        assert_eq!(prompt, "11: make: *** [all] Error 2\n12: dpkg-buildpackage: error\n");
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
        // Create lines that exceed MAX_PROMPT_CHARS
        let long_line = "x".repeat(500);
        let lines: Vec<&str> = std::iter::repeat(long_line.as_str()).take(20).collect();
        let (offset, selected) = truncate_lines(&lines);
        assert!(offset > 0);
        assert!(selected.len() < lines.len());
    }

    #[test]
    fn test_parse_response_simple() {
        let lines = vec!["ok", "error here", "ok"];
        let m = parse_response("2", &lines, "test").unwrap();
        assert_eq!(m.offset(), 1);
        assert_eq!(m.line(), "error here");
    }

    #[test]
    fn test_parse_response_with_surrounding_text() {
        let lines = vec!["ok", "error here", "ok"];
        let m = parse_response("Line 2", &lines, "test").unwrap();
        assert_eq!(m.offset(), 1);
    }

    #[test]
    fn test_parse_response_out_of_range() {
        let lines = vec!["ok", "ok"];
        assert!(parse_response("99", &lines, "test").is_none());
    }

    #[test]
    fn test_parse_response_zero() {
        let lines = vec!["ok"];
        assert!(parse_response("0", &lines, "test").is_none());
    }

    #[test]
    fn test_parse_response_no_number() {
        let lines = vec!["ok"];
        assert!(parse_response("no number here", &lines, "test").is_none());
    }
}
