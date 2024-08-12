use crate::lines::Lines;
use crate::problems::common::*;
/// Common code for all environments.
// TODO(jelmer): Right now this is just a straight port from Python. It needs a massive amount of
// refactoring, including a split of the file.
use crate::r#match::{Error, Matcher, MatcherGroup, RegexLineMatcher};
use crate::regex_line_matcher;
use crate::{Match, Problem};
use crate::{MultiLineMatch, Origin, SingleLineMatch};
use lazy_regex::{regex_captures, regex_is_match};
use regex::Captures;
use std::borrow::Cow;
use std::fmt::Display;

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

fn file_not_found_maybe_executable(p: &str) -> Result<Option<Box<dyn Problem>>, Error> {
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

#[derive(Debug, Clone)]
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
        for (j, line) in lines.enumerate_forward(None).skip(offset + 1) {
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

#[derive(Debug, Clone)]
struct HaskellMissingDependencyMatcher;

impl Matcher for HaskellMissingDependencyMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if !regex_is_match!(
            r"(.*): Encountered missing or private dependencies:",
            lines[offset].trim_end_matches('\n')
        ) {
            return Ok(None);
        }

        let mut deps = vec![];
        let mut offsets = vec![offset];

        for (offset, line) in lines.enumerate_forward(None).skip(offset + 1) {
            if line.trim().is_empty() {
                break;
            }
            if let Some((dep, _)) = line.trim().split_once(',') {
                deps.push(dep.to_string());
            }
            offsets.push(offset);
        }
        let m = MultiLineMatch {
            origin: Origin("haskell dependencies".into()),
            offsets: offsets.clone(),
            lines: offsets.iter().map(|i| lines[*i].to_string()).collect(),
        };
        let p = MissingHaskellDependencies(deps);
        Ok(Some((Box::new(m), Some(Box::new(p)))))
    }
}

#[derive(Debug, Clone)]
struct MissingSetupPyCommand(String);

impl Problem for MissingSetupPyCommand {
    fn kind(&self) -> Cow<str> {
        "missing-setup.py-command".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.0,
        })
    }
}

impl Display for MissingSetupPyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing setup.py command: {}", self.0)
    }
}

#[derive(Debug, Clone)]
struct SetupPyCommandMissingMatcher;

impl Matcher for SetupPyCommandMissingMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let first_offset = offset;
        let command =
            match regex_captures!(r"error: invalid command \'(.*)\'", lines[offset].trim()) {
                None => return Ok(None),
                Some((_, command)) => command,
            };

        for j in 0..20 {
            let offset = offset - j;
            let line = lines[offset].trim_end_matches('\n');

            if regex_is_match!(
                r"usage: setup.py \[global_opts\] cmd1 \[cmd1_opts\] \[cmd2 \[cmd2_opts\] \.\.\.\]",
                line,
            ) {
                let offsets: Vec<usize> = vec![first_offset];
                let m = MultiLineMatch {
                    origin: Origin("setup.py".into()),
                    offsets,
                    lines: vec![lines[first_offset].to_string()],
                };

                let p = MissingSetupPyCommand(command.to_string());
                return Ok(Some((Box::new(m), Some(Box::new(p)))));
            }
        }

        log::warn!("Unable to find setup.py usage line");
        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct PythonFileNotFoundErrorMatcher;

impl Matcher for PythonFileNotFoundErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if let Some((_, name)) = lazy_regex::regex_captures!(
            r"^(?:E  +)?FileNotFoundError: \[Errno 2\] No such file or directory: \'(.*)\'",
            lines[offset].trim_end_matches('\n')
        ) {
            if offset > 2 && lines[offset - 2].contains("subprocess") {
                return Ok(Some((
                    Box::new(SingleLineMatch {
                        origin: Origin("python".into()),
                        offset,
                        line: lines[offset].to_string(),
                    }),
                    Some(Box::new(MissingCommand(name.to_string()))),
                )));
            } else {
                return Ok(Some((
                    Box::new(SingleLineMatch {
                        origin: Origin("python".into()),
                        offset,
                        line: lines[offset].to_string(),
                    }),
                    file_not_found_maybe_executable(name)?,
                )));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NotExecutableFile(String);

impl NotExecutableFile {
    pub fn new(path: String) -> Self {
        Self(path)
    }
}

impl Problem for NotExecutableFile {
    fn kind(&self) -> Cow<str> {
        "not-executable-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0
        })
    }
}

impl Display for NotExecutableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Command not executable: {}", self.0)
    }
}

#[derive(Debug, Clone)]
struct DhMissingUninstalled(String);

impl DhMissingUninstalled {
    pub fn new(missing_file: String) -> Self {
        Self(missing_file)
    }
}

impl Problem for DhMissingUninstalled {
    fn kind(&self) -> Cow<str> {
        "dh-missing-uninstalled".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "missing_file": self.0
        })
    }
}

impl Display for DhMissingUninstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh_missing file not installed: {}", self.0)
    }
}

#[derive(Debug, Clone)]
struct DhLinkDestinationIsDirectory(String);

impl DhLinkDestinationIsDirectory {
    pub fn new(path: String) -> Self {
        Self(path)
    }
}

impl Problem for DhLinkDestinationIsDirectory {
    fn kind(&self) -> Cow<str> {
        "dh-link-destination-is-directory".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0
        })
    }
}

impl Display for DhLinkDestinationIsDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Link destination {} is directory", self.0)
    }
}

#[derive(Debug, Clone)]
struct MissingXmlEntity {
    url: String,
}

impl MissingXmlEntity {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Problem for MissingXmlEntity {
    fn kind(&self) -> Cow<str> {
        "missing-xml-entity".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url
        })
    }
}

impl Display for MissingXmlEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing XML entity: {}", self.url)
    }
}

#[derive(Debug, Clone)]
struct CcacheError(String);

impl CcacheError {
    pub fn new(error: String) -> Self {
        Self(error)
    }
}

impl Problem for CcacheError {
    fn kind(&self) -> Cow<str> {
        "ccache-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.0
        })
    }
}

impl Display for CcacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ccache error: {}", self.0)
    }
}

#[derive(Debug, Clone)]
struct DebianVersionRejected {
    version: String,
}

impl DebianVersionRejected {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl Problem for DebianVersionRejected {
    fn kind(&self) -> Cow<str> {
        "debian-version-rejected".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version
        })
    }
}

