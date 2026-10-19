//! Module providing pattern matching functionality for log analysis.
//!
//! This module contains tools for matching patterns in logs and extracting problems.
//! It includes regex-based matchers and a matcher group for combining multiple matchers.

use crate::SingleLineMatch;
use crate::{Failure, Origin, Problem};
use regex::{Captures, Regex};
use std::fmt::Display;

/// Result of trying to extract a failure from a log
pub type MatchResult = Result<Option<Failure>, Error>;

/// Type alias for a callback function that processes regex captures
pub type RegexCallback =
    Box<dyn Fn(&Captures) -> Result<Option<Box<dyn Problem>>, Error> + Send + Sync>;

/// Error raised by a matcher when log content can not be interpreted.
#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    offset: Option<usize>,
    origin: Option<Origin>,
}

impl Error {
    /// Creates a new error with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
            offset: None,
            origin: None,
        }
    }

    /// Creates a new error with the given message, caused by `source`.
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            source: Some(Box::new(source)),
            ..Self::new(message)
        }
    }

    /// Returns the error message, without context.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the offset of the line that was being matched, if known.
    ///
    /// The offset is relative to the lines passed to the matcher.
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    /// Returns the origin of the matcher that failed, if known.
    pub fn origin(&self) -> Option<&Origin> {
        self.origin.as_ref()
    }

    /// Records the offset, unless a more specific one is already known.
    pub(crate) fn or_offset(mut self, offset: usize) -> Self {
        self.offset.get_or_insert(offset);
        self
    }

    /// Records the origin, unless a more specific one is already known.
    pub(crate) fn or_origin(mut self, origin: impl FnOnce() -> Origin) -> Self {
        self.origin.get_or_insert_with(origin);
        self
    }

    /// Replaces the offset, e.g. when the error came from matching synthesized text.
    pub(crate) fn at_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.message)?;
        if let Some(offset) = self.offset {
            write!(f, " at line {}", offset + 1)?;
        }
        if let Some(origin) = &self.origin {
            write!(f, " ({})", origin)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::with_source("invalid integer", e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self::with_source("invalid number", e)
    }
}

impl From<pep508_rs::Pep508Error> for Error {
    fn from(e: pep508_rs::Pep508Error) -> Self {
        Self::with_source("invalid PEP 508 requirement", e)
    }
}

/// A matcher that uses regular expressions to find patterns in single lines.
///
/// This matcher applies a regex to individual lines and can extract problem information
/// through a callback function when a match is found.
pub struct RegexLineMatcher {
    /// The regular expression to match against each line.
    regex: Regex,
    /// Callback function that extracts problem information from regex captures.
    callback: RegexCallback,
}

/// Trait for pattern matchers that can extract matches and problems from logs.
///
/// Implementors of this trait can search through log lines to find patterns and
/// extract problem information.
pub trait Matcher: Sync {
    /// Extracts a match and optional problem from a specific line in a log.
    ///
    /// # Arguments
    /// * `lines` - The collection of log lines
    /// * `offset` - The line offset to analyze
    ///
    /// # Returns
    /// * `Ok(Some(failure))` - A match was found along with an optional problem
    /// * `Ok(None)` - No match was found
    /// * `Err(error)` - An error occurred during matching
    fn extract_from_lines(&self, lines: &[&str], offset: usize) -> MatchResult;
}

impl RegexLineMatcher {
    /// Creates a new `RegexLineMatcher` with the given regex and callback.
    ///
    /// # Arguments
    /// * `regex` - The regex pattern to match against lines
    /// * `callback` - Function that processes regex captures and returns an optional problem
    ///
    /// # Returns
    /// A new `RegexLineMatcher` instance
    pub fn new(regex: Regex, callback: RegexCallback) -> Self {
        Self { regex, callback }
    }

    /// Checks if a line matches the regex pattern.
    ///
    /// # Arguments
    /// * `line` - The line to check
    ///
    /// # Returns
    /// `true` if the line matches the pattern, `false` otherwise
    pub fn matches_line(&self, line: &str) -> bool {
        self.regex.is_match(line)
    }

