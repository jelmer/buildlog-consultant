use crate::SingleLineMatch;
use crate::{Match, Origin, Problem};
use regex::{Captures, Regex};
use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for Error {}

pub struct RegexLineMatcher {
    regex: Regex,
    callback: Box<dyn Fn(&Captures) -> Result<Option<Box<dyn Problem>>, Error> + Send + Sync>,
}

pub trait Matcher: Sync {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error>;
}

impl RegexLineMatcher {
    pub fn new(
        regex: Regex,
        callback: Box<dyn Fn(&Captures) -> Result<Option<Box<dyn Problem>>, Error> + Send + Sync>,
    ) -> Self {
        Self { regex, callback }
    }

    pub fn matches_line(&self, line: &str) -> bool {
        self.regex.is_match(line)
    }

    pub fn extract_from_line(&self, line: &str) -> Result<Option<Option<Box<dyn Problem>>>, Error> {
        let c = self.regex.captures(line);
        if let Some(c) = c {
            return Ok(Some((self.callback)(&c)?));
        }
        Ok(None)
    }

    fn origin(&self) -> Origin {
        Origin(format!("direct regex ({})", self.regex.as_str()))
    }
}

impl Matcher for RegexLineMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let line = lines[offset];
        if let Some(problem) = self.extract_from_line(line)? {
            let m = SingleLineMatch {
                offset,
                line: line.to_string(),
                origin: self.origin(),
            };
            return Ok(Some((Box::new(m), problem)));
        }
        Ok(None)
    }
}

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

pub struct MatcherGroup(Vec<Box<dyn Matcher>>);

impl MatcherGroup {
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
    pub fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        for matcher in self.0.iter() {
            if let Some(p) = matcher.extract_from_lines(lines, offset)? {
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
        fn kind(&self) -> Cow<str> {
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
        let error = Error {
            message: "test error".to_string(),
        };
        assert_eq!(error.to_string(), "test error");
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
        let (m, problem) = result.unwrap();
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
        let (m, problem) = result.unwrap();
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
        let (_m, problem) = result.unwrap();
        assert!(problem.is_none());
    }
}
