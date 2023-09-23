use crate::r#match::{Error, Matcher, MatcherGroup, RegexLineMatcher};
use crate::regex_line_matcher;
use crate::{Match, Problem};
use crate::{MultiLineMatch, Origin, SingleLineMatch};
use pyo3::prelude::*;
use regex::Captures;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::max;
use std::fmt::Display;

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

struct MissingPythonModule {
    module: String,
    python_version: Option<i32>,
    minimum_version: Option<String>,
}

impl Display for MissingPythonModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(python_version) = self.python_version {
            write!(
                f,
                "Missing {} Python module: {}",
                python_version, self.module
            )?;
        } else {
            write!(f, "Missing Python module: {}", self.module)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl MissingPythonModule {
    fn simple(module: String) -> MissingPythonModule {
        MissingPythonModule {
            module,
            python_version: None,
            minimum_version: None,
        }
    }
}

impl Problem for MissingPythonModule {
    fn kind(&self) -> Cow<str> {
        "missing-python-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.module,
            "python_version": self.python_version,
            "minimum_version": self.minimum_version,
        })
    }
}

struct MissingCommand(String);

impl Display for MissingCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command: {}", self.0)
    }
}

impl Problem for MissingCommand {
    fn kind(&self) -> Cow<str> {
        "command-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.0,
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

struct MissingLibrary(String);

impl Display for MissingLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing library: {}", self.0)
    }
}

impl Problem for MissingLibrary {
    fn kind(&self) -> Cow<str> {
        "missing-library".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "library": self.0,
        })
    }
}

struct MissingIntrospectionTypelib(String);

impl Display for MissingIntrospectionTypelib {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing introspection typelib: {}", self.0)
    }
}

impl Problem for MissingIntrospectionTypelib {
    fn kind(&self) -> Cow<str> {
        "missing-introspection-typelib".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "library": self.0,
        })
    }
}

struct MissingPytestFixture(String);

impl Display for MissingPytestFixture {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing pytest fixture: {}", self.0)
    }
}

impl Problem for MissingPytestFixture {
    fn kind(&self) -> Cow<str> {
        "missing-pytest-fixture".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "fixture": self.0,
        })
    }
}

struct UnsupportedPytestConfigOption(String);

impl Display for UnsupportedPytestConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest config option: {}", self.0)
    }
}

impl Problem for UnsupportedPytestConfigOption {
    fn kind(&self) -> Cow<str> {
        "unsupported-pytest-config-option".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.0,
        })
    }
}

struct UnsupportedPytestArguments(Vec<String>);

impl Display for UnsupportedPytestArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest arguments: {:?}", self.0)
    }
}

impl Problem for UnsupportedPytestArguments {
    fn kind(&self) -> Cow<str> {
        "unsupported-pytest-arguments".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "args": self.0,
        })
    }
}

struct MissingRPackage {
    package: String,
    minimum_version: Option<String>,
}

impl MissingRPackage {
    pub fn simple(package: String) -> Self {
        Self {
            package,
            minimum_version: None,
        }
    }
}

impl Display for MissingRPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing R package: {}", self.package)?;
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl Problem for MissingRPackage {
    fn kind(&self) -> Cow<str> {
        "missing-r-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "minimum_version": self.minimum_version,
        })
    }
}

struct MissingGoPackage {
    package: String,
}

impl Problem for MissingGoPackage {
    fn kind(&self) -> Cow<str> {
        "missing-go-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
        })
    }
}

impl Display for MissingGoPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Go package: {}", self.package)
    }
}

struct MissingCHeader {
    header: String,
}

impl Problem for MissingCHeader {
    fn kind(&self) -> Cow<str> {
        "missing-c-header".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "header": self.header,
        })
    }
}

impl Display for MissingCHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C header: {}", self.header)
    }
}

impl MissingCHeader {
    fn new(header: String) -> Self {
        Self { header }
    }
}

struct MissingNodeModule(String);

impl Problem for MissingNodeModule {
    fn kind(&self) -> Cow<str> {
        "missing-node-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.0,
        })
    }
}

impl Display for MissingNodeModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node module: {}", self.0)
    }
}

struct MissingNodePackage(String);

impl Problem for MissingNodePackage {
    fn kind(&self) -> Cow<str> {
        "missing-node-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.0,
        })
    }
}

impl Display for MissingNodePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node package: {}", self.0)
    }
}

fn node_module_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    if c.get(1).unwrap().as_str().starts_with("/<<PKGBUILDDIR>>/") {
        return Ok(None);
    }
    if c.get(1).unwrap().as_str().starts_with("./") {
        return Ok(None);
    }
    Ok(Some(Box::new(MissingNodeModule(
        c.get(1).unwrap().as_str().to_string(),
    ))))
}

struct MissingConfigure;

impl Problem for MissingConfigure {
    fn kind(&self) -> Cow<str> {
        "missing-configure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingConfigure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing ./configure")
    }
}

fn command_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let command = c.get(1).unwrap().as_str();
    if command.contains("PKGBUILDDIR") {
        return Ok(None);
    }
    if command == "./configure" {
        return Ok(Some(Box::new(MissingConfigure)));
    }
    if command.starts_with("./") || command.starts_with("../") {
        return Ok(None);
    }
    if command == "debian/rules" {
        return Ok(None);
    }
    Ok(Some(Box::new(MissingCommand(command.to_string()))))
}

struct MissingVagueDependency {
    name: String,
    url: Option<String>,
    minimum_version: Option<String>,
    current_version: Option<String>,
}

impl MissingVagueDependency {
    fn simple(name: &str) -> Self {
        Self {
            name: name.to_string(),
            url: None,
            minimum_version: None,
            current_version: None,
        }
    }
}

impl Problem for MissingVagueDependency {
    fn kind(&self) -> Cow<str> {
        "missing-vague-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "url": self.url,
            "minimum_version": self.minimum_version,
            "current_version": self.current_version,
        })
    }
}

impl Display for MissingVagueDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing dependency: {}", self.name)
    }
}

struct MissingQt;

impl Problem for MissingQt {
    fn kind(&self) -> Cow<str> {
        "missing-qt".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingQt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Qt")
    }
}

struct MissingX11;

impl Problem for MissingX11 {
    fn kind(&self) -> Cow<str> {
        "missing-x11".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingX11 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing X11")
    }
}

struct MissingAutoconfMacro {
    r#macro: String,
    need_rebuild: bool,
}

impl MissingAutoconfMacro {
    fn new(r#macro: String) -> Self {
        Self {
            r#macro,
            need_rebuild: false,
        }
    }
}

impl Problem for MissingAutoconfMacro {
    fn kind(&self) -> Cow<str> {
        "missing-autoconf-macro".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "macro": self.r#macro,
            "need_rebuild": self.need_rebuild,
        })
    }
}

impl Display for MissingAutoconfMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing autoconf macro: {}", self.r#macro)
    }
}

struct DirectoryNonExistant(String);

impl Problem for DirectoryNonExistant {
    fn kind(&self) -> Cow<str> {
        "local-directory-not-existing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0,
        })
    }
}

impl Display for DirectoryNonExistant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Directory does not exist: {}", self.0)
    }
}

fn interpreter_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    if c.get(1).unwrap().as_str().starts_with('/') {
        if c.get(1).unwrap().as_str().contains("PKGBUILDDIR") {
            return Ok(None);
        }
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(c.get(1).unwrap().as_str().to_string()),
        })));
    }
    if c.get(1).unwrap().as_str().contains('/') {
        return Ok(None);
    }
    return Ok(Some(Box::new(MissingCommand(
        c.get(1).unwrap().as_str().to_string(),
    ))));
}

struct MissingPostgresExtension(String);

impl Problem for MissingPostgresExtension {
    fn kind(&self) -> Cow<str> {
        "missing-postgresql-extension".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "extension": self.0,
        })
    }
}

impl Display for MissingPostgresExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PostgreSQL extension: {}", self.0)
    }
}

struct MissingPkgConfig {
    module: String,
    minimum_version: Option<String>,
}

impl Problem for MissingPkgConfig {
    fn kind(&self) -> Cow<str> {
        "missing-pkg-config-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.module,
            "minimum_version": self.minimum_version,
        })
    }
}