    /// Attempts to extract problem information from a line.
    ///
    /// # Arguments
    /// * `line` - The line to analyze
    ///
    /// # Returns
    /// * `Ok(Some(Some(problem)))` - A match was found and a problem was extracted
    /// * `Ok(Some(None))` - A match was found but no problem was extracted
    /// * `Ok(None)` - No match was found
    /// * `Err(error)` - An error occurred during matching or problem extraction
    pub fn extract_from_line(&self, line: &str) -> Result<Option<Option<Box<dyn Problem>>>, Error> {
        self.regex
            .captures(line)
            .map(|c| (self.callback)(&c).map_err(|e| e.or_origin(|| self.origin())))
            .transpose()
    }

    /// Creates an origin identifier for matches from this matcher.
    ///
    /// # Returns
    /// An `Origin` identifying the regex pattern used for matching
    fn origin(&self) -> Origin {
        Origin(format!("direct regex ({})", self.regex.as_str()))
    }
}

impl Matcher for RegexLineMatcher {
    fn extract_from_lines(&self, lines: &[&str], offset: usize) -> MatchResult {
        let line = lines[offset];
        if let Some(problem) = self
            .extract_from_line(line)
            .map_err(|e| e.or_offset(offset))?
        {
            let m = SingleLineMatch {
                offset,
                line: line.to_string(),
                origin: self.origin(),
            };
            return Ok(Some(Failure::new(m, problem)));
        }
        Ok(None)
    }
}

/// Tries each matcher in turn against a single line.
///
/// Returns the outcome of the first matcher whose regex matches `line`.
pub(crate) fn extract_from_line(
    matchers: &[Box<RegexLineMatcher>],
    line: &str,
) -> Result<Option<Option<Box<dyn Problem>>>, Error> {
    for matcher in matchers {
        if let Some(problem) = matcher.extract_from_line(line)? {
            return Ok(Some(problem));
        }
    }
    Ok(None)
}

/// Macro for creating regex-based line matchers.
///
/// This macro simplifies the creation of `RegexLineMatcher` instances by automatically
/// handling regex compilation and callback boxing.
///
/// # Examples
///
/// ```
/// # use buildlog_consultant::regex_line_matcher;
/// # use buildlog_consultant::r#match::RegexLineMatcher;
/// // With callback
/// let matcher = regex_line_matcher!(r"error: (.*)", |captures| {
///     let message = captures.get(1).unwrap().as_str();
///     // Process the error message
///     Ok(None)
/// });
///
/// // Without callback (just matches the pattern)
/// let simple_matcher = regex_line_matcher!(r"warning");
/// ```
#[macro_export]
macro_rules! regex_line_matcher {
    ($regex:expr, $callback:expr) => {
        Box::new(RegexLineMatcher::new(
            regex::Regex::new($regex).unwrap(),
            Box::new($callback),
        ))
    };
    ($regex: expr) => {
        Box::new(RegexLineMatcher::new(
            regex::Regex::new($regex).unwrap(),
            Box::new(|_| Ok(None)),
        ))
    };
}

/// Macro for creating regex-based paragraph matchers.
///
/// This macro is similar to `regex_line_matcher`, but creates matchers that can match
/// across multiple lines by automatically enabling the "dot matches newline" regex flag (?s).
///
/// # Examples
///
/// ```
/// # use buildlog_consultant::regex_para_matcher;
/// # use buildlog_consultant::r#match::RegexLineMatcher;
/// // With callback
/// let matcher = regex_para_matcher!(r"BEGIN(.*?)END", |captures| {
///     let content = captures.get(1).unwrap().as_str();
///     // Process the content between BEGIN and END
///     Ok(None)
/// });
///
/// // Without callback
/// let simple_matcher = regex_para_matcher!(r"function\s*\{.*?\}");
/// ```
#[macro_export]
macro_rules! regex_para_matcher {
    ($regex:expr, $callback:expr) => {{
        Box::new(RegexLineMatcher::new(
            regex::Regex::new(concat!("(?s)", $regex)).unwrap(),
            Box::new($callback),
        ))
    }};
    ($regex: expr) => {{
        Box::new(RegexLineMatcher::new(
            regex::Regex::new(concat!("(?s)", $regex)).unwrap(),
            Box::new(|_| Ok(None)),
        ))
    }};
}

/// A group of matchers that can be used to match multiple patterns.
///
/// This struct allows combining multiple matchers and trying them in sequence
/// until a match is found.
pub struct MatcherGroup(Vec<Box<dyn Matcher>>);

