//! Module for parsing and analyzing APT package manager logs.
//!
//! This module contains functions for detecting and diagnosing common issues
//! in APT package manager output, such as missing packages, disk space issues,
//! and dependency problems.

use crate::lines::Lines;
use crate::problems::common::NoSpaceOnDevice;
use crate::problems::debian::*;
use crate::r#match::{extract_from_line, Error, RegexLineMatcher};
use crate::regex_line_matcher;
use crate::sbuild::SectionFailure;
use crate::{Failure, MultiLineMatch, Problem};
use debian_control::lossless::relations::{Entry, Relations};

lazy_static::lazy_static! {
    static ref APT_ERROR_MATCHERS: [Box<RegexLineMatcher>; 3] = [
        regex_line_matcher!(
            "E: The repository '([^']+)' does not have a Release file.",
            |c| Ok(Some(Box::new(AptMissingReleaseFile {
                url: c[1].to_string(),
            })))
        ),
        regex_line_matcher!(
            "dpkg-deb: error: unable to write file '(.*)': No space left on device",
            |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
        regex_line_matcher!(
            r"E: You don't have enough free space in (.*)\.",
            |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
    ];
    static ref NO_SPACE_MATCHERS: [Box<RegexLineMatcher>; 2] = [
        regex_line_matcher!(
            r" cannot copy extracted data for '(.*)' to '(.*)': failed to write \(No space left on device\)",
            |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
        regex_line_matcher!(
            r" .*: No space left on device",
            |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
    ];
}

fn match_line(matchers: &[Box<RegexLineMatcher>], line: &str) -> Option<Option<Box<dyn Problem>>> {
    extract_from_line(matchers, line).expect("apt matchers do not fail")
}

/// Analyzes APT output to identify failures and their causes.
///
/// This function scans APT output for common error patterns and returns the matching line(s)
/// along with a problem description.
///
/// # Arguments
/// * `lines` - Lines from an APT log
///
/// # Returns
/// The failure, if one was found
pub fn find_apt_get_failure(lines: &[&str]) -> Option<Failure> {
    let mut ret = None;
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
                return Some(Failure::at_line(
                    lines,
                    lineno,
                    "direct regex",
                    Some(problem),
                ));
            }
            return Some(Failure::at_line(lines, lineno, "direct regex", None));
        }
        if line == "E: Broken packages" {
            // The preceding line describes the problem; fall back to this one
            // if there is none.
            let offset = lineno.saturating_sub(1);
            let error = Some(Box::new(AptBrokenPackages {
                description: lines[offset].trim().to_string(),
                broken: None,
            }) as Box<dyn Problem>);
            return Some(Failure::at_line(lines, offset, "direct match", error));
        }
        if line == "E: Unable to correct problems, you have held broken packages." {
            let mut offsets = vec![];
            let mut broken = vec![];
            for j in (0..lineno).rev() {
                if let Some((_, pkg, _)) = lazy_regex::regex_captures!(
                    r"^\s*Depends: (.*) but it is not (going to be installed|installable)",
                    lines[j]
                ) {
                    offsets.push(j);
                    broken.push(pkg.to_string());
                    continue;
                }
                if let Some((_, _, pkg, _)) = lazy_regex::regex_captures!(
                    r"^\s*(.*) : Depends: (.*) but it is not (going to be installed|installable)",
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
            // Keep offsets ascending, as highlight_lines expects.
            offsets.reverse();
            offsets.push(lineno);
            return Some(Failure::new(
                MultiLineMatch::from_lines(lines, offsets, Some("direct match")),
                error,
            ));
        }
        if let Some(problem) = match_line(&APT_ERROR_MATCHERS[..], line) {
            return Some(Failure::at_line(lines, lineno, "direct regex", problem));
        }
        if line.starts_with("E: ") && ret.is_none() {
            ret = Some(Failure::at_line(lines, lineno, "direct regex", None));
        }
        if let Some((_, pkg)) =
            lazy_regex::regex_captures!(r"E: Unable to locate package (.*)", line)
        {
            return Some(Failure::at_line(
                lines,
                lineno,
                "direct regex",
                Some(Box::new(AptPackageUnknown {
                    package: pkg.to_string(),
                })),
            ));
        }
        if line == "E: Write error - write (28: No space left on device)" {
            return Some(Failure::at_line(
                lines,
                lineno,
                "direct regex",
                Some(Box::new(NoSpaceOnDevice)),
            ));
        }
        if let Some((_, msg)) = lazy_regex::regex_captures!(r"dpkg: error: (.*)", line) {
            if msg.ends_with(": No space left on device") {
                return Some(Failure::at_line(
                    lines,
                    lineno,
                    "direct regex",
                    Some(Box::new(NoSpaceOnDevice)),
                ));
            }
            return Some(Failure::at_line(
                lines,
                lineno,
                "direct regex",
                Some(Box::new(DpkgError {
                    msg: msg.to_string(),
                })),
            ));
        }
        if let Some((_, pkg, msg)) =
            lazy_regex::regex_captures!(r"dpkg: error processing package (.*) \((.*)\):", line)
        {
            // The following line has the reason; fall back to this one if
            // the log ends here.
            let offset = if lineno + 1 < lines.len() {
                lineno + 1
            } else {
                lineno
            };
            return Some(Failure::at_line(
                lines,
                offset,
                "direct regex",
                Some(Box::new(DpkgError {
                    msg: format!("processing package {} ({})", pkg, msg),
                })),
            ));
        }
    }

    for (i, line) in lines.enumerate_forward(None) {
        if let Some(problem) = match_line(&NO_SPACE_MATCHERS[..], line) {
            return Some(Failure::at_line(lines, i, "direct regex", problem));
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
/// The section that was analyzed and the failure found in it, if any
pub fn find_apt_get_update_failure(
    sbuildlog: &crate::sbuild::SbuildLog,
) -> SectionFailure<'static> {
    let focus_section = "update chroot";
    let failure = sbuildlog
        .get_section_lines(Some(focus_section))
        .and_then(|lines| find_apt_get_failure(&lines));
    SectionFailure {
        section: Some(focus_section),
        failure,
    }
}

/// Finds and parses CUDF output in a log.
///
/// This function searches for and extracts CUDF (Common Upgradeability Description Format)
/// output from a log file.
///
/// # Arguments
/// * `lines` - Lines to search in
///
/// # Returns
/// An optional tuple containing:
/// * A vector of line offsets where the CUDF output was found
/// * The parsed CUDF data
///
/// An error is returned if the CUDF output can not be parsed.
pub(crate) fn find_cudf_output(
    lines: &[&str],
) -> Result<Option<(Vec<usize>, crate::cudf::Cudf)>, Error> {
    let Some((mut offset, _)) = lines
        .enumerate_backward(None)
        .find(|(_, line)| line.starts_with("output-version:"))
    else {
        return Ok(None);
    };
    let start = offset;
    let mut output = vec![];
    let mut offsets = vec![];
    while lines
        .get(offset)
        .is_some_and(|line| !line.trim().is_empty())
    {
        offsets.push(offset);
        output.push(lines[offset]);
        offset += 1;
    }

    let cudf = serde_yaml::from_str(&output.join("\n"))
        .map_err(|e| Error::with_source("invalid dose3 output", e).at_offset(start))?;
    Ok(Some((offsets, cudf)))
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
/// An optional problem description extracted from the reports, or an error if
/// the reports do not have the expected structure
pub(crate) fn error_from_dose3_reports(
    reports: &[crate::cudf::Report],
) -> Result<Option<Box<dyn Problem>>, Error> {
    let report = match reports {
        [report] if report.package == "sbuild-build-depends-main-dummy" => report,
        _ => {
            let packages = reports
                .iter()
                .map(|report| &report.package)
                .collect::<Vec<_>>();
            return Err(Error::new(format!(
                "unexpected packages in dose3 report: {:?}",
                packages
            )));
        }
    };
    if report.status != crate::cudf::Status::Broken {
        return Ok(None);
    }
    let mut missing = vec![];
    let mut conflict = vec![];
    for reason in &report.reasons {
        if let Some(this_missing) = &reason.missing {
            let relation: Entry = parse_dose3_relation(
                this_missing.pkg.unsat_dependency.as_deref(),
                "unsat-dependency",
            )?;
            missing.push(relation);
        }
        if let Some(this_conflict) = &reason.conflict {
            let relation: Relations = parse_dose3_relation(
                this_conflict.pkg1.unsat_conflict.as_deref(),
                "unsat-conflict",
            )?;
            conflict.extend(relation.entries());
        }
    }
    if !missing.is_empty() {
        let missing: Relations = missing.into();
        return Ok(Some(Box::new(UnsatisfiedAptDependencies {
            relations: missing.to_string(),
        }) as Box<dyn Problem>));
    }
    if !conflict.is_empty() {
        let conflict: Relations = conflict.into();
        return Ok(Some(Box::new(UnsatisfiedAptConflicts {
            relations: conflict.to_string(),
        }) as Box<dyn Problem>));
    }
    Ok(None)
}

fn parse_dose3_relation<T: std::str::FromStr<Err = String>>(
    text: Option<&str>,
    field: &str,
) -> Result<T, Error> {
    let text = text.ok_or_else(|| Error::new(format!("dose3 reason without {}", field)))?;
    text.parse().map_err(|e| {
        Error::new(format!(
            "invalid {} {:?} in dose3 report: {}",
            field, text, e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_just_match(lines: Vec<&str>, lineno: usize) {
        let (r#match, actual_err) = Failure::split(super::find_apt_get_failure(&lines));
        let r#match = r#match.expect("no match found");
        assert_eq!(&r#match.line(), &lines[lineno - 1]);
        assert_eq!(lineno, r#match.lineno());
        assert!(actual_err.is_none());
    }

    fn assert_match(lines: Vec<&str>, lineno: usize, expected: Option<impl Problem + 'static>) {
        let (r#match, actual_err) = Failure::split(super::find_apt_get_failure(&lines));
        let r#match = r#match.expect("no match found");
        assert_eq!(&r#match.line(), &lines[lineno - 1]);
        assert_eq!(lineno, r#match.lineno());
        assert_eq!(
            actual_err.as_ref().map(|x| x.as_ref()),
            expected.as_ref().map(|x| x as &dyn Problem)
        );
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
            Some(AptMissingReleaseFile { url: "https://janitor.debian.net/ blah/ Release".to_owned() })
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
            Some(DpkgError {
                msg: "some other error occurred".to_string(),
            }),
        );
    }

    #[test]
    fn test_dpkg_error_processing_package_direct() {
        let lines = vec![
            "dpkg: error processing package hello (--configure):",
            "subprocess installed post-installation script returned error exit status 1",
        ];

        let (match_result, problem) = Failure::split(find_apt_get_failure(&lines));

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            let dpkg_error = problem.as_any().downcast_ref::<DpkgError>();
            assert!(dpkg_error.is_some());
            let dpkg_error = dpkg_error.unwrap();
            assert_eq!(dpkg_error.msg, "processing package hello (--configure)");
        }
    }

    #[test]
    fn test_dpkg_error_processing_package_last_line() {
        let lines = vec!["dpkg: error processing package hello (--configure):"];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![0]);
        assert_eq!(
            failure
                .problem
                .unwrap()
                .as_any()
                .downcast_ref::<DpkgError>()
                .unwrap(),
            &DpkgError {
                msg: "processing package hello (--configure)".to_string(),
            }
        );
    }

    // Direct test of functions without using helper functions
    #[test]
    fn test_broken_packages_direct() {
        let lines = vec![
            "The following packages have unmet dependencies:",
            "E: Broken packages",
        ];

        let (match_result, problem) = Failure::split(find_apt_get_failure(&lines));

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
    fn test_broken_packages_first_line() {
        let lines = vec!["E: Broken packages"];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![0]);
        assert_eq!(
            failure
                .problem
                .unwrap()
                .as_any()
                .downcast_ref::<AptBrokenPackages>()
                .unwrap(),
            &AptBrokenPackages {
                description: "E: Broken packages".to_string(),
                broken: None,
            }
        );
    }

    #[test]
    fn test_held_broken_packages() {
        let lines = vec![
            "The following packages have unmet dependencies:",
            " sbuild-build-depends-main-dummy : Depends: libfoo-dev but it is not going to be installed",
            "                                   Depends: libbar-dev (>= 1.0) but it is not installable",
            "E: Unable to correct problems, you have held broken packages.",
        ];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![1, 2, 3]);
        assert_eq!(failure.r#match.lines(), vec![lines[1], lines[2], lines[3]]);
        assert_eq!(
            failure
                .problem
                .unwrap()
                .as_any()
                .downcast_ref::<AptBrokenPackages>()
                .unwrap(),
            &AptBrokenPackages {
                description: "E: Unable to correct problems, you have held broken packages."
                    .to_string(),
                broken: Some(vec![
                    "libbar-dev (>= 1.0)".to_string(),
                    "libfoo-dev".to_string()
                ]),
            }
        );
    }

    #[test]
    fn test_held_broken_packages_from_first_line() {
        let lines = vec![
            " sbuild-build-depends-main-dummy : Depends: libfoo-dev but it is not going to be installed",
            "E: Unable to correct problems, you have held broken packages.",
        ];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![0, 1]);
    }

    #[test]
    fn test_held_broken_packages_first_line() {
        let lines = vec!["E: Unable to correct problems, you have held broken packages."];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![0]);
        assert_eq!(
            failure
                .problem
                .unwrap()
                .as_any()
                .downcast_ref::<AptBrokenPackages>()
                .unwrap()
                .broken,
            Some(vec![])
        );
    }

    #[test]
    fn test_held_broken_packages_unrelated_depends() {
        let lines = vec![
            "Note: foo Depends: libfoo-dev but it is not installable",
            "Note: foo Depends: libfoo-dev but it is not installable",
            "E: Unable to correct problems, you have held broken packages.",
        ];

        let failure = find_apt_get_failure(&lines).unwrap();

        assert_eq!(failure.r#match.offsets(), vec![2]);
        assert_eq!(
            failure
                .problem
                .unwrap()
                .as_any()
                .downcast_ref::<AptBrokenPackages>()
                .unwrap()
                .broken,
            Some(vec![])
        );
    }

    #[test]
    fn test_unable_to_locate_package_direct() {
        let lines = vec!["E: Unable to locate package nonexistent-package"];

        let (match_result, problem) = Failure::split(find_apt_get_failure(&lines));

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            let pkg_unknown = problem.as_any().downcast_ref::<AptPackageUnknown>();
            assert!(pkg_unknown.is_some());
            let pkg_unknown = pkg_unknown.unwrap();
            assert_eq!(pkg_unknown.package, "nonexistent-package");
        }
    }

    #[test]
    fn test_copy_extracted_data_no_space_direct() {
        let lines = vec![
            "some text before",
            " cannot copy extracted data for '/var/cache/apt/archives/hello_2.10-2_amd64.deb' to '/tmp/hello': failed to write (No space left on device)",
            "some text after"
        ];

        let (match_result, problem) = Failure::split(find_apt_get_failure(&lines));

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

        let (match_result, problem) = Failure::split(find_apt_get_failure(&lines));

        assert!(match_result.is_some());
        assert!(problem.is_some());
        if let Some(problem) = problem {
            assert!(problem.as_any().is::<NoSpaceOnDevice>());
        }
    }

    #[test]
    fn test_find_apt_get_update_failure_missing_section() {
        use crate::sbuild::{SbuildLog, SbuildLogSection};
        let log = SbuildLog(vec![SbuildLogSection {
            title: Some("cleanup".to_string()),
            offsets: (1, 1),
            lines: vec![
                "E: Failed to fetch http://example.com/InRelease  Connection failed".to_string(),
            ],
        }]);

        let section_failure = find_apt_get_update_failure(&log);

        assert_eq!(section_failure.section, Some("update chroot"));
        assert!(section_failure.failure.is_none());
    }

    #[test]
    fn test_find_cudf_output() {
        use crate::cudf::*;
        let lines = include_str!("testdata/sbuild-cudf.log")
            .split_inclusive('\n')
            .collect::<Vec<_>>();
        let (offsets, report) = find_cudf_output(&lines).unwrap().unwrap();
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
    fn test_find_cudf_output_conflict() {
        let lines = vec![
            "output-version: 1.2\n",
            "native-architecture: amd64\n",
            "report:\n",
            " -\n",
            "  package: sbuild-build-depends-main-dummy\n",
            "  version: 0.invalid.0\n",
            "  architecture: amd64\n",
            "  status: broken\n",
            "  reasons:\n",
            "   -\n",
            "    conflict:\n",
            "     pkg1:\n",
            "      package: sbuild-build-depends-main-dummy\n",
            "      version: 0.invalid.0\n",
            "      architecture: amd64\n",
            "      unsat-conflict: libfoo:amd64 (<< 2.0)\n",
            "     pkg2:\n",
            "      package: libfoo\n",
            "      version: 1.0-1\n",
            "      architecture: amd64\n",
            " \n",
        ];
        let (_, cudf) = find_cudf_output(&lines).unwrap().unwrap();
        let conflict = cudf.report[0].reasons[0].conflict.as_ref().unwrap();
        assert_eq!(
            conflict.pkg1.unsat_conflict.as_deref(),
            Some("libfoo:amd64 (<< 2.0)")
        );
        let problem = error_from_dose3_reports(&cudf.report).unwrap().unwrap();
        let problem = problem
            .as_any()
            .downcast_ref::<UnsatisfiedAptConflicts>()
            .unwrap();
        assert_eq!(problem.relations, "libfoo:amd64 (<< 2.0)");
    }

    #[test]
    fn test_find_cudf_output_last() {
        let lines = vec![
            "output-version: 1.2\n",
            "native-architecture: amd64\n",
            "report: []\n",
            "\n",
            "output-version: 1.2\n",
            "native-architecture: arm64\n",
            "report: []\n",
            "\n",
        ];
        let (offsets, cudf) = find_cudf_output(&lines).unwrap().unwrap();
        assert_eq!(offsets, vec![4, 5, 6]);
        assert_eq!(cudf.native_architecture, "arm64");
    }

    #[test]
    fn test_find_cudf_output_invalid() {
        let lines = vec![
            "output-version: 1.2\n",
            "native-architecture: amd64\n",
            "\n",
        ];
        let err = find_cudf_output(&lines).unwrap_err();
        assert_eq!(err.message(), "invalid dose3 output");
        assert_eq!(err.offset(), Some(0));
    }

    #[test]
    fn test_find_cudf_output_at_end() {
        let lines = vec![
            "output-version: 1.2\n",
            "native-architecture: amd64\n",
            "report:\n",
            " -\n",
            "  package: sbuild-build-depends-main-dummy\n",
            "  version: 0.invalid.0\n",
            "  architecture: amd64\n",
            "  status: broken\n",
            "  reasons: []\n",
        ];
        let (offsets, cudf) = find_cudf_output(&lines).unwrap().unwrap();
        assert_eq!(offsets, (0..lines.len()).collect::<Vec<_>>());
        assert_eq!(cudf.report.len(), 1);
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

        let problem = error_from_dose3_reports(&missing_reports).unwrap();
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

        let problem = error_from_dose3_reports(&conflict_reports).unwrap();
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

        let problem = error_from_dose3_reports(&ok_reports).unwrap();
        assert!(problem.is_none());
    }

    fn dose3_pkg(
        package: &str,
        unsat_dependency: Option<&str>,
        unsat_conflict: Option<&str>,
    ) -> crate::cudf::Pkg {
        crate::cudf::Pkg {
            package: package.to_string(),
            version: "0.invalid.0".parse().unwrap(),
            architecture: "amd64".to_string(),
            unsat_dependency: unsat_dependency.map(str::to_string),
            unsat_conflict: unsat_conflict.map(str::to_string),
        }
    }

    fn dose3_report(package: &str, reasons: Vec<crate::cudf::Reason>) -> crate::cudf::Report {
        crate::cudf::Report {
            package: package.to_string(),
            version: "0.invalid.0".parse().unwrap(),
            architecture: "amd64".to_string(),
            status: crate::cudf::Status::Broken,
            reasons,
        }
    }

    fn dose3_missing(unsat_dependency: Option<&str>) -> crate::cudf::Reason {
        crate::cudf::Reason {
            missing: Some(crate::cudf::Missing {
                pkg: dose3_pkg("sbuild-build-depends-main-dummy", unsat_dependency, None),
            }),
            conflict: None,
        }
    }

    fn dose3_conflict(unsat_conflict: Option<&str>) -> crate::cudf::Reason {
        crate::cudf::Reason {
            missing: None,
            conflict: Some(crate::cudf::Conflict {
                pkg1: dose3_pkg("sbuild-build-depends-main-dummy", None, unsat_conflict),
                pkg2: dose3_pkg("libfoo", None, None),
            }),
        }
    }

    #[test]
    fn test_error_from_dose3_reports_no_reports() {
        let err = error_from_dose3_reports(&[]).unwrap_err();
        assert_eq!(err.message(), "unexpected packages in dose3 report: []");
    }

    #[test]
    fn test_error_from_dose3_reports_unexpected_package() {
        let err = error_from_dose3_reports(&[dose3_report("libfoo", vec![])]).unwrap_err();
        assert_eq!(
            err.message(),
            "unexpected packages in dose3 report: [\"libfoo\"]"
        );
    }

    #[test]
    fn test_error_from_dose3_reports_missing_without_dependency() {
        let reports = [dose3_report(
            "sbuild-build-depends-main-dummy",
            vec![dose3_missing(None)],
        )];
        let err = error_from_dose3_reports(&reports).unwrap_err();
        assert_eq!(err.message(), "dose3 reason without unsat-dependency");
    }

    #[test]
    fn test_error_from_dose3_reports_invalid_dependency() {
        let reports = [dose3_report(
            "sbuild-build-depends-main-dummy",
            vec![dose3_missing(Some("libfoo (>= 1.0"))],
        )];
        let err = error_from_dose3_reports(&reports).unwrap_err();
        assert_eq!(
            err.message(),
            "invalid unsat-dependency \"libfoo (>= 1.0\" in dose3 report: Expected ')'"
        );
    }

    #[test]
    fn test_error_from_dose3_reports_conflict_without_conflict() {
        let reports = [dose3_report(
            "sbuild-build-depends-main-dummy",
            vec![dose3_conflict(None)],
        )];
        let err = error_from_dose3_reports(&reports).unwrap_err();
        assert_eq!(err.message(), "dose3 reason without unsat-conflict");
    }

    #[test]
    fn test_error_from_dose3_reports_invalid_conflict() {
        let reports = [dose3_report(
            "sbuild-build-depends-main-dummy",
            vec![dose3_conflict(Some("libfoo (<< 2.0"))],
        )];
        let err = error_from_dose3_reports(&reports).unwrap_err();
        assert_eq!(
            err.message(),
            "invalid unsat-conflict \"libfoo (<< 2.0\" in dose3 report: Expected ')'"
        );
    }

    fn apt(lines: &[&str]) -> serde_json::Value {
        crate::failure_json(find_apt_get_failure(lines).as_ref())
    }

    #[test]
    fn test_apt_empty() {
        assert_eq!(apt(&[]), serde_json::json!(null));
    }

    #[test]
    fn test_apt_failed_to_fetch() {
        assert_eq!(
            apt(&["E: Failed to fetch http://deb.debian.org/foo.deb  404  Not Found [IP: 1.2.3.4 80]\n"]),
            serde_json::json!({"lines":["E: Failed to fetch http://deb.debian.org/foo.deb  404  Not Found [IP: 1.2.3.4 80]\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"error":"404  Not Found [IP: 1.2.3.4 80]","url":"http://deb.debian.org/foo.deb"},"kind":"apt-file-fetch-failure"}}),
        );
        assert_eq!(
            apt(&["E: Failed to fetch http://deb.debian.org/foo.deb  Write error - No space left on device\n"]),
            serde_json::json!({"lines":["E: Failed to fetch http://deb.debian.org/foo.deb  Write error - No space left on device\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_failed_to_fetch_unparsed() {
        assert_eq!(
            apt(&["E: Failed to fetch http://deb.debian.org/foo.deb 404\n"]),
            serde_json::json!({"lines":["E: Failed to fetch http://deb.debian.org/foo.deb 404\n"],"offsets":[0],"origin":"direct regex","problem":null}),
        );
    }

    #[test]
    fn test_apt_broken_packages() {
        assert_eq!(
            apt(&[
                "foo\n",
                " The following packages have unmet dependencies:  \n",
                "E: Broken packages\n",
            ]),
            serde_json::json!({"lines":[" The following packages have unmet dependencies:  \n"],"offsets":[1],"origin":"direct match","problem":{"details":{"broken":null,"description":"The following packages have unmet dependencies:"},"kind":"apt-broken-packages"}}),
        );
    }

    #[test]
    fn test_apt_held_broken_packages() {
        assert_eq!(
            apt(&[
                "The following packages have unmet dependencies:\n",
                " sbuild-build-depends-main-dummy : Depends: libfoo-dev but it is not going to be installed\n",
                "                                   Depends: libbar-dev (>= 1.0) but it is not installable\n",
                "E: Unable to correct problems, you have held broken packages.\n",
            ]),
            serde_json::json!({"lines":[" sbuild-build-depends-main-dummy : Depends: libfoo-dev but it is not going to be installed\n","                                   Depends: libbar-dev (>= 1.0) but it is not installable\n","E: Unable to correct problems, you have held broken packages.\n"],"offsets":[1,2,3],"origin":"direct match","problem":{"details":{"broken":["libbar-dev (>= 1.0)","libfoo-dev"],"description":"E: Unable to correct problems, you have held broken packages."},"kind":"apt-broken-packages"}}),
        );
    }

    #[test]
    fn test_apt_held_broken_packages_none_listed() {
        assert_eq!(
            apt(&[
                "foo\n",
                "bar\n",
                "E: Unable to correct problems, you have held broken packages.\n",
            ]),
            serde_json::json!({"lines":["E: Unable to correct problems, you have held broken packages.\n"],"offsets":[2],"origin":"direct match","problem":{"details":{"broken":[],"description":"E: Unable to correct problems, you have held broken packages."},"kind":"apt-broken-packages"}}),
        );
    }

    #[test]
    fn test_apt_missing_release_file() {
        assert_eq!(
            apt(&["E: The repository 'http://example.com unstable Release' does not have a Release file.\n"]),
            serde_json::json!({"lines":["E: The repository 'http://example.com unstable Release' does not have a Release file.\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"url":"http://example.com unstable Release"},"kind":"missing-release-file"}}),
        );
    }

    #[test]
    fn test_apt_dpkg_deb_no_space() {
        assert_eq!(
            apt(&["dpkg-deb: error: unable to write file '/foo': No space left on device\n"]),
            serde_json::json!({"lines":["dpkg-deb: error: unable to write file '/foo': No space left on device\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_not_enough_free_space() {
        assert_eq!(
            apt(&["E: You don't have enough free space in /var/cache/apt/archives/.\n"]),
            serde_json::json!({"lines":["E: You don't have enough free space in /var/cache/apt/archives/.\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_generic_error_latest_wins() {
        assert_eq!(
            apt(&["E: first\n", "foo\n", "E: second\n", "bar\n"]),
            serde_json::json!({"lines":["E: second\n"],"offsets":[2],"origin":"direct regex","problem":null}),
        );
    }

    #[test]
    fn test_apt_specific_error_before_generic() {
        assert_eq!(
            apt(&["E: Unable to locate package foo\n", "E: second\n"]),
            serde_json::json!({"lines":["E: Unable to locate package foo\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"package":"foo"},"kind":"apt-package-unknown"}}),
        );
    }

    #[test]
    fn test_apt_unable_to_locate_package() {
        assert_eq!(
            apt(&["E: Unable to locate package libfoo-dev\n"]),
            serde_json::json!({"lines":["E: Unable to locate package libfoo-dev\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"package":"libfoo-dev"},"kind":"apt-package-unknown"}}),
        );
    }

    #[test]
    fn test_apt_write_error() {
        assert_eq!(
            apt(&["E: Write error - write (28: No space left on device)\n"]),
            serde_json::json!({"lines":["E: Write error - write (28: No space left on device)\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
        assert_eq!(
            apt(&[
                "E: Unable to locate package foo\n",
                "E: Write error - write (28: No space left on device)\n",
            ]),
            serde_json::json!({"lines":["E: Write error - write (28: No space left on device)\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_dpkg_error() {
        assert_eq!(
            apt(&["dpkg: error: cannot write '/var/lib/dpkg/status': No space left on device\n"]),
            serde_json::json!({"lines":["dpkg: error: cannot write '/var/lib/dpkg/status': No space left on device\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
        assert_eq!(
            apt(&["dpkg: error: something else\n"]),
            serde_json::json!({"lines":["dpkg: error: something else\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"msg":"something else"},"kind":"dpkg-error"}}),
        );
    }

    #[test]
    fn test_apt_dpkg_error_processing_package() {
        assert_eq!(
            apt(&[
                "dpkg: error processing package foo (--configure):\n",
                " installed foo package post-installation script subprocess returned error exit status 1\n",
            ]),
            serde_json::json!({"lines":[" installed foo package post-installation script subprocess returned error exit status 1\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"msg":"processing package foo (--configure)"},"kind":"dpkg-error"}}),
        );
    }

    #[test]
    fn test_apt_cannot_copy_extracted_data() {
        assert_eq!(
            apt(&[
                "foo\n",
                "dpkg-deb (subprocess): cannot copy extracted data for './usr/lib/foo.so' to '/usr/lib/foo.so.dpkg-new': failed to write (No space left on device)\n",
            ]),
            serde_json::json!({"lines":["dpkg-deb (subprocess): cannot copy extracted data for './usr/lib/foo.so' to '/usr/lib/foo.so.dpkg-new': failed to write (No space left on device)\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_forward_no_space_overrides_generic() {
        assert_eq!(
            apt(&[
                "E: something\n",
                " foo: No space left on device\n",
                " bar: No space left on device\n"
            ]),
            serde_json::json!({"lines":[" foo: No space left on device\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}}),
        );
    }

    #[test]
    fn test_apt_scans_last_50_lines() {
        let mut lines = vec!["E: Unable to locate package libfoo-dev\n"];
        lines.extend(std::iter::repeat_n("filler\n", 49));
        assert_eq!(
            apt(&lines),
            serde_json::json!({"lines":["E: Unable to locate package libfoo-dev\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"package":"libfoo-dev"},"kind":"apt-package-unknown"}})
        );
        lines.push("filler\n");
        assert_eq!(apt(&lines), serde_json::json!(null));
    }

    #[test]
    fn test_apt_forward_scan_is_not_limited() {
        let mut lines = vec![" foo: No space left on device\n"];
        lines.extend(std::iter::repeat_n("filler\n", 60));
        assert_eq!(
            apt(&lines),
            serde_json::json!({"lines":[" foo: No space left on device\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"no-space-on-device"}})
        );
    }
}