impl Display for DebianVersionRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Debian Version Rejected; {}", self.version)
    }
}

#[derive(Debug, Clone)]
pub struct PatchApplicationFailed {
    pub patchname: String,
}

impl PatchApplicationFailed {
    pub fn new(patchname: String) -> Self {
        Self { patchname }
    }
}

impl Problem for PatchApplicationFailed {
    fn kind(&self) -> Cow<str> {
        "patch-application-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "patchname": self.patchname
        })
    }
}

impl Display for PatchApplicationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Patch application failed: {}", self.patchname)
    }
}

#[derive(Debug, Clone)]
struct NeedPgBuildExtUpdateControl {
    generated_path: String,
    template_path: String,
}

impl NeedPgBuildExtUpdateControl {
    pub fn new(generated_path: String, template_path: String) -> Self {
        Self {
            generated_path,
            template_path,
        }
    }
}

impl Problem for NeedPgBuildExtUpdateControl {
    fn kind(&self) -> Cow<str> {
        "need-pg-buildext-updatecontrol".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "generated_path": self.generated_path,
            "template_path": self.template_path
        })
    }
}

impl Display for NeedPgBuildExtUpdateControl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Need to run 'pg_buildext updatecontrol' to update {}",
            self.generated_path
        )
    }
}

#[derive(Debug, Clone)]
struct DhAddonLoadFailure {
    name: String,
    path: String,
}

impl DhAddonLoadFailure {
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }
}

impl Problem for DhAddonLoadFailure {
    fn kind(&self) -> Cow<str> {
        "dh-addon-load-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "path": self.path
        })
    }
}

impl Display for DhAddonLoadFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh addon loading failed: {}", self.name)
    }
}

#[derive(Debug, Clone)]
struct DhUntilUnsupported;

impl DhUntilUnsupported {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for DhUntilUnsupported {
    fn kind(&self) -> Cow<str> {
        "dh-until-unsupported".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for DhUntilUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh --until is no longer supported")
    }
}

fn r_missing_package(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let fragment = m.get(1).unwrap().as_str();
    let deps = fragment
        .split(",")
        .map(|dep| {
            dep.trim_matches('‘')
                .trim_matches('’')
                .trim_matches('\'')
                .to_string()
        })
        .collect::<Vec<_>>();
    Ok(Some(Box::new(MissingRPackage::simple(deps[0].clone()))))
}

#[derive(Debug, Clone)]
struct DebhelperPatternNotFound {
    pattern: String,
    tool: String,
    directories: Vec<String>,
}

impl DebhelperPatternNotFound {
    pub fn new(pattern: String, tool: String, directories: Vec<String>) -> Self {
        Self {
            pattern,
            tool,
            directories,
        }
    }
}

impl Problem for DebhelperPatternNotFound {
    fn kind(&self) -> Cow<str> {
        "debhelper-pattern-not-found".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern": self.pattern,
            "tool": self.tool,
            "directories": self.directories
        })
    }
}

impl Display for DebhelperPatternNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "debhelper ({}) expansion failed for {:?} (directories: {:?})",
            self.tool, self.pattern, self.directories
        )
    }
}

#[derive(Debug, Clone)]
struct MissingPerlManifest;

impl MissingPerlManifest {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingPerlManifest {
    fn kind(&self) -> Cow<str> {
        "missing-perl-manifest".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingPerlManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Perl MANIFEST")
    }
}

#[derive(Debug, Clone)]
struct ImageMagickDelegateMissing {
    delegate: String,
}

impl ImageMagickDelegateMissing {
    pub fn new(delegate: String) -> Self {
        Self { delegate }
    }
}

impl Problem for ImageMagickDelegateMissing {
    fn kind(&self) -> Cow<str> {
        "imagemagick-delegate-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "delegate": self.delegate
        })
    }
}

impl Display for ImageMagickDelegateMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Imagemagick missing delegate: {}", self.delegate)
    }
}

#[derive(Debug, Clone)]
struct Cancelled;

impl Cancelled {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for Cancelled {
    fn kind(&self) -> Cow<str> {
        "cancelled".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cancelled by runner or job manager")
    }
}

fn webpack_file_missing(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let path = std::path::Path::new(m.get(1).unwrap().as_str());
    let container = std::path::Path::new(m.get(2).unwrap().as_str());
    let path = container.join(path);
    if path.starts_with("/") && !path.as_path().starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile { path })));
    }
    Ok(None)
}

#[derive(Debug, Clone)]
struct DisappearedSymbols;

impl DisappearedSymbols {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for DisappearedSymbols {
    fn kind(&self) -> Cow<str> {
        "disappeared-symbols".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for DisappearedSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Disappeared symbols")
    }
}

#[derive(Debug, Clone)]
struct DuplicateDHCompatLevel {
    command: String,
}

impl DuplicateDHCompatLevel {
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl Problem for DuplicateDHCompatLevel {
    fn kind(&self) -> Cow<str> {
        "duplicate-dh-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.command
        })
    }
}

impl Display for DuplicateDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "DH Compat Level specified twice (command: {})",
            self.command
        )
    }
}

#[derive(Debug, Clone)]
struct MissingDHCompatLevel {
    command: String,
}

impl MissingDHCompatLevel {
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl Problem for MissingDHCompatLevel {
    fn kind(&self) -> Cow<str> {
        "missing-dh-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.command
        })
    }
}

impl Display for MissingDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing DH Compat Level (command: {})", self.command)
    }
}

#[derive(Debug, Clone)]
struct MissingJVM;

impl MissingJVM {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingJVM {
    fn kind(&self) -> Cow<str> {
        "missing-jvm".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingJVM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing JVM")
    }
}

#[derive(Debug, Clone)]
struct MissingRubyGem {
    gem: String,
    version: Option<String>,
}

impl MissingRubyGem {
    pub fn new(gem: String, version: Option<String>) -> Self {
        Self { gem, version }
    }

    pub fn simple(gem: String) -> Self {
        Self::new(gem, None)
    }
}

impl Problem for MissingRubyGem {
    fn kind(&self) -> Cow<str> {
        "missing-ruby-gem".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "gem": self.gem,
            "version": self.version
        })
    }
}

impl Display for MissingRubyGem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "missing ruby gem: {} (>= {})", self.gem, version)
        } else {
            write!(f, "missing ruby gem: {}", self.gem)
        }
    }
}

