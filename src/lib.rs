use std::borrow::Cow;
use std::ops::Index;

pub mod apt;
pub mod autopkgtest;
pub mod brz;
pub mod cudf;
pub mod lines;
pub mod problems;

#[cfg(feature = "chatgpt")]
pub mod chatgpt;

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
        assert_eq!(origin.0, "test");
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
        assert_eq!(origin.0, "test");
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

pub trait Match: Send + Sync + std::fmt::Debug + std::fmt::Display {
    fn line(&self) -> String;

    fn origin(&self) -> Origin;

    fn offset(&self) -> usize;

    fn lineno(&self) -> usize {
        self.offset() + 1
    }

    fn linenos(&self) -> Vec<usize> {
        self.offsets().iter().map(|&x| x + 1).collect()
    }

    fn offsets(&self) -> Vec<usize>;

    fn lines(&self) -> Vec<String>;

    fn add_offset(&self, offset: usize) -> Box<dyn Match>;
}

#[derive(Clone, Debug)]
pub struct Origin(String);

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug)]
pub struct SingleLineMatch {
    pub origin: Origin,
    pub offset: usize,
    pub line: String,
}

impl Match for SingleLineMatch {
    fn line(&self) -> String {
        self.line.clone()
    }

    fn origin(&self) -> Origin {
        self.origin.clone()
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn offsets(&self) -> Vec<usize> {
        vec![self.offset]
    }

    fn lines(&self) -> Vec<String> {
        vec![self.line.clone()]
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

#[derive(Clone, Debug)]
pub struct MultiLineMatch {
    pub origin: Origin,
    pub offsets: Vec<usize>,
    pub lines: Vec<String>,
}

impl MultiLineMatch {
    pub fn new(origin: Origin, offsets: Vec<usize>, lines: Vec<String>) -> Self {
        assert!(!offsets.is_empty());
        assert!(offsets.len() == lines.len());
        Self {
            origin,
            offsets,
            lines,
        }
    }

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
    fn line(&self) -> String {
        self.lines.last().unwrap().clone()
    }

    fn origin(&self) -> Origin {
        self.origin.clone()
    }

    fn offset(&self) -> usize {
        *self.offsets.last().unwrap()
    }

    fn lineno(&self) -> usize {
        self.offset() + 1
    }

    fn offsets(&self) -> Vec<usize> {
        self.offsets.clone()
    }

    fn lines(&self) -> Vec<String> {
        self.lines.clone()
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

pub trait Problem: std::fmt::Display + Send + Sync + std::fmt::Debug {
    fn kind(&self) -> Cow<str>;

    fn json(&self) -> serde_json::Value;

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

pub mod common;

pub mod r#match;

pub mod sbuild;

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
    for i in max(0, m.offsets()[0] - context)
        ..min(lines.len(), m.offsets().last().unwrap() + context + 1)
    {
        println!(
            " {}  {}",
            if m.offsets().contains(&i) { ">" } else { " " },
            lines[i].trim_end_matches('\n')
        );
    }
}