impl MatcherGroup {
    /// Creates a new `MatcherGroup` with the given matchers.
    ///
    /// # Arguments
    /// * `matchers` - Vector of boxed matchers
    ///
    /// # Returns
    /// A new `MatcherGroup` instance
    pub fn new(matchers: Vec<Box<dyn Matcher>>) -> Self {
        Self(matchers)
    }
}

impl Default for MatcherGroup {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl From<Vec<Box<dyn Matcher>>> for MatcherGroup {
    fn from(matchers: Vec<Box<dyn Matcher>>) -> Self {
        Self::new(matchers)
    }
}

impl MatcherGroup {
    /// Tries each matcher in the group until one finds a match.
    ///
    /// This method attempts to extract a match and problem from a specific line
    /// by trying each matcher in the group in sequence until one succeeds.
    ///
    /// # Arguments
    /// * `lines` - The collection of log lines
    /// * `offset` - The line offset to analyze
    ///
    /// # Returns
    /// * `Ok(Some(failure))` - A match was found by one of the matchers
    /// * `Ok(None)` - No match was found by any matcher
    /// * `Err(error)` - An error occurred during matching
    pub fn extract_from_lines(&self, lines: &[&str], offset: usize) -> MatchResult {
        for matcher in self.0.iter() {
            if let Some(p) = matcher
                .extract_from_lines(lines, offset)
                .map_err(|e| e.or_offset(offset))?
            {
                return Ok(Some(p));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[derive(Debug)]
    struct TestProblem {
        description: String,
    }

    impl std::fmt::Display for TestProblem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.description)
        }
    }

    impl Problem for TestProblem {
        fn kind(&self) -> Cow<'_, str> {
            Cow::Borrowed("test")
        }

        fn json(&self) -> serde_json::Value {
            serde_json::json!({
                "description": self.description,
            })
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_error_display() {
        let error = Error::new("test error");
        assert_eq!(error.to_string(), "test error");
        assert_eq!(error.message(), "test error");
        assert_eq!(error.offset(), None);
        assert!(error.origin().is_none());
        assert!(std::error::Error::source(&error).is_none());
    }

    #[test]
    fn test_error_from_parse_int() {
        let error: Error = "x".parse::<u32>().unwrap_err().into();
        assert_eq!(error.to_string(), "invalid integer");
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "invalid digit found in string"
        );
    }

    fn failing_matcher() -> RegexLineMatcher {
        RegexLineMatcher::new(
            Regex::new(r"^fail$").unwrap(),
            Box::new(|_| Err(Error::new("broken"))),
        )
    }

    #[test]
    fn test_regex_line_matcher_error_context() {
        let lines = vec!["line 1", "fail"];
        let error = failing_matcher().extract_from_lines(&lines, 1).unwrap_err();
        assert_eq!(error.message(), "broken");
        assert_eq!(error.offset(), Some(1));
        assert_eq!(error.origin().unwrap().as_str(), "direct regex (^fail$)");
        assert_eq!(
            error.to_string(),
            "broken at line 2 (direct regex (^fail$))"
        );
    }

    #[test]
    fn test_matcher_group_error() {
        let group = MatcherGroup::new(vec![Box::new(failing_matcher())]);
        let lines = vec!["fail"];
        let error = group.extract_from_lines(&lines, 0).unwrap_err();
        assert_eq!(
            error.to_string(),
            "broken at line 1 (direct regex (^fail$))"
        );
    }

    #[test]
    fn test_matcher_group_error_adds_offset() {
        struct Failing;
        impl Matcher for Failing {
            fn extract_from_lines(&self, _lines: &[&str], _offset: usize) -> MatchResult {
                Err(Error::new("broken"))
            }
        }
        let group = MatcherGroup::new(vec![Box::new(Failing)]);
        let error = group.extract_from_lines(&["a", "b"], 1).unwrap_err();
        assert_eq!(error.to_string(), "broken at line 2");
    }

    #[test]
    fn test_regex_line_matcher_new() {
        let regex = Regex::new(r"test").unwrap();
        let callback = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let matcher = RegexLineMatcher::new(regex, callback);
        assert!(matcher.matches_line("test line"));
        assert!(!matcher.matches_line("other line"));
    }

    #[test]
    fn test_regex_line_matcher_matches_line() {
        let regex = Regex::new(r"test").unwrap();
        let callback =
            Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> { Ok(None) });
        let matcher = RegexLineMatcher::new(regex, callback);
        assert!(matcher.matches_line("test line"));
        assert!(!matcher.matches_line("other line"));
    }

