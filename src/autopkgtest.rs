use crate::lines::Lines;
use crate::problems::autopkgtest::*;
use crate::{Match, Problem, SingleLineMatch};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Packet<'a> {
    Source,
    Summary,
    TestBeginOutput(&'a str),
    TestEndOutput(&'a str),
    Results(&'a str),
    Stderr(&'a str),
    TestbedSetup(&'a str),
    TestOutput(&'a str, &'a str),
    Error(&'a str),
    Other(&'a str),
}

fn parse_autopgktest_line(line: &str) -> Option<(&str, Packet)> {
    let (timestamp, message) =
        match lazy_regex::regex_captures!(r"autopkgtest \[([0-9:]+)\]: (.*)", line) {
            Some((_, timestamp, message)) => (timestamp, message),
            None => {
                return None;
            }
        };

    if message.starts_with("@@@@@@@@@@@@@@@@@@@@ source ") {
        Some((timestamp, Packet::Source))
    } else if message.starts_with("@@@@@@@@@@@@@@@@@@@@ summary") {
        return Some((timestamp, Packet::Summary));
    } else if let Some(message) = message.strip_prefix("test ") {
        let (testname, test_status) = message.trim_end_matches('\n').split_once(": ").unwrap();
        if test_status == "[-----------------------" {
            return Some((timestamp, Packet::TestBeginOutput(testname)));
        } else if test_status == "-----------------------]" {
            return Some((timestamp, Packet::TestEndOutput(testname)));
        } else if test_status == " - - - - - - - - - - results - - - - - - - - - -" {
            return Some((timestamp, Packet::Results(testname)));
        } else if test_status == " - - - - - - - - - - stderr - - - - - - - - - -" {
            return Some((timestamp, Packet::Stderr(testname)));
        } else if test_status == "preparing testbed" {
            return Some((timestamp, Packet::TestbedSetup(testname)));
        } else {
            return Some((timestamp, Packet::TestOutput(testname, test_status)));
        }
    } else if let Some(message) = message.strip_prefix("ERROR: ") {
        return Some((timestamp, Packet::Error(message)));
    } else {
        log::warn!("unhandled autopkgtest message: {}", message);
        return Some((timestamp, Packet::Other(message)));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestResult {
    Pass,
    Fail,
    Skip,
    Flaky,
}

impl std::str::FromStr for TestResult {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PASS" => Ok(Self::Pass),
            "FAIL" => Ok(Self::Fail),
            "SKIP" => Ok(Self::Skip),
            "FLAKY" => Ok(Self::Flaky),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Summary {
    pub offset: usize,
    pub name: String,
    pub result: TestResult,
    pub reason: Option<String>,
    pub extra: Vec<String>,
}

impl Summary {
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn lineno(&self) -> usize {
        self.offset + 1
    }
}

pub fn parse_autopkgtest_summary(lines: Vec<&str>) -> Vec<Summary> {
    let mut i = 0;
    let mut ret = vec![];
    while i < lines.len() {
        let line = lines[i];
        if let Some((_, name)) = lazy_regex::regex_captures!("([^ ]+)(?:[ ]+)PASS", line) {
            ret.push(Summary {
                offset: i,
                name: name.to_string(),
                result: TestResult::Pass,
                reason: None,
                extra: vec![],
            });
            i += 1;
            continue;
        }
        if let Some((_, testname, result, reason)) =
            lazy_regex::regex_captures!("([^ ]+)(?:[ ]+)(FAIL|PASS|SKIP|FLAKY) (.+)", line)
        {
            let offset = i;
            let mut extra = vec![];
            if reason == "badpkg" {
                while i + 1 < lines.len()
                    && (lines[i + 1].starts_with("badpkg:") || lines[i + 1].starts_with("blame:"))
                {
                    extra.push(lines[i + 1]);
                    i += 1;
                }
            }
            ret.push(Summary {
                offset,
                name: testname.to_string(),
                result: result.parse().unwrap(),
                reason: Some(reason.to_string()),
                extra: extra.iter().map(|x| x.to_string()).collect(),
            });
            i += 1;
        } else {
            i += 1;
            continue;
        }
    }
    ret
}

#[derive(Debug, PartialEq, Eq, Clone, std::hash::Hash)]
enum Field {
    Output(String),
    FieldOutput(String, String),
    Stderr(String),
    Results(String),
    PrepareTestbed(String),
    Summary,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Field::Output(testname) => write!(f, "output for test {}", testname),
            Field::Stderr(testname) => write!(f, "stderr for test {}", testname),
            Field::Results(testname) => write!(f, "results for test {}", testname),
            Field::PrepareTestbed(testname) => write!(f, "testbed setup for test {}", testname),
            Field::FieldOutput(testname, field) => {
                write!(f, "{} for test {}", field, testname)
            }
            Field::Summary => write!(f, "summary"),
        }
    }
}

impl Field {
    fn testname(&self) -> Option<&str> {
        match self {
            Field::Output(testname) => Some(testname),
            Field::Stderr(testname) => Some(testname),
            Field::Results(testname) => Some(testname),
            Field::PrepareTestbed(testname) => Some(testname),
            Field::FieldOutput(testname, _) => Some(testname),
            Field::Summary => None,
        }
    }
}

/// Find the autopkgtest failure in output.
pub fn find_autopkgtest_failure_description(
    mut lines: Vec<&str>,
) -> (
    Option<Box<dyn Match>>,
    Option<String>,
    Option<Box<dyn Problem>>,
    Option<String>,
) {
    let mut test_output: HashMap<Field, (Vec<String>, usize)> = HashMap::new();
    let mut current_field: Option<Field> = None;
    let mut it = lines.iter().enumerate().peekable();
    while let Some((i, line)) = it.next() {
        let line = lines[i];
        match parse_autopgktest_line(line) {
            Some((_, Packet::Source)) => {}
            Some((_, Packet::Other(_))) => {}
            Some((_, Packet::Error(msg))) => {
                let msg = if msg.starts_with('"') && msg.chars().filter(|x| *x == '"').count() == 1
                {
                    let mut sublines = vec![msg];
                    while i < lines.len() {
                        let (i, line) = it.next().unwrap();
                        sublines.push(line);
                        if line.chars().filter(|x| x == &'"').count() == 1 {
                            break;
                        }
                    }
                    sublines.join("\n")
                } else {
                    msg.to_string()
                };
                let last_test = if let Some(current_field) = current_field.as_ref() {
                    current_field.testname().map(|x| x.to_owned())
                } else {
                    None
                };
                if let Some((_, _, stderr, _)) =
                    lazy_regex::regex_captures!(r#""(.*)" failed with stderr "(.*)("?)"#, &msg)
                {
                    if lazy_regex::regex_is_match!(
                        "W: (.*): Failed to stat file: No such file or directory",
                        stderr
                    ) {
                        let error =
                            Some(Box::new(AutopkgtestDepChrootDisappeared) as Box<dyn Problem>);
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines,
                                i,
                                Some("direct regex"),
                            )) as Box<dyn Match>),
                            last_test,
                            error,
                            Some(stderr.to_owned()),
                        );
                    }
                }
                if let Some((_, testbed_failure_reason)) =
                    lazy_regex::regex_captures!(r"testbed failure: (.*)", &msg)
                {
                    if current_field.is_some()
                        && testbed_failure_reason == "testbed auxverb failed with exit code 255"
                    {
                        let field = Field::Output(
                            current_field
                                .as_ref()
                                .unwrap()
                                .testname()
                                .unwrap()
                                .to_owned(),
                        );
                        let (r#match, error) = crate::common::find_build_failure_description(
                            test_output
                                .get(&field)
                                .map_or(vec![], |x| x.0.iter().map(|x| x.as_str()).collect()),
                        );
                        if let Some(error) = error {
                            assert!(r#match.is_some());
                            let description = r#match.as_ref().unwrap().line();
                            return (
                                Some(Box::new(SingleLineMatch::from_lines(
                                    &lines,
                                    test_output.get(&field).unwrap().1 + r#match.unwrap().offset(),
                                    Some("direct regex"),
                                )) as Box<dyn Match>),
                                last_test.map(|x| x.to_owned()),
                                Some(error),
                                Some(description),
                            );
                        }
                    }

                    if testbed_failure_reason
                        == "sent `auxverb_debug_fail', got `copy-failed', expected `ok...'"
                    {
                        let (r#match, error) =
                            crate::common::find_build_failure_description(lines.clone());
                        if let Some(error) = error {
                            let description = r#match.as_ref().unwrap().line();
                            return (r#match, last_test, Some(error), Some(description));
                        }
                    }

                    if testbed_failure_reason == "cannot send to testbed: [Errno 32] Broken pipe" {
                        let (r#match, error) = find_testbed_setup_failure(lines.clone());
                        if error.is_some() && r#match.is_some() {
                            let description = r#match.as_ref().unwrap().line();
                            return (r#match, last_test, error, Some(description));
                        }
                    }
                    if testbed_failure_reason == "apt repeatedly failed to download packages" {
                        let (r#match, error) = crate::apt::find_apt_get_failure(lines.clone());
                        if error.is_some() && r#match.is_some() {
                            let description = r#match.as_ref().unwrap().line();
                            return (Some(r#match.unwrap()), last_test, error, Some(description));
                        }
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines,
                                i,
                                Some("direct regex"),
                            )) as Box<dyn Match>),
                            last_test,
                            Some(Box::new(crate::problems::debian::AptFetchFailure {
                                url: None,
                                error: testbed_failure_reason.to_owned(),
                            }) as Box<dyn Problem>),
                            None,
                        );
                    }
                    return (
                        Some(
                            Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                                as Box<dyn Match>,
                        ),
                        last_test.map(|x| x.to_owned()),
                        Some(
                            Box::new(AutopkgtestTestbedFailure(testbed_failure_reason.to_owned()))
                                as Box<dyn Problem>,
                        ),
                        None,
                    );
                }
                if let Some((_, pkg)) =
                    lazy_regex::regex_captures!(r"erroneous package: (.*)", &msg)
                {
                    let (r#match, error) =
                        crate::common::find_build_failure_description(lines[..i].to_vec());
                    let description = r#match.as_ref().unwrap().line();
                    if error.is_some() && r#match.is_some() {
                        return (
                            r#match,
                            last_test.map(|x| x.to_owned()),
                            error,
                            Some(description),
                        );
                    }
                    return (
                        Some(
                            Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                                as Box<dyn Match>,
                        ),
                        last_test.map(|x| x.to_owned()),
                        Some(Box::new(AutopkgtestErroneousPackage(pkg.to_string()))
                            as Box<dyn Problem>),
                        None,
                    );
                }
                if msg == "unexpected error:" {
                    let (r#match, error) =
                        crate::common::find_build_failure_description(lines[(i + 1)..].to_vec());
                    let description = r#match.as_ref().unwrap().line();
                    if error.is_some() && r#match.is_some() {
                        return (
                            r#match,
                            last_test.map(|x| x.to_owned()),
                            error,
                            Some(description),
                        );
                    }
                }
                if let Some(current_field) = current_field.as_ref() {
                    let (r#match, error) = crate::apt::find_apt_get_failure(
                        test_output
                            .get(current_field)
                            .unwrap()
                            .0
                            .iter()
                            .map(|x| x.as_str())
                            .collect(),
                    );
                    if error.is_some()
                        && r#match.is_some()
                        && test_output.contains_key(current_field)
                    {
                        let description = r#match.as_ref().unwrap().line();
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines,
                                test_output.get(current_field).unwrap().1
                                    + r#match.unwrap().offset(),
                                Some("direct regex"),
                            )) as Box<dyn Match>),
                            last_test.map(|x| x.to_owned()),
                            error,
                            Some(description),
                        );
                    }
                }
                if msg == "autopkgtest" && lines[i + 1].trim_end() == ": error cleaning up:" {
                    let description = lines[i - 1].trim_end().to_owned();
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            &lines,
                            test_output.get(current_field.as_ref().unwrap()).unwrap().1,
                            Some("direct regex"),
                        )) as Box<dyn Match>),
                        last_test.map(|x| x.to_owned()),
                        Some(Box::new(AutopkgtestTimedOut) as Box<dyn Problem>),
                        Some(description),
                    );
                }
                return (
                    Some(
                        Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                            as Box<dyn Match>,
                    ),
                    last_test.map(|x| x.to_owned()),
                    None,
                    Some(msg),
                );
            }
            Some((_, Packet::Summary)) => {
                current_field = Some(Field::Summary);
                test_output.insert(current_field.clone().unwrap(), (vec![], i + 1));
            }
            Some((
                _,
                p @ Packet::TestBeginOutput(..)
                | p @ Packet::TestEndOutput(..)
                | p @ Packet::Stderr(..)
                | p @ Packet::Results(..)
                | p @ Packet::TestbedSetup(..)
                | p @ Packet::TestOutput(..),
            )) => {
                match p {
                    Packet::TestBeginOutput(testname) => {
                        current_field = Some(Field::Output(testname.to_owned()));
                    }
                    Packet::TestEndOutput(testname) => {
                        match &current_field {
                            Some(Field::Output(current_testname)) => {
                                if current_testname != testname {
                                    log::warn!(
                                        "unexpected test end output for {}, expected {}",
                                        current_testname,
                                        testname
                                    );
                                }
                            }
                            Some(f) => {
                                log::warn!(
                                    "unexpected test end output for {} while in {}",
                                    testname,
                                    f
                                );
                            }
                            None => {
                                log::warn!("unexpected test end output for {}", testname);
                            }
                        }
                        current_field = None;
                        continue;
                    }
                    Packet::Results(testname) => {
                        current_field = Some(Field::Results(testname.to_owned()));
                    }
                    Packet::Stderr(testname) => {
                        current_field = Some(Field::Stderr(testname.to_owned()));
                    }
                    Packet::TestbedSetup(testname) => {
                        current_field = Some(Field::PrepareTestbed(testname.to_owned()));
                    }
                    Packet::TestOutput(testname, field) => {
                        current_field =
                            Some(Field::FieldOutput(testname.to_owned(), field.to_owned()));
                    }
                    _ => {}
                }
                if test_output.contains_key(current_field.as_ref().unwrap()) {
                    log::warn!(
                        "duplicate output fields for {}",
                        current_field.as_ref().unwrap()
                    );
                }
                test_output.insert(current_field.clone().unwrap(), (vec![], i + 1));
            }
            None => {
                if let Some(current_field) = current_field.as_ref() {
                    test_output
                        .entry(current_field.clone())
                        .or_insert((vec![], i))
                        .0
                        .push(line.to_owned());
                }
            }
        }
    }

    let summary_field = Field::Summary;
    let (summary_lines, summary_offset) = match test_output.get(&summary_field) {
        Some((lines, _)) => (lines, test_output.get(&summary_field).unwrap().1),
        None => {
            while !lines.is_empty() && lines.last().unwrap().trim().is_empty() {
                lines.pop();
            }
            if lines.is_empty() {
                return (None, None, None, None);
            }
            let offset = lines.len() - 1;
            let last_line = lines.last().map(|x| x.to_string());
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    offset,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                last_line,
                None,
                None,
            );
        }
    };
    for packet in parse_autopkgtest_summary(summary_lines.iter().map(|x| x.as_str()).collect()) {
        if [TestResult::Pass, TestResult::Skip].contains(&packet.result) {
            continue;
        }
        assert!([TestResult::Fail, TestResult::Flaky].contains(&packet.result));
        if packet.reason.as_deref() == Some("timed out") {
            let error = Some(Box::new(AutopkgtestTimedOut) as Box<dyn Problem>);
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    summary_offset + packet.offset(),
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(packet.name),
                error,
                packet.reason,
            );
        } else if let Some(output) = packet
            .reason
            .as_ref()
            .and_then(|x| x.strip_prefix("stderr: "))
        {
            let field = Field::Stderr(packet.name.to_string());
            let (stderr_lines, stderr_offset) = test_output.get(&field).map_or_else(
                || (vec![], None),
                |x| (x.0.iter().map(|x| x.as_str()).collect(), Some(x.1)),
            );
            let description;
            let mut offset = None;
            let r#match;
            let mut error;
            if !stderr_lines.is_empty() {
                (r#match, error) =
                    crate::common::find_build_failure_description(stderr_lines.clone());
                if r#match.is_some() && stderr_offset.is_some() {
                    offset = Some(r#match.as_ref().unwrap().offset() + stderr_offset.unwrap());
                    description = Some(r#match.as_ref().unwrap().line());
                } else if stderr_lines.len() == 1
                    && lazy_regex::regex_is_match!(
                        r"QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to \'(.*)\'",
                        &stderr_lines[0],
                    )
                {
                    error = Some(Box::new(XDGRunTimeNotSet) as Box<dyn Problem>);
                    description = Some(stderr_lines[0].to_owned());
                    offset = stderr_offset;
                } else {
                    if let Some(stderr_offset) = stderr_offset {
                        offset = Some(stderr_offset);
                    }
                    description = None;
                }
            } else {
                (r#match, error) = crate::common::find_build_failure_description(vec![output]);

                (offset, description) = if let Some(r#match) = r#match.as_ref() {
                    (
                        Some(summary_offset + packet.offset() + r#match.offset()),
                        Some(r#match.line()),
                    )
                } else {
                    (None, None)
                };
            }
            let offset = offset.unwrap_or_else(|| summary_offset + packet.offset());
            let error =
                error.unwrap_or_else(|| Box::new(AutopkgtestStderrFailure(output.to_owned())));
            let description = description.unwrap_or_else(|| {
                format!(
                    "Test {} failed due to unauthorized stderr output: {}",
                    packet.name, output
                )
            });
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    offset,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(packet.name),
                Some(error),
                Some(description),
            );
        } else if packet.reason.as_deref() == Some("badpkg") {
            let field = Field::Output(packet.name.to_string());
            let (output_lines, output_offset) = test_output.get(&field).map_or_else(
                || (vec![], None),
                |x| (x.0.iter().map(|x| x.as_str()).collect(), Some(x.1)),
            );
            if !output_lines.is_empty() && output_offset.is_some() {
                let (r#match, error) = crate::apt::find_apt_get_failure(output_lines);
                if error.is_some() && r#match.is_some() {
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            &lines,
                            r#match.unwrap().offset() + output_offset.unwrap(),
                            Some("direct regex"),
                        )) as Box<dyn Match>),
                        Some(packet.name),
                        error,
                        None,
                    );
                }
            }
            let mut badpkg = None;
            let mut blame = None;
            let mut blame_offset = None;
            for (extra_offset, line) in packet.extra.iter().enumerate() {
                let extra_offset = extra_offset + 1;
                badpkg = line.strip_prefix("badpkg: ");
                if line.starts_with("blame: ") {
                    blame = Some(line);
                    blame_offset = Some(extra_offset);
                }
            }
            let description = if let Some(badpkg) = badpkg {
                format!(
                    "Test {} failed: {}",
                    packet.name,
                    badpkg.trim_end_matches('\n')
                )
            } else {
                format!("Test {} failed", packet.name)
            };

            let error = blame.map(|blame| {
                Box::new(AutopkgtestDepsUnsatisfiable::from_blame_line(blame)) as Box<dyn Problem>
            });
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    summary_offset + packet.offset() + blame_offset.unwrap(),
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(packet.name),
                error,
                Some(description),
            );
        } else {
            let field = Field::Output(packet.name.to_string());
            let (output_lines, output_offset) = test_output.get(&field).map_or_else(
                || (vec![], None),
                |x| (x.0.iter().map(|x| x.as_str()).collect(), Some(x.1)),
            );
            let (r#match, error) = crate::common::find_build_failure_description(output_lines);
            let offset = if r#match.is_none() || output_offset.is_none() {
                summary_offset + packet.offset()
            } else {
                r#match.as_ref().unwrap().offset() + output_offset.unwrap()
            };
            let description = if let Some(r#match) = r#match.as_ref() {
                r#match.line()
            } else if let Some(reason) = packet.reason {
                format!("Test {} failed: {}", packet.name, reason)
            } else {
                format!("Test {} failed", packet.name)
            };
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    offset,
                    Some("direct regex"),
                )) as Box<dyn Match>),
                Some(packet.name),
                error,
                Some(description),
            );
        }
    }

    (None, None, None, None)
}