impl Display for MissingPkgConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(minimum_version) = &self.minimum_version {
            write!(
                f,
                "Missing pkg-config module: {} >= {}",
                self.module, minimum_version
            )
        } else {
            write!(f, "Missing pkg-config module: {}", self.module)
        }
    }
}

impl MissingPkgConfig {
    fn new(module: String, minimum_version: Option<String>) -> Self {
        Self {
            module,
            minimum_version,
        }
    }

    fn simple(module: String) -> Self {
        Self {
            module,
            minimum_version: None,
        }
    }
}

fn pkg_config_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let expr = c.get(1).unwrap().as_str().split('\t').next().unwrap();
    if let Some((pkg, minimum)) = expr.split_once(">=") {
        return Ok(Some(Box::new(MissingPkgConfig {
            module: pkg.trim().to_string(),
            minimum_version: Some(minimum.trim().to_string()),
        })));
    }
    if !expr.contains(' ') {
        return Ok(Some(Box::new(MissingPkgConfig {
            module: expr.to_string(),
            minimum_version: None,
        })));
    }
    // Hmmm
    Ok(None)
}

lazy_static::lazy_static! {
    static ref CONFIGURE_LINE_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(
            r"^\s*Unable to find (.*) \(http(.*)\)",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(1).unwrap().as_str().to_string(),
                url: Some(m.get(2).unwrap().as_str().to_string()),
                minimum_version: None,
                current_version: None,
            })))
        ),
        regex_line_matcher!(
            r"^\s*Unable to find (.*)\.",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(1).unwrap().as_str().to_string(),
                url: None,
                minimum_version: None,
                current_version: None,
            })))
        ),
    ]);
}

struct MultiLineConfigureErrorMatcher;

impl Matcher for MultiLineConfigureErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if lines[offset].trim_end_matches(|c| c == '\r' || c == '\n') != "configure: error:" {
            return Ok(None);
        }

        let mut relevant_linenos = vec![];
        for (j, line) in lines.iter().enumerate().skip(offset + 1) {
            if line.trim().is_empty() {
                continue;
            }
            relevant_linenos.push(j);
            let m = CONFIGURE_LINE_MATCHERS.extract_from_lines(lines, j)?;
            if let Some(m) = m {
                return Ok(Some(m));
            }
        }

        let m = MultiLineMatch::new(
            Origin("configure".into()),
            relevant_linenos.clone(),
            lines
                .iter()
                .enumerate()
                .filter(|(i, _)| relevant_linenos.contains(i))
                .map(|(_, l)| l.to_string())
                .collect(),
        );

        Ok(Some((Box::new(m), None)))
    }
}

struct MissingPerlModule {
    filename: Option<String>,
    module: String,
    inc: Option<Vec<String>>,
    minimum_version: Option<String>,
}

impl Problem for MissingPerlModule {
    fn kind(&self) -> Cow<str> {
        "missing-perl-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
            "module": self.module,
            "inc": self.inc,
            "minimum_version": self.minimum_version,
        })
    }
}

impl Display for MissingPerlModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(filename) = &self.filename {
            write!(
                f,
                "Missing Perl module: {} (from {})",
                self.module, filename
            )?;
        } else {
            write!(f, "Missing Perl module: {}", self.module)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " >= {}", minimum_version)?;
        }
        if let Some(inc) = &self.inc {
            write!(f, " (INC: {})", inc.join(", "))?;
        }
        Ok(())
    }
}

impl MissingPerlModule {
    fn simple(module: &str) -> Self {
        Self {
            filename: None,
            module: module.to_string(),
            inc: None,
            minimum_version: None,
        }
    }
}

struct MultiLinePerlMissingModulesErrorMatcher;

impl Matcher for MultiLinePerlMissingModulesErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let line = lines[offset].trim_end_matches(|c| c == '\r' || c == '\n');
        if line != "# The following modules are not available." {
            return Ok(None);
        }
        if lines[offset + 1].trim_end_matches(|c| c == '\r' || c == '\n')
            != "# `perl Makefile.PL | cpanm` will install them:"
        {
            return Ok(None);
        }

        let relevant_linenos = vec![offset, offset + 1, offset + 2];

        let m = MultiLineMatch::new(
            Origin("perl line match".into()),
            relevant_linenos.clone(),
            lines
                .iter()
                .enumerate()
                .filter(|(i, _)| relevant_linenos.contains(i))
                .map(|(_, l)| l.to_string())
                .collect(),
        );

        let problem: Option<Box<dyn Problem>> = Some(Box::new(MissingPerlModule::simple(
            lines[offset + 2].trim(),
        )));

        Ok(Some((Box::new(m), problem)))
    }
}

lazy_static::lazy_static! {
    static ref VIGNETTE_LINE_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(r"^([^ ]+) is not available", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_line_matcher!(r"^The package `(.*)` is required\.", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^Package '(.*)' required.*", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^The '(.*)' package must be installed.*", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str().to_string()))))),
    ]);
}

struct MultiLineVignetteErrorMatcher;

impl Matcher for MultiLineVignetteErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let header_m =
            regex::Regex::new(r"^Error: processing vignette '(.*)' failed with diagnostics:")
                .unwrap();

        if !header_m.is_match(lines[offset]) {
            return Ok(None);
        }

        if let Some((m, p)) = VIGNETTE_LINE_MATCHERS.extract_from_lines(lines, offset + 1)? {
            return Ok(Some((m, p)));
        }

        Ok(Some((
            Box::new(SingleLineMatch {
                origin: Origin("vignette line match".into()),
                offset: offset + 1,
                line: lines[offset + 1].to_string(),
            }),
            None,
        )))
    }
}

struct MissingCSharpCompiler;

impl Problem for MissingCSharpCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-c#-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingCSharpCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C# compiler")
    }
}

struct MissingRustCompiler;

impl Problem for MissingRustCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-rust-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingRustCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Rust compiler")
    }
}

struct MissingAssembler;

impl Problem for MissingAssembler {
    fn kind(&self) -> Cow<str> {
        "missing-assembler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing assembler")
    }
}

struct AutoconfUnexpectedMacroMatcher;

impl Matcher for AutoconfUnexpectedMacroMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let regexp1 = regex::Regex::new(
            r"\./configure: line [0-9]+: syntax error near unexpected token `.+'",
        )
        .unwrap();
        if !regexp1.is_match(lines[offset]) {
            return Ok(None);
        }

        let regexp2 =
            regex::Regex::new(r"^\./configure: line [0-9]+: `[\s\t]*([A-Z0-9_]+)\(.*").unwrap();

        let c = regexp2.captures(lines[offset + 1]).unwrap();
        if c.len() != 2 {
            return Ok(None);
        }

        let m = MultiLineMatch::new(
            Origin("autoconf unexpected macro".into()),
            vec![offset + 1, offset],
            vec![lines[offset + 1].to_string(), lines[offset].to_string()],
        );

        Ok(Some((
            Box::new(m),
            Some(Box::new(MissingAutoconfMacro {
                r#macro: c.get(1).unwrap().as_str().to_string(),
                need_rebuild: true,
            })),
        )))
    }
}

struct MissingCargoCrate {
    crate_name: String,
    requirement: Option<String>,
}

impl Problem for MissingCargoCrate {
    fn kind(&self) -> Cow<str> {
        "missing-cargo-crate".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.crate_name,
            "requirement": self.requirement
        })
    }
}

impl MissingCargoCrate {
    fn simple(crate_name: String) -> Self {
        Self {
            crate_name,
            requirement: None,
        }
    }
}

impl Display for MissingCargoCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(requirement) = self.requirement.as_ref() {
            write!(
                f,
                "Missing Cargo crate {} (required by {})",
                self.crate_name, requirement
            )
        } else {
            write!(f, "Missing Cargo crate {}", self.crate_name)
        }
    }
}

struct DhWithOrderIncorrect;

impl Problem for DhWithOrderIncorrect {
    fn kind(&self) -> Cow<str> {
        "debhelper-argument-order".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for DhWithOrderIncorrect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh argument order is incorrect")
    }
}

struct NoSpaceOnDevice;

impl Problem for NoSpaceOnDevice {
    fn kind(&self) -> Cow<str> {
        "no-space-on-device".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for NoSpaceOnDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No space left on device")
    }
}

