//! Module for parsing and analyzing APT package manager logs.
//!
//! This module contains functions for detecting and diagnosing common issues
//! in APT package manager output, such as missing packages, disk space issues,
//! and dependency problems.

use crate::lines::Lines;
use crate::problems::common::NoSpaceOnDevice;
use crate::problems::debian::*;
use crate::{Match, MultiLineMatch, Problem, SingleLineMatch};
use debian_control::lossless::relations::{Entry, Relations};

/// Type alias for APT failure result
pub type AptFailureResult = (Option<Box<dyn Match>>, Option<Box<dyn Problem>>);

/// Type alias for APT dependency failure result
pub type AptDependencyResult = (
    Option<String>,
    Option<Box<dyn Match>>,
    Option<Box<dyn Problem>>,
);

/// Analyzes APT output to identify failures and their causes.
///
/// This function scans APT output for common error patterns and returns the matching line(s)
/// along with a problem description.
///
/// # Arguments
/// * `lines` - Vector of lines from an APT log
///
/// # Returns
/// A tuple containing:
/// * An optional match with the location of the error
/// * An optional problem description
pub fn find_apt_get_failure(lines: Vec<&str>) -> AptFailureResult {
    let mut ret: AptFailureResult = (None, None);
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

/// Analyzes APT update failure in an sbuild log.
///
/// This function extracts the "update chroot" section from an sbuild log
/// and analyzes it for APT failures.
///
/// # Arguments
/// * `sbuildlog` - The sbuild log to analyze
///
/// # Returns
/// A tuple containing:
/// * An optional section name where the failure was found
/// * An optional match with the location of the error
/// * An optional problem description
pub fn find_apt_get_update_failure(sbuildlog: &crate::sbuild::SbuildLog) -> AptDependencyResult {
    let focus_section = "update chroot";
    let lines = sbuildlog.get_section_lines(Some(focus_section));
    let (match_, problem) = find_apt_get_failure(lines.unwrap());
    (Some(focus_section.to_string()), match_, problem)
}

/// Finds and parses CUDF output in a log.
///
/// This function searches for and extracts CUDF (Common Upgradeability Description Format)
/// output from a log file.
///
/// # Arguments
/// * `lines` - Vector of lines to search in
///
/// # Returns
/// An optional tuple containing:
/// * A vector of line offsets where the CUDF output was found
/// * The parsed CUDF data
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

/// Extracts error information from DOSE3 reports.
///
/// This function analyzes DOSE3 reports to identify dependency problems
/// and conflicts.
///
/// # Arguments
/// * `reports` - Slice of CUDF reports from DOSE3
///
/// # Returns
/// An optional problem description extracted from the reports
pub(crate) fn error_from_dose3_reports(
    reports: &[crate::cudf::Report],
) -> Option<Box<dyn Problem>> {
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
    fn test_no_space_on_device() {
        assert_match(
            vec![
                "E: Failed to fetch http://apt.example.com/pool/main/h/hello/hello_2.10.orig.tar.gz  No space left on device"
            ],
            1,
            Some(NoSpaceOnDevice {}),
        );
    }

    #[test]
    fn test_dpkg_no_space_on_device() {
        assert_match(
            vec![
                "dpkg-deb: error: unable to write file '/var/cache/apt/archives/hello_2.10-2_amd64.deb': No space left on device"
            ],
            1,
            Some(NoSpaceOnDevice {}),
        );
    }

    #[test]
    fn test_apt_no_space_error() {
        assert_match(
            vec!["E: You don't have enough free space in /var."],
            1,
            Some(NoSpaceOnDevice {}),
        );
    }

    #[test]
    fn test_write_error_no_space() {
        assert_match(
            vec!["E: Write error - write (28: No space left on device)"],
            1,
            Some(NoSpaceOnDevice {}),
        );
    }

    #[test]
    fn test_dpkg_error_no_space() {
        assert_match(
            vec!["dpkg: error: writing to '/var/lib/dpkg/status': No space left on device"],
            1,
            Some(NoSpaceOnDevice {}),
        );
    }

    #[test]
    fn test_dpkg_error_general() {
        assert_match(
            vec!["dpkg: error: some other error occurred"],
            1,
            Some(DpkgError("some other error occurred".to_string())),
        );
    }

    #[test]
    fn test_dpkg_error_processing_package_direct() {
        let lines = vec![
            "dpkg: error processing package hello (--configure):",
            "subprocess installed post-installation script returned error exit status 1",
        ];

        let (match_result, problem) = find_apt_get_failure(lines);

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            let dpkg_error = problem.as_any().downcast_ref::<DpkgError>();
            assert!(dpkg_error.is_some());
            let dpkg_error = dpkg_error.unwrap();
            assert_eq!(dpkg_error.0, "processing package hello (--configure)");
        }
    }

    // Direct test of functions without using helper functions
    #[test]
    fn test_broken_packages_direct() {
        let lines = vec![
            "The following packages have unmet dependencies:",
            "E: Broken packages",
        ];

        let (match_result, problem) = find_apt_get_failure(lines);

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            let broken_packages = problem.as_any().downcast_ref::<AptBrokenPackages>();
            assert!(broken_packages.is_some());
            let broken_packages = broken_packages.unwrap();
            assert_eq!(
                broken_packages.description,
                "The following packages have unmet dependencies:"
            );
            assert!(broken_packages.broken.is_none());
        }
    }

    #[test]
    fn test_unable_to_locate_package_direct() {
        let lines = vec!["E: Unable to locate package nonexistent-package"];

        let (match_result, problem) = find_apt_get_failure(lines);

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            let pkg_unknown = problem.as_any().downcast_ref::<AptPackageUnknown>();
            assert!(pkg_unknown.is_some());
            let pkg_unknown = pkg_unknown.unwrap();
            assert_eq!(pkg_unknown.0, "nonexistent-package");
        }
    }

    #[test]
    fn test_copy_extracted_data_no_space_direct() {
        let lines = vec![
            "some text before",
            " cannot copy extracted data for '/var/cache/apt/archives/hello_2.10-2_amd64.deb' to '/tmp/hello': failed to write (No space left on device)",
            "some text after"
        ];

        let (match_result, problem) = find_apt_get_failure(lines);

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            assert!(problem.as_any().is::<NoSpaceOnDevice>());
        }
    }

    #[test]
    fn test_generic_no_space_error_direct() {
        let lines = vec![
            "some text before",
            " /var/cache/apt/archives/hello_2.10-2_amd64.deb: No space left on device",
            "some text after",
        ];

        let (match_result, problem) = find_apt_get_failure(lines);

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            assert!(problem.as_any().is::<NoSpaceOnDevice>());
        }
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

    #[test]
    fn test_error_from_dose3_reports() {
        use crate::cudf::*;

        // Test missing dependencies case
        let missing_reports = vec![Report {
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
                        unsat_dependency: Some("libfoo (>= 1.0)".to_string()),
                    },
                }),
                conflict: None,
            }],
        }];

        let problem = error_from_dose3_reports(&missing_reports);
        assert!(problem.is_some());
        let problem = problem.unwrap();
        assert!(problem.as_any().is::<UnsatisfiedAptDependencies>());

        // Test conflict case
        let conflict_reports = vec![Report {
            package: "sbuild-build-depends-main-dummy".to_string(),
            version: "0.invalid.0".parse().unwrap(),
            architecture: "amd64".to_string(),
            status: Status::Broken,
            reasons: vec![Reason {
                missing: None,
                conflict: Some(Conflict {
                    pkg1: Pkg {
                        package: "sbuild-build-depends-main-dummy".to_string(),
                        version: "0.invalid.0".parse().unwrap(),
                        architecture: "amd64".to_string(),
                        unsat_conflict: Some("libbar (>= 2.0)".to_string()),
                        unsat_dependency: None,
                    },
                    pkg2: Pkg {
                        package: "libbar".to_string(),
                        version: "2.1".parse().unwrap(),
                        architecture: "amd64".to_string(),
                        unsat_conflict: None,
                        unsat_dependency: None,
                    },
                }),
            }],
        }];

        let problem = error_from_dose3_reports(&conflict_reports);
        assert!(problem.is_some());
        let problem = problem.unwrap();
        assert!(problem.as_any().is::<UnsatisfiedAptConflicts>());

        // Test non-broken status - for this test we set empty reasons to simulate non-broken
        let ok_reports = vec![Report {
            package: "sbuild-build-depends-main-dummy".to_string(),
            version: "0.invalid.0".parse().unwrap(),
            architecture: "amd64".to_string(),
            status: Status::Broken, // The cudf.rs only has Broken enum variant
            reasons: vec![],        // Empty reasons meaning it's actually "ok" for our test
        }];

        let problem = error_from_dose3_reports(&ok_reports);
        assert!(problem.is_none());
    }
}
