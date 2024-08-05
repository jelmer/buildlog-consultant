//! Parsing of Debian sbuild logs
//!
//! This module provides a parser for Debian sbuild logs. It extracts the different sections of the
//! log file, and makes them accessible.

use crate::common::{find_build_failure_description, NoSpaceOnDevice, PatchApplicationFailed};
use crate::{Match, Problem, SingleLineMatch};
use debversion::Version;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;
use std::str::FromStr;
use std::time::Duration;

pub fn find_failed_stage<'a>(lines: &'a [&'a str]) -> Option<&'a str> {
    for line in lines {
        if let Some(value) = line.strip_prefix("Fail-Stage: ") {
            return Some(value.trim());
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    build_architecture: Option<String>,
    build_type: Option<String>,
    build_time: Option<Duration>,
    build_space: Option<u64>,
    host_architecture: Option<String>,
    install_time: Option<Duration>,
    lintian: Option<String>,
    package: Option<String>,
    package_time: Option<Duration>,
    distribution: Option<String>,
    fail_stage: Option<String>,
    job: Option<String>,
    autopkgtest: Option<String>,
    source_version: Option<Version>,
    machine_architecture: Option<String>,
    status: Option<String>,
    space: Option<u64>,
}

pub fn parse_summary(lines: &[&str]) -> Summary {
    let mut build_architecture = None;
    let mut build_type = None;
    let mut build_time = None;
    let mut build_space = None;
    let mut host_architecture = None;
    let mut install_time = None;
    let mut lintian = None;
    let mut package = None;
    let mut distribution = None;
    let mut job = None;
    let mut autopkgtest = None;
    let mut status = None;
    let mut package_time = None;
    let mut source_version = None;
    let mut machine_architecture = None;
    let mut fail_stage = None;
    let mut space = None;
    for line in lines {
        if line.trim() == "" {
            continue;
        }
        if let Some((key, value)) = line.trim_end().split_once(": ") {
            let value = value.trim();
            match key {
                "Fail-Stage" => fail_stage = Some(value.to_string()),
                "Build Architecture" => build_architecture = Some(value.to_string()),
                "Build Type" => build_type = Some(value.to_string()),
                "Build-Time" => build_time = Some(Duration::from_secs(value.parse().unwrap())),
                "Build-Space" => build_space = Some(value.parse().unwrap()),
                "Host Architecture" => host_architecture = Some(value.to_string()),
                "Install-Time" => install_time = Some(Duration::from_secs(value.parse().unwrap())),
                "Lintian" => lintian = Some(value.to_string()),
                "Package" => package = Some(value.to_string()),
                "Package-Time" => package_time = Some(Duration::from_secs(value.parse().unwrap())),
                "Source-Version" => source_version = Some(value.parse().unwrap()),
                "Job" => job = Some(value.parse().unwrap()),
                "Machine Architecture" => machine_architecture = Some(value.to_string()),
                "Distribution" => distribution = Some(value.to_string()),
                "Autopkgtest" => autopkgtest = Some(value.to_string()),
                "Status" => status = Some(value.to_string()),
                "Space" => space = Some(value.parse().unwrap()),
                n => {
                    log::warn!("Unknown key in summary: {}", n);
                }
            }
        } else {
            log::warn!("Unknown line in summary: {}", line);
        }
    }
    Summary {
        build_architecture,
        build_type,
        build_time,
        build_space,
        host_architecture,
        install_time,
        lintian,
        package,
        package_time,
        distribution,
        fail_stage,
        job,
        autopkgtest,
        source_version,
        machine_architecture,
        status,
        space,
    }
}

#[derive(Debug, Clone)]
pub struct SbuildLog(pub Vec<SbuildLogSection>);

impl SbuildLog {
    /// Get the first section with the given title, if it exists.
    pub fn get_section(&self, title: Option<&str>) -> Option<&SbuildLogSection> {
        self.0.iter().find(|s| s.title.as_deref() == title)
    }

    /// Get the lines of a section, if it exists.
    pub fn get_section_lines(&self, title: Option<&str>) -> Option<Vec<&str>> {
        self.get_section(title)
            .map(|s| s.lines.iter().map(|l| l.as_str()).collect::<Vec<_>>())
    }

    /// Get the titles of sections
    pub fn section_titles(&self) -> Vec<&str> {
        self.0.iter().filter_map(|s| s.title.as_deref()).collect()
    }

    /// Get the failed stage, if it is provided
    pub fn get_failed_stage(&self) -> Option<String> {
        if let Some(summary) = self.summary() {
            summary.fail_stage
        } else {
            None
        }
    }

    /// Iterate ove the sections
    pub fn sections(&self) -> impl Iterator<Item = &SbuildLogSection> {
        self.0.iter()
    }

    pub fn summary(&self) -> Option<Summary> {
        let lines = self.get_section_lines(Some("Summary"));
        lines.map(|lines| parse_summary(lines.as_slice()))
    }
}

impl TryFrom<File> for SbuildLog {
    type Error = std::io::Error;

    fn try_from(f: File) -> Result<Self, Self::Error> {
        let reader = BufReader::new(f);
        let sections = parse_sbuild_log(reader);
        Ok(SbuildLog(sections.collect()))
    }
}

impl FromStr for SbuildLog {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());
        let sections = parse_sbuild_log(reader);
        Ok(SbuildLog(sections.collect()))
    }
}

#[derive(Debug, Clone)]
pub struct SbuildLogSection {
    pub title: Option<String>,
    pub offsets: (usize, usize),
    pub lines: Vec<String>,
}

pub fn parse_sbuild_log<R: BufRead>(mut reader: R) -> impl Iterator<Item = SbuildLogSection> {
    let mut begin_offset = 1;
    let mut lines = Vec::new();
    let mut title: Option<String> = None;

    // Separator line (78 '-' characters, bookended by '+').
    let sep = "+".to_string() + &"-".repeat(78) + "+";
    let mut lineno = 0;

    // We'll store our sections in this Vec and return it as an iterator at the end.
    let mut sections = Vec::new();

    loop {
        let mut line = String::new();

        // Read a line from the file. Break if EOF.
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }

        lineno += 1;

        // Trim trailing whitespace and newline characters.
        let line_trimmed = line.trim().to_string();

        if line_trimmed == sep {
            // Read next two lines
            let mut l1 = String::new();
            let mut l2 = String::new();

            reader.read_line(&mut l1).unwrap();
            reader.read_line(&mut l2).unwrap();

            lineno += 2;

            // Trim trailing whitespace and newline characters.
            let l1_trimmed = l1.trim();
            let l2_trimmed = l2.trim();

            if l1_trimmed.starts_with('|') && l1_trimmed.ends_with('|') && l2_trimmed == sep {
                let mut end_offset = lineno - 3;

                // Drop trailing empty lines
                while lines.last() == Some(&"\n".to_string()) {
                    lines.pop();
                    end_offset -= 1;
                }

                if !lines.is_empty() {
                    // The unwrap_or_else is to provide a default value in case 'title' is None.
                    sections.push(SbuildLogSection {
                        title: title.clone(),
                        offsets: (begin_offset, end_offset),
                        lines: lines.clone(),
                    });
                }

                title = Some(l1_trimmed[1..l1.len() - 2].trim().to_string());
                lines.clear();
                begin_offset = lineno;
            } else {
                lines.push(line);
                lines.push(l1);
                lines.push(l2);
            }
        } else {
            lines.push(line);
        }
    }

    // Generate the final section.
    sections.push(SbuildLogSection {
        title,
        offsets: (begin_offset, lineno),
        lines,
    });

    // Return the sections as an iterator.
    sections.into_iter()
}