pub fn find_testbed_setup_failure(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    for (i, line) in lines.enumerate_backward(None) {
        if let Some((_, command, status_code, stderr)) = lazy_regex::regex_captures!(
            r"\[(.*)\] failed \(exit status ([0-9]+), stderr \'(.*)\'\)\n",
            line
        ) {
            if let Some((_, chroot)) =
                lazy_regex::regex_captures!(r"E: (.*): Chroot not found\\n", stderr)
            {
                return (
                    Some(
                        Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                            as Box<dyn Match>,
                    ),
                    Some(Box::new(crate::problems::common::ChrootNotFound {
                        chroot: chroot.to_owned(),
                    }) as Box<dyn Problem>),
                );
            }
            return (
                Some(
                    Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                        as Box<dyn Match>,
                ),
                Some(Box::new(AutopkgtestTestbedSetupFailure {
                    command: command.to_string(),
                    exit_status: status_code.parse().unwrap(),
                    error: stderr.to_string(),
                }) as Box<dyn Problem>),
            );
        }
        if let Some((_, command, stderr_group)) = lazy_regex::regex_captures!(
            r"<VirtSubproc>: failure: \['(.*)'\] unexpectedly produced stderr output `(.*)\n",
            line
        ) {
            if lazy_regex::regex_is_match!(
                r"W: /var/lib/schroot/session/(.*): Failed to stat file: No such file or directory",
                stderr_group
            ) {
                return (
                    Some(
                        Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                            as Box<dyn Match>,
                    ),
                    Some(Box::new(AutopkgtestDepChrootDisappeared) as Box<dyn Problem>),
                );
            }
            return (
                Some(
                    Box::new(SingleLineMatch::from_lines(&lines, i, Some("direct regex")))
                        as Box<dyn Match>,
                ),
                Some(Box::new(AutopkgtestTestbedSetupFailure {
                    command: command.to_string(),
                    exit_status: 1,
                    error: stderr_group.to_string(),
                }) as Box<dyn Problem>),
            );
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use crate::problems::autopkgtest::*;
    use crate::problems::common::*;
    use super::*;

    fn assert_autopkgtest_match(
        lines: Vec<&str>,
        expected_offsets: Vec<usize>,
        expected_testname: Option<&str>,
        expected_error: Option<Box<dyn super::Problem>>,
        expected_description: Option<&str>,
    ) {
        let (r#match, testname, error, description) =
            super::find_autopkgtest_failure_description(lines);
        if !expected_offsets.is_empty() {
            assert_eq!(r#match.as_ref().unwrap().offsets(), expected_offsets);
        } else {
            assert!(r#match.is_none());
        }
        assert_eq!(testname, expected_testname.map(|x| x.to_string()));
        assert_eq!(error, expected_error);
        assert_eq!(description, expected_description.map(|x| x.to_string()));
    }

    #[test]
    fn test_empty() {
        assert_autopkgtest_match(vec![], vec![], None, None, None);
    }

    #[test]
    fn test_no_match() {
        let lines = vec!["blalblala\n"];
        assert_autopkgtest_match(lines, vec![0],Some("blalblala\n"), None, None);
    }

    #[test]
    fn test_unknown_error() {
        assert_autopkgtest_match(
            vec![
                "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "python-bcolz           FAIL some error\n",
            ],
            vec![1],
            Some("python-bcolz"),
            None,
            Some("Test python-bcolz failed: some error"),
        );
    }

    #[test]
    fn test_timed_out() {
        let error = super::AutopkgtestTimedOut;
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "unit-tests           FAIL timed out\n",
        ];
        assert_autopkgtest_match(lines, vec![1], Some("unit-tests"), Some(Box::new(error)), Some("timed out"));
    }

    #[test]
    fn test_deps() {
        let error = AutopkgtestDepsUnsatisfiable(vec![
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmppvupofwl/build-area/bcolz-doc_1.2.1+ds2-4~jan+lint1_all.deb".to_string(),
                ),
                (Some("deb".to_string()), "bcolz-doc".to_string()),
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb".to_string(),
                ),
                (Some("deb".to_string()), "python-bcolz-dbgsym".to_string()),
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb".to_string(),
                ),
                (Some("deb".to_string()), "python-bcolz".to_string()),
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb".to_string(),
                ),
                (Some("deb".to_string()), "python3-bcolz-dbgsym".to_string()),
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb".to_string(),
                ),
                (Some("deb".to_string()), "python3-bcolz".to_string()),
                (
                    None,
                    "/home/janitor/tmp/tmppvupofwl/build-area/bcolz_1.2.1+ds2-4~jan+lint1.dsc".to_string(),
                ),
            ]
        );

        let lines = vec![
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "python-bcolz         FAIL badpkg\n",
                    "blame: arg:/home/janitor/tmp/tmppvupofwl/build-area/bcolz-doc_1.2.1+ds2-4~jan+lint1_all.deb deb:bcolz-doc arg:/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python-bcolz-dbgsym arg:/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python-bcolz arg:/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python3-bcolz-dbgsym arg:/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python3-bcolz /home/janitor/tmp/tmppvupofwl/build-area/bcolz_1.2.1+ds2-4~jan+lint1.dsc\n",
                    "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.\n",
                ];

        assert_autopkgtest_match(lines, vec![2], Some("python-bcolz"), Some(Box::new(error)), Some("Test python-bcolz failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));

        let error = AutopkgtestDepsUnsatisfiable(vec![
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb".to_string(),
                ),
                (Some("deb".to_string()), "cmake-extras".to_string()),
                (
                    None,
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc".to_string(),
                ),
            ]
        );
        let lines = vec![
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "intltool             FAIL badpkg",
                    "blame: arg:/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb deb:cmake-extras /home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc",
                    "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.",
                ];

        assert_autopkgtest_match(lines, vec![2], Some("intltool"), Some(Box::new(error)), Some("Test intltool failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));
    }

    #[test]
    fn test_session_disappeared() {
        let error = AutopkgtestDepChrootDisappeared;

        let lines = vec![
"autopkgtest [22:52:18]: starting date: 2021-04-01\n",
"autopkgtest [22:52:18]: version 5.16\n",
"autopkgtest [22:52:18]: host osuosl167-amd64; command line: /usr/bin/autopkgtest '/tmp/tmpb0o8ai2j/build-area/liquid-dsp_1.2.0+git20210131.9ae84d8-1~jan+deb1_amd64.changes' --no-auto-control -- schroot unstable-amd64-sbuild\n",
"<VirtSubproc>: failure: ['chmod', '1777', '/tmp/autopkgtest.JLqPpH'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2-53ed-4f84-8f0d-2c53ebe71010: Failed to stat file: No such file or directory\n",
"'\n",
"autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n"
        ];
        assert_autopkgtest_match(lines, vec![3], 
                None,
                Some(Box::new(error)),
                Some("<VirtSubproc>: failure: ['chmod', '1777', '/tmp/autopkgtest.JLqPpH'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2-53ed-4f84-8f0d-2c53ebe71010: Failed to stat file: No such file or directory\n")
            );
    }

    #[test]
    fn test_stderr() {
        let error = AutopkgtestStderrFailure("some output".to_string());
        let lines = vec![
            "intltool            FAIL stderr: some output",
            "autopkgtest [20:49:00]: test intltool:  - - - - - - - - - - stderr - - - - - - - - - -",
            "some output",
            "some more output",
            "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary",
            "intltool            FAIL stderr: some output",
        ];

        assert_autopkgtest_match(lines, vec![2], 
                Some("intltool"),
                Some(Box::new(error)),
                Some("Test intltool failed due to unauthorized stderr output: some output"),
        );
        let lines =                 vec![
                    "autopkgtest [20:49:00]: test intltool:  - - - - - - - - - - stderr - - - - - - - - - -",
                    "/tmp/bla: 12: ss: not found",
                    "some more output",
                    "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary",
                    "intltool            FAIL stderr: /tmp/bla: 12: ss: not found",
                ];
        let error = MissingCommand("ss".to_owned());
        assert_autopkgtest_match(lines, vec![1], Some("intltool"), Some(Box::new(error)), Some("/tmp/bla: 12: ss: not found"));

        let lines = vec![
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    r#"command10            FAIL stderr: Can't exec "uptime": No such file or directory at /usr/lib/nagios/plugins/check_uptime line 529."#,
        ];

        let error = MissingCommand("uptime".to_owned());
        assert_autopkgtest_match(
            lines, vec![1], Some("command10"), Some(Box::new(error)), Some(r#"Can't exec "uptime": No such file or directory at /usr/lib/nagios/plugins/check_uptime line 529."#));
    }

    #[test]
    fn test_testbed_failure() {
        let error = AutopkgtestTestbedFailure(
            "sent `copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ /tmp/autopkgtest.output.icg0g8e6/tests-tree/', got `timeout', expected `ok...'".to_owned()
        );
        let lines = vec![
                    "autopkgtest [12:46:18]: ERROR: testbed failure: sent `copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ /tmp/autopkgtest.output.icg0g8e6/tests-tree/', got `timeout', expected `ok...'\n"
                ];

        assert_autopkgtest_match(lines, vec![0], None, Some(Box::new(error)), None);
    }

    #[test]
    fn test_testbed_failure_with_test() {
        let error = AutopkgtestTestbedFailure("testbed auxverb failed with exit code 255".to_owned());

        let lines = vec!["Removing autopkgtest-satdep (0) ...\n",
        "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
        "PHP Fatal error:  Declaration of Wicked_TestCase::setUp() must be compatible with PHPUnit\\Framework\\TestCase::setUp(): void in /tmp/autopkgtest.5ShOBp/build.ViG/src/wicked-2.0.8/test/Wicked/TestCase.php on line 31\n",
"autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n",
"Exiting with 16\n"
        ];
        assert_autopkgtest_match(lines, vec![3], Some("phpunit"), Some(Box::new(error)), None);
    }

    #[test]
    fn test_test_command_failure() {
        let lines = vec![
            "Removing autopkgtest-satdep (0) ...\n",
"autopkgtest [01:30:11]: test command2: phpunit --bootstrap /usr/autoload.php\n",
"autopkgtest [01:30:11]: test command2: [-----------------------\n",
"PHPUnit 8.5.2 by Sebastian Bergmann and contributors.\n",
"\n",
"Cannot open file \"/usr/share/php/Pimple/autoload.php\".\n",
"\n",
"autopkgtest [01:30:12]: test command2: -----------------------]\n",
"autopkgtest [01:30:12]: test command2:  - - - - - - - - - - results - - - - - - - - - -\n",
"command2             FAIL non-zero exit status 1\n",
"autopkgtest [01:30:12]: @@@@@@@@@@@@@@@@@@@@ summary\n",
"command1             PASS\n",
"command2             FAIL non-zero exit status 1\n",
"Exiting with 4\n"
        ];

        let error = MissingFile::new("/usr/share/php/Pimple/autoload.php".into());

        assert_autopkgtest_match(lines, vec![5], Some("command2"), Some(Box::new(error)), Some("Cannot open file \"/usr/share/php/Pimple/autoload.php\".\n"));
    }

    #[test]
    fn test_dpkg_failure() {
        let lines = vec![
            "autopkgtest [19:19:19]: test require: [-----------------------\n",
            "autopkgtest [19:19:20]: test require: -----------------------]\n",
            "autopkgtest [19:19:20]: test require:  - - - - - - - - - - results - - - - - - - - - -\n",
            "require              PASS\n",
            "autopkgtest [19:19:20]: test runtestsuite: preparing testbed\n",
            "Get:1 file:/tmp/autopkgtest.hdIETy/binaries  InRelease\n",
            "Ign:1 file:/tmp/autopkgtest.hdIETy/binaries  InRelease\n",
            "autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/autopkgtest.hdIETy/4-autopkgtest-satdep.deb\" failed with stderr \"W: /var/lib/schroot/session/unstable-amd64-sbuild-7fb1b836-14f9-4709-8584-cbbae284db97: Failed to stat file: No such file or directory\n",
        ];

        let error = AutopkgtestDepChrootDisappeared;

        assert_autopkgtest_match(lines, vec![7], Some("runtestsuite"), Some(Box::new(error)), Some("W: /var/lib/schroot/session/unstable-amd64-sbuild-7fb1b836-14f9-4709-8584-cbbae284db97: Failed to stat file: No such file or directory"));
    }

    #[test]
    fn test_last_stderr_line() {
        let lines = vec![
            "autopkgtest [17:38:49]: test unmunge: [-----------------------\n",
            "munge: Error: Failed to access \"/run/munge/munge.socket.2\": No such file or directory\n",
            "unmunge: Error: No credential specified\n",
            "autopkgtest [17:38:50]: test unmunge: -----------------------]\n",
            "autopkgtest [17:38:50]: test unmunge:  - - - - - - - - - - results - - - - - - - - - -\n",
            "unmunge              FAIL non-zero exit status 2\n",
            "autopkgtest [17:38:50]: test unmunge:  - - - - - - - - - - stderr - - - - - - - - - -\n",
            "munge: Error: Failed to access \"/run/munge/munge.socket.2\": No such file or directory\n",
            "unmunge: Error: No credential specified\n",
            "autopkgtest [17:38:50]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "unmunge              FAIL non-zero exit status 2\n",
            "Exiting with 4\n"
        ];

        assert_autopkgtest_match(lines, vec![10], Some("unmunge"), None, Some("Test unmunge failed: non-zero exit status 2"));
    }

    #[test]
    fn test_python_error_in_output() {
        let lines = vec![
"autopkgtest [14:55:35]: test unit-tests-3: [-----------------------",
" File \"twisted/test/test_log.py\", line 511, in test_getTimezoneOffsetWithout",
"   self._getTimezoneOffsetTest(\"Africa/Johannesburg\", -7200, -7200)",
" File \"twisted/test/test_log.py\", line 460, in _getTimezoneOffsetTest",
"   daylight = time.mktime(localDaylightTuple)",
"builtins.OverflowError: mktime argument out of range",
"-------------------------------------------------------------------------------",
"Ran 12377 tests in 143.490s",
"",
"143.4904797077179 12377 12377 1 0 2352",
"autopkgtest [14:58:01]: test unit-tests-3: -----------------------]",
"autopkgtest [14:58:01]: test unit-tests-3:  - - - - - - - - - - results - - - - - - - - - -",
"unit-tests-3         FAIL non-zero exit status 1",
"autopkgtest [14:58:01]: @@@@@@@@@@@@@@@@@@@@ summary",
"unit-tests-3         FAIL non-zero exit status 1",
"Exiting with 4"
        ];

        assert_autopkgtest_match(lines, vec![5], Some("unit-tests-3"), None, Some("builtins.OverflowError: mktime argument out of range"));
    }

    mod parse_autopkgtest_summary {
        use super::*;

        #[test]
        fn test_empty() {
            assert_eq!(parse_autopkgtest_summary(vec![]), vec![]);
        }

        #[test]
        fn test_single_pass() {
            assert_eq!(
                parse_autopkgtest_summary(vec!["python-bcolz PASS"]),
                vec![Summary {
                    offset: 0,
                    name: "python-bcolz".to_string(),
                    result: TestResult::Pass,
                    reason: None,
                    extra: vec![]
                }]
            );
        }

        #[test]
        fn test_single_fail() {
            assert_eq!(
                parse_autopkgtest_summary(vec!["python-bcolz FAIL some error"]),
                vec![Summary {
                    offset: 0,
                    name: "python-bcolz".to_string(),
                    result: TestResult::Fail,
                    reason: Some("some error".to_string()),
                    extra: vec![]
                }]
            );
        }

        #[test]
        fn test_single_skip() {
            assert_eq!(
                parse_autopkgtest_summary(vec!["python-bcolz SKIP some reason"]),
                vec![Summary {
                    offset: 0,
                    name: "python-bcolz".to_string(),
                    result: TestResult::Skip,
                    reason: Some("some reason".to_string()),
                    extra: vec![]
                }]
            );
        }

        #[test]
        fn test_single_flaky() {
            assert_eq!(
                parse_autopkgtest_summary(vec!["python-bcolz FLAKY some reason"]),
                vec![Summary {
                    offset: 0,
                    name: "python-bcolz".to_string(),
                    result: TestResult::Flaky,
                    reason: Some("some reason".to_string()),
                    extra: vec![]
                }]
            );
        }

        #[test]
        fn test_multiple() {
            assert_eq!(
                parse_autopkgtest_summary(vec![
                    "python-bcolz PASS",
                    "python-bcolz FAIL some error",
                    "python-bcolz SKIP some reason",
                    "python-bcolz FLAKY some reason"
                ]),
                vec![
                    Summary {
                        offset: 0,
                        name: "python-bcolz".to_string(),
                        result: TestResult::Pass,
                        reason: None,
                        extra: vec![]
                    },
                    Summary {
                        offset: 1,
                        name: "python-bcolz".to_string(),
                        result: TestResult::Fail,
                        reason: Some("some error".to_string()),
                        extra: vec![]
                    },
                    Summary {
                        offset: 2,
                        name: "python-bcolz".to_string(),
                        result: TestResult::Skip,
                        reason: Some("some reason".to_string()),
                        extra: vec![]
                    },
                    Summary {
                        offset: 3,
                        name: "python-bcolz".to_string(),
                        result: TestResult::Flaky,
                        reason: Some("some reason".to_string()),
                        extra: vec![]
                    }
                ]
            );
        }
    }

    mod parse_autopkgtest_line {
        #[test]
        fn test_source() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ source "),
                Some(("07:58:03", super::Packet::Source))
            );
        }

        #[test]
        fn test_summary() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary"),
                Some(("07:58:03", super::Packet::Summary))
            );
        }

        #[test]
        fn test_test_begin_output() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests: [-----------------------"),
                Some(("07:58:03", super::Packet::TestBeginOutput("unit-tests")))
            );
        }

        #[test]
        fn test_test_end_output() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests: -----------------------]"),
                Some(("07:58:03", super::Packet::TestEndOutput("unit-tests")))
            );
        }

        #[test]
        fn test_results() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests:  - - - - - - - - - - results - - - - - - - - - -"),
                Some(("07:58:03", super::Packet::Results("unit-tests")))
            );
        }

        #[test]
        fn test_stderr() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests:  - - - - - - - - - - stderr - - - - - - - - - -"),
                Some(("07:58:03", super::Packet::Stderr("unit-tests")))
            );
        }

        #[test]
        fn test_testbed_setup() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests: preparing testbed"),
                Some(("07:58:03", super::Packet::TestbedSetup("unit-tests")))
            );
        }

        #[test]
        fn test_test_output() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test unit-tests: some output"),
                Some(("07:58:03", super::Packet::TestOutput("unit-tests", "some output")))
            );
        }

        #[test]
        fn test_error() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: ERROR: some error"),
                Some(("07:58:03", super::Packet::Error("some error")))
            );
        }
    }


}
