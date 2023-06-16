use crate::{Match, Problem};
use crate::{Origin, SingleLineMatch};
use pyo3::prelude::*;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MissingFile {
    path: std::path::PathBuf,
}

impl Problem for MissingFile {
    fn kind(&self) -> Cow<str> {
        "missing-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path.to_string_lossy(),
        })
    }
}

impl Display for MissingFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing file: {}", self.path.display())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MissingBuildFile {
    filename: String,
}

impl Problem for MissingBuildFile {
    fn kind(&self) -> Cow<str> {
        "missing-build-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
        })
    }
}

impl Display for MissingBuildFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing build file: {}", self.filename)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MissingCommandOrBuildFile {
    filename: String,
}

impl Problem for MissingCommandOrBuildFile {
    fn kind(&self) -> Cow<str> {
        "missing-command-or-build-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
        })
    }
}

impl Display for MissingCommandOrBuildFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command or build file: {}", self.filename)
    }
}

impl MissingCommandOrBuildFile {
    pub fn command(&self) -> String {
        self.filename.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VcsControlDirectoryNeeded {
    vcs: Vec<String>,
}

impl Problem for VcsControlDirectoryNeeded {
    fn kind(&self) -> Cow<str> {
        "vcs-control-directory-needed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "vcs": self.vcs,
        })
    }
}

struct MissingPythonDistribution {
    distribution: String,
    python_version: Option<i32>,
    minimum_version: Option<String>,
}

impl Display for MissingPythonDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(python_version) = self.python_version {
            write!(
                f,
                "Missing {} Python distribution: {}",
                python_version, self.distribution
            )?;
        } else {
            write!(f, "Missing Python distribution: {}", self.distribution)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl Problem for MissingPythonDistribution {
    fn kind(&self) -> Cow<str> {
        "missing-python-distribution".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "distribution": self.distribution,
            "python_version": self.python_version,
            "minimum_version": self.minimum_version,
        })
    }
}

impl MissingPythonDistribution {
    pub fn from_requirement_str(
        text: &str,
        python_version: Option<i32>,
    ) -> PyResult<MissingPythonDistribution> {
        Python::with_gil(|py| {
            let requirement = py
                .import("requirements.requirement")?
                .getattr("Requirement")?
                .call_method1("parse", (text,))?;
            let distribution = requirement.getattr("name")?.extract::<String>()?;
            let specs = requirement
                .getattr("specs")?
                .extract::<Vec<(String, String)>>()?;

            Ok(if specs.len() == 1 && specs[0].0 == ">=" {
                MissingPythonDistribution {
                    distribution,
                    python_version,
                    minimum_version: Some(specs[0].1.clone()),
                }
            } else {
                MissingPythonDistribution {
                    distribution,
                    python_version,
                    minimum_version: None,
                }
            })
        })
    }
}

impl Display for VcsControlDirectoryNeeded {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VCS control directory needed: {}", self.vcs.join(", "))
    }
}

pub struct RegexLineMatcher {
    regex: Regex,
    callback: Box<dyn Fn(&Captures) -> Result<Option<Box<dyn Problem>>, Error> + Send + Sync>,
}

impl RegexLineMatcher {
    pub fn origin(&self) -> Origin {
        Origin(format!("direct regex ({})", self.regex.as_str()))
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

    pub fn extract_from_lines(
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

macro_rules! regex_line_matcher {
    ($regex:expr, $callback:expr) => {
        RegexLineMatcher {
            regex: Regex::new($regex).unwrap(),
            callback: Box::new($callback),
        }
    };
    ($regex: expr) => {
        RegexLineMatcher {
            regex: Regex::new($regex).unwrap(),
            callback: Box::new(|_| Ok(None)),
        }
    };
}

fn file_not_found(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let path = c.get(1).unwrap().as_str();
    if path.starts_with('/') && !path.starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(path),
        })));
    }
    if let Some(filename) = path.strip_prefix("/<<PKGBUILDDIR>>/") {
        return Ok(Some(Box::new(MissingBuildFile {
            filename: filename.to_string(),
        })));
    }
    if path == ".git/HEAD" {
        return Ok(Some(Box::new(VcsControlDirectoryNeeded {
            vcs: vec!["git".to_string()],
        })));
    }
    if path == "CVS/Root" {
        return Ok(Some(Box::new(VcsControlDirectoryNeeded {
            vcs: vec!["cvs".to_string()],
        })));
    }
    if !path.contains('/') {
        // Maybe a missing command?
        return Ok(Some(Box::new(MissingBuildFile {
            filename: path.to_string(),
        })));
    }
    Ok(None)
}

fn file_not_found_maybe_executable(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let p = c.get(1).unwrap().as_str();
    if p.starts_with('/') && !p.starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(p),
        })));
    }

    if !p.contains('/') {
        // Maybe a missing command?
        return Ok(Some(Box::new(MissingCommandOrBuildFile {
            filename: p.to_string(),
        })));
    }
    Ok(None)
}

lazy_static::lazy_static! {
    static ref LINE_MATCHERS: Vec<RegexLineMatcher> = vec![
        regex_line_matcher!(
            r"^make\[[0-9]+\]: \*\*\* No rule to make target '(.*)', needed by '.*'\.  Stop\.$",
            file_not_found
        ),
        regex_line_matcher!(r"^[^:]+:\d+: (.*): No such file or directory$", file_not_found_maybe_executable),
        regex_line_matcher!(
        r"^(distutils.errors.DistutilsError|error): Could not find suitable distribution for Requirement.parse\('([^']+)'\)$",
        |c| {
            let req = c.get(2).unwrap().as_str().split(';').next().unwrap();
            Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(req, None).unwrap())))
        }),
    ];
}

pub fn match_line(line: &str) -> Result<Option<(Option<Box<dyn Problem>>, Origin)>, Error> {
    for matcher in LINE_MATCHERS.iter() {
        if let Some(p) = matcher.extract_from_line(line)? {
            return Ok(Some((p, matcher.origin())));
        }
    }
    Ok(None)
}

pub fn match_lines(
    lines: &[&str],
    offset: usize,
) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
    for matcher in LINE_MATCHERS.iter() {
        if let Some(p) = matcher.extract_from_lines(lines, offset)? {
            return Ok(Some(p));
        }
    }
    Ok(None)
}