pub struct SbuildFailure {
    stage: Option<String>,
    description: Option<String>,
    error: Option<Box<dyn Problem>>,
    phase: Option<Vec<String>>,
    section: Option<SbuildLogSection>,
    r#match: Option<Box<dyn Match>>,
}

impl Serialize for SbuildFailure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SbuildFailure", 6)?;
        state.serialize_field("stage", &self.stage)?;
        state.serialize_field("phase", &self.phase)?;
        state.serialize_field(
            "section",
            &self.section.as_ref().map(|s| s.title.as_deref()),
        )?;
        state.serialize_field("origin", &self.r#match.as_ref().map(|m| m.origin().0))?;
        state.serialize_field(
            "lineno",
            &self
                .section
                .as_ref()
                .map(|s| s.offsets.0 + self.r#match.as_ref().unwrap().lineno()),
        )?;
        if let Some(error) = &self.error {
            state.serialize_field("kind", &error.kind())?;
            state.serialize_field("details", &error.json())?;
        }
        state.end()
    }
}

pub struct DpkgSourceLocalChanges {
    diff_file: Option<String>,
    files: Option<Vec<String>>,
}

impl Problem for DpkgSourceLocalChanges {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unexpected-local-upstream-changes".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "diff_file": self.diff_file,
            "files": self.files,
        })
    }
}

impl std::fmt::Display for DpkgSourceLocalChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(files) = self.files.as_ref() {
            if files.len() < 5 {
                write!(f, "Tree has local changes: {:?}", files)?;
                return Ok(());
            }

            write!(f, "Tree has local changes: {} files", files.len())?;
        } else {
            write!(f, "Tree has local changes")?;
        }
        Ok(())
    }
}

pub struct DpkgSourceUnrepresentableChanges;

impl Problem for DpkgSourceUnrepresentableChanges {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unrepresentable-local-changes".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for DpkgSourceUnrepresentableChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unrepresentable changes")
    }
}

pub struct DpkgUnwantedBinaryFiles;

impl Problem for DpkgUnwantedBinaryFiles {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unwanted-binary-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for DpkgUnwantedBinaryFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unwanted binary files")
    }
}

pub struct DpkgBinaryFileChanged(Vec<String>);

impl Problem for DpkgBinaryFileChanged {
    fn kind(&self) -> std::borrow::Cow<str> {
        "binary-file-changed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "files": self.0,
        })
    }
}

impl std::fmt::Display for DpkgBinaryFileChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binary file changed")
    }
}

struct MissingControlFile(std::path::PathBuf);

impl Problem for MissingControlFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-control-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for MissingControlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing control file: {}", self.0.display())
    }
}

struct UnableToFindUpstreamTarball {
    package: String,
    version: Version,
}

