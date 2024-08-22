use crate::lines::Lines;
use crate::problems::common::NoSpaceOnDevice;
use crate::problems::debian::*;
use crate::Problem;

pub fn find_brz_build_error(lines: Vec<&str>) -> Option<(Option<Box<dyn Problem>>, String)> {
    for (i, line) in lines.enumerate_backward(None) {
        if let Some(suffix) = line.strip_prefix("brz: ERROR: ") {
            let mut rest = vec![suffix.to_string()];
            for n in lines[i + 1..].iter() {
                if n.starts_with(" ") {
                    rest.push(n.to_string());
                }
            }
            let reflowed = rest.join("\n");
            let (err, line) = parse_brz_error(&reflowed, lines[..i].to_vec());
            return Some((err, line.to_string()));
        }
    }
    None
}

fn parse_debcargo_failure(_: &regex::Captures, prior_lines: Vec<&str>) -> Option<Box<dyn Problem>> {
    const MORE_TAIL: &str = "\x1b[0m\n";
    const MORE_HEAD1: &str = "\x1b[1;31mSomething failed: ";
    const MORE_HEAD2: &str = "\x1b[1;31mdebcargo failed: ";
    if let Some(extra) = prior_lines
        .last()
        .unwrap()
        .strip_suffix(MORE_TAIL)
    {
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
    static ref BRZ_ERRORS: Vec<(regex::Regex, fn(&regex::Captures, Vec<&str>) -> Option<Box<dyn Problem>>)> = vec![
        regex_line_matcher!("Unable to find the needed upstream tarball for package (.*), version (.*)\\.",
        |m, _| Some(Box::new(UnableToFindUpstreamTarball{package: m.get(1).unwrap().as_str().to_string(), version: m.get(2).unwrap().as_str().parse().unwrap()}))),
        regex_line_matcher!("Unknown mercurial extra fields in (.*): b'(.*)'.", |m, _| Some(Box::new(UnknownMercurialExtraFields(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!("UScan failed to run: In watchfile (.*), reading webpage (.*) failed: 429 too many requests\\.", |m, _| Some(Box::new(UScanTooManyRequests(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!("UScan failed to run: OpenPGP signature did not verify..", |_, _| Some(Box::new(UpstreamPGPSignatureVerificationFailed))),
        regex_line_matcher!(r"Inconsistency between source format and version: version is( not)? native, format is( not)? native\.", |m, _| Some(Box::new(InconsistentSourceFormat{version: m.get(1).is_some(), source_format: m.get(2).is_some()}))),
        regex_line_matcher!(r"UScan failed to run: In (.*) no matching hrefs for version (.*) in watch line", |m, _| Some(Box::new(UScanRequestVersionMissing(m.get(2).unwrap().as_str().to_string())))),
        regex_line_matcher!(r"UScan failed to run: In directory ., downloading (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"UScan failed to run: In watchfile debian/watch, reading webpage\n  (.*) failed: (.*)", |m, _| Some(Box::new(UScanFailed{url: m.get(1).unwrap().as_str().to_string(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"Unable to parse upstream metadata file (.*): (.*)", |m, _| Some(Box::new(UpstreamMetadataFileParseError{path: m.get(1).unwrap().as_str().to_string().into(), reason: m.get(2).unwrap().as_str().to_string()}))),
        regex_line_matcher!(r"Debcargo failed to run\.", parse_debcargo_failure),
        regex_line_matcher!(r"\[Errno 28\] No space left on device", |_, _| Some(Box::new(NoSpaceOnDevice)))
    ];
}

pub fn parse_brz_error<'a>(
    line: &'a str,
    prior_lines: Vec<&'a str>,
) -> (Option<Box<dyn Problem>>, String) {
    let line = line.trim();
    for (re, f) in BRZ_ERRORS.iter() {
        if let Some(m) = re.captures(line) {
            let err = f(&m, prior_lines);
            let description= err.as_ref().unwrap().to_string();
            return (err, description);
        }
    }
    if let Some(suffix) = line.strip_prefix("UScan failed to run: ") {
        return (Some(Box::new(UScanError(suffix.to_owned()))), line.to_string());
    }
    if let Some(suffix) = line.strip_prefix("Unable to parse changelog: ") {
        return (
            Some(Box::new(ChangelogParseError(
                suffix.to_string().to_string(),
            ))),
            line.to_string(),
        );
    }
    return (None, line.split_once('\n').unwrap().0.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inconsistent_source_format() {
        let (err, line) = 
            parse_brz_error(
                "Inconsistency between source format and version: version is not native, format is native.",
                vec![]);
        assert_eq!(
            line,
                "Inconsistent source format between version and source format",
        );
        assert_eq!(
            Some(Box::new(InconsistentSourceFormat { version: true, source_format: false }) as Box<dyn Problem>),
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
    fn test_missing_debcargo_crate2() {
        let lines = vec!["Running 'sbuild -A -s -v'\n",
        "Building using working tree\n",
        "Building package in merge mode\n",
        "Using crate name: utf8parse, version 0.10.1+git20220116.1.dfac57e\n",
"    Updating crates.io index\n",
"    Updating crates.io index\n",
"\x1b[1;31mdebcargo failed: Couldn't find any crate matching utf8parse =0.10.1\n",
"Try `debcargo update` to update the crates.io index.\x1b[0m\n",
"brz: ERROR: Debcargo failed to run.\n"
        ];
        let (err, line) = find_brz_build_error(lines).unwrap();
        assert_eq!(
            line,
            "debcargo can't find crate utf8parse (version: 0.10.1)"
        );
        assert_eq!(err, Some(Box::new(MissingDebcargoCrate{cratename: "utf8parse".to_owned(), version: Some("0.10.1".to_owned())}) as Box<dyn Problem>));
    }
}
