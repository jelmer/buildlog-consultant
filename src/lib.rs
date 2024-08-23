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
