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