impl Problem for UnableToFindUpstreamTarball {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unable-to-find-upstream-tarball".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "version": self.version.to_string(),
        })
    }
}

impl std::fmt::Display for UnableToFindUpstreamTarball {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to find upstream tarball for {} {}",
            self.package, self.version
        )
    }
}

pub struct SourceFormatUnbuildable {
    source_format: String,
    reason: String,
}

impl Problem for SourceFormatUnbuildable {
    fn kind(&self) -> std::borrow::Cow<str> {
        "source-format-unbuildable".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "source_format": self.source_format,
            "reason": self.reason,
        })
    }
}

impl std::fmt::Display for SourceFormatUnbuildable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Source format {} is unbuildable: {}",
            self.source_format, self.reason
        )
    }
}

pub struct SourceFormatUnsupported(String);

impl Problem for SourceFormatUnsupported {
    fn kind(&self) -> std::borrow::Cow<str> {
        "source-format-unsupported".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "source_format": self.0,
        })
    }
}

impl std::fmt::Display for SourceFormatUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source format {} is unsupported", self.0)
    }
}

pub struct PatchFileMissing(std::path::PathBuf);

impl Problem for PatchFileMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "patch-file-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0.display().to_string(),
        })
    }
}

impl std::fmt::Display for PatchFileMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Patch file missing: {}", self.0.display())
    }
}

struct UnknownMercurialExtraFields(String);

impl Problem for UnknownMercurialExtraFields {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unknown-mercurial-extra-fields".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "field": self.0,
        })
    }
}

impl std::fmt::Display for UnknownMercurialExtraFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown Mercurial extra field: {}", self.0)
    }
}

pub struct UpstreamPGPSignatureVerificationFailed;

impl Problem for UpstreamPGPSignatureVerificationFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "upstream-pgp-signature-verification-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for UpstreamPGPSignatureVerificationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream PGP signature verification failed")
    }
}

pub struct UScanRequestVersionMissing(String);

impl Problem for UScanRequestVersionMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-request-version-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.0,
        })
    }
}

impl std::fmt::Display for UScanRequestVersionMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan request version missing: {}", self.0)
    }
}

pub struct DebcargoFailure(String);

impl Problem for DebcargoFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }
}

impl std::fmt::Display for DebcargoFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debcargo failure: {}", self.0)
    }
}

pub struct ChangelogParseError(String);

impl Problem for ChangelogParseError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "changelog-parse-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }
}

impl std::fmt::Display for ChangelogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Changelog parse error: {}", self.0)
    }
}

pub struct UScanFailed {
    url: String,
    reason: String,
}

impl Problem for UScanFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "reason": self.reason,
        })
    }
}

impl std::fmt::Display for UScanFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan failed: {}", self.reason)
    }
}

pub struct InconsistentSourceFormat {
    version: Option<String>,
    source_format: Option<String>,
}

impl Problem for InconsistentSourceFormat {
    fn kind(&self) -> std::borrow::Cow<str> {
        "inconsistent-source-format".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version.as_ref().map(|v| v.to_string()),
            "source_format": self.source_format,
        })
    }
}

impl std::fmt::Display for InconsistentSourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inconsistent source format between version and source format"
        )
    }
}

pub struct UpstreamMetadataFileParseError {
    path: std::path::PathBuf,
    reason: String,
}

impl Problem for UpstreamMetadataFileParseError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debian-upstream-metadata-invalid".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path.display().to_string(),
            "reason": self.reason,
        })
    }
}

impl std::fmt::Display for UpstreamMetadataFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream metadata file parse error: {}", self.reason)
    }
}

pub struct DpkgSourcePackFailed(String);

impl Problem for DpkgSourcePackFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-source-pack-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }
}

impl std::fmt::Display for DpkgSourcePackFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dpkg source pack failed: {}", self.0)
    }
}

pub struct DpkgBadVersion {
    version: String,
    reason: Option<String>,
}

impl Problem for DpkgBadVersion {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-bad-version".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "reason": self.reason,
        })
    }
}

impl std::fmt::Display for DpkgBadVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(reason) = &self.reason {
            write!(f, "Version {} is invalid: {}", self.version, reason)
        } else {
            write!(f, "Version {} is invalid", self.version)
        }
    }
}

pub struct MissingDebcargoCrate {
    cratename: String,
    version: Option<String>,
}

impl Problem for MissingDebcargoCrate {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-missing-crate".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.cratename,
            "version": self.version,
        })
    }
}

impl std::fmt::Display for MissingDebcargoCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "Missing debcargo crate: {}={}", self.cratename, version)
        } else {
            write!(f, "Missing debcargo crate: {}", self.cratename)
        }
    }
}

impl MissingDebcargoCrate {
    pub fn from_string(text: &str) -> Self {
        let text = text.trim();
        if let Some((cratename, version)) = text.split_once('=') {
            Self {
                cratename: cratename.trim().to_string(),
                version: Some(version.trim().to_string()),
            }
        } else {
            Self {
                cratename: text.to_string(),
                version: None,
            }
        }
    }
}

pub struct PristineTarTreeMissing(String);

impl Problem for PristineTarTreeMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "pristine-tar-missing-tree".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "treeish": self.0,
        })
    }
}

impl std::fmt::Display for PristineTarTreeMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pristine-tar tree missing: {}", self.0)
    }
}

