//! Buildlog-consultant provides tools for analyzing build logs to identify problems.
//!
//! This crate contains functionality for parsing and analyzing build logs from various
//! build systems, primarily focusing on Debian package building tools.

#![deny(missing_docs)]

use std::borrow::Cow;
use std::ops::Index;

/// Module for handling apt-related logs and problems.
pub mod apt;
/// Module for processing autopkgtest logs.
pub mod autopkgtest;
/// Module for Bazaar (brz) version control system logs.
pub mod brz;
/// Module for Common Upgradeability Description Format (CUDF) logs.
pub mod cudf;
/// Module for line-level processing.
pub mod lines;
/// Module containing problem definitions for various build systems.
pub mod problems;

#[cfg(feature = "chatgpt")]
/// Module for interacting with ChatGPT for log analysis.
pub mod chatgpt;

/// Common utilities and helpers for build log analysis.
pub mod common;

/// Match-related functionality for finding patterns in logs.
pub mod r#match;

/// Module for handling sbuild logs and related problems.
pub mod sbuild;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singlelinematch_line() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.line(), "test line");
    }

    #[test]
    fn test_singlelinematch_origin() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        let origin = m.origin();
        assert_eq!(origin.as_str(), "test");
    }

    #[test]
    fn test_singlelinematch_offset() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.offset(), 10);
    }

    #[test]
    fn test_singlelinematch_lineno() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.lineno(), 11);
    }

    #[test]
    fn test_singlelinematch_linenos() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.linenos(), vec![11]);
    }

    #[test]
    fn test_singlelinematch_offsets() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.offsets(), vec![10]);
    }

    #[test]
    fn test_singlelinematch_lines() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        assert_eq!(m.lines(), vec!["test line"]);
    }

    #[test]
    fn test_singlelinematch_add_offset() {
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 10,
            line: "test line".to_string(),
        };
        let new_m = m.add_offset(5);
        assert_eq!(new_m.offset(), 15);
    }

    #[test]
    fn test_multilinelmatch_line() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(m.line(), "line 3");
    }

    #[test]
    fn test_multilinelmatch_origin() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        let origin = m.origin();
        assert_eq!(origin.as_str(), "test");
    }

    #[test]
    fn test_multilinelmatch_offset() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(m.offset(), 12);
    }

    #[test]
    fn test_multilinelmatch_lineno() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(m.lineno(), 13);
    }

    #[test]
    fn test_multilinelmatch_offsets() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(m.offsets(), vec![10, 11, 12]);
    }

    #[test]
    fn test_multilinelmatch_lines() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(m.lines(), vec!["line 1", "line 2", "line 3"]);
    }

    #[test]
    fn test_multilinelmatch_add_offset() {
        let m = MultiLineMatch {
            origin: Origin("test".to_string()),
            offsets: vec![10, 11, 12],
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        let new_m = m.add_offset(5);
        assert_eq!(new_m.offsets(), vec![15, 16, 17]);
    }

    #[test]
    fn test_highlight_lines() {
        let lines = vec!["line 1", "line 2", "line 3", "line 4", "line 5"];
        let m = SingleLineMatch {
            origin: Origin("test".to_string()),
            offset: 2,
            line: "line 3".to_string(),
        };
        // This test just ensures the function doesn't panic
        highlight_lines(&lines, &m, 1);
    }
}

/// Trait for representing a match of content in a log file.
///
/// This trait defines the interface for working with matched content in logs,
/// providing methods to access the content and its location information.
pub trait Match: Send + Sync + std::fmt::Debug + std::fmt::Display {
    /// Returns the matched line of text.
    fn line(&self) -> &str;

    /// Returns the origin information for this match.
    fn origin(&self) -> &Origin;

    /// Returns the 0-based offset of the match in the source.
    fn offset(&self) -> usize;

    /// Returns the 1-based line number of the match in the source.
    fn lineno(&self) -> usize {
        self.offset() + 1
    }

    /// Returns all 1-based line numbers for this match.
    fn linenos(&self) -> Vec<usize> {
        self.offsets().iter().map(|&x| x + 1).collect()
    }

    /// Returns all 0-based offsets for this match.
    fn offsets(&self) -> Vec<usize>;

    /// Returns all lines of text in this match.
    fn lines(&self) -> Vec<&str>;

    /// Creates a new match with all offsets shifted by the given amount.
    fn add_offset(&self, offset: usize) -> Box<dyn Match>;
}

/// Source identifier for a match.
///
/// This struct represents the source/origin of a match, typically a file name or other identifier.
#[derive(Clone, Debug)]
pub struct Origin(String);

impl Origin {
    /// Returns the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A match for a single line in a log file.
///
/// This struct implements the `Match` trait for single-line matches.
#[derive(Clone, Debug)]
pub struct SingleLineMatch {
    /// Source identifier for the match.
    pub origin: Origin,
    /// Zero-based line offset in the source.
    pub offset: usize,
    /// The matched line content.
    pub line: String,
}

impl Match for SingleLineMatch {
    fn line(&self) -> &str {
        &self.line
    }

    fn origin(&self) -> &Origin {
        &self.origin
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn offsets(&self) -> Vec<usize> {
        vec![self.offset]
    }

    fn lines(&self) -> Vec<&str> {
        vec![&self.line]
    }

    fn add_offset(&self, offset: usize) -> Box<dyn Match> {
        Box::new(Self {
            origin: self.origin.clone(),
            offset: self.offset + offset,
            line: self.line.clone(),
        })
    }
}

impl std::fmt::Display for SingleLineMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.origin.0, self.lineno(), self.line)
    }
}

