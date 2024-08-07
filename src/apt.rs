use crate::common::NoSpaceOnDevice;
use crate::{Match, MultiLineMatch, Problem, SingleLineMatch};

#[derive(Debug)]
pub struct DpkgError(pub String);

impl Problem for DpkgError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "msg": self.0,
        })
    }
}

impl std::fmt::Display for DpkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dpkg error: {}", self.0)
    }
}

pub struct AptUpdateError;

impl Problem for AptUpdateError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-update-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for AptUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt update error")
    }
}

pub struct AptFetchFailure {
    pub url: Option<String>,
    pub error: String,
}

impl Problem for AptFetchFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-fetch-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "error": self.error,
        })
    }
}

impl std::fmt::Display for AptFetchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(url) = &self.url {
            write!(f, "apt fetch failure: {} ({})", url, self.error)
        } else {
            write!(f, "apt fetch failure: {}", self.error)
        }
    }
}

pub struct AptMissingReleaseFile(pub String);

impl Problem for AptMissingReleaseFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-missing-release-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.0,
        })
    }
}

impl std::fmt::Display for AptMissingReleaseFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt missing release file: {}", self.0)
    }
}

pub struct AptPackageUnknown(pub String);

impl Problem for AptPackageUnknown {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-package-unknown".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.0,
        })
    }
}

impl std::fmt::Display for AptPackageUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt package unknown: {}", self.0)
    }
}

pub struct AptBrokenPackages {
    pub description: String,
    pub broken: Option<Vec<String>>,
}

impl Problem for AptBrokenPackages {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-broken-packages".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "description": self.description,
            "broken": self.broken,
        })
    }
}

impl std::fmt::Display for AptBrokenPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt broken packages: {}", self.description)
    }
}

pub fn find_apt_get_failure(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    let mut ret: (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) = (None, None);
    const OFFSET: usize = 50;
    for i in 1..OFFSET {
        if i >= lines.len() {
            break;
        }
        let lineno = lines.len() - i;
        let line = lines[lineno].trim_end_matches('\n');
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

    for (i, line) in lines.iter().enumerate() {
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
    for i in (0..(lines.len() - 1)).rev() {
        if lines[i].starts_with("cudf output:") {
            offset = Some(i);
        }
    }
    let mut offset = if let Some(offset) = offset {
        offset
    } else {
        return None;
    };
    let mut output = vec![];
    while !lines[offset].trim().is_empty() {
        output.push(lines[offset]);
        offset += 1;
    }

    serde_yaml::from_str(&output.join("\n")).ok()
}