pub struct MissingRevision(Vec<u8>);

impl Problem for MissingRevision {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-revision".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "revision": String::from_utf8_lossy(&self.0),
        })
    }
}

impl std::fmt::Display for MissingRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing revision: {}", String::from_utf8_lossy(&self.0))
    }
}

pub fn find_preamble_failure_description(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    let mut ret: (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) = (None, None);
    const OFFSET: usize = 100;
    for i in 1..OFFSET {
        if i >= lines.len() {
            break;
        }
        let lineno = lines.len() - i;
        let line = lines[lineno].trim_end_matches('\n');
        if let Some((_, diff_file)) = lazy_regex::regex_captures!(
            "dpkg-source: error: aborting due to unexpected upstream changes, see (.*)",
            line
        ) {
            let mut j = lineno - 1;
            let mut files = vec![];
            while j > 0 {
                if lines[j]
                    == ("dpkg-source: info: local changes detected, the modified files are:\n")
                {
                    let err = Some(Box::new(DpkgSourceLocalChanges {
                        diff_file: Some(diff_file.to_string()),
                        files: Some(files),
                    }) as Box<dyn Problem>);
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            lines,
                            lineno,
                            Some("direct regex"),
                        ))),
                        err,
                    );
                }
                files.push(lines[j].trim().to_string());
                j -= 1;
            }
            let err = Some(Box::new(DpkgSourceLocalChanges {
                diff_file: Some(diff_file.to_string()),
                files: Some(files),
            }) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if line == "dpkg-source: error: unrepresentable changes to source" {
            let err = Some(Box::new(DpkgSourceUnrepresentableChanges) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct match"),
                ))),
                err,
            );
        }
        if lazy_regex::regex_is_match!(
            r"dpkg-source: error: detected ([0-9]+) unwanted binary file.*",
            line
        ) {
            let err = Some(Box::new(DpkgUnwantedBinaryFiles) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                ))),
                err,
            );
        }
        if let Some((_, path)) = lazy_regex::regex_captures!(
            "dpkg-source: error: cannot read (.*/debian/control): No such file or directory",
            line,
        ) {
            let err = Some(Box::new(MissingControlFile(path.into())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                ))),
                err,
            );
        }
        if lazy_regex::regex_is_match!("dpkg-source: error: .*: No space left on device", line) {
            let err = Some(Box::new(NoSpaceOnDevice) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if lazy_regex::regex_is_match!("tar: .*: Cannot write: No space left on device", line) {
            let err = Some(Box::new(NoSpaceOnDevice) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if let Some((_, path)) = lazy_regex::regex_captures!(
            "dpkg-source: error: cannot represent change to (.*): binary file contents changed",
            line
        ) {
            let err =
                Some(Box::new(DpkgBinaryFileChanged(vec![path.to_string()])) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }

        if let Some((_, format, _, _, _)) = lazy_regex::regex_captures!(
            r"dpkg-source: error: source package format \'(.*)\' is not supported: Can\'t locate (.*) in \@INC \(you may need to install the (.*) module\) \(\@INC contains: (.*)\) at \(eval [0-9]+\) line [0-9]+\.",
            line
        ) {
            let err =
                Some(Box::new(SourceFormatUnsupported(format.to_string())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }

        if let Some((_, reason)) =
            lazy_regex::regex_captures!("E: Failed to package source directory (.*)", line)
        {
            let err = Some(Box::new(DpkgSourcePackFailed(reason.to_string())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }

        if let Some((_, path)) = lazy_regex::regex_captures!("E: Bad version unknown in (.*)", line)
        {
            if lines[lineno - 1].starts_with("LINE: ") {
                if let Some((_, version, reason)) = lazy_regex::regex_captures!(
                    r"dpkg-parsechangelog: warning: .*\(l[0-9]+\): version \'(.*)\' is invalid: (.*)",
                    lines[lineno - 2]
                ) {
                    let err = Some(Box::new(DpkgBadVersion {
                        version: version.to_string(),
                        reason: Some(reason.to_string()),
                    }) as Box<dyn Problem>);
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            lines,
                            lineno,
                            Some("direct regex"),
                        )) as Box<dyn Match>),
                        err,
                    );
                }
            }
        }

        if let Some((_, patchname)) =
            lazy_regex::regex_captures!("Patch (.*) does not apply \\(enforce with -f\\)\n", line)
        {
            let patchname = patchname.rsplit_once('/').unwrap().1;
            let err = Some(Box::new(PatchApplicationFailed {
                patchname: patchname.to_string(),
            }) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if let Some((_, patchname)) = lazy_regex::regex_captures!(
            r"dpkg-source: error: LC_ALL=C patch .* --reject-file=- < .*\/debian\/patches\/([^ ]+) subprocess returned exit status 1",
            line
        ) {
            let err = Some(Box::new(PatchApplicationFailed {
                patchname: patchname.to_string(),
            }) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if let Some((_, source_format, reason)) = lazy_regex::regex_captures!(
            "dpkg-source: error: can't build with source format '(.*)': (.*)",
            line
        ) {
            let err = Some(Box::new(SourceFormatUnbuildable {
                source_format: source_format.to_string(),
                reason: reason.to_string(),
            }) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if let Some((_, path)) = lazy_regex::regex_captures!(
            "dpkg-source: error: cannot read (.*): No such file or directory",
            line
        ) {
            let patchname = path.rsplit_once('/').unwrap().1;
            let err = Some(Box::new(PatchFileMissing(path.into())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
        if let Some((_, format, msg)) = lazy_regex::regex_captures!(
            "dpkg-source: error: source package format '(.*)' is not supported: (.*)",
            line
        ) {
            let (unused_match, p) = find_build_failure_description(vec![msg]);
            let p = p.unwrap_or_else(|| {
                Box::new(SourceFormatUnsupported(format.to_string())) as Box<dyn Problem>
            });
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(p),
            );
        }
        if let Some((_, _, revid)) = lazy_regex::regex_captures!(
            "breezy.errors.NoSuchRevision: (.*) has no revision b'(.*)'",
            line
        ) {
            let err =
                Some(Box::new(MissingRevision(revid.as_bytes().to_vec())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                ))),
                err,
            );
        }

        if let Some((_, arg)) = lazy_regex::regex_captures!(
            r"fatal: ambiguous argument \'(.*)\': unknown revision or path not in the working tree.",
            line
        ) {
            let err = Some(Box::new(PristineTarTreeMissing(arg.to_string())) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }

        if let Some((_, msg)) = lazy_regex::regex_captures!("dpkg-source: error: (.*)", line) {
            let err = Some(Box::new(DpkgSourcePackFailed(msg.to_string())) as Box<dyn Problem>);
            ret = (
                Some(Box::new(SingleLineMatch::from_lines(
                    lines.clone(),
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sbuild_log() {
        let log = r###"\
sbuild (Debian sbuild) 0.85.2 (11 March 2023) on charis.vpn.jelmer.uk

+==============================================================================+
| rust-always-assert 0.1.3-1 (amd64)           Sat, 16 Sep 2023 16:46:46 +0000 |
+==============================================================================+

Package: rust-always-assert
Version: 0.1.3-1
Source Version: 0.1.3-1
Distribution: unstable
Machine Architecture: amd64
Host Architecture: amd64
Build Architecture: amd64
Build Type: binary

Unpacking /home/jelmer/.cache/sbuild/debcargo-unstable-amd64-sbuild.tar.xz to /dev/shm/tmp.sbuild.3lAQxFDuCZ...
I: NOTICE: Log filtering will replace 'sbuild-unshare-dummy-location' with '<<CHROOT>>'
I: NOTICE: Log filtering will replace 'build/rust-always-assert-SPcsaf/resolver-Vb7vZ2' with '<<RESOLVERDIR>>'

+------------------------------------------------------------------------------+
| Update chroot                                                                |
+------------------------------------------------------------------------------+

...

+------------------------------------------------------------------------------+
| Fetch source files                                                           |
+------------------------------------------------------------------------------+


Local sources
-------------

...
Install main build dependencies (apt-based resolver)
----------------------------------------------------
...

+------------------------------------------------------------------------------+
| Check architectures                                                          |
+------------------------------------------------------------------------------+

Arch check ok (amd64 included in any)

+------------------------------------------------------------------------------+
| Build environment                                                            |
+------------------------------------------------------------------------------+

Kernel: Linux 6.4.0-4-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.4.13-1 (2023-08-31) amd64 (x86_64)
Toolchain package versions: binutils_2.41-5 dpkg-dev_1.22.0 g++-12_12.3.0-9 g++-13_13.2.0-4 gcc-12_12.3.0-9 gcc-13_13.2.0-4 libc6-dev_2.37-10 libstdc++-12-dev_12.3.0-9 libstdc++-13-dev_13.2.0-4 libstdc++6_13.2.0-4 linux-libc-dev_6.5.3-1
Package versions: adduser_3.137 apt_2.7.5 autoconf_2.71-3 automake_1:1.16.5-1.3 autopoint_0.21-13 autotools-dev_20220109.1 base-files_13 base-passwd_3.6.1 bash_5.2.15-2+b5 binutils_2.41-5 binutils-common_2.41-5 binutils-x86-64-linux-gnu_2.41-5 bsdextrautils_2.39.2-1 bsdutils_1:2.39.2-1 build-essential_12.10 bzip2_1.0.8-5+b1 ca-certificates_20230311 cargo_0.66.0+ds1-1 ccache_4.8.3-1 coreutils_9.1-1 cpp_4:13.2.0-1 cpp-12_12.3.0-9 cpp-13_13.2.0-4 dash_0.5.12-6 debconf_1.5.82 debhelper_13.11.6 debian-archive-keyring_2023.4 debianutils_5.12 dh-autoreconf_20 dh-cargo_30 dh-strip-nondeterminism_1.13.1-1 diffstat_1.65-1 diffutils_1:3.8-4 dirmngr_2.2.40-1.1 dpkg_1.22.0 dpkg-dev_1.22.0 dwz_0.15-1 e2fsprogs_1.47.0-2+b1 eatmydata_131-1 fakeroot_1.32.1-1 file_1:5.45-2 findutils_4.9.0-5 g++_4:13.2.0-1 g++-12_12.3.0-9 g++-13_13.2.0-4 gcc_4:13.2.0-1 gcc-12_12.3.0-9 gcc-12-base_12.3.0-9 gcc-13_13.2.0-4 gcc-13-base_13.2.0-4 gettext_0.21-13+b1 gettext-base_0.21-13+b1 gnupg_2.2.40-1.1 gnupg-l10n_2.2.40-1.1 gnupg-utils_2.2.40-1.1 gpg_2.2.40-1.1 gpg-agent_2.2.40-1.1 gpg-wks-client_2.2.40-1.1 gpg-wks-server_2.2.40-1.1 gpgconf_2.2.40-1.1 gpgsm_2.2.40-1.1 gpgv_2.2.40-1.1 grep_3.11-3 groff-base_1.23.0-2 gzip_1.12-1 hostname_3.23+nmu1 init-system-helpers_1.65.2 intltool-debian_0.35.0+20060710.6 iso-codes_4.15.0-1 libacl1_2.3.1-3 libaliased-perl_0.34-3 libapt-pkg-perl_0.1.40+b2 libapt-pkg6.0_2.7.5 libarchive-zip-perl_1.68-1 libasan8_13.2.0-4 libassuan0_2.5.6-1 libatomic1_13.2.0-4 libattr1_1:2.5.1-4 libaudit-common_1:3.1.1-1 libaudit1_1:3.1.1-1 libb-hooks-endofscope-perl_0.26-1 libb-hooks-op-check-perl_0.22-2+b1 libberkeleydb-perl_0.64-2+b1 libbinutils_2.41-5 libblkid1_2.39.2-1 libbrotli1_1.0.9-2+b6 libbsd0_0.11.7-4 libbz2-1.0_1.0.8-5+b1 libc-bin_2.37-10 libc-dev-bin_2.37-10 libc6_2.37-10 libc6-dev_2.37-10 libcap-ng0_0.8.3-1+b3 libcap2_1:2.66-4 libcapture-tiny-perl_0.48-2 libcc1-0_13.2.0-4 libcgi-pm-perl_4.57-1 libclass-data-inheritable-perl_0.08-3 libclass-method-modifiers-perl_2.15-1 libclass-xsaccessor-perl_1.19-4+b1 libclone-perl_0.46-1 libcom-err2_1.47.0-2+b1 libconfig-tiny-perl_2.29-1 libconst-fast-perl_0.014-2 libcpanel-json-xs-perl_4.37-1 libcrypt-dev_1:4.4.36-2 libcrypt1_1:4.4.36-2 libctf-nobfd0_2.41-5 libctf0_2.41-5 libcurl3-gnutls_8.3.0-1 libdata-dpath-perl_0.58-2 libdata-messagepack-perl_1.02-1+b1 libdata-optlist-perl_0.114-1 libdata-validate-domain-perl_0.10-1.1 libdata-validate-ip-perl_0.31-1 libdata-validate-uri-perl_0.07-2 libdb5.3_5.3.28+dfsg2-2 libdebconfclient0_0.270 libdebhelper-perl_13.11.6 libdevel-callchecker-perl_0.008-2 libdevel-size-perl_0.83-2+b1 libdevel-stacktrace-perl_2.0400-2 libdpkg-perl_1.22.0 libdynaloader-functions-perl_0.003-3 libeatmydata1_131-1 libedit2_3.1-20230828-1 libelf1_0.189-4 libemail-address-xs-perl_1.05-1+b1 libencode-locale-perl_1.05-3 libexception-class-perl_1.45-1 libexpat1_2.5.0-2 libext2fs2_1.47.0-2+b1 libfakeroot_1.32.1-1 libffi8_3.4.4-1 libfile-basedir-perl_0.09-2 libfile-find-rule-perl_0.34-3 libfile-listing-perl_6.15-1 libfile-stripnondeterminism-perl_1.13.1-1 libfont-ttf-perl_1.06-2 libgcc-12-dev_12.3.0-9 libgcc-13-dev_13.2.0-4 libgcc-s1_13.2.0-4 libgcrypt20_1.10.2-2 libgdbm-compat4_1.23-3 libgdbm6_1.23-3 libgit2-1.5_1.5.1+ds-1 libgmp10_2:6.3.0+dfsg-2 libgnutls30_3.8.1-4+b1 libgomp1_13.2.0-4 libgpg-error0_1.47-2 libgprofng0_2.41-5 libgssapi-krb5-2_1.20.1-4 libhiredis0.14_0.14.1-4 libhogweed6_3.9.1-2 libhtml-form-perl_6.11-1 libhtml-html5-entities-perl_0.004-3 libhtml-parser-perl_3.81-1 libhtml-tagset-perl_3.20-6 libhtml-tokeparser-simple-perl_3.16-4 libhtml-tree-perl_5.07-3 libhttp-cookies-perl_6.10-1 libhttp-date-perl_6.05-2 libhttp-message-perl_6.44-2 libhttp-negotiate-perl_6.01-2 libhttp-parser2.9_2.9.4-6 libhwasan0_13.2.0-4 libicu72_72.1-3 libidn2-0_2.3.4-1+b1 libimport-into-perl_1.002005-2 libio-html-perl_1.004-3 libio-interactive-perl_1.023-2 libio-socket-ssl-perl_2.083-1 libio-string-perl_1.08-4 libipc-run3-perl_0.048-3 libipc-system-simple-perl_1.30-2 libisl23_0.26-3 libiterator-perl_0.03+ds1-2 libiterator-util-perl_0.02+ds1-2 libitm1_13.2.0-4 libjansson4_2.14-2 libjson-maybexs-perl_1.004005-1 libk5crypto3_1.20.1-4 libkeyutils1_1.6.3-2 libkrb5-3_1.20.1-4 libkrb5support0_1.20.1-4 libksba8_1.6.4-2 libldap-2.5-0_2.5.13+dfsg-5 liblist-compare-perl_0.55-2 liblist-someutils-perl_0.59-1 liblist-utilsby-perl_0.12-2 libllvm14_1:14.0.6-16 libllvm15_1:15.0.7-10 liblsan0_13.2.0-4 liblwp-mediatypes-perl_6.04-2 liblwp-protocol-https-perl_6.11-1 liblz1_1.13-6 liblz4-1_1.9.4-1 liblzma5_5.4.4-0.1 liblzo2-2_2.10-2 libmagic-mgc_1:5.45-2 libmagic1_1:5.45-2 libmarkdown2_2.2.7-2 libmbedcrypto7_2.28.4-1 libmbedtls14_2.28.4-1 libmbedx509-1_2.28.4-1 libmd0_1.1.0-1 libmldbm-perl_2.05-4 libmodule-implementation-perl_0.09-2 libmodule-runtime-perl_0.016-2 libmoo-perl_2.005005-1 libmoox-aliases-perl_0.001006-2 libmount1_2.39.2-1 libmouse-perl_2.5.10-1+b3 libmpc3_1.3.1-1 libmpfr6_4.2.1-1 libnamespace-clean-perl_0.27-2 libncursesw6_6.4+20230625-2 libnet-domain-tld-perl_1.75-3 libnet-http-perl_6.23-1 libnet-ipv6addr-perl_1.02-1 libnet-netmask-perl_2.0002-2 libnet-ssleay-perl_1.92-2+b1 libnetaddr-ip-perl_4.079+dfsg-2+b1 libnettle8_3.9.1-2 libnghttp2-14_1.56.0-1 libnpth0_1.6-3 libnsl-dev_1.3.0-2 libnsl2_1.3.0-2 libnumber-compare-perl_0.03-3 libp11-kit0_0.25.0-4 libpackage-stash-perl_0.40-1 libpam-modules_1.5.2-7 libpam-modules-bin_1.5.2-7 libpam-runtime_1.5.2-7 libpam0g_1.5.2-7 libparams-classify-perl_0.015-2+b1 libparams-util-perl_1.102-2+b1 libpath-tiny-perl_0.144-1 libpcre2-8-0_10.42-4 libperl5.36_5.36.0-9 libperlio-gzip-perl_0.20-1+b1 libperlio-utf8-strict-perl_0.010-1 libpipeline1_1.5.7-1 libproc-processtable-perl_0.636-1 libpsl5_0.21.2-1+b1 libpython3-stdlib_3.11.4-5+b1 libpython3.11-minimal_3.11.5-3 libpython3.11-stdlib_3.11.5-3 libquadmath0_13.2.0-4 libreadline8_8.2-1.3 libregexp-ipv6-perl_0.03-3 libregexp-wildcards-perl_1.05-3 librole-tiny-perl_2.002004-1 librtmp1_2.4+20151223.gitfa8646d.1-2+b2 libsasl2-2_2.1.28+dfsg1-3 libsasl2-modules-db_2.1.28+dfsg1-3 libseccomp2_2.5.4-1+b3 libselinux1_3.5-1 libsemanage-common_3.5-1 libsemanage2_3.5-1 libsepol2_3.5-1 libsereal-decoder-perl_5.004+ds-1 libsereal-encoder-perl_5.004+ds-1 libsframe1_2.41-5 libsmartcols1_2.39.2-1 libsort-versions-perl_1.62-3 libsqlite3-0_3.43.1-1 libss2_1.47.0-2+b1 libssh2-1_1.11.0-2 libssl3_3.0.10-1 libstd-rust-1.63_1.63.0+dfsg1-2 libstd-rust-1.69_1.69.0+dfsg1-1 libstd-rust-dev_1.69.0+dfsg1-1 libstdc++-12-dev_12.3.0-9 libstdc++-13-dev_13.2.0-4 libstdc++6_13.2.0-4 libstrictures-perl_2.000006-1 libsub-exporter-perl_0.990-1 libsub-exporter-progressive-perl_0.001013-3 libsub-identify-perl_0.14-3 libsub-install-perl_0.929-1 libsub-name-perl_0.27-1 libsub-override-perl_0.09-4 libsub-quote-perl_2.006008-1 libsyntax-keyword-try-perl_0.29-1 libsystemd0_254.3-1 libtasn1-6_4.19.0-3 libterm-readkey-perl_2.38-2+b1 libtext-glob-perl_0.11-3 libtext-levenshteinxs-perl_0.03-5+b1 libtext-markdown-discount-perl_0.16-1 libtext-xslate-perl_3.5.9-1+b2 libtime-duration-perl_1.21-2 libtime-moment-perl_0.44-2+b1 libtimedate-perl_2.3300-2 libtinfo6_6.4+20230625-2 libtirpc-common_1.3.3+ds-1 libtirpc-dev_1.3.3+ds-1 libtirpc3_1.3.3+ds-1 libtool_2.4.7-7 libtry-tiny-perl_0.31-2 libtsan2_13.2.0-4 libubsan1_13.2.0-4 libuchardet0_0.0.7-1 libudev1_254.3-1 libunicode-utf8-perl_0.62-2 libunistring2_1.0-2 libunistring5_1.1-2 liburi-perl_5.21-1 libuuid1_2.39.2-1 libvariable-magic-perl_0.63-1+b1 libwww-mechanize-perl_2.17-1 libwww-perl_6.72-1 libwww-robotrules-perl_6.02-1 libxml-libxml-perl_2.0207+dfsg+really+2.0134-1+b1 libxml-namespacesupport-perl_1.12-2 libxml-sax-base-perl_1.09-3 libxml-sax-perl_1.02+dfsg-3 libxml2_2.9.14+dfsg-1.3 libxs-parse-keyword-perl_0.38-1 libxxhash0_0.8.2-2 libyaml-0-2_0.2.5-1 libyaml-libyaml-perl_0.86+ds-1 libz3-4_4.8.12-3.1 libzstd1_1.5.5+dfsg2-1 lintian_2.116.3 linux-libc-dev_6.5.3-1 login_1:4.13+dfsg1-1+b1 logsave_1.47.0-2+b1 lzop_1.04-2 m4_1.4.19-4 make_4.3-4.1 man-db_2.11.2-3 mawk_1.3.4.20230808-1 media-types_10.1.0 mount_2.39.2-1 ncurses-base_6.4+20230625-2 ncurses-bin_6.4+20230625-2 netbase_6.4 openssl_3.0.10-1 passwd_1:4.13+dfsg1-1+b1 patch_2.7.6-7 patchutils_0.4.2-1 perl_5.36.0-9 perl-base_5.36.0-9 perl-modules-5.36_5.36.0-9 perl-openssl-defaults_7+b1 pinentry-curses_1.2.1-1 plzip_1.10-6 po-debconf_1.0.21+nmu1 python3_3.11.4-5+b1 python3-minimal_3.11.4-5+b1 python3.11_3.11.5-3 python3.11-minimal_3.11.5-3 readline-common_8.2-1.3 rpcsvc-proto_1.4.3-1 rustc_1.69.0+dfsg1-1 sbuild-build-depends-main-dummy_0.invalid.0 sed_4.9-1 sensible-utils_0.0.20 sysvinit-utils_3.07-1 t1utils_1.41-4 tar_1.34+dfsg-1.2 tzdata_2023c-10 ucf_3.0043+nmu1 unzip_6.0-28 usrmerge_37 util-linux_2.39.2-1 util-linux-extra_2.39.2-1 xz-utils_5.4.4-0.1 zlib1g_1:1.2.13.dfsg-3

+------------------------------------------------------------------------------+
| Cleanup                                                                      |
+------------------------------------------------------------------------------+

Purging /<<BUILDDIR>>
Not cleaning session: cloned chroot in use

+------------------------------------------------------------------------------+
| Summary                                                                      |
+------------------------------------------------------------------------------+

Autopkgtest: pass
Build Architecture: amd64
Build Type: binary
Build-Space: 41428
Build-Time: 3
Distribution: unstable
Host Architecture: amd64
Install-Time: 4
Job: /home/jelmer/src/debcargo-conf/build/rust-always-assert_0.1.3-1.dsc
Lintian: warn
Machine Architecture: amd64
Package: rust-always-assert
Package-Time: 72
Source-Version: 0.1.3-1
Space: 41428
Status: successful
Version: 0.1.3-1
--------------------------------------------------------------------------------
Finished at 2023-09-16T16:47:58Z
Build needed 00:01:12, 41428k disk space
"###;
        let sbuild_log: SbuildLog = log.parse().unwrap();
        assert_eq!(
            sbuild_log.section_titles(),
            vec![
                "Update chroot",
                "Fetch source files",
                "Check architectures",
                "Build environment",
                "Cleanup",
                "Summary"
            ]
        );
        assert_eq!(sbuild_log.get_failed_stage(), None);
        assert_eq!(
            sbuild_log.summary().unwrap(),
            Summary {
                fail_stage: None,
                autopkgtest: Some("pass".to_string()),
                build_architecture: Some("amd64".to_string()),
                build_type: Some("binary".to_string()),
                build_space: Some(41428),
                build_time: Some(Duration::from_secs(3)),
                distribution: Some("unstable".to_string()),
                host_architecture: Some("amd64".to_string()),
                install_time: Some(Duration::from_secs(4)),
                job: Some(
                    "/home/jelmer/src/debcargo-conf/build/rust-always-assert_0.1.3-1.dsc"
                        .to_string()
                ),
                lintian: Some("warn".to_string()),
                machine_architecture: Some("amd64".to_string()),
                package: Some("rust-always-assert".to_string()),
                package_time: Some(Duration::from_secs(72)),
                source_version: Some("0.1.3-1".parse().unwrap()),
                space: Some(41428),
                status: Some("successful".to_string()),
            }
        );
    }
}