impl SingleLineMatch {
    /// Creates a new `SingleLineMatch` from a collection of lines, an offset, and an optional origin.
    ///
    /// # Arguments
    /// * `lines` - Collection of lines that can be indexed
    /// * `offset` - Zero-based offset of the line to match
    /// * `origin` - Optional source identifier
    ///
    /// # Returns
    /// A new `SingleLineMatch` instance
    pub fn from_lines<'a>(
        lines: &impl Index<usize, Output = &'a str>,
        offset: usize,
        origin: Option<&str>,
    ) -> Self {
        let line = &lines[offset];
        let origin = origin
            .map(|s| Origin(s.to_string()))
            .unwrap_or_else(|| Origin("".to_string()));
        Self {
            origin,
            offset,
            line: line.to_string(),
        }
    }
}

/// A match for multiple consecutive lines in a log file.
///
/// This struct implements the `Match` trait for multi-line matches.
#[derive(Clone, Debug)]
pub struct MultiLineMatch {
    /// Source identifier for the match.
    pub origin: Origin,
    /// Zero-based line offsets for each matching line.
    pub offsets: Vec<usize>,
    /// The matched line contents.
    pub lines: Vec<String>,
}

impl MultiLineMatch {
    /// Creates a new `MultiLineMatch` with the specified origin, offsets, and lines.
    ///
    /// # Arguments
    /// * `origin` - The source identifier
    /// * `offsets` - Vector of zero-based line offsets
    /// * `lines` - Vector of matched line contents
    ///
    /// # Returns
    /// A new `MultiLineMatch` instance
    pub fn new(origin: Origin, offsets: Vec<usize>, lines: Vec<String>) -> Self {
        assert!(!offsets.is_empty());
        assert!(offsets.len() == lines.len());
        Self {
            origin,
            offsets,
            lines,
        }
    }

    /// Creates a new `MultiLineMatch` from a collection of lines, a vector of offsets, and an optional origin.
    ///
    /// # Arguments
    /// * `lines` - Collection of lines that can be indexed
    /// * `offsets` - Vector of zero-based line offsets to match
    /// * `origin` - Optional source identifier
    ///
    /// # Returns
    /// A new `MultiLineMatch` instance
    pub fn from_lines<'a>(
        lines: &impl Index<usize, Output = &'a str>,
        offsets: Vec<usize>,
        origin: Option<&str>,
    ) -> Self {
        let lines = offsets
            .iter()
            .map(|&offset| lines[offset].to_string())
            .collect();
        let origin = origin
            .map(|s| Origin(s.to_string()))
            .unwrap_or_else(|| Origin("".to_string()));
        Self::new(origin, offsets, lines)
    }
}

impl Match for MultiLineMatch {
    fn line(&self) -> &str {
        self.lines
            .last()
            .expect("MultiLineMatch should have at least one line")
    }

    fn origin(&self) -> &Origin {
        &self.origin
    }

    fn offset(&self) -> usize {
        *self
            .offsets
            .last()
            .expect("MultiLineMatch should have at least one offset")
    }

    fn lineno(&self) -> usize {
        self.offset() + 1
    }

    fn offsets(&self) -> Vec<usize> {
        self.offsets.clone()
    }

    fn lines(&self) -> Vec<&str> {
        self.lines.iter().map(|s| s.as_str()).collect()
    }

    fn add_offset(&self, extra: usize) -> Box<dyn Match> {
        let offsets = self.offsets.iter().map(|&offset| offset + extra).collect();
        Box::new(Self {
            origin: self.origin.clone(),
            offsets,
            lines: self.lines.clone(),
        })
    }
}

impl std::fmt::Display for MultiLineMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.origin.0, self.lineno(), self.line())
    }
}

/// Trait for representing a problem found in build logs.
///
/// This trait defines the interface for working with problems identified in build logs,
/// providing methods to access problem information and properties.
pub trait Problem: std::fmt::Display + Send + Sync + std::fmt::Debug {
    /// Returns the kind/type of problem.
    fn kind(&self) -> Cow<'_, str>;

    /// Returns the problem details as a JSON value.
    fn json(&self) -> serde_json::Value;

    /// Returns the problem as a trait object that can be downcast.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Is this problem universal, i.e. applicable to all build steps?
    ///
    /// Good examples of universal problems are e.g. disk full, out of memory, etc.
    fn is_universal(&self) -> bool {
        false
    }
}

impl PartialEq for dyn Problem {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.json() == other.json()
    }
}

impl Eq for dyn Problem {}

impl serde::Serialize for dyn Problem {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        map.insert(
            "kind".to_string(),
            serde_json::Value::String(self.kind().to_string()),
        );
        map.insert("details".to_string(), self.json());
        map.serialize(serializer)
    }
}

impl std::hash::Hash for dyn Problem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.json().hash(state);
    }
}

/// Prints highlighted lines from a match with surrounding context.
///
/// # Arguments
/// * `lines` - All lines from the source
/// * `m` - The match to highlight
/// * `context` - Number of lines of context to display before and after the match
pub fn highlight_lines(lines: &[&str], m: &dyn Match, context: usize) {
    use std::cmp::{max, min};
    if m.linenos().len() == 1 {
        println!("Issue found at line {}:", m.lineno());
    } else {
        println!(
            "Issue found at lines {}-{}:",
            m.linenos().first().unwrap(),
            m.linenos().last().unwrap()
        );
    }
    let start = max(0, m.offsets()[0].saturating_sub(context));
    let end = min(lines.len(), m.offsets().last().unwrap() + context + 1);

    for (i, line) in lines.iter().enumerate().take(end).skip(start) {
        println!(
            " {}  {}",
            if m.offsets().contains(&i) { ">" } else { " " },
            line.trim_end_matches('\n')
        );
    }
}
