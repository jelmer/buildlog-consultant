use crate::lines::Lines;
use crate::problems::common::NoSpaceOnDevice;
use crate::problems::debian::*;
use crate::{Match, MultiLineMatch, Problem, SingleLineMatch};
use debian_control::lossless::relations::{Entry,Relations};

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

pub(crate) fn find_cudf_output(lines: Vec<&str>) -> Option<(Vec<usize>, crate::cudf::Cudf)> {
    let mut offset = None;
    for (i, line) in lines.enumerate_backward(None) {
        if line.starts_with("output-version:") {
            offset = Some(i);
        }
    }
    let mut offset = offset?;
    let mut output = vec![];
    let mut offsets = vec![];
    while !lines[offset].trim().is_empty() {
        offsets.push(offset);
        output.push(lines[offset]);
        offset += 1;
    }

    Some((offsets, serde_yaml::from_str(&output.join("\n")).unwrap()))
}

pub(crate) fn error_from_dose3_reports(reports: &[crate::cudf::Report]) -> Option<Box<dyn Problem>> {
    let packages = reports
        .iter()
        .map(|report| &report.package)
        .collect::<Vec<_>>();
    assert_eq!(packages, ["sbuild-build-depends-main-dummy"]);
    if reports[0].status != crate::cudf::Status::Broken {
        return None;
    }
    let mut missing = vec![];
    let mut conflict = vec![];
    for reason in &reports[0].reasons {
        if let Some(this_missing) = &reason.missing {
            let relation: Entry = this_missing
                .pkg
                .unsat_dependency
                .as_ref()
                .unwrap()
                .parse()
                .unwrap();
            missing.push(relation);
        }
        if let Some(this_conflict) = &reason.conflict {
            let relation: Relations = this_conflict
                .pkg1
                .unsat_conflict
                .as_ref()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_just_match(lines: Vec<&str>, lineno: usize) {
        let (r#match, actual_err) = super::find_apt_get_failure(lines.clone());
        assert!(actual_err.is_none());
        if let Some(r#match) = r#match.as_ref() {
            assert_eq!(&r#match.line(), &lines[lineno - 1]);
            assert_eq!(lineno, r#match.lineno());
        } else {
            assert!(r#match.is_none());
        }
    }

    fn assert_match(lines: Vec<&str>, lineno: usize, mut expected: Option<impl Problem + 'static>) {
        let (r#match, actual_err) = super::find_apt_get_failure(lines.clone());
        if let Some(r#match) = r#match.as_ref() {
            assert_eq!(&r#match.line(), &lines[lineno - 1]);
            assert_eq!(lineno, r#match.lineno());
        } else {
            assert!(r#match.is_none());
        }
        if let Some(expected) = expected.take() {
            assert!(
                r#match.is_some(),
                "err ({:?}) provided but match missing",
                &expected
            );
            assert_eq!(
                actual_err.as_ref().map(|x| x.as_ref()),
                Some(&expected as &dyn Problem)
            );
        } else {
            assert!(actual_err.is_none());
        }
    }

    #[test]
    fn test_make_missing_rule() {
        assert_match(
            vec![
                "E: Failed to fetch http://janitor.debian.net/blah/Packages.xz  File has unexpected size (3385796 != 3385720). Mirror sync in progress? [IP]"
            ],
            1,
            Some(AptFetchFailure{
                url: Some("http://janitor.debian.net/blah/Packages.xz".to_owned()),
                error: "File has unexpected size (3385796 != 3385720). Mirror sync in progress? [IP]".to_owned(),
            }),
        );
    }

    #[test]
    fn test_missing_release_file() {
        assert_match(
            vec![
                "E: The repository 'https://janitor.debian.net/ blah/ Release' does not have a Release file.",
            ],
            1,
            Some(AptMissingReleaseFile("https://janitor.debian.net/ blah/ Release".to_owned()))
        );
    }

    #[test]
    fn test_vague() {
        assert_just_match(vec!["E: Stuff is broken"], 1);
    }

    #[test]
    fn test_find_cudf_output() {
        use crate::cudf::*;
        let lines = include_str!("testdata/sbuild-cudf.log")
            .split_inclusive('\n')
            .collect::<Vec<_>>();
        let (offsets, report) = find_cudf_output(lines).unwrap();
        assert_eq!(offsets, (104..=119).collect::<Vec<_>>());
        let expected = Cudf {
            output_version: (1, 2),
            native_architecture: "amd64".to_string(),
            report: vec![Report {
                package: "sbuild-build-depends-main-dummy".to_string(),
                version: "0.invalid.0".parse().unwrap(),
                architecture: "amd64".to_string(),
                status: Status::Broken,
                reasons: vec![Reason {
                    missing: Some(Missing {
                        pkg: Pkg {
                            package: "sbuild-build-depends-main-dummy".to_string(),
                            version: "0.invalid.0".parse().unwrap(),
                            architecture: "amd64".to_string(),
                            unsat_conflict: None,
                            unsat_dependency: Some(
                                "librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-~~)"
                                    .to_string(),
                            ),
                        },
                    }),
                    conflict: None,
                }],
            }],
        };

        assert_eq!(report, expected);
    }
}