    #[test]
    fn test_regex_line_matcher_extract_from_line() {
        let regex = Regex::new(r"test").unwrap();
        let callback = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let matcher = RegexLineMatcher::new(regex, callback);
        let result = matcher.extract_from_line("test line").unwrap();
        assert!(result.is_some());
        let problem = result.unwrap();
        assert!(problem.is_some());
        let problem = problem.unwrap();
        assert_eq!(problem.kind(), "test");
    }

    #[test]
    fn test_regex_line_matcher_extract_from_line_no_match() {
        let regex = Regex::new(r"test").unwrap();
        let callback = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let matcher = RegexLineMatcher::new(regex, callback);
        let result = matcher.extract_from_line("other line").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_regex_line_matcher_extract_from_line_no_problem() {
        let regex = Regex::new(r"test").unwrap();
        let callback =
            Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> { Ok(None) });
        let matcher = RegexLineMatcher::new(regex, callback);
        let result = matcher.extract_from_line("test line").unwrap();
        assert!(result.is_some());
        let problem = result.unwrap();
        assert!(problem.is_none());
    }

    #[test]
    fn test_regex_line_matcher_extract_from_lines() {
        let regex = Regex::new(r"test").unwrap();
        let callback = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let matcher = RegexLineMatcher::new(regex, callback);
        let lines = vec!["line 1", "test line", "line 3"];
        let result = matcher.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_some());
        let Failure {
            r#match: m,
            problem,
        } = result.unwrap();
        assert_eq!(m.line(), "test line");
        assert_eq!(m.offset(), 1);
        assert!(problem.is_some());
        let problem = problem.unwrap();
        assert_eq!(problem.kind(), "test");
    }

    #[test]
    fn test_regex_line_matcher_extract_from_lines_no_match() {
        let regex = Regex::new(r"test").unwrap();
        let callback = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let matcher = RegexLineMatcher::new(regex, callback);
        let lines = vec!["line 1", "line 2", "line 3"];
        let result = matcher.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_matcher_group() {
        let regex1 = Regex::new(r"test1").unwrap();
        let callback1 = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem 1".to_string(),
            })))
        });
        let matcher1 = RegexLineMatcher::new(regex1, callback1);

        let regex2 = Regex::new(r"test2").unwrap();
        let callback2 = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem 2".to_string(),
            })))
        });
        let matcher2 = RegexLineMatcher::new(regex2, callback2);

        let group = MatcherGroup::new(vec![Box::new(matcher1), Box::new(matcher2)]);
        let lines = vec!["line 1", "test2 line", "line 3"];
        let result = group.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_some());
        let Failure {
            r#match: m,
            problem,
        } = result.unwrap();
        assert_eq!(m.line(), "test2 line");
        assert_eq!(m.offset(), 1);
        assert!(problem.is_some());
        let problem = problem.unwrap();
        assert_eq!(problem.kind(), "test");
    }

    #[test]
    fn test_matcher_group_no_match() {
        let regex1 = Regex::new(r"test1").unwrap();
        let callback1 = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem 1".to_string(),
            })))
        });
        let matcher1 = RegexLineMatcher::new(regex1, callback1);

        let regex2 = Regex::new(r"test2").unwrap();
        let callback2 = Box::new(|_: &Captures| -> Result<Option<Box<dyn Problem>>, Error> {
            Ok(Some(Box::new(TestProblem {
                description: "test problem 2".to_string(),
            })))
        });
        let matcher2 = RegexLineMatcher::new(regex2, callback2);

        let group = MatcherGroup::new(vec![Box::new(matcher1), Box::new(matcher2)]);
        let lines = vec!["line 1", "line 2", "line 3"];
        let result = group.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_regex_line_matcher_macro() {
        let matcher = regex_line_matcher!(r"test", |_| {
            Ok(Some(Box::new(TestProblem {
                description: "test problem".to_string(),
            })))
        });
        let lines = vec!["line 1", "test line", "line 3"];
        let result = matcher.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_regex_line_matcher_macro_simple() {
        let matcher = regex_line_matcher!(r"test");
        let lines = vec!["line 1", "test line", "line 3"];
        let result = matcher.extract_from_lines(&lines, 1).unwrap();
        assert!(result.is_some());
        let problem = result.unwrap().problem;
        assert!(problem.is_none());
    }
}
