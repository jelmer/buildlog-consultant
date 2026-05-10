//! Module for parsing and analyzing Bazaar (brz) version control system logs.
//!
//! This module contains functions to identify and diagnose common issues in
//! Bazaar/Breezy output, particularly in the context of Debian packaging.

use crate::lines::Lines;
use crate::problems::common::NoSpaceOnDevice;
use crate::problems::debian::*;
use crate::Problem;

/// Type alias for brz error handler
pub type BrzErrorHandler = Vec<(
    regex::Regex,
    fn(&regex::Captures, Vec<&str>) -> Option<Box<dyn Problem>>,
)>;

/// Searches for Bazaar (brz) build errors in log lines.
///
/// This function scans log lines for Bazaar errors and extracts problem information.
///
/// # Arguments
/// * `lines` - Vector of log lines to analyze
///
/// # Returns
/// An optional tuple containing:
/// * An optional problem description
/// * A string representation of the error
pub fn find_brz_build_error(lines: Vec<&str>) -> Option<(Option<Box<dyn Problem>>, String)> {
    for (i, line) in lines.enumerate_backward(None) {
        if let Some(suffix) = line.strip_prefix("brz: ERROR: ") {
            // Section lines retain their trailing newline. Strip it before
            // joining so multi-line brz errors collapse to a single `\n`
            // between segments instead of `\n\n`, otherwise patterns like
            //   In watchfile debian/watch, reading webpage\n  ... failed: ...
            // won't match.
            let mut rest = vec![suffix.trim_end_matches('\n').to_string()];
            rest.extend(
                lines[i + 1..]
                    .iter()
                    .filter(|n| n.starts_with(" "))
                    .map(|n| n.trim_end_matches('\n').to_string()),
            );
            let reflowed = rest.join("\n");
            let (err, line) = parse_brz_error(&reflowed, lines[..i].to_vec());
            return Some((err, line.to_string()));
        }
    }
    None
}

/// Extracts debcargo-specific failures from brz error output.
///
/// This function parses error output specific to debcargo failures,
/// looking for patterns like "Couldn't find any crate matching".
///
/// # Arguments
/// * `_` - The regex captures (unused)
/// * `prior_lines` - Lines preceding the error to analyze for context
///
/// # Returns
/// An optional problem description
fn parse_debcargo_failure(_: &regex::Captures, prior_lines: Vec<&str>) -> Option<Box<dyn Problem>> {
    const MORE_TAIL: &str = "\x1b[0m\n";
    const MORE_HEAD1: &str = "\x1b[1;31mSomething failed: ";
    const MORE_HEAD2: &str = "\x1b[1;31mdebcargo failed: ";
    if let Some(extra) = prior_lines.last().unwrap().strip_suffix(MORE_TAIL) {
        let mut extra = vec![extra];
        for line in prior_lines[..prior_lines.len() - 1].iter().rev() {
            if let Some(middle) = extra[0].strip_prefix(MORE_HEAD1) {
                extra[0] = middle;
                break;
            }
            if let Some(middle) = extra[0].strip_prefix(MORE_HEAD2) {
                extra[0] = middle;
                break;
            }
            extra.insert(0, line);
        }
        if extra.len() == 1 {
            extra = vec![];
        }
        if extra
            .last()
            .and_then(|l| l.strip_prefix("Try `debcargo update` to update the crates.io index."))
            .is_some()
        {
            if let Some((_, n)) = lazy_regex::regex_captures!(
                r"Couldn't find any crate matching (.*)",
                extra[extra.len() - 2].trim_end()
            ) {
                return Some(Box::new(MissingDebcargoCrate::from_string(n)));
            } else {
                return Some(Box::new(DpkgSourcePackFailed(
                    extra[extra.len() - 2].to_owned(),
                )));
            }
        } else if !extra.is_empty() {
            if let Some((_, d, p)) = lazy_regex::regex_captures!(
                r"Cannot represent prerelease part of dependency: (.*) Predicate \{ (.*) \}",
                extra[0]
            ) {
                return Some(Box::new(DebcargoUnacceptablePredicate {
                    cratename: d.to_owned(),
                    predicate: p.to_owned(),
                }));
            } else if let Some((_, d, c)) = lazy_regex::regex_captures!(
                r"Cannot represent prerelease part of dependency: (.*) Comparator \{ (.*) \}",
                extra[0]
            ) {
                return Some(Box::new(DebcargoUnacceptableComparator {
                    cratename: d.to_owned(),
                    comparator: c.to_owned(),
                }));
            }
        } else {
            return Some(Box::new(DebcargoFailure(extra.join(""))));
        }
    }

    Some(Box::new(DebcargoFailure(
        "Debcargo failed to run".to_string(),
    )))
}