fn ruby_missing_gem(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let mut minimum_version = None;
    for grp in m.get(2).unwrap().as_str().split(",") {
        if let Some((cond, val)) = grp.trim().split_once(" ") {
            if cond == ">=" {
                minimum_version = Some(val.to_string());
                break;
            }
            if cond == "~>" {
                minimum_version = Some(val.to_string());
            }
        }
    }
    Ok(Some(Box::new(MissingRubyGem::new(
        m.get(1).unwrap().as_str().to_string(),
        minimum_version,
    ))))
}

#[derive(Debug, Clone)]
struct MissingLibtool;

impl MissingLibtool {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingLibtool {
    fn kind(&self) -> Cow<str> {
        "missing-libtool".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingLibtool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Libtool is missing")
    }
}

#[derive(Debug, Clone)]
struct MissingJavaScriptRuntime;

impl MissingJavaScriptRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingJavaScriptRuntime {
    fn kind(&self) -> Cow<str> {
        "javascript-runtime-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingJavaScriptRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JavaScript Runtime")
    }
}

#[derive(Debug, Clone)]
struct MissingRubyFile {
    filename: String,
}

impl MissingRubyFile {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl Problem for MissingRubyFile {
    fn kind(&self) -> Cow<str> {
        "missing-ruby-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename
        })
    }
}

impl Display for MissingRubyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing ruby file: {}", self.filename)
    }
}

#[derive(Debug, Clone)]
struct MissingPhpClass {
    php_class: String,
}

impl MissingPhpClass {
    pub fn new(php_class: String) -> Self {
        Self { php_class }
    }

    pub fn simple(php_class: String) -> Self {
        Self::new(php_class)
    }
}

impl Problem for MissingPhpClass {
    fn kind(&self) -> Cow<str> {
        "missing-php-class".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "php_class": self.php_class
        })
    }
}

impl Display for MissingPhpClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing PHP class: {}", self.php_class)
    }
}

#[derive(Debug, Clone)]
struct MissingJavaClass {
    classname: String,
}

impl MissingJavaClass {
    pub fn new(classname: String) -> Self {
        Self { classname }
    }

    pub fn simple(classname: String) -> Self {
        Self::new(classname)
    }
}

impl Problem for MissingJavaClass {
    fn kind(&self) -> Cow<str> {
        "missing-java-class".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "classname": self.classname
        })
    }
}

impl Display for MissingJavaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Java class: {}", self.classname)
    }
}

#[derive(Debug, Clone)]
struct MissingSprocketsFile {
    name: String,
    content_type: String,
}

impl MissingSprocketsFile {
    pub fn new(name: String, content_type: String) -> Self {
        Self { name, content_type }
    }
}

impl Problem for MissingSprocketsFile {
    fn kind(&self) -> Cow<str> {
        "missing-sprockets-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "content_type": self.content_type
        })
    }
}

impl Display for MissingSprocketsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "missing sprockets file: {} (type: {})",
            self.name, self.content_type
        )
    }
}

#[derive(Debug, Clone)]
struct MissingXfceDependency {
    package: String,
}

impl MissingXfceDependency {
    pub fn new(package: String) -> Self {
        Self { package }
    }
}

impl Problem for MissingXfceDependency {
    fn kind(&self) -> Cow<str> {
        "missing-xfce-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package
        })
    }
}

impl Display for MissingXfceDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing XFCE build dependency: {}", self.package)
    }
}

#[derive(Debug, Clone)]
struct GnomeCommonMissing;

impl Problem for GnomeCommonMissing {
    fn kind(&self) -> Cow<str> {
        "missing-gnome-common".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for GnomeCommonMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "gnome-common is not installed")
    }
}

#[derive(Debug, Clone)]
struct MissingConfigStatusInput {
    path: String,
}

impl MissingConfigStatusInput {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Problem for MissingConfigStatusInput {
    fn kind(&self) -> Cow<str> {
        "missing-config.status-input".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path
        })
    }
}

impl Display for MissingConfigStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing config.status input {}", self.path)
    }
}

#[derive(Debug, Clone)]
struct MissingGnomeCommonDependency {
    package: String,
    minimum_version: Option<String>,
}

impl MissingGnomeCommonDependency {
    pub fn new(package: String, minimum_version: Option<String>) -> Self {
        Self {
            package,
            minimum_version,
        }
    }

    pub fn simple(package: String) -> Self {
        Self::new(package, None)
    }
}

impl Problem for MissingGnomeCommonDependency {
    fn kind(&self) -> Cow<str> {
        "missing-gnome-common-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "minimum_version": self.minimum_version
        })
    }
}

impl Display for MissingGnomeCommonDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "missing gnome-common dependency: {}: (>= {})",
            self.package,
            self.minimum_version.as_deref().unwrap_or("any")
        )
    }
}

#[derive(Debug, Clone)]
struct MissingAutomakeInput {
    path: String,
}

impl MissingAutomakeInput {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Problem for MissingAutomakeInput {
    fn kind(&self) -> Cow<str> {
        "missing-automake-input".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path
        })
    }
}

impl Display for MissingAutomakeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "automake input file {} missing", self.path)
    }
}

#[derive(Debug, Clone)]
pub struct ChrootNotFound {
    pub chroot: String,
}

impl ChrootNotFound {
    pub fn new(chroot: String) -> Self {
        Self { chroot }
    }
}

impl Problem for ChrootNotFound {
    fn kind(&self) -> Cow<str> {
        "chroot-not-found".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "chroot": self.chroot
        })
    }
}

impl Display for ChrootNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "chroot not found: {}", self.chroot)
    }
}

const MAVEN_ERROR_PREFIX: &str = "(?:\\[ERROR\\]|\\[\x1b\\[1;31mERROR\x1b\\[m\\]) ";

