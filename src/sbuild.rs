//! Parsing of Debian sbuild logs
//!
//! This module provides a parser for Debian sbuild logs. It extracts the different sections of the
//! log file, and makes them accessible.

use crate::common::find_build_failure_description;
use crate::lines::Lines;
use crate::problems::common::{NoSpaceOnDevice, PatchApplicationFailed};
use crate::problems::debian::*;
use crate::{Match, Problem, SingleLineMatch};
use debversion::Version;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;
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

pub fn find_preamble_failure_description(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    let mut ret: (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) = (None, None);
    for (lineno, line) in lines.enumerate_backward(Some(100)) {
        let line = line.trim_end_matches('\n');
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
                            &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }

        if let Some((_, _path)) =
            lazy_regex::regex_captures!("E: Bad version unknown in (.*)", line)
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
                            &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
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
            let (_, p) = find_build_failure_description(vec![msg]);
            let p = p.unwrap_or_else(|| {
                Box::new(SourceFormatUnsupported(format.to_string())) as Box<dyn Problem>
            });
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
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
                    &lines,
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
                    &lines,
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
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                err,
            );
        }
    }

    ret
}

pub const DEFAULT_LOOK_BACK: usize = 50;

pub fn strip_build_tail<'a>(
    lines: &'a [&'a str],
    look_back: Option<usize>,
) -> (&'a [&'a str], HashMap<&'a str, &'a [&'a str]>) {
    let look_back = look_back.unwrap_or(DEFAULT_LOOK_BACK);

    let mut interesting_lines: &'a [&'a str] = lines;

    // Strip off unuseful tail
    for (i, line) in lines.enumerate_tail_forward(look_back) {
        if line.starts_with("Build finished at ") {
            interesting_lines = &lines[..i];
            if let Some(last_line) = interesting_lines.last() {
                if last_line == &("-".repeat(80)) {
                    interesting_lines = &interesting_lines[..interesting_lines.len() - 1];
                }
            }
            break;
        }
    }

    let mut files: HashMap<&'a str, &'a [&'a str]> = std::collections::HashMap::new();
    let mut body = interesting_lines;
    let mut current_file = None;
    let mut current_contents: &[&str] = &[];
    let mut start = 0;

    for (i, line) in interesting_lines.iter().enumerate() {
        if let Some((_, header)) = lazy_regex::regex_captures!(r"==> (.*) <==", line) {
            if let Some(current_file) = current_file {
                files.insert(current_file, current_contents);
            } else {
                body = current_contents;
            }
            current_file = Some(header);
            current_contents = &[];
            start = i + 1;
            continue;
        } else {
            current_contents = &interesting_lines[start..i + 1];
        }
    }
    if let Some(current_file) = current_file {
        files.insert(current_file, current_contents);
    } else {
        body = current_contents;
    }

    (body, files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sbuild_log() {
        let log = include_str!("testdata/sbuild.0.log");
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

    #[test]
    fn test_strip_build_tail() {
        assert_eq!(
            strip_build_tail(
                &[
                    "Build finished at 2023-09-16T16:47:58Z",
                    "--------------------------------------------------------------------------------",
                    "Finished at 2023-09-16T16:47:58Z",
                    "Build needed 00:01:12, 41428k disk space",
                ],
                None
            ),
            (
                &[
                ][..],
                HashMap::new()
            )
        );
        let meson_log_lines = r#"Build started at 2022-07-21T04:21:47.088879
Main binary: /usr/bin/python3
Build Options: -Dprefix=/usr -Dlibdir=lib/x86_64-linux-gnu -Dlocalstatedir=/var -Dsysconfdir=/etc -Dbuildtype=plain -Dwrap_mode=nodownload
Python system: Linux
The Meson build system
Version: 0.56.2
Source dir: /<<PKGBUILDDIR>>
Build dir: /<<PKGBUILDDIR>>/obj-x86_64-linux-gnu
Build type: native build

../meson.build:1:0: ERROR: Meson version is 0.56.2 but project requires >= 0.57.0
dh_auto_configure: error: cd obj-x86_64-linux-gnu && LC_ALL=C.UTF-8 meson .. --wrap-mode=nodownload --buildtype=plain --prefix=/usr --sysconfdir=/etc --localstatedir=/var --libdir=lib/x86_64-linux-gnu returned exit code 1
make: *** [debian/rules:13: binary] Error 25
dpkg-buildpackage: error: debian/rules binary subprocess returned exit status 2
"#.lines().collect::<Vec<_>>();
        assert_eq!(
            strip_build_tail(
                include_str!("testdata/sbuild.meson.log")
                    .lines()
                    .collect::<Vec<_>>()
                    .as_slice(),
                None
            ),
            (
                r#" --sysconfdir=/etc --localstatedir=/var --libdir=lib/x86_64-linux-gnu
The Meson build system
Version: 0.56.2
Source dir: /<<PKGBUILDDIR>>
Build dir: /<<PKGBUILDDIR>>/obj-x86_64-linux-gnu
Build type: native build

../meson.build:1:0: ERROR: Meson version is 0.56.2 but project requires >= 0.57.0

A full log can be found at /<<PKGBUILDDIR>>/obj-x86_64-linux-gnu/meson-logs/meson-log.txt
cd obj-x86_64-linux-gnu && tail -v -n \+0 meson-logs/meson-log.txt
"#
                .lines()
                .collect::<Vec<_>>()
                .as_slice(),
                maplit::hashmap! {
                        "meson-logs/meson-log.txt" => meson_log_lines.as_slice(),
                }
            )
        );
    }
}
