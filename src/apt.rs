use crate::problems::common::NoSpaceOnDevice;
use crate::lines::Lines;
use crate::problems::debian::{
    AptBrokenPackages, AptFetchFailure, AptMissingReleaseFile, AptPackageUnknown, DpkgError,
};
use crate::{Match, MultiLineMatch, Problem, SingleLineMatch};
use debian_control::relations::Relations;
use serde::{Deserialize, Serialize};

pub fn find_apt_get_failure(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    let mut ret: (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) = (None, None);
    for (lineno, line) in lines.enumerate_backward(Some(50)) {
        let line = line.trim_end_matches('\n');
        if line.starts_with("E: Failed to fetch ") {
            if let Some((_, pkg, msg)) =
                lazy_regex::regex_captures!("^E: Failed to fetch ([^ ]+)  (.*)", line)
            {
                let problem: Box<dyn Problem> = if msg.contains("No space left on device") {
                    Box::new(NoSpaceOnDevice)
                } else {
                    Box::new(AptFetchFailure {
                        url: Some(pkg.to_string()),
                        error: msg.to_string(),
                    })
                };
                return (
                    Some(Box::new(SingleLineMatch::from_lines(
                        &lines,
                        lineno,
                        Some("direct regex"),
                    )) as Box<dyn Match>),
                    Some(problem),
                );
            }
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                None,
            );
        }
        if line == "E: Broken packages" {
            let error = Some(Box::new(AptBrokenPackages {
                description: lines[lineno - 1].trim().to_string(),
                broken: None,
            }) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno - 1,
                    Some("direct match"),
                )) as Box<dyn Match>),
                error,
            );
        }
        if line == "E: Unable to correct problems, you have held broken packages." {
            let mut offsets = vec![];
            let mut broken = vec![];
            for j in (0..(lineno - 1)).rev() {
                if let Some((_, pkg, _)) = lazy_regex::regex_captures!(
                    r"\s*Depends: (.*) but it is not (going to be installed|installable)",
                    lines[j]
                ) {
                    offsets.push(j);
                    broken.push(pkg.to_string());
                    continue;
                }
                if let Some((_, _, pkg, _)) = lazy_regex::regex_captures!(
                    r"\s*(.*) : Depends: (.*) but it is not (going to be installed|installable)",
                    lines[j]
                ) {
                    offsets.push(j);
                    broken.push(pkg.to_string());
                    continue;
                }
                break;
            }
            let error = Some(Box::new(AptBrokenPackages {
                description: lines[lineno].trim().to_string(),
                broken: Some(broken),
            }) as Box<dyn Problem>);
            offsets.push(lineno);
            let r#match = Some(Box::new(MultiLineMatch::from_lines(
                &lines,
                offsets,
                Some("direct match"),
            )) as Box<dyn Match>);
            return (r#match, error);
        }
        if let Some((_, repo)) = lazy_regex::regex_captures!(
            "E: The repository '([^']+)' does not have a Release file.",
            line
        ) {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(AptMissingReleaseFile(repo.to_string()))),
            );
        }
        if let Some((_, _path)) = lazy_regex::regex_captures!(
            "dpkg-deb: error: unable to write file '(.*)': No space left on device",
            line
        ) {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(NoSpaceOnDevice)),
            );
        }
        if let Some((_, _path)) =
            lazy_regex::regex_captures!(r"E: You don't have enough free space in (.*)\.", line)
        {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(NoSpaceOnDevice)),
            );
        }
        if line.starts_with("E: ") && ret.0.is_none() {
            ret = (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                None,
            );
        }
        if let Some((_, pkg)) =
            lazy_regex::regex_captures!(r"E: Unable to locate package (.*)", line)
        {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(AptPackageUnknown(pkg.to_string()))),
            );
        }
        if line == "E: Write error - write (28: No space left on device)" {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(NoSpaceOnDevice)),
            );
        }
        if let Some((_, msg)) = lazy_regex::regex_captures!(r"dpkg: error: (.*)", line) {
            if msg.ends_with(": No space left on device") {
                return (
                    Some(Box::new(SingleLineMatch::from_lines(
                        &lines,
                        lineno,
                        Some("direct regex"),
                    )) as Box<dyn Match>),
                    Some(Box::new(NoSpaceOnDevice)),
                );
            }
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(DpkgError(msg.to_string()))),
            );
        }
        if let Some((_, pkg, msg)) =
            lazy_regex::regex_captures!(r"dpkg: error processing package (.*) \((.*)\):", line)
        {
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    lineno + 1,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(Box::new(DpkgError(format!(
                    "processing package {} ({})",
                    pkg, msg
                )))),
            );
        }
    }

    for (i, line) in lines.enumerate_forward(None) {
        if lazy_regex::regex_is_match!(
            r" cannot copy extracted data for '(.*)' to '(.*)': failed to write \(No space left on device\)",
            line,
        ) {
            return (
                Some(
                    Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                        as Box<dyn Match>,
                ),
                Some(Box::new(NoSpaceOnDevice)),
            );
        }
        if lazy_regex::regex_is_match!(r" .*: No space left on device", line) {
            return (
                Some(
                    Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                        as Box<dyn Match>,
                ),
                Some(Box::new(NoSpaceOnDevice)),
            );
        }
    }
    ret
}