struct MissingJRE;

impl Problem for MissingJRE {
    fn kind(&self) -> Cow<str> {
        "missing-jre".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingJRE {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JRE")
    }
}

struct MissingJDK {
    jdk_path: String,
}

impl MissingJDK {
    pub fn new(jdk_path: String) -> Self {
        Self { jdk_path }
    }
}

impl Problem for MissingJDK {
    fn kind(&self) -> Cow<str> {
        "missing-jdk".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "jdk_path": self.jdk_path
        })
    }
}

impl Display for MissingJDK {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK at {}", self.jdk_path)
    }
}

struct MissingJDKFile {
    jdk_path: String,
    filename: String,
}

impl MissingJDKFile {
    pub fn new(jdk_path: String, filename: String) -> Self {
        Self { jdk_path, filename }
    }
}

impl Problem for MissingJDKFile {
    fn kind(&self) -> Cow<str> {
        "missing-jdk-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "jdk_path": self.jdk_path,
            "filename": self.filename
        })
    }
}

impl Display for MissingJDKFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK file {} at {}", self.filename, self.jdk_path)
    }
}

struct MissingPerlFile {
    filename: String,
    inc: Option<Vec<String>>,
}

impl MissingPerlFile {
    pub fn new(filename: String, inc: Option<Vec<String>>) -> Self {
        Self { filename, inc }
    }
}

impl Problem for MissingPerlFile {
    fn kind(&self) -> Cow<str> {
        "missing-perl-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
            "inc": self.inc
        })
    }
}

impl Display for MissingPerlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(inc) = self.inc.as_ref() {
            write!(
                f,
                "Missing Perl file {} (INC: {})",
                self.filename,
                inc.join(":")
            )
        } else {
            write!(f, "Missing Perl file {}", self.filename)
        }
    }
}

struct UnsupportedDebhelperCompatLevel {
    oldest_supported: u32,
    requested: u32,
}

impl UnsupportedDebhelperCompatLevel {
    pub fn new(oldest_supported: u32, requested: u32) -> Self {
        Self {
            oldest_supported,
            requested,
        }
    }
}

impl Problem for UnsupportedDebhelperCompatLevel {
    fn kind(&self) -> Cow<str> {
        "unsupported-debhelper-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "oldest_supported": self.oldest_supported,
            "requested": self.requested
        })
    }
}

impl Display for UnsupportedDebhelperCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Request debhlper compat level {} lower than supported {}",
            self.requested, self.oldest_supported
        )
    }
}

struct SetuptoolScmVersionIssue;

impl Problem for SetuptoolScmVersionIssue {
    fn kind(&self) -> Cow<str> {
        "setuptools-scm-version-issue".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for SetuptoolScmVersionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "setuptools_scm was unable to find version")
    }
}

struct MissingMavenArtifacts(Vec<String>);

impl Problem for MissingMavenArtifacts {
    fn kind(&self) -> Cow<str> {
        "missing-maven-artifacts".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "artifacts": self.0
        })
    }
}

impl Display for MissingMavenArtifacts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Maven artifacts: {}", self.0.join(", "))
    }
}

fn maven_missing_artifact(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let artifacts = m
        .get(1)
        .unwrap()
        .as_str()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    Ok(Some(Box::new(MissingMavenArtifacts(artifacts))))
}

const MAVEN_ERROR_PREFIX: &str = "(?:\\[ERROR\\]|\\[\x1b\\[1;31mERROR\x1b\\[m\\]) ";