lazy_static::lazy_static! {
    static ref COMMON_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(
            r"^make\[[0-9]+\]: \*\*\* No rule to make target '(.*)', needed by '.*'\.  Stop\.$",
            file_not_found
        ),
        regex_line_matcher!(r"^[^:]+:\d+: (.*): No such file or directory$", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
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
        r"^error: \[Errno 2\] No such file or directory: '(.*)'",
        |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())
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
    regex_line_matcher!(r"^OSError: No such file (.*)", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
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
    regex_line_matcher!(
        r"E: eatmydata: unable to find '(.*)' in PATH",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"'(.*)' not found in PATH at (.*) line ([0-9]+)\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"/usr/bin/eatmydata: [0-9]+: exec: (.*): not found",
        command_missing
    ),
    regex_line_matcher!(
        r"/usr/bin/eatmydata: [0-9]+: exec: (.*): Permission denied",
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"(.*): exec: "(.*)": executable file not found in \$PATH"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at (.*) line ([0-9]+)\."#,
        command_missing
    ),
    regex_line_matcher!(
        r"dh_missing: (warning: )?(.*) exists in debian/.* but is not installed to anywhere",
        |m| Ok(Some(Box::new(DhMissingUninstalled(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"dh_link: link destination (.*) is a directory",
                        |m| Ok(Some(Box::new(DhLinkDestinationIsDirectory(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"I/O error : Attempt to load network entity (.*)",
                        |m| Ok(Some(Box::new(MissingXmlEntity::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"ccache: error: (.*)",
    |m| Ok(Some(Box::new(CcacheError(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"dh: The --until option is not supported any longer \(#932537\). Use override targets instead.",
        |_| Ok(Some(Box::new(DhUntilUnsupported::new())))
    ),
    regex_line_matcher!(
        r"dh: unable to load addon (.*): (.*) did not return a true value at \(eval 11\) line ([0-9]+).",
        |m| Ok(Some(Box::new(DhAddonLoadFailure::new(m.get(1).unwrap().as_str().to_string(), m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "ERROR: dependencies (.*) are not available for package [‘'](.*)['’]",
        r_missing_package
    ),
    regex_line_matcher!(
        "ERROR: dependency [‘'](.*)['’] is not available for package [‘'](.*)[’']",
        r_missing_package
    ),
    regex_line_matcher!(
        r"Error in library\(.*\) : there is no package called \'(.*)\'",
        r_missing_package
    ),
    regex_line_matcher!(r"Error in .* : there is no package called \'(.*)\'", r_missing_package),
    regex_line_matcher!(r"there is no package called \'(.*)\'", r_missing_package),
    regex_line_matcher!(
        r"  namespace ‘(.*)’ ([^ ]+) is being loaded, but >= ([^ ]+) is required",
        |m| Ok(Some(Box::new(MissingRPackage{ package: m.get(1).unwrap().as_str().to_string(), minimum_version: Some(m.get(3).unwrap().as_str().to_string())})))
    ),
    regex_line_matcher!(
        r"  namespace ‘(.*)’ ([^ ]+) is already loaded, but >= ([^ ]+) is required",
        |m| Ok(Some(Box::new(MissingRPackage{package: m.get(1).unwrap().as_str().to_string(), minimum_version: Some(m.get(3).unwrap().as_str().to_string())})))
    ),
    regex_line_matcher!(r"b\'convert convert: Unable to read font \((.*)\) \[No such file or directory\]\.\\n\'",
     file_not_found),
    regex_line_matcher!(r"mv: cannot stat \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(r"mv: cannot move \'.*\' to \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(
        r"(/usr/bin/install|mv): will not overwrite just-created \'(.*)\' with \'(.*)\'",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"^IOError: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"^error: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"^E   IOError: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!("FAIL\t(.+\\/.+\\/.+)\t([0-9.]+)s", |_| Ok(None)),
    regex_line_matcher!(
        r#"dh_(.*): Cannot find \(any matches for\) "(.*)" \(tried in (.*)\)"#,
        |m| Ok(Some(Box::new(DebhelperPatternNotFound {
            pattern: m.get(2).unwrap().as_str().to_string(),
            tool: m.get(1).unwrap().as_str().to_string(),
            directories: m.get(3).unwrap().as_str().split(',').map(|s| s.trim().to_string()).collect(),
        })))
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at /usr/share/perl5/Debian/Debhelper/Dh_Lib.pm line [0-9]+."#,
        command_missing
    ),
    regex_line_matcher!(
        r#"Can\'t exec "(.*)": Permission denied at (.*) line [0-9]+\."#,
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"/usr/bin/fakeroot: [0-9]+: (.*): Permission denied",
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r".*: error: (.*) command not found", command_missing),
    regex_line_matcher!(r"error: command '(.*)' failed: No such file or directory",
     command_missing),
    regex_line_matcher!(
        r"dh_install: Please use dh_missing --list-missing/--fail-missing instead",
        |_| Ok(None)
    ),

    regex_line_matcher!(
        r#"dh([^:]*): Please use the third-party "pybuild" build system instead of python-distutils"#,
        |_| Ok(None)
    ),
    // A Python error, but not likely to be actionable. The previous line will have the actual line that failed.
    regex_line_matcher!(r"ImportError: cannot import name (.*)", |_| Ok(None)),
    // Rust ?
    regex_line_matcher!(r"\s*= note: /usr/bin/ld: cannot find -l([^ ]+): .*", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"\s*= note: /usr/bin/ld: cannot find -l([^ ]+)", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/usr/bin/ld: cannot find -l([^ ]+): .*", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/usr/bin/ld: cannot find -l([^ ]+)", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"Could not find gem \'([^ ]+) \(([^)]+)\)\', which is required by gem.*",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"Could not find gem \'([^ \']+)\', which is required by gem.*",
        |m| Ok(Some(Box::new(MissingRubyGem::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`to_specs\': Could not find \'(.*)\' \(([^)]+)\) among [0-9]+ total gem\(s\) \(Gem::MissingSpecError\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`to_specs\': Could not find \'(.*)\' \(([^)]+)\) - .* \(Gem::MissingSpecVersionError\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`block in verify_gemfile_dependencies_are_found\!\': Could not find gem \'(.*)\' in any of the gem sources listed in your Gemfile\. \(Bundler::GemNotFound\)",
        |m| Ok(Some(Box::new(MissingRubyGem::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"Exception: (.*) not in path[!.]*",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"Exception: Building sdist requires that ([^ ]+) be installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`find_spec_for_exe\': can\'t find gem (.*) \(([^)]+)\) with executable (.*) \(Gem::GemNotFoundException\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r".?PHP Fatal error:  Uncaught Error: Class \'(.*)\' not found in (.*):([0-9]+)",
        |m| Ok(Some(Box::new(MissingPhpClass::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Caused by: java.lang.ClassNotFoundException: (.*)", |m| Ok(Some(Box::new(MissingJavaClass::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"\[(.*)\] \t\t:: (.*)\#(.*);\$\{(.*)\}: not found",
        |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![format!("{}:{}:jar:debian", m.get(2).unwrap().as_str(), m.get(3).unwrap().as_str())]))))
    ),
    regex_line_matcher!(
        r"Caused by: java.lang.IllegalArgumentException: Cannot find JAR \'(.*)\' required by module \'(.*)\' using classpath or distribution directory \'(.*)\'",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r".*\.xml:[0-9]+: Unable to find a javac compiler;",
        |_| Ok(Some(Box::new(MissingJavaClass::simple("com.sun.tools.javac.Main".to_string()))))
    ),
    regex_line_matcher!(
        r#"checking for (.*)\.\.\. configure: error: "Cannot check for existence of module (.*) without pkgconf""#,
        |_| Ok(Some(Box::new(MissingCommand("pkgconf".to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: Could not find '(.*)' in path\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"autoreconf was not found; .*",
        |_| Ok(Some(Box::new(MissingCommand("autoreconf".to_string()))))
    ),
    regex_line_matcher!(r"^g\+\+: error: (.*): No such file or directory", file_not_found),
    regex_line_matcher!(r"strip: \'(.*)\': No such file", file_not_found),
    regex_line_matcher!(
        r"Sprockets::FileNotFound: couldn\'t find file \'(.*)\' with type \'(.*)\'",
        |m| Ok(Some(Box::new(MissingSprocketsFile{ name: m.get(1).unwrap().as_str().to_string(), content_type: m.get(2).unwrap().as_str().to_string()})))
    ),
    regex_line_matcher!(
        r#"xdt-autogen: You must have "(.*)" installed. You can get if from"#,
        |m| Ok(Some(Box::new(MissingXfceDependency::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"autogen.sh: You must have GNU autoconf installed.",
        |_| Ok(Some(Box::new(MissingCommand("autoconf".to_string()))))
    ),
    regex_line_matcher!(
        r"\s*You must have (autoconf|automake|aclocal|libtool|libtoolize) installed to compile (.*)\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"It appears that Autotools is not correctly installed on this system.",
        |_| Ok(Some(Box::new(MissingCommand("autoconf".to_string()))))
    ),
    regex_line_matcher!(
        r"\*\*\* No autoreconf found \*\*\*",
        |_| Ok(Some(Box::new(MissingCommand("autoreconf".to_string()))))
    ),
    regex_line_matcher!(r"You need to install gnome-common module and make.*", |_| Ok(Some(Box::new(GnomeCommonMissing)))),
    regex_line_matcher!(r"You need to install the gnome-common module and make.*", |_| Ok(Some(Box::new(GnomeCommonMissing)))),
    regex_line_matcher!(
        r"You need to install gnome-common from the GNOME (git|CVS|SVN)",
        |_| Ok(Some(Box::new(GnomeCommonMissing)))
    ),
    regex_line_matcher!(
        r"automake: error: cannot open < (.*): No such file or directory",
        |m| Ok(Some(Box::new(MissingAutomakeInput::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure(|\.in|\.ac):[0-9]+: error: possibly undefined macro: (.*)",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure.(in|ac):[0-9]+: error: macro (.*) is not defined; is a m4 file missing\?",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"config.status: error: cannot find input file: `(.*)\'",
        |m| Ok(Some(Box::new(MissingConfigStatusInput::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\*\*\*Error\*\*\*: You must have glib-gettext >= (.*) installed.*",
        |m| Ok(Some(Box::new(MissingGnomeCommonDependency::new("glib-gettext".to_string(), Some(m.get(1).unwrap().as_str().to_string())))))
    ),
    regex_line_matcher!(
        r"ERROR: JAVA_HOME is set to an invalid directory: /usr/lib/jvm/default-java/",
        |_| Ok(Some(Box::new(MissingJVM)))
    ),
    regex_line_matcher!(
        r#"Error: The file "MANIFEST" is missing from this distribution\. The MANIFEST lists all files included in the distribution\."#,
        |_| Ok(Some(Box::new(MissingPerlManifest)))
    ),
    regex_line_matcher!(
        r"dh_installdocs: --link-doc not allowed between (.*) and (.*) \(one is arch:all and the other not\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh: unable to load addon systemd: dh: The systemd-sequence is no longer provided in compat >= 11, please rely on dh_installsystemd instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh: The --before option is not supported any longer \(#932537\). Use override targets instead.",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"\(.*\): undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!("(.*):([0-9]+): undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!("(.*):([0-9]+): error: undefined reference to '(.*)'", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld:(.*): multiple definition of `*.\'; (.*): first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(r".+\.go:[0-9]+: undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!(r"ar: libdeps specified more than once", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld: .*\(.*\):\(.*\): multiple definition of `*.\'; (.*):\((.*)\) first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"\/usr\/bin\/ld:(.*): multiple definition of `*.\'; (.*):\((.*)\) first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"\/usr\/bin\/ld: (.*): undefined reference to `(.*)\'", |_| Ok(None)),
    regex_line_matcher!(r"\/usr\/bin\/ld: (.*): undefined reference to symbol \'(.*)\'", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld: (.*): relocation (.*) against symbol `(.*)\' can not be used when making a shared object; recompile with -fPIC",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "(.*):([0-9]+): multiple definition of `(.*)'; (.*):([0-9]+): first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "(dh.*): debhelper compat level specified both in debian/compat and via build-dependency on debhelper-compat",
        |m| Ok(Some(Box::new(DuplicateDHCompatLevel::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "(dh.*): (error: )?Please specify the compatibility level in debian/compat",
        |m| Ok(Some(Box::new(MissingDHCompatLevel::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "dh_makeshlibs: The udeb (.*) does not contain any shared libraries but --add-udeb=(.*) was passed!?",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "dpkg-gensymbols: error: some symbols or patterns disappeared in the symbols file: see diff output below",
        |_| Ok(Some(Box::new(DisappearedSymbols)))
    ),
    regex_line_matcher!(
        r"Failed to copy \'(.*)\': No such file or directory at /usr/share/dh-exec/dh-exec-install-rename line [0-9]+.*",
        file_not_found
    ),
    regex_line_matcher!(r"Invalid gemspec in \[.*\]: No such file or directory - (.*)", command_missing),
    regex_line_matcher!(
        r".*meson.build:[0-9]+:[0-9]+: ERROR: Program\(s\) \[\'(.*)\'\] not found or not executable",
        command_missing
    ),
    regex_line_matcher!(
        r".*meson.build:[0-9]+:[0-9]: ERROR: Git program not found\.",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r"Failed: [pytest] section in setup.cfg files is no longer supported, change to [tool:pytest] instead.",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"cp: cannot stat \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(r"cp: \'(.*)\' and \'(.*)\' are the same file", |_| Ok(None)),
    regex_line_matcher!(r".?PHP Fatal error: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"sed: no input files", |_| Ok(None)),
    regex_line_matcher!(r"sed: can\'t read (.*): No such file or directory", file_not_found),
    regex_line_matcher!(
        r"ERROR in Entry module not found: Error: Can\'t resolve \'(.*)\' in \'(.*)\'",
        webpack_file_missing
    ),
    regex_line_matcher!(
        r".*:([0-9]+): element include: XInclude error : could not load (.*), and no fallback was found",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"E: Child terminated by signal ‘Terminated’",
     |_| Ok(Some(Box::new(Cancelled)))
     ),
    regex_line_matcher!(r"E: Caught signal ‘Terminated’",
     |_| Ok(Some(Box::new(Cancelled)))
     ),
    regex_line_matcher!(r"E: Failed to execute “(.*)”: No such file or directory", command_missing),
    regex_line_matcher!(r"E ImportError: Bad (.*) executable(\.?)", command_missing),
    regex_line_matcher!(r"E: The Debian version .* cannot be used as an ELPA version.", |_| Ok(None)),
    // ImageMagick
    regex_line_matcher!(
        r"convert convert: Image pixel limit exceeded \(see -limit Pixels\) \(-1\).",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"convert convert: Improper image header \(.*\).", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: invalid primitive argument \([0-9]+\).", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unexpected end-of-file \(\)\.", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unrecognized option \((.*)\)\.", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unrecognized channel type \((.*)\)\.", |_| Ok(None)),
    regex_line_matcher!(
        r"convert convert: Unable to read font \((.*)\) \[No such file or directory\].",
        file_not_found
    ),
    regex_line_matcher!(
        r"convert convert: Unable to open file (.*) \[No such file or directory\]\.",
        file_not_found
    ),
    regex_line_matcher!(
        r"convert convert: No encode delegate for this image format \((.*)\) \[No such file or directory\].",
        |m| Ok(Some(Box::new(ImageMagickDelegateMissing::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"ERROR: Sphinx requires at least Python (.*) to run.", |_| Ok(None)),
    regex_line_matcher!(r"Can\'t find (.*) directory in (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"/bin/sh: [0-9]: cannot create (.*): Directory nonexistent",
        |m|  Ok(Some(Box::new(DirectoryNonExistant(std::path::Path::new(m.get(1).unwrap().as_str()).to_path_buf().parent().unwrap().display().to_string()))))
    ),
    regex_line_matcher!(r"dh: Unknown sequence (.*) \(choose from: .*\)", |_| Ok(None)),
    regex_line_matcher!(r".*\.vala:[0-9]+\.[0-9]+-[0-9]+.[0-9]+: error: (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"error: Package `(.*)\' not found in specified Vala API directories or GObject-Introspection GIR directories",
        |m| Ok(Some(Box::new(MissingValaPackage(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r".*.scala:[0-9]+: error: (.*)", |_| Ok(None)),
    // JavaScript
    regex_line_matcher!(r"error TS6053: File \'(.*)\' not found.", file_not_found),
    // Mocha
    regex_line_matcher!(r"Error \[ERR_MODULE_NOT_FOUND\]: Cannot find package '(.*)' imported from (.*)", |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"\s*Uncaught Error \[ERR_MODULE_NOT_FOUND\]: Cannot find package '(.*)' imported from (.*)",
    |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"(.*\.ts)\([0-9]+,[0-9]+\): error TS[0-9]+: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"(.*.nim)\([0-9]+, [0-9]+\) Error: .*", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_installinit: upstart jobs are no longer supported\!  Please remove (.*) and check if you need to add a conffile removal",
        |m| Ok(Some(Box::new(UpstartFilePresent(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"dh_installinit: --no-restart-on-upgrade has been renamed to --no-stop-on-upgrade",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"find: paths must precede expression: .*", |_| Ok(None)),
    regex_line_matcher!(r"find: ‘(.*)’: No such file or directory", file_not_found),
    regex_line_matcher!(r"ninja: fatal: posix_spawn: Argument list too long", |_| Ok(None)),
    regex_line_matcher!("ninja: fatal: chdir to '(.*)' - No such file or directory", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    // Java
    regex_line_matcher!(r"error: Source option [0-9] is no longer supported. Use [0-9] or later.", |_| Ok(None)),
    regex_line_matcher!(
        r"(dh.*|jh_build): -s/--same-arch has been removed; please use -a/--arch instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh_systemd_start: dh_systemd_start is no longer used in compat >= 11, please use dh_installsystemd instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"Trying patch (.*) at level 1 \.\.\. 0 \.\.\. 2 \.\.\. failure.", |_| Ok(None)),
    // QMake
    regex_line_matcher!(r"Project ERROR: (.*) development package not found", pkg_config_missing),
    regex_line_matcher!(r"Package \'(.*)\', required by \'(.*)\', not found\n", pkg_config_missing),
    regex_line_matcher!(r"pkg-config cannot find (.*)", pkg_config_missing),
    regex_line_matcher!(
        r"configure: error: .* not found: Package dependency requirement \'([^\']+)\' could not be satisfied.",
        pkg_config_missing
    ),
    regex_line_matcher!(
        r"configure: error: (.*) is required to build documentation",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r".*:[0-9]+: (.*) does not exist.", file_not_found),
    // uglifyjs
    regex_line_matcher!(r"ERROR: can\'t read file: (.*)", file_not_found),
    regex_line_matcher!(r#"jh_build: Cannot find \(any matches for\) "(.*)" \(tried in .*\)"#, |_| Ok(None)),
    regex_line_matcher!(
        r"--   Package \'(.*)\', required by \'(.*)\', not found",
        |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r".*.rb:[0-9]+:in `require_relative\': cannot load such file -- (.*) \(LoadError\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"<internal:.*>:[0-9]+:in `require': cannot load such file -- (.*) \(LoadError\)",
        |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r".*.rb:[0-9]+:in `require\': cannot load such file -- (.*) \(LoadError\)",
        |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"LoadError: cannot load such file -- (.*)", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"  cannot load such file -- (.*)", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    // TODO(jelmer): This is a fairly generic string; perhaps combine with other checks for ruby?
    regex_line_matcher!(r"File does not exist: ([a-z/]+)$", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r".*:[0-9]+:in `do_check_dependencies\': E: dependency resolution check requested but no working gemspec available \(RuntimeError\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"rm: cannot remove \'(.*)\': Is a directory", |_| Ok(None)),
    regex_line_matcher!(r"rm: cannot remove \'(.*)\': No such file or directory", file_not_found),
    // Invalid option from Python
    regex_line_matcher!(r"error: option .* not recognized", |_| Ok(None)),
    // Invalid option from go
    regex_line_matcher!(r"flag provided but not defined: .*", |_| Ok(None)),
    regex_line_matcher!(r#"CMake Error: The source directory "(.*)" does not exist."#, |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*: [0-9]+: cd: can\'t cd to (.*)", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/bin/sh: 0: Can\'t open (.*)", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"/bin/sh: [0-9]+: cannot open (.*): No such file", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r".*: line [0-9]+: (.*): No such file or directory", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"/bin/sh: [0-9]+: Syntax error: .*", |_| Ok(None)),
    regex_line_matcher!(r"error: No member named \$memberName", |_| Ok(None)),
    regex_line_matcher!(
        r"(?:/usr/bin/)?install: cannot create regular file \'(.*)\': Permission denied",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"(?:/usr/bin/)?install: cannot create directory .(.*).: File exists", |_| Ok(None)),
    regex_line_matcher!(r"/usr/bin/install: missing destination file operand after .*", |_| Ok(None)),
    // Ruby
    regex_line_matcher!(r"rspec .*\.rb:[0-9]+ # (.*)", |_| Ok(None)),
    // help2man
    regex_line_matcher!(r"Addendum (.*) does NOT apply to (.*) \(translation discarded\).", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_installchangelogs: copy\((.*), (.*)\): No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(r"dh_installman: mv (.*) (.*): No such file or directory", file_not_found),
    regex_line_matcher!(r"dh_installman: Could not determine section for (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"failed to initialize build cache at (.*): mkdir (.*): permission denied",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at (.*) line ([0-9]+)."#,
        command_missing
    ),
    regex_line_matcher!(
        r#"E OSError: No command "(.*)" found on host .*"#,
        command_missing
    ),
    // PHPUnit
    regex_line_matcher!(r#"Cannot open file "(.*)"."#, file_not_found),
    regex_line_matcher!(
        r".*Could not find a JavaScript runtime\. See https://github.com/rails/execjs for a list of available runtimes\..*",
        |_| Ok(Some(Box::new(MissingJavaScriptRuntime)))
    ),
    Box::new(PythonFileNotFoundErrorMatcher),
    // ruby
    regex_line_matcher!(r"Errno::ENOENT: No such file or directory - (.*)", file_not_found),
    regex_line_matcher!(r"(.*.rb):[0-9]+:in `.*\': .* \(.*\) ", |_| Ok(None)),
    // JavaScript
    regex_line_matcher!(r".*: ENOENT: no such file or directory, open \'(.*)\'", file_not_found),
    regex_line_matcher!(r"\[Error: ENOENT: no such file or directory, stat \'(.*)\'\] \{", file_not_found),
    regex_line_matcher!(
        r"(.*):[0-9]+: error: Libtool library used but \'LIBTOOL\' is undefined",
        |_| Ok(Some(Box::new(MissingLibtool)))
    ),
    // libtoolize
    regex_line_matcher!(r"libtoolize:   error: \'(.*)\' does not exist.", file_not_found),
    // Seen in python-cogent
    regex_line_matcher!(
        "(OSError|RuntimeError): (.*) required but not found.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"RuntimeError: The (.*) executable cannot be found\. Please check if it is in the system path\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_lowercase()))))
    ),
    regex_line_matcher!(
        r".*: [0-9]+: cannot open (.*): No such file",
        file_not_found
    ),
    regex_line_matcher!(
        r"Cannot find Git. Git is required for .*",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r"E ImportError: Bad (.*) executable\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "RuntimeError: (.*) is missing",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): Could not find (.*) library\..*",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): We need package (.*), but not importable",
        |m| Ok(Some(Box::new(MissingPythonDistribution{ distribution: m.get(2).unwrap().as_str().to_string(), minimum_version: None, python_version: None })))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): No (.*) was found: .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),

    regex_line_matcher!(
        r"(.*)meson.build:[0-9]+:[0-9]+: ERROR: Meson version is (.+) but project requires >=\s*(.+)",
        |m| Ok(Some(Box::new(MissingVagueDependency{
            name: "meson".to_string(), url: None,
            minimum_version: Some(m.get(3).unwrap().as_str().trim_end_matches('.').to_string()),
            current_version: Some(m.get(2).unwrap().as_str().to_string())}
        )))
    ),

    // Seen in cpl-plugin-giraf
    regex_line_matcher!(
        r"ImportError: Numpy version (.*) or later must be installed to use .*",
        |m| Ok(Some(Box::new(MissingPythonModule{ module: "numpy".to_string(), python_version: None, minimum_version: Some(m.get(1).unwrap().as_str().to_string())})))
    ),
    // Seen in mayavi2
    regex_line_matcher!(r"\w+Numpy is required to build.*", |_| Ok(Some(Box::new(MissingPythonModule::simple("numpy".to_string()))))),
    // autoconf
    regex_line_matcher!(r"configure.ac:[0-9]+: error: required file \'(.*)\' not found", file_not_found),
    regex_line_matcher!(r"/usr/bin/m4:(.*):([0-9]+): cannot open `(.*)\': No such file or directory", |m| Ok(Some(Box::new(MissingFile{path: std::path::PathBuf::from(m.get(3).unwrap().as_str().to_string())})))),
    // automake
    regex_line_matcher!(r"Makefile.am: error: required file \'(.*)\' not found", file_not_found),
    // sphinx
    regex_line_matcher!(r"config directory doesn\'t contain a conf.py file \((.*)\)", |_| Ok(None)),
    // vcversioner
    regex_line_matcher!(
        r"vcversioner: no VCS could be detected in \'/<<PKGBUILDDIR>>\' and \'/<<PKGBUILDDIR>>/version.txt\' isn\'t present.",
        |_| Ok(None)
    ),
    // rst2html (and other Python?)
    regex_line_matcher!(r"  InputError: \[Errno 2\] No such file or directory: \'(.*)\'", file_not_found),
    // gpg
    regex_line_matcher!(r"gpg: can\'t connect to the agent: File name too long", |_| Ok(None)),
    regex_line_matcher!(r"(.*.lua):[0-9]+: assertion failed", |_| Ok(None)),
    regex_line_matcher!(r"\s+\^\-\-\-\-\^ SC[0-4][0-9][0-9][0-9]: .*", |_| Ok(None)),
    regex_line_matcher!(
        r"Error: (.*) needs updating from (.*)\. Run \'pg_buildext updatecontrol\'.",
        |m| Ok(Some(Box::new(NeedPgBuildExtUpdateControl::new(m.get(1).unwrap().as_str().to_string(), m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Patch (.*) does not apply \(enforce with -f\)", |m| Ok(Some(Box::new(PatchApplicationFailed::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"java.io.FileNotFoundException: ([^ ]+) \(No such file or directory\)",
        file_not_found
    ),
    // Pytest
    regex_line_matcher!(r"INTERNALERROR> PluginValidationError: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"[0-9]+ out of [0-9]+ hunks FAILED -- saving rejects to file (.*\.rej)", |_| Ok(None)),
    regex_line_matcher!(r"pkg_resources.UnknownExtra: (.*) has no such extra feature \'(.*)\'", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_auto_configure: invalid or non-existing path to the source directory: .*",
        |_| Ok(None)
    ),
    // Sphinx
    regex_line_matcher!(
        r"(.*) is no longer a hard dependency since version (.*). Please install it manually.\(pip install (.*)\)",
        |m| Ok(Some(Box::new(MissingPythonModule::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"There is a syntax error in your configuration file: (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"E: The Debian version (.*) cannot be used as an ELPA version.",
        |m| Ok(Some(Box::new(DebianVersionRejected::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r#""(.*)" is not exported by the ExtUtils::MakeMaker module"#, |_| Ok(None)),
    regex_line_matcher!(
        r"E: Please add appropriate interpreter package to Build-Depends, see pybuild\(1\) for details\..*",
        |_| Ok(Some(Box::new(DhAddonLoadFailure::new("pybuild".to_string(), "Debian/Debhelper/Buildsystem/pybuild.pm".to_string()))))
    ),
    regex_line_matcher!(r"dpkg: error: .*: No space left on device", |_| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(
        r"You need the GNU readline library\(ftp://ftp.gnu.org/gnu/readline/\s+\) to build",
        |_| Ok(Some(Box::new(MissingLibrary("readline".to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: Could not find lib(.*)",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"    Could not find module ‘(.*)’",
        |m| Ok(Some(Box::new(MissingHaskellModule::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"E: session: (.*): Chroot not found", |m| Ok(Some(Box::new(ChrootNotFound::new(m.get(1).unwrap().as_str().to_string()))))),
    Box::new(HaskellMissingDependencyMatcher),
    Box::new(SetupPyCommandMissingMatcher),
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
    for (offset, line) in lines.enumerate_tail_forward(start_offset) {
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

#[derive(Debug, Clone)]
pub struct CMakeFilesMissing {
    pub filenames: Vec<String>,
    pub version: Option<String>,
}

impl Problem for CMakeFilesMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-cmake-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filenames": self.filenames,
            "version": self.version,
        })
    }
}

impl std::fmt::Display for CMakeFilesMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CMake files missing: {:?}", self.filenames)
    }
}

/// Find the key failure line in build output.
///
/// # Returns
/// A tuple with (match object, error object)
pub fn find_build_failure_description(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    pub const OFFSET: usize = 250;
    // Is this cmake-specific, or rather just kf5 / qmake ?
    let mut cmake = false;
    // We search backwards for clear errors.
    for (lineno, line) in lines.enumerate_backward(Some(250)) {
        if line.contains("cmake") {
            cmake = true;
        }
        if let Some((mm, merr)) = match_lines(lines.as_slice(), lineno).unwrap() {
            return (Some(mm), merr);
        }
    }

    // TODO(jelmer): Remove this in favour of CMakeErrorMatcher above.
    if cmake {
        // Urgh, multi-line regexes---
        for (mut lineno, line) in lines.enumerate_forward(None) {
            let line = line.trim_end_matches('\n');
            if let Some((_, target)) =
                lazy_regex::regex_captures!(r"  Could NOT find (.*) \(missing: .*\)", line)
            {
                return (
                    Some(Box::new(SingleLineMatch::from_lines(
                        &lines,
                        lineno,
                        Some("direct regex"),
                    )) as Box<dyn Match>),
                    Some(Box::new(MissingCommand(target.to_lowercase())) as Box<dyn Problem>),
                );
            }
            if let Some((_, _target)) = lazy_regex::regex_captures!(
                r#"\s*The imported target "(.*)" references the file"#,
                line
            ) {
                lineno += 1;
                while lineno < lines.len() && !line.is_empty() {
                    lineno += 1;
                }
                if lines[lineno + 2].starts_with("  but this file does not exist.") {
                    let filename = if let Some((_, entry)) =
                        lazy_regex::regex_captures!(r#"\s*"(.*)""#, line)
                    {
                        entry
                    } else {
                        line
                    };
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            &lines,
                            lineno,
                            Some("direct regex"),
                        )) as Box<dyn Match>),
                        Some(Box::new(MissingFile {
                            path: filename.into(),
                        }) as Box<dyn Problem>),
                    );
                }
                continue;
            }
            if lineno + 1 < lines.len() {
                if let Some((_, _pkg)) = lazy_regex::regex_captures!("^  Could not find a package configuration file provided by \"(.*)\" with any of the following names:", &(line.to_string() + " " + lines[lineno + 1].trim_start_matches(' ').trim_end_matches('\n'))) {
                    if lines[lineno + 2] == "\n" {
                        let mut i = 3;
                        let mut filenames = vec![];
                        while !lines[lineno + i].trim().is_empty() {
                            filenames.push(lines[lineno + i].trim().to_string());
                            i += 1;
                        }
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines, lineno, Some("direct regex (cmake)")
                            )) as Box<dyn Match>),
                            Some(Box::new(CMakeFilesMissing{filenames, version: None}) as Box<dyn Problem>),
                        )
                    }
                }
            }
        }
    }

    // And forwards for vague ("secondary") errors.
    let m = find_secondary_build_failure(lines.as_slice(), OFFSET);
    if let Some(m) = m {
        return (Some(Box::new(m)), None);
    }

    (None, None)
}