pub fn find_apt_get_update_failure(
    sbuildlog: &crate::sbuild::SbuildLog,
) -> (
    Option<String>,
    Option<Box<dyn Match>>,
    Option<Box<dyn Problem>>,
) {
    let focus_section = "update chroot";
    let lines = sbuildlog.get_section_lines(Some(focus_section));
    let (match_, problem) = find_apt_get_failure(lines.unwrap());
    (Some(focus_section.to_string()), match_, problem)
}

pub fn find_cudf_output(lines: Vec<&str>) -> Option<serde_yaml::Value> {
    let mut offset = None;
    for (i, line) in lines.enumerate_backward(None) {
        if line.starts_with("cudf output:") {
            offset = Some(i);
        }
    }
    let mut offset = offset?;
    let mut output = vec![];
    while !lines[offset].trim().is_empty() {
        output.push(lines[offset]);
        offset += 1;
    }

    serde_yaml::from_str(&output.join("\n")).ok()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dose3Report {
    pub package: String,
    pub status: String,
    pub reasons: Vec<Dose3Reason>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dose3Reason {
    pub missing: Option<Dose3Missing>,
    pub conflict: Option<Dose3Conflict>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dose3Pkg {
    pub name: String,
    pub version: String,
    #[serde(rename = "unsat-dependency")]
    pub unsat_dependency: Option<String>,
    #[serde(rename = "unsat-conflict")]
    pub unsat_conflict: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dose3Missing {
    pub pkg: Dose3Pkg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dose3Conflict {
    pub pkg1: Dose3Pkg,
    pub pkg2: Dose3Pkg,
}

#[derive(Debug)]
pub struct UnsatisfiedAptConflicts(String);

impl Problem for UnsatisfiedAptConflicts {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unsatisfied-apt-conflicts".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "relations": self.0
        })
    }
}

impl std::fmt::Display for UnsatisfiedAptConflicts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt conflicts: {}", self.0)
    }
}

impl std::error::Error for UnsatisfiedAptConflicts {}

#[derive(Debug)]
pub struct UnsatisfiedAptDependencies(String);

impl Problem for UnsatisfiedAptDependencies {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unsatisfied-apt-dependencies".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "relations": self.0
        })
    }
}

impl std::fmt::Display for UnsatisfiedAptDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt dependencies: {}", self.0)
    }
}

pub fn error_from_dose3_report(report: serde_json::Value) -> Option<Box<dyn Problem>> {
    let entries: Vec<Dose3Report> = serde_json::from_value(report).ok()?;
    let packages = entries
        .iter()
        .map(|entry| &entry.package)
        .collect::<Vec<_>>();
    assert_eq!(packages, ["sbuild-build-depends-main-dummy"]);
    if entries[0].status != "broken" {
        return None;
    }
    let mut missing = vec![];
    let mut conflict = vec![];
    for reason in &entries[0].reasons {
        if let Some(this_missing) = &reason.missing {
            let relation: Relations = this_missing
                .pkg
                .unsat_dependency
                .clone()
                .unwrap()
                .parse()
                .unwrap();
            missing.extend(relation.entries());
        }
        if let Some(this_conflict) = &reason.conflict {
            let relation: Relations = this_conflict
                .pkg1
                .unsat_conflict
                .clone()
                .unwrap()
                .parse()
                .unwrap();
            conflict.extend(relation.entries());
        }
    }
    if !missing.is_empty() {
        let missing: Relations = missing.into();
        return Some(Box::new(UnsatisfiedAptDependencies(missing.to_string())) as Box<dyn Problem>);
    }
    if !conflict.is_empty() {
        let conflict: Relations = conflict.into();
        return Some(Box::new(UnsatisfiedAptConflicts(conflict.to_string())) as Box<dyn Problem>);
    }
    None
}