macro_rules! regex_line_matcher {
    ($re:expr, $f:expr) => {
        (regex::Regex::new($re).unwrap(), $f)
    };
}

lazy_static::lazy_static! {
    static ref BRZ_ERRORS: BrzErrorHandler = vec![
        regex_line_matcher!("Unable to find the needed upstream tarball for package (.*), version (.*)\\.",
        |m, _| Some(Box::new(UnableToFindUpstreamTarball{package: m.get(1).unwrap().as_str().to_string(), version: m.get(2).unwrap().as_str().parse().unwrap()}))),
        regex_line_matcher!("Unknown mercurial extra fields in (.*): b'(.*)'.", |m, _| Some(Box::new(UnknownMercurialExtraFields(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!("UScan failed to run: In watchfile (.*), reading webpage (.*) failed: 429 too many requests\\.", |m, _| Some(Box::new(UScanTooManyRequests(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!("UScan failed to run: OpenPGP signature did not verify..", |_, _| Some(Box::new(UpstreamPGPSignatureVerificationFailed))),
        regex_line_matcher!(r"Inconsistency between source format and version: version is( not)? native, format is( not)? native\.", |m, _| Some(Box::new(InconsistentSourceFormat{version: m.get(1).is_some(), source_format: m.get(2).is_some()}))),
        regex_line_matcher!(r"UScan failed to run: In (.*) no matching hrefs for version (.*) in watch line", |m, _| Some(Box::new(UScanRequestVersionMissing(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!(r"UScan failed to run: In (.*) no matching files for version (.*) in watch line", |m, _| Some(Box::new(UScanRequestVersionMissing(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!(r"UScan failed to run: In directory ., downloading (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"UScan failed to run: In directory \., downloading\n  (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"UScan failed to run: In watchfile debian/watch, reading webpage\n  (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"UScan failed to run: In watchfile debian/watch, reading webpage (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        // brz renames the source package version to the changelog
        // version, then asks uscan to fetch that exact version. When
        // upstream's watchfile points at a moving target (master tag,
        // git snapshot etc.) the latest tarball doesn't carry the
        // requested label and uscan emits this exact line. Distinct
        // from the "no matching files" case (UScanRequestVersionMissing)
        // because here uscan *did* find files — just for a different
        // version. ~24 of the 80 `unidentified` runs in
        // janitor.debian.net's lintian-fixes pipeline matched this.
        regex_line_matcher!(
            r"UScan failed to run: Newest version of (.*) on remote site is (.*), specified download version is (.*)\.",
            |m, _| Some(Box::new(UScanRemoteVersionMismatch{
                package: m.get(1).unwrap().as_str().to_string(),
                remote_version: m.get(2).unwrap().as_str().to_string(),
                wanted_version: m.get(3).unwrap().as_str().to_string(),
            }))
        ),
        // uscan refuses to read debian/watch when one paragraph fails
        // its strict parser. The trailing `<<==EOF==` etc. wraps the
        // offending text in brz's reflowed output; capture greedily
        // up to end of line and stash the raw paragraph for
        // operator-side debugging. Actionable: fix debian/watch.
        regex_line_matcher!(
            r"UScan failed to run: The following paragraph isn't well formatted, skipping it: ([\s\S]*)",
            |m, _| Some(Box::new(UScanWatchfileMalformed{
                paragraph: m.get(1).unwrap().as_str().trim().to_string(),
            }))
        ),
        regex_line_matcher!(r"Unable to parse upstream metadata file (.*): (.*)", |m, _| Some(Box::new(UpstreamMetadataFileParseError{path: m.get(1).unwrap().as_str().to_string().into(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"Debcargo failed to run\.", parse_debcargo_failure),
        // Ad-hoc ad-hoc brz error suffixes that surface in the logs:
        regex_line_matcher!(
            r"The nested tree for (.*) can not be resolved\.",
            |m, _| Some(Box::new(NestedTreeUnresolvable{
                name: m.get(1).unwrap().as_str().to_string(),
            }))
        ),
        // dulwich raises this when its packfile walker sees a
        // `gitlink` (submodule) entry without `--include-submodules`.
        // The error format varies (Python tuple of `(path, sha)`,
        // sometimes with `b'...'` byte-string repr for either field);
        // accept either repr.
        regex_line_matcher!(
            r"dulwich\.objects\.SubmoduleEncountered: \(b?'?([^']+)'?, b?'?([0-9a-f]+)'?\)",
            |m, _| Some(Box::new(SubmoduleEncountered{
                path: m.get(1).unwrap().as_str().to_string(),
                sha: m.get(2).unwrap().as_str().to_string(),
            }))
        ),
        // brz's "Breezy has encountered an internal error" banner
        // emits the underlying Python exception type + message on the
        // first ERROR line. Empty messages (assertion errors) are
        // common — preserve the empty string to keep the matcher
        // honest about what brz reported.
        regex_line_matcher!(
            r"^([A-Z][A-Za-z]*Error): ?(.*)$",
            |m, _| Some(Box::new(BrzInternalError{
                exception_type: m.get(1).unwrap().as_str().to_string(),
                exception_message: m.get(2).unwrap().as_str().trim().to_string(),
            }))
        ),
        regex_line_matcher!(r"\[Errno 28\] No space left on device", |_, _| Some(Box::new(NoSpaceOnDevice)))
    ];
}

/// Parses a brz error message to identify the specific problem.
///
/// This function analyzes a Bazaar error message line and the preceding context
/// to determine the specific problem type.
///
/// # Arguments
/// * `line` - The error message to parse
/// * `prior_lines` - Vector of lines preceding the error message in the log
///
/// # Returns
/// A tuple containing:
/// * An optional problem description if the error is recognized
/// * A string representation of the error
pub fn parse_brz_error<'a>(
    line: &'a str,
    prior_lines: Vec<&'a str>,
) -> (Option<Box<dyn Problem>>, String) {
    let line = line.trim();
    for (re, f) in BRZ_ERRORS.iter() {
        if let Some(m) = re.captures(line) {
            let err = f(&m, prior_lines);
            let description = err.as_ref().unwrap().to_string();
            return (err, description);
        }
    }
    if let Some(suffix) = line.strip_prefix("UScan failed to run: ") {
        return (
            Some(Box::new(UScanError(suffix.to_owned()))),
            line.to_string(),
        );
    }
    if let Some(suffix) = line.strip_prefix("Unable to parse changelog: ") {
        return (
            Some(Box::new(ChangelogParseError(
                suffix.to_string().to_string(),
            ))),
            line.to_string(),
        );
    }
    let first_line = line.split_once('\n').map_or(line, |(head, _)| head);
    (None, first_line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inconsistent_source_format() {
        let (err, line) = parse_brz_error(
                "Inconsistency between source format and version: version is not native, format is native.",
                vec![]);
        assert_eq!(
            line,
            "Inconsistent source format between version and source format",
        );
        assert_eq!(
            Some(Box::new(InconsistentSourceFormat {
                version: true,
                source_format: false
            }) as Box<dyn Problem>),
            err
        );
    }

    #[test]
    fn test_missing_debcargo_crate() {
        let lines = vec![
            "Using crate name: version-check, version 0.9.2   Updating crates.io index\n",
            "\x1b[1;31mSomething failed: Couldn't find any crate matching version-check = 0.9.2\n",
            "Try `debcargo update` to update the crates.io index.\x1b[0m\n",
            "brz: ERROR: Debcargo failed to run.\n",
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            line,
            "debcargo can't find crate version-check (version: 0.9.2)"
        );
        assert_eq!(
            err,
            Some(Box::new(MissingDebcargoCrate {
                cratename: "version-check".to_string(),
                version: Some("0.9.2".to_string())
            }) as Box<dyn Problem>)
        );
    }

    #[test]
    fn test_uscan_no_matching_files_for_version() {
        let lines = vec![
            "Using uscan to look for the upstream tarball.\n",
            "uscan warn: In debian/watch no matching files for version 1.2.0 in watch line\n",
            "brz: ERROR: UScan failed to run: In debian/watch no matching files for version 1.2.0 in watch line.\n",
        ];
        let (err, _line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(UScanRequestVersionMissing("1.2.0".to_string())) as Box<dyn Problem>)
        );
    }

    /// Regression: brz wraps long uscan errors across two lines, with the
    /// second line indented by two spaces. The previous reflow joined the
    /// segments with `\n`, but section lines retain their own trailing `\n`,
    /// so the result was `\n\n  ...` and the matcher regex (which expects a
    /// single `\n  `) silently failed.
    #[test]
    fn test_uscan_failed_multiline_reflow() {
        let lines = vec![
            "Using uscan to look for the upstream tarball.\n",
            "uscan warn: In watchfile debian/watch, reading webpage\n",
            "  https://example.com/dist/ failed: 404 Not Found\n",
            "brz: ERROR: UScan failed to run: In watchfile debian/watch, reading webpage\n",
            "  https://example.com/dist/ failed: 404 Not Found.\n",
        ];
        let (err, _line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(UScanFailed {
                url: "https://example.com/dist/".to_string(),
                reason: "404 Not Found.".to_string(),
            }) as Box<dyn Problem>)
        );
    }

    /// Regression: a brz error line without a continuation (so no
    /// embedded `\n` after reflow) used to panic in the fallback
    /// branch when calling `split_once('\n').unwrap()`. Also pins
    /// that the nested-tree pattern is now classified (was `None`
    /// before; surfaced as `unidentified` in janitor.debian.net).
    #[test]
    fn test_brz_error_nested_tree_unresolvable() {
        let lines = vec!["brz: ERROR: The nested tree for lib can not be resolved.\n"];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(NestedTreeUnresolvable {
                name: "lib".to_string(),
            }) as Box<dyn Problem>)
        );
        assert_eq!(line, "Nested tree for lib cannot be resolved");
    }

    /// uscan: requested version vs newest-on-remote mismatch. brz
    /// renames the changelog version to the source version then asks
    /// uscan to fetch *that* exact version; when upstream's
    /// watchfile points at a moving target the labels don't line
    /// up. Real example from janitor.debian.net (castor 1.3.2 vs
    /// 1.3.2 — equality is fine; the mismatch in the live logs was
    /// for sonnet 5.116.0 vs 5.116.0 with internal version-suffix
    /// quirks). Use the documented "newest is X, specified Y"
    /// shape that brz emits regardless.
    #[test]
    fn test_brz_error_uscan_remote_version_mismatch() {
        let lines = vec![
            "Using uscan to look for the upstream tarball.\n",
            "brz: ERROR: UScan failed to run: Newest version of td1.8.11 on remote site is 1.8.0+git20260425.8fc2344, specified download version is 1.8.11~git20230202.3179d35.\n",
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(UScanRemoteVersionMismatch {
                package: "td1.8.11".to_string(),
                remote_version: "1.8.0+git20260425.8fc2344".to_string(),
                wanted_version: "1.8.11~git20230202.3179d35".to_string(),
            }) as Box<dyn Problem>)
        );
        assert_eq!(
            line,
            "uscan: td1.8.11 latest is 1.8.0+git20260425.8fc2344, wanted 1.8.11~git20230202.3179d35"
        );
    }

    /// uscan: refuses to parse a malformed paragraph in
    /// debian/watch. The capture is the paragraph text, not the
    /// `<<==EOF==` markers around it.
    #[test]
    fn test_brz_error_uscan_watchfile_malformed() {
        let lines = vec![
            "brz: ERROR: UScan failed to run: The following paragraph isn't well formatted, skipping it: << ==EOF==\n",
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(UScanWatchfileMalformed {
                paragraph: "<< ==EOF==".to_string(),
            }) as Box<dyn Problem>)
        );
        assert_eq!(line, "uscan rejected malformed debian/watch paragraph");
    }

    /// dulwich submodule: the (path, sha) error format.
    #[test]
    fn test_brz_error_submodule_encountered() {
        let lines = vec![
            "brz: ERROR: dulwich.objects.SubmoduleEncountered: (b'subprojects/libcmatrix', b'6c260ee37bd2eff096ee44c29690f30718566c1c')\n",
        ];
        let (err, _line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(SubmoduleEncountered {
                path: "subprojects/libcmatrix".to_string(),
                sha: "6c260ee37bd2eff096ee44c29690f30718566c1c".to_string(),
            }) as Box<dyn Problem>)
        );
    }

    /// brz internal error: empty AssertionError (the td1.8.11 case).
    /// Pre-fix this surfaced as `unidentified` because no matcher
    /// claimed the bare `AssertionError:` line.
    #[test]
    fn test_brz_error_internal_assertion_error_empty() {
        let lines = vec!["brz: ERROR: AssertionError: \n"];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(BrzInternalError {
                exception_type: "AssertionError".to_string(),
                exception_message: String::new(),
            }) as Box<dyn Problem>)
        );
        assert_eq!(line, "brz internal error: AssertionError");
    }

    /// brz internal error with a non-empty message also surfaces.
    #[test]
    fn test_brz_error_internal_attribute_error_with_message() {
        let lines = vec![
            "brz: ERROR: AttributeError: 'RemoteGitRepository' object has no attribute '_git'\n",
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            err,
            Some(Box::new(BrzInternalError {
                exception_type: "AttributeError".to_string(),
                exception_message: "'RemoteGitRepository' object has no attribute '_git'"
                    .to_string(),
            }) as Box<dyn Problem>)
        );
        assert_eq!(
            line,
            "brz internal error: AttributeError: 'RemoteGitRepository' object has no attribute '_git'"
        );
    }

    #[test]
    fn test_missing_debcargo_crate2() {
        let lines = vec![
            "Running 'sbuild -A -s -v'\n",
            "Building using working tree\n",
            "Building package in merge mode\n",
            "Using crate name: utf8parse, version 0.10.1+git20220116.1.dfac57e\n",
            "    Updating crates.io index\n",
            "    Updating crates.io index\n",
            "\x1b[1;31mdebcargo failed: Couldn't find any crate matching utf8parse =0.10.1\n",
            "Try `debcargo update` to update the crates.io index.\x1b[0m\n",
            "brz: ERROR: Debcargo failed to run.\n",
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            line,
            "debcargo can't find crate utf8parse (version: 0.10.1)"
        );
        assert_eq!(
            err,
            Some(Box::new(MissingDebcargoCrate {
                cratename: "utf8parse".to_owned(),
                version: Some("0.10.1".to_owned())
            }) as Box<dyn Problem>)
        );
    }
}