lazy_static::lazy_static! {
    static ref COMMON_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
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
        regex_line_matcher!(
            r"^We need the Python library (.*) to be installed. Try runnning: python -m ensurepip$",
            |c| Ok(Some(Box::new(MissingPythonDistribution { distribution: c.get(1).unwrap().as_str().to_string(), python_version: None, minimum_version: None })))),
        regex_line_matcher!(
            r"^pkg_resources.DistributionNotFound: The '([^']+)' distribution was not found and is required by the application$",
            |c| Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(c.get(1).unwrap().as_str(), None).unwrap())))),
        regex_line_matcher!(
            r"^pkg_resources.DistributionNotFound: The '([^']+)' distribution was not found and is required by (.*)$",
            |c| Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(c.get(1).unwrap().as_str(), None).unwrap())))),
        regex_line_matcher!(
            r"^Please install cmake version >= (.*) and re-run setup$",
            |_| Ok(Some(Box::new(MissingCommand("cmake".to_string()))))),
        regex_line_matcher!(
            r"^pluggy.manager.PluginValidationError: Plugin '.*' could not be loaded: \(.* \(/usr/lib/python2.[0-9]/dist-packages\), Requirement.parse\('(.*)'\)\)!$",
            |c| {
                let expr = c.get(1).unwrap().as_str();
                let python_version = Some(2);
                if let Some((pkg, minimum)) = expr.split_once(">=") {
                    Ok(Some(Box::new(MissingPythonModule {
                        module: pkg.trim().to_string(),
                        python_version,
                        minimum_version: Some(minimum.trim().to_string()),
                    })))
                } else if !expr.contains(' ') {
                    Ok(Some(Box::new(MissingPythonModule {
                        module: expr.trim().to_string(),
                        python_version,
                        minimum_version: None,
                    })))
                }
                else {
                    Ok(None)
                }
            }),
        regex_line_matcher!(r"^E ImportError: (.*) could not be imported\.$", |m| Ok(Some(Box::new(MissingPythonModule {
            module: m.get(1).unwrap().as_str().to_string(),
            python_version: None,
            minimum_version: None
        })))),
        regex_line_matcher!(r"^ImportError: could not find any library for ([^ ]+) .*$", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^ImportError: cannot import name (.*), introspection typelib not found$", |m| Ok(Some(Box::new(MissingIntrospectionTypelib(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^ValueError: Namespace (.*) not available$", |m| Ok(Some(Box::new(MissingIntrospectionTypelib(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^  namespace '(.*)' ([^ ]+) is being loaded, but >= ([^ ]+) is required$", |m| {
            let package = m.get(1).unwrap().as_str();
            let min_version = m.get(3).unwrap().as_str();

            Ok(Some(Box::new(MissingRPackage {
                package: package.to_string(),
                minimum_version: Some(min_version.to_string()),
            })))
        }),
        regex_line_matcher!("^ImportError: cannot import name '(.*)' from '(.*)'$", |m| {
            let module = m.get(2).unwrap().as_str();
            let name = m.get(1).unwrap().as_str();
            // TODO(jelmer): This name won't always refer to a module
            let name = format!("{}.{}", module, name);
            Ok(Some(Box::new(MissingPythonModule {
                module: name,
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E       fixture '(.*)' not found$", |m| Ok(Some(Box::new(MissingPytestFixture(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!("^pytest: error: unrecognized arguments: (.*)$", |m| {
            let args = shlex::split(m.get(1).unwrap().as_str()).unwrap();
            Ok(Some(Box::new(UnsupportedPytestArguments(args))))
        }),
        regex_line_matcher!(
            "^INTERNALERROR> pytest.PytestConfigWarning: Unknown config option: (.*)$",
            |m| Ok(Some(Box::new(UnsupportedPytestConfigOption(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!("^E   ImportError: cannot import name '(.*)' from '(.*)'", |m| {
            let name = m.get(1).unwrap().as_str();
            let module = m.get(2).unwrap().as_str();
            Ok(Some(Box::new(MissingPythonModule {
                module: format!("{}.{}", module, name),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E   ImportError: cannot import name ([^']+)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^django.core.exceptions.ImproperlyConfigured: Error loading .* module: No module named '(.*)'", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E   ImportError: No module named (.*)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^\s*ModuleNotFoundError: No module named '(.*)'",|m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^Could not import extension .* \(exception: No module named (.*)\)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().trim().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^Could not import (.*)\.", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().trim().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^(.*): Error while finding module specification for '(.*)' \(ModuleNotFoundError: No module named '(.*)'\)", |m| {
            let exec = m.get(1).unwrap().as_str();
            let python_version = if exec.ends_with("python3") {
                Some(3)
            } else if exec.ends_with("python2") {
                Some(2)
            } else {
                None
            };

            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(3).unwrap().as_str().trim().to_string(),
                python_version,
                minimum_version: None,
            })))}),
        regex_line_matcher!("^E   ModuleNotFoundError: No module named '(.*)'", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None
            })))
        }),
        regex_line_matcher!(r"^/usr/bin/python3: No module named ([^ ]+).*", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r#"^(.*:[0-9]+|package .*): cannot find package "(.*)" in any of:"#, |m| Ok(Some(Box::new(MissingGoPackage { package: m.get(2).unwrap().as_str().to_string() })))),
        regex_line_matcher!(r#"^ImportError: Error importing plugin ".*": No module named (.*)"#, |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^ImportError: No module named (.*)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^[^:]+:\d+:\d+: fatal error: (.+\.h|.+\.hh|.+\.hpp): No such file or directory", |m| Ok(Some(Box::new(MissingCHeader { header: m.get(1).unwrap().as_str().to_string() })))),
        regex_line_matcher!(r"^[^:]+:\d+:\d+: fatal error: (.+\.xpm): No such file or directory", file_not_found),
        regex_line_matcher!(r".*fatal: not a git repository \(or any parent up to mount point /\)", |_| Ok(Some(Box::new(VcsControlDirectoryNeeded { vcs: vec!["git".to_string()] })))),
        regex_line_matcher!(r".*fatal: not a git repository \(or any of the parent directories\): \.git", |_| Ok(Some(Box::new(VcsControlDirectoryNeeded { vcs: vec!["git".to_string()] })))),
        regex_line_matcher!(r"[^:]+\.[ch]:\d+:\d+: fatal error: (.+): No such file or directory", |m| Ok(Some(Box::new(MissingCHeader { header: m.get(1).unwrap().as_str().to_string() })))),
        regex_line_matcher!("^.*␛\x1b\\[31mERROR:␛\x1b\\[39m Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[2mError: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[1m\x1b\\[31m\\[!\\] \x1b\\[1mError: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^✖ \x1b\\[31mERROR:\x1b\\[39m Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[0;31m  Error: To use the transpile option, you must have the '(.*)' module installed",
     node_module_missing),
    regex_line_matcher!(r#"^\[31mError: No test files found: "(.*)"\[39m"#),
    regex_line_matcher!(r#"^\x1b\[31mError: No test files found: "(.*)"\x1b\[39m"#),
    regex_line_matcher!(r"^\s*Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^>> Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^>> Error: Cannot find module '(.*)' from '.*'", node_module_missing),
    regex_line_matcher!(r"^Error: Failed to load parser '.*' declared in '.*': Cannot find module '(.*)'", |m| Ok(Some(Box::new(MissingNodeModule(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^    Cannot find module '(.*)' from '.*'", |m| Ok(Some(Box::new(MissingNodeModule(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^>> Error: Grunt attempted to load a \.coffee file but CoffeeScript was not installed\.", |_| Ok(Some(Box::new(MissingNodePackage("coffeescript".to_string()))))),
    regex_line_matcher!(r"^>> Got an unexpected exception from the coffee-script compiler. The original exception was: Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^\s*Module not found: Error: Can't resolve '(.*)' in '(.*)'", node_module_missing),
    regex_line_matcher!(r"^  Module (.*) in the transform option was not found\.", node_module_missing),
    regex_line_matcher!(
        r"^libtool/glibtool not found!",
        |_| Ok(Some(Box::new(MissingVagueDependency::simple("libtool"))))),
    regex_line_matcher!(r"^qmake: could not find a Qt installation of ''", |_| Ok(Some(Box::new(MissingQt)))),
    regex_line_matcher!(r"^Cannot find X include files via .*", |_| Ok(Some(Box::new(MissingX11)))),
    regex_line_matcher!(
        r"^\*\*\* No X11! Install X-Windows development headers/libraries! \*\*\*",
        |_| Ok(Some(Box::new(MissingX11)))
    ),
    regex_line_matcher!(
        r"^configure: error: \*\*\* No X11! Install X-Windows development headers/libraries! \*\*\*",
        |_| Ok(Some(Box::new(MissingX11)))
    ),
    regex_line_matcher!(
        r"^configure: error: The Java compiler javac failed.*",
        |_| Ok(Some(Box::new(MissingCommand("javac".to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: No ([^ ]+) command found",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^ERROR: InvocationError for command could not find executable (.*)",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^  \*\*\* The (.*) script could not be found\. .*",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^(.*)" command could not be found. (.*)"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: cannot find lib ([^ ]+)",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r#"^>> Local Npm module "(.*)" not found. Is it installed?"#, node_module_missing),
    regex_line_matcher!(
        r"^npm ERR! CLI for webpack must be installed.",
        |_| Ok(Some(Box::new(MissingNodePackage("webpack-cli".to_string()))))
    ),
    regex_line_matcher!(r"^npm ERR! \[!\] Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(
        r#"^npm ERR! >> Local Npm module "(.*)" not found. Is it installed\?"#,
        node_module_missing
    ),
    regex_line_matcher!(r"^npm ERR! Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(
        r"^npm ERR! ERROR in Entry module not found: Error: Can't resolve '(.*)' in '.*'",
        node_module_missing
    ),
    regex_line_matcher!(r"^npm ERR! sh: [0-9]+: (.*): not found", command_missing),
    regex_line_matcher!(r"^npm ERR! (.*\.ts)\([0-9]+,[0-9]+\): error TS[0-9]+: Cannot find module '(.*)' or its corresponding type declarations.", |m| Ok(Some(Box::new(MissingNodeModule(m.get(2).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^npm ERR! Error: spawn (.*) ENOENT", command_missing),

    regex_line_matcher!(
        r"^(\./configure): line \d+: ([A-Z0-9_]+): command not found",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"^.*: line \d+: ([^ ]+): command not found", command_missing),
    regex_line_matcher!(r"^.*: line \d+: ([^ ]+): Permission denied"),
    regex_line_matcher!(r"^make\[[0-9]+\]: .*: Permission denied"),
    regex_line_matcher!(r"^/usr/bin/texi2dvi: TeX neither supports -recorder nor outputs \\openout lines in its log file"),
    regex_line_matcher!(r"^/bin/sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^.*\.sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^.*: 1: cd: can't cd to (.*)", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^/bin/bash: (.*): command not found", command_missing),
    regex_line_matcher!(r"^bash: ([^ ]+): command not found", command_missing),
    regex_line_matcher!(r"^env: ‘(.*)’: No such file or directory", interpreter_missing),
    regex_line_matcher!(r"^/bin/bash: .*: (.*): bad interpreter: No such file or directory", interpreter_missing),
    // SH Errors
    regex_line_matcher!(r"^.*: [0-9]+: exec: (.*): not found", command_missing),
    regex_line_matcher!(r"^.*: [0-9]+: (.*): not found", command_missing),
    regex_line_matcher!(r"^/usr/bin/env: [‘'](.*)['’]: No such file or directory", command_missing),
    regex_line_matcher!(r"^make\[[0-9]+\]: (.*): Command not found", command_missing),
    regex_line_matcher!(r"^make: (.*): Command not found", command_missing),
    regex_line_matcher!(r"^make: (.*): No such file or directory", command_missing),
    regex_line_matcher!(r"^xargs: (.*): No such file or directory", command_missing),
    regex_line_matcher!(r"^make\[[0-9]+\]: ([^/ :]+): No such file or directory", command_missing),
    regex_line_matcher!(r"^.*: failed to exec '(.*)': No such file or directory", command_missing),
    regex_line_matcher!(r"^No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(r"^--\s* No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(
        r"^\-\- Please install Git, make sure it is in your path, and then try again.",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r#"^\+ERROR:  could not access file "(.*)": No such file or directory"#,
        |m| Ok(Some(Box::new(MissingPostgresExtension(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^configure: error: (Can't|Cannot) find "(.*)" in your PATH.*"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: Cannot find (.*) in your system path",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^> Cannot run program "(.*)": error=2, No such file or directory"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"^(.*) binary '(.*)' not available .*", |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^An error has occurred: FatalError: git failed\. Is it installed, and are you in a Git repository directory\?",
     |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!("^Please install '(.*)' seperately and try again.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"^> A problem occurred starting process 'command '(.*)''", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^vcver.scm.git.GitCommandError: 'git .*' returned an error code 127",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    Box::new(MultiLineConfigureErrorMatcher),
    Box::new(MultiLinePerlMissingModulesErrorMatcher),
    Box::new(MultiLineVignetteErrorMatcher),
    regex_line_matcher!(r"^configure: error: No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(r"^configure: error: (doxygen|asciidoc) is not available and maintainer mode is enabled", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: Documentation enabled but rst2html not found.", |_| Ok(Some(Box::new(MissingCommand("rst2html".to_string()))))),
    regex_line_matcher!(r"^cannot run pkg-config to check .* version at (.*) line [0-9]+\.", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^Error: pkg-config not found!", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^\*\*\* pkg-config (.*) or newer\. You can download pkg-config", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: "pkg-config".to_string(),
        minimum_version: Some(m.get(1).unwrap().as_str().to_string()),
        url: None,
        current_version: None
    })))),
    // Tox
    regex_line_matcher!(r"^ERROR: InterpreterNotFound: (.*)", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^ERROR: unable to find python", |_| Ok(Some(Box::new(MissingCommand("python".to_string()))))),
    regex_line_matcher!(r"^ ERROR: BLAS not found!", |_| Ok(Some(Box::new(MissingLibrary("blas".to_string()))))),
    Box::new(AutoconfUnexpectedMacroMatcher),
    regex_line_matcher!(r"^\./configure: [0-9]+: \.: Illegal option .*"),
    regex_line_matcher!(r"^Requested '(.*)' but version of ([^ ]+) is ([^ ]+)", pkg_config_missing),
    regex_line_matcher!(r"^.*configure: error: Package requirements \((.*)\) were not met:", pkg_config_missing),
    regex_line_matcher!(r"^configure: error: [a-z0-9_-]+-pkg-config (.*) couldn't be found", pkg_config_missing),
    regex_line_matcher!(r#"^configure: error: C preprocessor "/lib/cpp" fails sanity check"#),
    regex_line_matcher!(r"^configure: error: .*\. Please install (bison|flex)", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: No C\# compiler found. You need to install either mono \(>=(.*)\) or \.Net", |_| Ok(Some(Box::new(MissingCSharpCompiler)))),
    regex_line_matcher!(r"^configure: error: No C\# compiler found", |_| Ok(Some(Box::new(MissingCSharpCompiler)))),
    regex_line_matcher!(r"^error: can't find Rust compiler", |_| Ok(Some(Box::new(MissingRustCompiler)))),
    regex_line_matcher!(r"^Found no assembler", |_| Ok(Some(Box::new(MissingAssembler)))),
    regex_line_matcher!(r"^error: failed to get `(.*)` as a dependency of package `(.*)`", |m| Ok(Some(Box::new(MissingCargoCrate::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: (.*) requires libkqueue \(or system kqueue\). .*", |_| Ok(Some(Box::new(MissingPkgConfig::simple("libkqueue".to_string()))))),
    regex_line_matcher!(r"^Did not find pkg-config by name 'pkg-config'", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^configure: error: Required (.*) binary is missing. Please install (.*).", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r#".*meson.build:([0-9]+):([0-9]+): ERROR: Dependency "(.*)" not found"#, |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Problem encountered: No XSLT processor found, .*", |_| Ok(Some(Box::new(MissingVagueDependency::simple("xsltproc"))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): Unknown compiler\(s\): \[\['(.*)'.*\]", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: python3 \"(.*)\" missing", |m| Ok(Some(Box::new(MissingPythonModule {
        module: m.get(3).unwrap().as_str().to_string(),
        python_version: Some(3),
        minimum_version: None,
    })))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Program \'(.*)\' not found", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Git program not found, .*", |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C header \'(.*)\' not found", |m| Ok(Some(Box::new(MissingCHeader::new(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: (.+\.h) could not be found\. Please set CPPFLAGS\.", |m| Ok(Some(Box::new(MissingCHeader::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Unknown compiler\(s\): \['(.*)'\]", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Dependency \"(.*)\" not found, tried pkgconfig", |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r#".*meson.build:([0-9]+):([0-9]+): ERROR: Could not execute Vala compiler "(.*)""#, |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: python3 is missing modules: (.*)", |m| Ok(Some(Box::new(MissingPythonModule::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Invalid version of dependency, need '([^']+)' \['>=\s*([^']+)'\] found '([^']+)'\.", |m| Ok(Some(Box::new(MissingPkgConfig::new(m.get(3).unwrap().as_str().to_string(), Some(m.get(4).unwrap().as_str().to_string())))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C shared or static library '(.*)' not found", |m| Ok(Some(Box::new(MissingLibrary(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C\\+\\++ shared or static library '(.*)' not found", |m| Ok(Some(Box::new(MissingLibrary(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Pkg-config binary for machine .* not found. Giving up.", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(".*meson.build([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) require (.*) >= (.*), (.*) which were not found.", |m| Ok(Some(Box::new(MissingVagueDependency{name: m.get(4).unwrap().as_str().to_string(), current_version: None, url: None, minimum_version: Some(m.get(5).unwrap().as_str().to_string())})))),
    regex_line_matcher!(".*meson.build([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) is required to .*", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(4).unwrap().as_str()))))),
    regex_line_matcher!(r"^ERROR: (.*) is not installed\. Install at least (.*) version (.+) to continue\.", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
        current_version: None,
        url: None,
    })))),
    regex_line_matcher!(r"^configure: error: Library requirements \((.*)\) not met\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: (.*) is missing -- (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Cannot find (.*), check (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        url: Some(m.get(2).unwrap().as_str().to_string()),
        minimum_version: None,
        current_version: None
    })))),
    regex_line_matcher!(r"^configure: error: \*\*\* Unable to find (.* library)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: unable to find (.*)\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Perl Module (.*) not available", |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"(.*) was not found in your path\. Please install (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Please install (.*) >= (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(
        r"^configure: error: the required package (.*) is not installed", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: \*\*\* (.*) >= (.*) not installed.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure: error: you should install (.*) first", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: cannot locate (.*) >= (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure: error: !!! Please install (.*) !!!", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: (.*) version (.*) or higher is required", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure.(ac|in):[0-9]+: error: libtool version (.*) or higher is required", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(2).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) ([^ ]+) or better is required.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) ([^ ]+) or greater is required.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) or greater is required.*", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: (.*) library is required",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: (.*) library is not installed\.",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: OpenSSL developer library 'libssl-dev' or 'openssl-devel' not installed; cannot continue.",
        |_m| Ok(Some(Box::new(MissingLibrary("ssl".to_string()))))),
    regex_line_matcher!(
        r"configure: error: \*\*\* Cannot find (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: (.*) is required to compile .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\s*You must have (.*) installed to compile .*\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"You must install (.*) to compile (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\*\*\* No (.*) found, please in(s?)tall it \*\*\*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) required, please in(s?)tall it",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\*\* ERROR \*\* : You must have `(.*)' installed on your system\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"autogen\.sh: ERROR: You must have `(.*)' installed to compile this package\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"autogen\.sh: You must have (.*) installed\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\s*Error! You need to have (.*) installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"(configure: error|\*\*Error\*\*): You must have (.*) installed.*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required for building this package.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required to build (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required for (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: \*\*\* (.*) is required\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required, please get it from (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency{
            name: m.get(1).unwrap().as_str().to_string(),
            url: Some(m.get(2).unwrap().as_str().to_string()),
            minimum_version: None, current_version: None})))),
    regex_line_matcher!(
        r".*meson.build:\d+:\d+: ERROR: Assert failed: (.*) support explicitly required, but (.*) not found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: .*, (lib[^ ]+) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"dh: Unknown sequence --(.*) \(options should not come before the sequence\)",
        |_| Ok(Some(Box::new(DhWithOrderIncorrect)))),
    regex_line_matcher!(
        r"(dh: |dh_.*: error: )Compatibility levels before ([0-9]+) are no longer supported \(level ([0-9]+) requested\)",
        |m| {
            let l1 = m.get(2).unwrap().as_str().parse().unwrap();
            let l2 = m.get(3).unwrap().as_str().parse().unwrap();
            Ok(Some(Box::new(UnsupportedDebhelperCompatLevel::new(l1, l2))))
        }
    ),
    regex_line_matcher!(r"\{standard input\}: Error: (.*)"),
    regex_line_matcher!(r"dh: Unknown sequence (.*) \(choose from: .*\)"),
    regex_line_matcher!(r".*: .*: No space left on device", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(r"^No space left on device.", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(
        r".*Can't locate (.*).pm in @INC \(you may need to install the (.*) module\) \(@INC contains: (.*)\) at .* line [0-9]+\.",
        |m| {
            let path = format!("{}.pm", m.get(1).unwrap().as_str());
            let inc = m.get(3).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();

            Ok(Some(Box::new(MissingPerlModule{ filename: Some(path), module: m.get(2).unwrap().as_str().to_string(), minimum_version: None, inc: Some(inc)})))
        }
    ),
    regex_line_matcher!(
        r".*Can't locate (.*).pm in @INC \(you may need to install the (.*) module\) \(@INC contains: (.*)\)\.",
        |m| {
            let path = format!("{}.pm", m.get(1).unwrap().as_str());
            let inc = m.get(3).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();

            Ok(Some(Box::new(MissingPerlModule{ filename: Some(path), module: m.get(2).unwrap().as_str().to_string(), inc: Some(inc), minimum_version: None })))
        }
    ),
    regex_line_matcher!(
        r"\[DynamicPrereqs\] Can't locate (.*) at inline delegation in .*",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"Can't locate object method "(.*)" via package "(.*)" \(perhaps you forgot to load "(.*)"\?\) at .*.pm line [0-9]+\."#,
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r">\(error\): Could not expand \[(.*)'",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str().trim().trim_matches('\'')))))),

    regex_line_matcher!(
        r"\[DZ\] could not load class (.*) for license (.*)",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\- ([^\s]+)\s+\.\.\.missing. \(would need (.*)\)",
        |m| Ok(Some(Box::new(MissingPerlModule {
            filename: None,
            module: m.get(1).unwrap().as_str().to_string(),
            inc: None,
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        })))),

    regex_line_matcher!(
        r"Required plugin bundle ([^ ]+) isn't installed.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"Required plugin ([^ ]+) isn't installed.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r".*Can't locate (.*) in @INC \(@INC contains: (.*)\) at .* line .*.",
        |m| {
            let inc = m.get(2).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();
            Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), Some(inc)))))
        }),

    regex_line_matcher!(
        r"Can't find author dependency ([^ ]+) at (.*) line ([0-9]+).",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"Can't find author dependency ([^ ]+) version (.*) at (.*) line ([0-9]+).",
        |m| Ok(Some(Box::new(MissingPerlModule {
            filename: None,
            module: m.get(1).unwrap().as_str().to_string(),
            inc: None,
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        })))),
    regex_line_matcher!(
        r"> Could not find (.*)\. Please check that (.*) contains a valid JDK installation.",
        |m| Ok(Some(Box::new(MissingJDKFile::new(m.get(2).unwrap().as_str().to_string(), m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> Could not find (.*)\. Please check that (.*) contains a valid \(and compatible\) JDK installation.",
        |m| Ok(Some(Box::new(MissingJDKFile::new(m.get(2).unwrap().as_str().to_string(), m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> Kotlin could not find the required JDK tools in the Java installation '(.*)' used by Gradle. Make sure Gradle is running on a JDK, not JRE.",
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> JDK_5 environment variable is not defined. It must point to any JDK that is capable to compile with Java 5 target \((.*)\)",
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH.",
        |_| Ok(Some(Box::new(MissingJRE)))),

    regex_line_matcher!(
        r#"Error: environment variable "JAVA_HOME" must be set to a JDK \(>= v(.*)\) installation directory"#,
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"(?:/usr/bin/)?install: cannot create regular file '(.*)': No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(
        r"Cannot find source directory \((.*)\)",
        file_not_found
    ),
    regex_line_matcher!(
        r"python[0-9.]*: can't open file '(.*)': \[Errno 2\] No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(
        r"error: \[Errno 2\] No such file or directory: '(.*)'",
        file_not_found_maybe_executable
    ),
    regex_line_matcher!(
        r".*:[0-9]+:[0-9]+: ERROR: <ExternalProgram 'python3' -> \['/usr/bin/python3'\]> is not a valid python or it is missing setuptools",
        |_| Ok(Some(Box::new(MissingPythonDistribution {
            distribution: "setuptools".to_string(),
            python_version: Some(3),
            minimum_version: None,
        })))
    ),
    regex_line_matcher!(r"OSError: \[Errno 28\] No space left on device", |_| Ok(Some(Box::new(NoSpaceOnDevice)))),
    // python:setuptools_scm
    regex_line_matcher!(
        r"^LookupError: setuptools-scm was unable to detect version for '.*'\.",
        |_| Ok(Some(Box::new(SetuptoolScmVersionIssue)))
    ),
    regex_line_matcher!(
        r"^LookupError: setuptools-scm was unable to detect version for .*\.",
        |_| Ok(Some(Box::new(SetuptoolScmVersionIssue)))
    ),
    regex_line_matcher!(r"^OSError: 'git' was not found", |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!(r"^OSError: No such file (.*)", file_not_found_maybe_executable),
    regex_line_matcher!(
        r"^Could not open '(.*)': No such file or directory at /usr/share/perl/[0-9.]+/ExtUtils/MM_Unix.pm line [0-9]+.",
        |m| Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), None))))
    ),
    regex_line_matcher!(
        r#"^Can't open perl script "(.*)": No such file or directory"#,
        |m| Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), None))))),
    // Maven
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: \x1b\[1;31mCould not resolve dependencies for project .*: The following artifacts could not be resolved: (.*): Could not find artifact (.*) in (.*) \((.*)\)\x1b\[m -> \x1b\[1m\[Help 1\]\x1b\[m").as_str(), maven_missing_artifact),

    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: \x1b\[1;31mCould not resolve dependencies for project .*: Could not find artifact (.*)\x1b\[m .*").as_str(),
        maven_missing_artifact
    ),

    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: The following artifacts could not be resolved: (.*): Cannot access central \(https://repo\.maven\.apache\.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before..*").as_str(), maven_missing_artifact
    ),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Unresolveable build extension: Plugin (.*) or one of its dependencies could not be resolved: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before. @").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Non-resolvable import POM: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before. @ line [0-9]+, column [0-9]+").as_str(), maven_missing_artifact),
    regex_line_matcher!(
        r"\[FATAL\] Non-resolvable parent POM for .*: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before. .*", maven_missing_artifact),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX,r"Plugin (.*) or one of its dependencies could not be resolved: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before. -> \[Help 1\]").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Plugin (.+) or one of its dependencies could not be resolved: Failed to read artifact descriptor for (.*): (.*)").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: Cannot access .* \([^\)]+\) in offline mode and the artifact (.*) has not been downloaded from it before. -> \[Help 1\]").as_str(), maven_missing_artifact),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before..*").as_str(), maven_missing_artifact),
    regex_line_matcher!(format!("{}{}", MAVEN_ERROR_PREFIX, "Failed to execute goal (.*) on project (.*): (.*)").as_str(), |_| Ok(None)),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Error resolving version for plugin \'(.*)\' from the repositories \[.*\]: Plugin not found in any plugin repository -> \[Help 1\]").as_str(),
        |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))
    ),
    ]);
}

pub fn match_lines(
    lines: &[&str],
    offset: usize,
) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
    COMMON_MATCHERS.extract_from_lines(lines, offset)
}

macro_rules! secondary_matcher {
    ($re:expr) => {
        fancy_regex::Regex::new($re).unwrap()
    };
}

lazy_static::lazy_static! {
    /// Regexps that hint at an error of some sort, but not the error itself.
    static ref SECONDARY_MATCHERS: Vec<fancy_regex::Regex> = vec![
    secondary_matcher!(r"E: pybuild pybuild:[0-9]+: test: plugin [^ ]+ failed with:"),
    secondary_matcher!(r"[^:]+: error: (.*)"),
    secondary_matcher!(r"[^:]+:[0-9]+: error: (.*)"),
    secondary_matcher!(r"[^:]+:[0-9]+:[0-9]+: error: (.*)"),
    secondary_matcher!(r"error TS[0-9]+: (.*)"),

    secondary_matcher!(r"mount: .*: mount failed: Operation not permitted\."),

    secondary_matcher!(r"  [0-9]+:[0-9]+\s+error\s+.+"),

    secondary_matcher!(r"fontmake: Error: In '(.*)': (.*)"),

    secondary_matcher!(r"#   Failed test at t\/.*\.t line [0-9]+\."),

    secondary_matcher!(r"Gradle build daemon disappeared unexpectedly \(it may have been killed or may have crashed\)"),

    // ocaml
    secondary_matcher!(r"\*\*\* omake error:"),
    secondary_matcher!(r".*ocamlc.*: OCam has been configured with -force-safe-string: -unsafe-string is not available\."),

    // latex
    secondary_matcher!(r"\! LaTeX Error: .*"),

    secondary_matcher!(r"Killed"),

    // Java
    secondary_matcher!(r#"Exception in thread "(.*)" (.*): (.*);"#),
    secondary_matcher!(r"error: Unrecognized option: \'.*\'"),
    secondary_matcher!(r"Segmentation fault"),
    secondary_matcher!(r"\[ERROR\] (.*\.java):\[[0-9]+,[0-9]+\] (.*)"),
    secondary_matcher!(r"make: \*\*\* No targets specified and no makefile found\.  Stop\."),
    secondary_matcher!(r"make\[[0-9]+\]: \*\*\* No targets specified and no makefile found\.  Stop\."),
    secondary_matcher!(r#"make: \*\*\* No rule to make target \'(.*)\'\.  Stop\."#),
    secondary_matcher!(r"make\[[0-9]+\]: (.*): No such file or directory"),
    secondary_matcher!(r"make\[[0-9]+\]: \*\*\* \[.*:[0-9]+: .*\] Segmentation fault"),
    secondary_matcher!(
        r"make\[[0-9]+\]: \*\*\* No rule to make target \'(?!maintainer-clean)(?!clean)(.*)\'\.  Stop\."),
    secondary_matcher!(
    r".*:[0-9]+: \*\*\* empty variable name.  Stop."),
    secondary_matcher!(
    r"error: can't copy '(.*)': doesn't exist or not a regular file"),
    secondary_matcher!(
    r"error: ([0-9]+) test executed, ([0-9]+) fatal tests failed, "),
    secondary_matcher!(
    r"([0-9]+) nonfatal test failed\."),
    secondary_matcher!(
    r".*\.rst:toctree contains ref to nonexisting file \'.*\'"),
    secondary_matcher!(
    r".*\.rst:[0-9]+:term not in glossary: .*"),
    secondary_matcher!(
    r"Try adding AC_PREREQ\(\[(.*)\]\) to your configure\.ac\."),
    // Erlang
    secondary_matcher!(
    r"  (.*_test): (.+)\.\.\.\*failed\*"),
    secondary_matcher!(
    r"(.*\.erl):[0-9]+:[0-9]+: erlang:.*"),
    // Clojure
    secondary_matcher!(
    r"Could not locate (.*) or (.*) on classpath\."),
    // QMake
    secondary_matcher!(
    r"Project ERROR: .*"),
    // pdflatex
    secondary_matcher!(
    r"\!  ==> Fatal error occurred, no output PDF file produced\!"),
    // latex
    secondary_matcher!(
    r"\! Undefined control sequence\."),
    secondary_matcher!(
    r"\! Emergency stop\."),
    secondary_matcher!(r"\!pdfTeX error: pdflatex: fwrite\(\) failed"),
    // inkscape
    secondary_matcher!(r"Unknown option (?!.*ignoring.*)"),
    // CTest
    secondary_matcher!(
    r"not ok [0-9]+ .*"),
    secondary_matcher!(
    r"Errors while running CTest"),
    secondary_matcher!(
    r"dh_auto_install: error: .*"),
    secondary_matcher!(
    r"dh_quilt_patch: error: (.*)"),
    secondary_matcher!(
    r"dh.*: Aborting due to earlier error"),
    secondary_matcher!(
    r"dh.*: unknown option or error during option parsing; aborting"),
    secondary_matcher!(
    r"Could not import extension .* \(exception: .*\)"),
    secondary_matcher!(
    r"configure.ac:[0-9]+: error: (.*)"),
    secondary_matcher!(
    r"Reconfigure the source tree (via './config' or 'perl Configure'), please."),
    secondary_matcher!(
    r"dwz: Too few files for multifile optimization"),
    secondary_matcher!(
    r"\[CJM/MatchManifest\] Aborted because of MANIFEST mismatch"),
    secondary_matcher!(
    r"dh_dwz: dwz -q -- .* returned exit code [0-9]+"),
    secondary_matcher!(
    r"help2man: can\'t get `-?-help\' info from .*"),
    secondary_matcher!(
    r"[^:]+: line [0-9]+:\s+[0-9]+ Segmentation fault.*"),
    secondary_matcher!(
    r"dpkg-gencontrol: error: (.*)"),
    secondary_matcher!(
    r".*:[0-9]+:[0-9]+: (error|ERROR): (.*)"),
    secondary_matcher!(
    r".*[.]+FAILED .*"),
    secondary_matcher!(
    r"FAIL: (.*)"),
    secondary_matcher!(
    r"FAIL\!  : (.*)"),
    secondary_matcher!(
    r"\s*FAIL (.*) \(.*\)"),
    secondary_matcher!(
    r"FAIL\s+(.*) \[.*\] ?"),
    secondary_matcher!(
    r"([0-9]+)% tests passed, ([0-9]+) tests failed out of ([0-9]+)"),
    secondary_matcher!(
    r"TEST FAILURE"),
    secondary_matcher!(
    r"make\[[0-9]+\]: \*\*\* \[.*\] Error [0-9]+"),
    secondary_matcher!(
    r"make\[[0-9]+\]: \*\*\* \[.*\] Aborted"),
    secondary_matcher!(
    r"exit code=[0-9]+: .*"),
    secondary_matcher!(
    r"chmod: cannot access \'.*\': .*"),
    secondary_matcher!(
    r"dh_autoreconf: autoreconf .* returned exit code [0-9]+"),
    secondary_matcher!(
    r"make: \*\*\* \[.*\] Error [0-9]+"),
    secondary_matcher!(
    r".*:[0-9]+: \*\*\* missing separator\.  Stop\."),
    secondary_matcher!(
    r"[0-9]+ tests: [0-9]+ ok, [0-9]+ failure\(s\), [0-9]+ test\(s\) skipped"),
    secondary_matcher!(
    r"\*\*Error:\*\* (.*)"),
    secondary_matcher!(
    r"^Error: (.*)"),
    secondary_matcher!(
    r"Failed [0-9]+ tests? out of [0-9]+, [0-9.]+% okay."),
    secondary_matcher!(
    r"Failed [0-9]+\/[0-9]+ test programs. [0-9]+/[0-9]+ subtests failed."),
    secondary_matcher!(
    r"Original error was: (.*)"),
    secondary_matcher!(
    r"-- Error \(.*\.R:[0-9]+:[0-9]+\): \(.*\) [-]*"),
    secondary_matcher!(
    r"^Error \[ERR_.*\]: .*"),
    secondary_matcher!(
    r"^FAILED \(.*\)"),
    secondary_matcher!(
    r"FAILED .*"),
    // Random Python errors
    secondary_matcher!(
    "^(E  +)?(SyntaxError|TypeError|ValueError|AttributeError|NameError|django.core.exceptions..*|RuntimeError|subprocess.CalledProcessError|testtools.matchers._impl.MismatchError|PermissionError|IndexError|TypeError|AssertionError|IOError|ImportError|SerialException|OSError|qtawesome.iconic_font.FontError|redis.exceptions.ConnectionError|builtins.OverflowError|ArgumentError|httptools.parser.errors.HttpParserInvalidURLError|HypothesisException|SSLError|KeyError|Exception|rnc2rng.parser.ParseError|pkg_resources.UnknownExtra|tarfile.ReadError|numpydoc.docscrape.ParseError|distutils.errors.DistutilsOptionError|datalad.support.exceptions.IncompleteResultsError|AssertionError|Cython.Compiler.Errors.CompileError|UnicodeDecodeError|UnicodeEncodeError): .*"),
    // Rust
    secondary_matcher!(
    r"error\[E[0-9]+\]: .*"),
    secondary_matcher!(
    "^E   DeprecationWarning: .*"),
    secondary_matcher!(
    "^E       fixture '(.*)' not found"),
    // Rake
    secondary_matcher!(
    r"[0-9]+ runs, [0-9]+ assertions, [0-9]+ failures, [0-9]+ errors, [0-9]+ skips"),
    // Node
    secondary_matcher!(
    r"# failed [0-9]+ of [0-9]+ tests"),
    // Pytest
    secondary_matcher!(
    r"(.*).py:[0-9]+: AssertionError"),
    secondary_matcher!(
    r"============================ no tests ran in ([0-9.]+)s ============================="),
    // Perl
    secondary_matcher!(
    r"  Failed tests:  [0-9-]+"),
    secondary_matcher!(
    r"Failed (.*\.t): output changed"),
    // Go
    secondary_matcher!(
    r"no packages to test"),
    secondary_matcher!(
    "FAIL\t(.*)\t[0-9.]+s"),
    secondary_matcher!(
    r".*.go:[0-9]+:[0-9]+: (?!note:).*"),
    secondary_matcher!(
    r"can\'t load package: package \.: no Go files in /<<PKGBUILDDIR>>/(.*)"),
    // Ld
    secondary_matcher!(
    r"\/usr\/bin\/ld: cannot open output file (.*): No such file or directory"),
    secondary_matcher!(
    r"configure: error: (.+)"),
    secondary_matcher!(
    r"config.status: error: (.*)"),
    secondary_matcher!(
    r"E: Build killed with signal TERM after ([0-9]+) minutes of inactivity"),
    secondary_matcher!(
    r"    \[javac\] [^: ]+:[0-9]+: error: (.*)"),
    secondary_matcher!(
    r"1\) TestChannelFeature: ([^:]+):([0-9]+): assert failed"),
    secondary_matcher!(
    r"cp: target \'(.*)\' is not a directory"),
    secondary_matcher!(
    r"cp: cannot create regular file \'(.*)\': No such file or directory"),
    secondary_matcher!(
    r"couldn\'t determine home directory at (.*)"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': File exists"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': No such file or directory"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': Permission denied"),
    secondary_matcher!(
    r"ln: invalid option -- .*"),
    secondary_matcher!(
    r"mkdir: cannot create directory [‘'](.*)['’]: No such file or directory"),
    secondary_matcher!(
    r"mkdir: cannot create directory [‘'](.*)['’]: File exists"),
    secondary_matcher!(
    r"mkdir: missing operand"),
    secondary_matcher!(
    r"rmdir: failed to remove '.*': No such file or directory"),
    secondary_matcher!(
    r"Fatal error: .*"),
    secondary_matcher!(
    "Fatal Error: (.*)"),
    secondary_matcher!(
    r"Alert: (.*)"),
    secondary_matcher!(
    r#"ERROR: Test "(.*)" failed. Exiting."#),
    // scons
    secondary_matcher!(
    r"ERROR: test\(s\) failed in (.*)"),
    secondary_matcher!(
    r"./configure: line [0-9]+: syntax error near unexpected token `.*\'"),
    secondary_matcher!(
    r"scons: \*\*\* \[.*\] ValueError : unsupported pickle protocol: .*"),
    // yarn
    secondary_matcher!(
    r"ERROR: There are no scenarios; must have at least one."),
    // perl
    secondary_matcher!(
    r"Execution of (.*) aborted due to compilation errors."),
    // Mocha
    secondary_matcher!(
    r"     AssertionError \[ERR_ASSERTION\]: Missing expected exception."),
    // lt (C++)
    secondary_matcher!(
    r".*: .*:[0-9]+: .*: Assertion `.*\' failed."),
    secondary_matcher!(
    r"(.*).xml: FAILED:"),
    secondary_matcher!(
    r" BROKEN .*"),
    secondary_matcher!(
    r"failed: [0-9]+-.*"),
    // ninja
    secondary_matcher!(
    r"ninja: build stopped: subcommand failed."),
    secondary_matcher!(
    r".*\.s:[0-9]+: Error: .*"),
    // rollup
    secondary_matcher!(r"\[\!\] Error: Unexpected token"),
    // glib
    secondary_matcher!(r"\(.*:[0-9]+\): [a-zA-Z0-9]+-CRITICAL \*\*: [0-9:.]+: .*"),
    secondary_matcher!(
    r"tar: option requires an argument -- \'.\'"),
    secondary_matcher!(
    r"tar: .*: Cannot stat: No such file or directory"),
    secondary_matcher!(
    r"tar: .*: Cannot open: No such file or directory"),
    // rsvg-convert
    secondary_matcher!(
    r"Could not render file (.*.svg)"),
    // pybuild tests
    secondary_matcher!(
    r"ERROR: file not found: (.*)"),
    // msgfmt
    secondary_matcher!(
    r"/usr/bin/msgfmt: found [0-9]+ fatal errors"),
    // Docker
    secondary_matcher!(
    r"Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running\?"),
    secondary_matcher!(
    r"dh_makeshlibs: failing due to earlier errors"),
    // Ruby
    secondary_matcher!(
    r"([^:]+)\.rb:[0-9]+:in `([^\'])+\': (.*) \((.*)\)"),
    secondary_matcher!(
    r".*: \*\*\* ERROR: There where errors/warnings in server logs after running test cases."),
    secondary_matcher!(
    r"Errno::EEXIST: File exists @ dir_s_mkdir - .*"),
    secondary_matcher!(
    r"Test environment was found to be incomplete at configuration time,"),
    secondary_matcher!(
    r"libtool:   error: cannot find the library \'(.*)\' or unhandled argument \'(.*)\'"),
    secondary_matcher!(
    r"npm ERR\! (.*)"),
    secondary_matcher!(
    r"install: failed to access \'(.*)\': (.*)"),
    secondary_matcher!(
    r"MSBUILD: error MSBUILD[0-9]+: Project file \'(.*)\' not found."),
    secondary_matcher!(
    r"E: (.*)"),
    secondary_matcher!(
    r"(.*)\(([0-9]+),([0-9]+)\): Error: .*"),
    // C #
    secondary_matcher!(
    r"(.*)\.cs\([0-9]+,[0-9]+\): error CS[0-9]+: .*"),
    secondary_matcher!(
    r".*Segmentation fault.*"),
    secondary_matcher!(
    r"a2x: ERROR: (.*) returned non-zero exit status ([0-9]+)"),
    secondary_matcher!(
    r"-- Configuring incomplete, errors occurred\!"),
    secondary_matcher!(
    r#"Error opening link script "(.*)""#),
    secondary_matcher!(
    r"cc: error: (.*)"),
    secondary_matcher!(
    r"\[ERROR\] .*"),
    secondary_matcher!(
    r"dh_auto_(test|build): error: (.*)"),
    secondary_matcher!(
    r"tar: This does not look like a tar archive"),
    secondary_matcher!(
    r"\[DZ\] no (name|version) was ever set"),
    secondary_matcher!(
    r"\[Runtime\] No -phase or -relationship specified at .* line [0-9]+\."),
    secondary_matcher!(
    r"diff: (.*): No such file or directory"),
    secondary_matcher!(
    r"gpg: signing failed: .*"),
    // mh_install
    secondary_matcher!(
    r"Cannot find the jar to install: (.*)"),
    secondary_matcher!(
    r"ERROR: .*"),
    secondary_matcher!(
    r"> error: (.*)"),
    secondary_matcher!(
    r"error: (.*)"),
    secondary_matcher!(
    r"(.*\.hs):[0-9]+:[0-9]+: error:"),
    secondary_matcher!(
    r"go1: internal compiler error: .*"),
];
}

pub fn find_secondary_build_failure(
    lines: &[&str],
    start_offset: usize,
) -> Option<SingleLineMatch> {
    let start = max(0, (lines.len() as isize) - (start_offset as isize)) as usize;
    for offset in start..lines.len() {
        let line = lines[offset];
        let match_line = line.trim_end_matches('\n');
        for regexp in SECONDARY_MATCHERS.iter() {
            if regexp.is_match(match_line).unwrap() {
                let origin = Origin(format!("secondary regex {:?}", regexp));
                log::debug!(
                    "Found match against {:?} on {:?} (line {})",
                    regexp,
                    line,
                    offset + 1
                );
                return Some(SingleLineMatch {
                    origin,
                    offset,
                    line: line.to_string(),
                });
            }
        }
    }
    None
}
