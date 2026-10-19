//! Module for parsing and analyzing autopkgtest logs.
//!
//! This module provides functionality for parsing and analyzing logs from
//! autopkgtest, the Debian automated testing framework. It can identify and
//! extract test results, errors, and other information from autopkgtest logs.

use crate::lines::Lines;
use crate::problems::autopkgtest::*;
use crate::r#match::Error;
use crate::{Failure, Problem};
use std::collections::HashMap;

/// A failure found in an autopkgtest log.
#[derive(Debug)]
pub struct AutopkgtestFailure {
    /// The matched lines and, if identified, the problem.
    pub failure: Failure,
    /// Name of the test that failed, if known.
    pub testname: Option<String>,
    /// Human-readable description of the failure.
    pub description: Option<String>,
}

/// Represents different types of log packets in autopkgtest output.
///
/// Each variant corresponds to a specific type of log entry in autopkgtest output,
/// such as test results, source information, or error messages.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Packet<'a> {
    /// Source information marker in the log.
    Source,

    /// Summary section marker in the log.
    Summary,

    /// Beginning of test output for a specific test.
    TestBeginOutput(&'a str),

    /// End of test output for a specific test.
    TestEndOutput(&'a str),

    /// Test results section for a specific test.
    Results(&'a str),

    /// Standard error output from a specific test.
    Stderr(&'a str),

    /// Testbed setup information for a specific test.
    TestbedSetup(&'a str),

    /// General test output with test name and status.
    TestOutput(&'a str, &'a str),

    /// Error message in the log.
    Error(&'a str),

    /// Other unrecognized log entries.
    Other(&'a str),
}

/// Parses a single line from an autopkgtest log.
///
/// This function extracts the timestamp and message content from a log line
/// and categorizes it into a specific type of packet.
///
/// # Arguments
/// * `line` - The log line to parse
///
/// # Returns
/// An option containing a tuple of (timestamp, packet) if the line is a valid autopkgtest log line,
/// or None if the line doesn't match the expected format
fn parse_autopgktest_line(line: &str) -> Option<(&str, Packet<'_>)> {
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
    } else if let Some((testname, test_status)) = message
        .strip_prefix("test ")
        .and_then(|m| m.trim_end_matches('\n').split_once(": "))
    {
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

/// Represents the result of an individual autopkgtest test.
///
/// This enum represents the possible outcomes of an autopkgtest test case.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestResult {
    /// Test passed successfully.
    Pass,

    /// Test failed.
    Fail,

    /// Test was skipped.
    Skip,

    /// Test produced inconsistent results (sometimes passes, sometimes fails).
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

/// Summary of an autopkgtest test result.
///
/// This structure represents a summary of an individual test result from an autopkgtest run,
/// including the test name, result status, and additional information.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Summary {
    /// The line offset of this summary in the log.
    pub offset: usize,

    /// The name of the test.
    pub name: String,

    /// The result of the test (pass, fail, skip, or flaky).
    pub result: TestResult,

    /// Optional reason for the result, especially useful for failures.
    pub reason: Option<String>,

    /// Additional information about the test result.
    pub extra: Vec<String>,
}

impl Summary {
    /// Gets the line offset of this summary in the log.
    ///
    /// # Returns
    /// The zero-based line offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Gets the line number of this summary in the log.
    ///
    /// # Returns
    /// The one-based line number
    pub fn lineno(&self) -> usize {
        self.offset + 1
    }
}

/// Parses autopkgtest summary lines to extract test results.
///
/// This function analyzes the summary section of an autopkgtest log
/// and extracts information about individual test results.
///
/// # Arguments
/// * `lines` - Log lines from the summary section
///
/// # Returns
/// A vector of `Summary` structures, one for each identified test result
pub fn parse_autopkgtest_summary(lines: &[&str]) -> Vec<Summary> {
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

/// Represents different field types in an autopkgtest log.
///
/// This enum is used internally to categorize different sections of the log
/// based on their content type and associated test.
#[derive(Debug, PartialEq, Eq, Clone, std::hash::Hash)]
enum Field {
    /// General output for a specific test.
    Output(String),

    /// Specific field output for a test.
    SpecificOutput(String, String),

    /// Standard error output for a test.
    Stderr(String),

    /// Results output for a test.
    Results(String),

    /// Testbed preparation output for a test.
    PrepareTestbed(String),

    /// Summary section.
    Summary,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Field::Output(testname) => write!(f, "output for test {}", testname),
            Field::Stderr(testname) => write!(f, "stderr for test {}", testname),
            Field::Results(testname) => write!(f, "results for test {}", testname),
            Field::PrepareTestbed(testname) => write!(f, "testbed setup for test {}", testname),
            Field::SpecificOutput(testname, field) => {
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
            Field::SpecificOutput(testname, _) => Some(testname),
            Field::Summary => None,
        }
    }
}

/// Lines collected for a field.
///
/// Autopkgtest messages inside a field are not collected, so the lines are
/// not necessarily contiguous in the log.
struct FieldOutput<'a> {
    /// Offset of the line following the field header.
    start: usize,
    lines: Vec<&'a str>,
    /// Offset in the log of each entry in `lines`.
    offsets: Vec<usize>,
}

impl<'a> FieldOutput<'a> {
    fn new(start: usize) -> Self {
        Self {
            start,
            lines: vec![],
            offsets: vec![],
        }
    }

    fn push(&mut self, offset: usize, line: &'a str) {
        self.lines.push(line);
        self.offsets.push(offset);
    }
}

/// Find the autopkgtest failure in output.
///
/// Returns an error if a matcher could not interpret the log.
pub fn find_autopkgtest_failure_description(
    lines: &[&str],
) -> Result<Option<AutopkgtestFailure>, Error> {
    let mut test_output: HashMap<Field, FieldOutput> = HashMap::new();
    let mut current_field: Option<Field> = None;
    let mut it = lines.iter().enumerate().peekable();
    while let Some((i, line)) = it.next() {
        match parse_autopgktest_line(line) {
            Some((_, Packet::Source)) => {}
            Some((_, Packet::Other(_))) => {}
            Some((_, Packet::Error(msg))) => {
                let msg = if msg.starts_with('"') && msg.chars().filter(|x| *x == '"').count() == 1
                {
                    let mut sublines = vec![msg];
                    let mut terminated = false;
                    for (_, line) in it.by_ref() {
                        sublines.push(line);
                        if line.chars().filter(|x| x == &'"').count() == 1 {
                            terminated = true;
                            break;
                        }
                    }
                    if !terminated {
                        log::warn!(
                            "unterminated quoted autopkgtest error at line {}: {}",
                            i + 1,
                            msg
                        );
                    }
                    // Continuation lines still carry their line terminator.
                    sublines
                        .iter()
                        .map(|l| l.trim_end_matches('\n'))
                        .collect::<Vec<_>>()
                        .join("\n")
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
                        return Ok(Some(AutopkgtestFailure {
                            failure: Failure::at_line(lines, i, "direct regex", error),
                            testname: last_test,
                            description: Some(stderr.to_owned()),
                        }));
                    }
                }
                if let Some((_, testbed_failure_reason)) =
                    lazy_regex::regex_captures!(r"testbed failure: (.*)", &msg)
                {
                    if let (Some(testname), "testbed auxverb failed with exit code 255") =
                        (last_test.as_deref(), testbed_failure_reason)
                    {
                        let field = Field::Output(testname.to_owned());
                        if let Some(Failure {
                            r#match: m,
                            problem: Some(error),
                        }) = crate::common::find_build_failure_description(
                            test_output.get(&field).map_or(&[], |x| &x.lines),
                        )? {
                            let description = m.line().to_string();
                            return Ok(Some(AutopkgtestFailure {
                                failure: Failure::at_line(
                                    lines,
                                    test_output[&field].offsets[m.offset()],
                                    "direct regex",
                                    Some(error),
                                ),
                                testname: last_test.map(|x| x.to_owned()),
                                description: Some(description),
                            }));
                        }
                    }

                    if testbed_failure_reason
                        == "sent `auxverb_debug_fail', got `copy-failed', expected `ok...'"
                    {
                        if let Some(failure) = crate::common::find_build_failure_description(lines)?
                            .filter(|f| f.problem.is_some())
                        {
                            let description = failure.r#match.line().to_string();
                            return Ok(Some(AutopkgtestFailure {
                                failure,
                                testname: last_test,
                                description: Some(description),
                            }));
                        }
                    }

                    if testbed_failure_reason == "cannot send to testbed: [Errno 32] Broken pipe" {
                        if let Some(failure) =
                            find_testbed_setup_failure(lines)?.filter(|f| f.problem.is_some())
                        {
                            let description = failure.r#match.line().to_string();
                            return Ok(Some(AutopkgtestFailure {
                                failure,
                                testname: last_test,
                                description: Some(description),
                            }));
                        }
                    }
                    if testbed_failure_reason == "apt repeatedly failed to download packages" {
                        if let Some(failure) =
                            crate::apt::find_apt_get_failure(lines).filter(|f| f.problem.is_some())
                        {
                            let description = failure.r#match.line().to_string();
                            return Ok(Some(AutopkgtestFailure {
                                failure,
                                testname: last_test,
                                description: Some(description),
                            }));
                        }
                        return Ok(Some(AutopkgtestFailure {
                            failure: Failure::at_line(
                                lines,
                                i,
                                "direct regex",
                                Some(Box::new(crate::problems::debian::AptFetchFailure {
                                    url: None,
                                    error: testbed_failure_reason.to_owned(),
                                }) as Box<dyn Problem>),
                            ),
                            testname: last_test,
                            description: None,
                        }));
                    }
                    return Ok(Some(AutopkgtestFailure {
                        failure: Failure::at_line(
                            lines,
                            i,
                            "direct regex",
                            Some(Box::new(AutopkgtestTestbedFailure {
                                reason: testbed_failure_reason.to_owned(),
                            }) as Box<dyn Problem>),
                        ),
                        testname: last_test.map(|x| x.to_owned()),
                        description: None,
                    }));
                }
                if let Some((_, pkg)) =
                    lazy_regex::regex_captures!(r"erroneous package: (.*)", &msg)
                {
                    if let Some(failure) =
                        crate::common::find_build_failure_description(&lines[..i])?
                            .filter(|f| f.problem.is_some())
                    {
                        let description = failure.r#match.line().to_string();
                        return Ok(Some(AutopkgtestFailure {
                            failure,
                            testname: last_test.map(|x| x.to_owned()),
                            description: Some(description),
                        }));
                    }
                    return Ok(Some(AutopkgtestFailure {
                        failure: Failure::at_line(
                            lines,
                            i,
                            "direct regex",
                            Some(Box::new(AutopkgtestErroneousPackage {
                                reason: pkg.to_string(),
                            }) as Box<dyn Problem>),
                        ),
                        testname: last_test.map(|x| x.to_owned()),
                        description: None,
                    }));
                }
                if msg == "unexpected error:" {
                    if let Some(failure) =
                        crate::common::find_build_failure_description(&lines[(i + 1)..])?
                            .filter(|f| f.problem.is_some())
                    {
                        let description = failure.r#match.line().to_string();
                        return Ok(Some(AutopkgtestFailure {
                            failure,
                            testname: last_test.map(|x| x.to_owned()),
                            description: Some(description),
                        }));
                    }
                }
                if let Some(current_field) = current_field.as_ref() {
                    if let Some(Failure {
                        r#match: m,
                        problem: Some(e),
                    }) = crate::apt::find_apt_get_failure(
                        &test_output.get(current_field).unwrap().lines,
                    ) {
                        if let Some(test_output_entry) = test_output.get(current_field) {
                            let description = m.line().to_string();
                            return Ok(Some(AutopkgtestFailure {
                                failure: Failure::at_line(
                                    lines,
                                    test_output_entry.offsets[m.offset()],
                                    "direct regex",
                                    Some(e),
                                ),
                                testname: last_test.map(|x| x.to_owned()),
                                description: Some(description),
                            }));
                        }
                    }
                }
                if msg == "autopkgtest"
                    && lines
                        .get(i + 1)
                        .is_some_and(|l| l.trim_end() == ": error cleaning up:")
                {
                    let description = i.checked_sub(1).map(|p| lines[p].trim_end().to_owned());
                    // Point at the start of the current field's output, or
                    // at the ERROR line itself when outside of any field.
                    let offset = current_field.as_ref().map_or(i, |f| test_output[f].start);
                    return Ok(Some(AutopkgtestFailure {
                        failure: Failure::at_line(
                            lines,
                            offset,
                            "direct regex",
                            Some(Box::new(AutopkgtestTimedOut) as Box<dyn Problem>),
                        ),
                        testname: last_test.map(|x| x.to_owned()),
                        description,
                    }));
                }
                return Ok(Some(AutopkgtestFailure {
                    failure: Failure::at_line(lines, i, "direct regex", None),
                    testname: last_test.map(|x| x.to_owned()),
                    description: Some(msg),
                }));
            }
            Some((_, Packet::Summary)) => {
                current_field = Some(Field::Summary);
                test_output.insert(current_field.clone().unwrap(), FieldOutput::new(i + 1));
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
                            Some(Field::SpecificOutput(testname.to_owned(), field.to_owned()));
                    }
                    _ => {}
                }
                if test_output.contains_key(current_field.as_ref().unwrap()) {
                    log::warn!(
                        "duplicate output fields for {}",
                        current_field.as_ref().unwrap()
                    );
                }
                test_output.insert(current_field.clone().unwrap(), FieldOutput::new(i + 1));
            }
            None => {
                if let Some(current_field) = current_field.as_ref() {
                    test_output
                        .entry(current_field.clone())
                        .or_insert_with(|| FieldOutput::new(i))
                        .push(i, line);
                }
            }
        }
    }

    let Some(summary) = test_output.get(&Field::Summary) else {
        let Some(offset) = lines.iter().rposition(|l| !l.trim().is_empty()) else {
            return Ok(None);
        };
        return Ok(Some(AutopkgtestFailure {
            failure: Failure::at_line(lines, offset, "direct regex", None),
            testname: Some(lines[offset].to_string()),
            description: None,
        }));
    };
    for packet in parse_autopkgtest_summary(&summary.lines) {
        if [TestResult::Pass, TestResult::Skip].contains(&packet.result) {
            continue;
        }
        let packet_offset = summary.offsets[packet.offset()];
        assert!([TestResult::Fail, TestResult::Flaky].contains(&packet.result));
        if packet.reason.as_deref() == Some("timed out") {
            let error = Some(Box::new(AutopkgtestTimedOut) as Box<dyn Problem>);
            return Ok(Some(AutopkgtestFailure {
                failure: Failure::at_line(lines, packet_offset, "direct regex", error),
                testname: Some(packet.name),
                description: packet.reason,
            }));
        } else if let Some(output) = packet
            .reason
            .as_ref()
            .and_then(|x| x.strip_prefix("stderr: "))
        {
            let field = Field::Stderr(packet.name.to_string());
            let stderr = test_output.get(&field);
            let stderr_lines = stderr.map_or(&[][..], |x| &x.lines[..]);
            let description;
            let offset;
            let r#match;
            let mut error;
            if !stderr_lines.is_empty() {
                (r#match, error) =
                    Failure::split(crate::common::find_build_failure_description(stderr_lines)?);
                if let (Some(m), Some(stderr)) = (r#match.as_ref(), stderr) {
                    offset = Some(stderr.offsets[m.offset()]);
                    description = Some(m.line().to_string());
                } else if stderr_lines.len() == 1
                    && lazy_regex::regex_is_match!(
                        r"QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to \'(.*)\'",
                        &stderr_lines[0],
                    )
                {
                    error = Some(Box::new(XDGRunTimeNotSet) as Box<dyn Problem>);
                    description = Some(stderr_lines[0].to_string());
                    offset = stderr.map(|x| x.offsets[0]);
                } else {
                    offset = stderr.map(|x| x.start);
                    description = None;
                }
            } else {
                (r#match, error) =
                    Failure::split(crate::common::find_build_failure_description(&[output])?);

                (offset, description) = if let Some(r#match) = r#match.as_ref() {
                    (
                        Some(packet_offset + r#match.offset()),
                        Some(r#match.line().to_string()),
                    )
                } else {
                    (None, None)
                };
            }
            let offset = offset.unwrap_or(packet_offset);
            let error = error.unwrap_or_else(|| {
                Box::new(AutopkgtestStderrFailure {
                    stderr_line: output.to_owned(),
                })
            });
            let description = description.unwrap_or_else(|| {
                format!(
                    "Test {} failed due to unauthorized stderr output: {}",
                    packet.name, output
                )
            });
            return Ok(Some(AutopkgtestFailure {
                failure: Failure::at_line(lines, offset, "direct regex", Some(error)),
                testname: Some(packet.name),
                description: Some(description),
            }));
        } else if packet.reason.as_deref() == Some("badpkg") {
            let field = Field::Output(packet.name.to_string());
            if let Some(output) = test_output.get(&field) {
                if let Some(Failure {
                    r#match: m,
                    problem: Some(e),
                }) = crate::apt::find_apt_get_failure(&output.lines)
                {
                    return Ok(Some(AutopkgtestFailure {
                        failure: Failure::at_line(
                            lines,
                            output.offsets[m.offset()],
                            "direct regex",
                            Some(e),
                        ),
                        testname: Some(packet.name),
                        description: None,
                    }));
                }
            }
            let badpkg = packet
                .extra
                .iter()
                .find_map(|line| line.strip_prefix("badpkg: "));
            // A blame line that names nothing gives no information on what
            // was unsatisfiable.
            let blame = packet
                .extra
                .iter()
                .enumerate()
                .find(|(_, line)| line.starts_with("blame:"))
                .map(|(i, line)| (i, AutopkgtestDepsUnsatisfiable::from_blame_line(line)))
                .filter(|(_, blame)| !blame.args.is_empty());
            // Point at the blame line, falling back to the badpkg line and
            // then the summary line itself.
            let extra_offset = blame
                .as_ref()
                .map(|(i, _)| *i)
                .or_else(|| {
                    packet
                        .extra
                        .iter()
                        .position(|line| line.starts_with("badpkg: "))
                })
                .map_or(0, |i| i + 1);
            let description = if let Some(badpkg) = badpkg {
                format!(
                    "Test {} failed: {}",
                    packet.name,
                    badpkg.trim_end_matches('\n')
                )
            } else {
                format!("Test {} failed", packet.name)
            };

            let error = blame.map(|(_, blame)| Box::new(blame) as Box<dyn Problem>);
            return Ok(Some(AutopkgtestFailure {
                failure: Failure::at_line(
                    lines,
                    summary.offsets[packet.offset() + extra_offset],
                    "direct regex",
                    error,
                ),
                testname: Some(packet.name),
                description: Some(description),
            }));
        } else {
            let field = Field::Output(packet.name.to_string());
            let output = test_output.get(&field);
            let (r#match, error) = Failure::split(crate::common::find_build_failure_description(
                output.map_or(&[], |x| &x.lines),
            )?);
            let offset = match (r#match.as_ref(), output) {
                (Some(m), Some(output)) => output.offsets[m.offset()],
                _ => packet_offset,
            };
            let description = if let Some(r#match) = r#match.as_ref() {
                r#match.line().to_string()
            } else if let Some(reason) = packet.reason {
                format!("Test {} failed: {}", packet.name, reason)
            } else {
                format!("Test {} failed", packet.name)
            };
            return Ok(Some(AutopkgtestFailure {
                failure: Failure::at_line(lines, offset, "direct regex", error),
                testname: Some(packet.name),
                description: Some(description),
            }));
        }
    }

    Ok(None)
}

/// Searches for testbed setup failures in autopkgtest logs.
///
/// This function analyzes log lines to identify errors that occurred during
/// the autopkgtest testbed setup phase, such as chroot not found errors.
///
/// # Arguments
/// * `lines` - Log lines to analyze
///
/// # Returns
/// The failure, if one was found, or an error if a matching line could not
/// be interpreted
pub fn find_testbed_setup_failure(lines: &[&str]) -> Result<Option<Failure>, Error> {
    for (i, line) in lines.enumerate_backward(None) {
        // stderr is a Python repr(), which is double-quoted if the string
        // contains ' but no ".
        if let Some((_, command, status_code, stderr)) = lazy_regex::regex_captures!(
            r#"\[(.*)\] failed \(exit status (-?[0-9]+), stderr ('.*'|".*")\)\n"#,
            line
        ) {
            let stderr = &stderr[1..stderr.len() - 1];
            if let Some((_, chroot)) =
                lazy_regex::regex_captures!(r"E: (.*): Chroot not found\\n", stderr)
            {
                return Ok(Some(Failure::at_line(
                    lines,
                    i,
                    "direct regex",
                    Some(Box::new(crate::problems::common::ChrootNotFound {
                        chroot: chroot.to_owned(),
                    }) as Box<dyn Problem>),
                )));
            }
            return Ok(Some(Failure::at_line(
                lines,
                i,
                "direct regex",
                Some(Box::new(AutopkgtestTestbedSetupFailure {
                    command: command.to_string(),
                    exit_status: status_code
                        .parse()
                        .map_err(|e| Error::from(e).or_offset(i))?,
                    error: stderr.to_string(),
                }) as Box<dyn Problem>),
            )));
        }
        if let Some((_, command, stderr_group)) = lazy_regex::regex_captures!(
            r"<VirtSubproc>: failure: \['(.*)'\] unexpectedly produced stderr output `(.*)\n",
            line
        ) {
            if lazy_regex::regex_is_match!(
                r"W: /var/lib/schroot/session/(.*): Failed to stat file: No such file or directory",
                stderr_group
            ) {
                return Ok(Some(Failure::at_line(
                    lines,
                    i,
                    "direct regex",
                    Some(Box::new(AutopkgtestDepChrootDisappeared) as Box<dyn Problem>),
                )));
            }
            return Ok(Some(Failure::at_line(
                lines,
                i,
                "direct regex",
                Some(Box::new(AutopkgtestTestbedSetupFailure {
                    command: command.to_string(),
                    exit_status: 1,
                    error: stderr_group.to_string(),
                }) as Box<dyn Problem>),
            )));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::problems::common::*;

    fn assert_autopkgtest_match(
        lines: Vec<&str>,
        expected_offsets: Vec<usize>,
        expected_testname: Option<&str>,
        expected_error: Option<Box<dyn super::Problem>>,
        expected_description: Option<&str>,
    ) {
        let (r#match, testname, error, description) =
            match super::find_autopkgtest_failure_description(&lines).unwrap() {
                Some(f) => (
                    Some(f.failure.r#match),
                    f.testname,
                    f.failure.problem,
                    f.description,
                ),
                None => (None, None, None, None),
            };
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
        assert_autopkgtest_match(lines, vec![0], Some("blalblala\n"), None, None);
    }

    #[test]
    fn test_no_match_trailing_blank_lines() {
        let lines = vec!["first\n", "blalblala\n", "\n", "  \n"];
        assert_autopkgtest_match(lines, vec![1], Some("blalblala\n"), None, None);
        assert_autopkgtest_match(vec!["\n", " \n"], vec![], None, None, None);
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
    fn test_propagates_match_error() {
        let error = super::find_autopkgtest_failure_description(&[
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "unit-tests           FAIL stderr: E: Build killed with signal TERM after 99999999999999999999 minutes of inactivity\n",
        ])
        .unwrap_err();
        assert_eq!(error.message(), "invalid integer");
        assert_eq!(error.offset(), Some(0));
    }

    #[test]
    fn test_testbed_setup_exit_status_overflow() {
        let error = super::find_autopkgtest_failure_description(&[
            "[schroot -c foo] failed (exit status 99999999999, stderr 'boom')\n",
            "autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n",
        ])
        .unwrap_err();
        assert_eq!(error.message(), "invalid integer");
        assert_eq!(error.offset(), Some(0));
    }

    #[test]
    fn test_testbed_setup_double_quoted_stderr() {
        let failure = super::find_testbed_setup_failure(&[
            "['schroot', '-c', 'foo'] failed (exit status 1, stderr \"E: can't find session\\n\")\n",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(failure.r#match.offsets(), vec![0]);
        assert_eq!(
            failure.problem,
            Some(Box::new(AutopkgtestTestbedSetupFailure {
                command: "'schroot', '-c', 'foo'".to_owned(),
                exit_status: 1,
                error: "E: can't find session\\n".to_owned(),
            }) as Box<dyn Problem>)
        );
    }

    #[test]
    fn test_testbed_setup_negative_exit_status() {
        let failure = super::find_testbed_setup_failure(&[
            "['schroot', '-c', 'foo'] failed (exit status -9, stderr '')\n",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(failure.r#match.offsets(), vec![0]);
        assert_eq!(
            failure.problem,
            Some(Box::new(AutopkgtestTestbedSetupFailure {
                command: "'schroot', '-c', 'foo'".to_owned(),
                exit_status: -9,
                error: "".to_owned(),
            }) as Box<dyn Problem>)
        );
    }

    #[test]
    fn test_timed_out() {
        let error = super::AutopkgtestTimedOut;
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "unit-tests           FAIL timed out\n",
        ];
        assert_autopkgtest_match(
            lines,
            vec![1],
            Some("unit-tests"),
            Some(Box::new(error)),
            Some("timed out"),
        );
    }

    #[test]
    fn test_deps() {
        let error = AutopkgtestDepsUnsatisfiable { args: vec![
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
            ] };

        let lines = vec![
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "python-bcolz         FAIL badpkg\n",
                    "blame: arg:/home/janitor/tmp/tmppvupofwl/build-area/bcolz-doc_1.2.1+ds2-4~jan+lint1_all.deb deb:bcolz-doc arg:/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python-bcolz-dbgsym arg:/home/janitor/tmp/tmppvupofwl/build-area/python-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python-bcolz arg:/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz-dbgsym_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python3-bcolz-dbgsym arg:/home/janitor/tmp/tmppvupofwl/build-area/python3-bcolz_1.2.1+ds2-4~jan+lint1_amd64.deb deb:python3-bcolz /home/janitor/tmp/tmppvupofwl/build-area/bcolz_1.2.1+ds2-4~jan+lint1.dsc\n",
                    "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.\n",
                ];

        assert_autopkgtest_match(lines, vec![2], Some("python-bcolz"), Some(Box::new(error)), Some("Test python-bcolz failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));

        let error = AutopkgtestDepsUnsatisfiable { args: vec![
                (
                    Some("arg".to_string()),
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb".to_string(),
                ),
                (Some("deb".to_string()), "cmake-extras".to_string()),
                (
                    None,
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc".to_string(),
                ),
            ] };
        let lines = vec![
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                    "intltool             FAIL badpkg",
                    "blame: arg:/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan+unchanged1_all.deb deb:cmake-extras /home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc",
                    "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.",
                ];

        assert_autopkgtest_match(lines, vec![2], Some("intltool"), Some(Box::new(error)), Some("Test intltool failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));
    }

    #[test]
    fn test_deps_badpkg_before_blame() {
        let error = AutopkgtestDepsUnsatisfiable {
            args: vec![
                (Some("deb".to_string()), "cmake-extras".to_string()),
                (
                    None,
                    "/home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc"
                        .to_string(),
                ),
            ],
        };
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "intltool             FAIL badpkg\n",
            "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.\n",
            "blame: deb:cmake-extras /home/janitor/tmp/tmpgbn5jhou/build-area/cmake-extras_1.3+17.04.20170310-6~jan.dsc\n",
        ];

        assert_autopkgtest_match(lines, vec![3], Some("intltool"), Some(Box::new(error)), Some("Test intltool failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));
    }

    #[test]
    fn test_deps_badpkg_without_blame() {
        // autopkgtest emits "blame: " with nothing blamed; trailing whitespace
        // is often stripped from stored logs.
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "intltool             FAIL badpkg\n",
            "blame:\n",
            "badpkg: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U.\n",
        ];

        assert_autopkgtest_match(lines, vec![3], Some("intltool"), None, Some("Test intltool failed: Test dependencies are unsatisfiable. A common reason is that your testbed is out of date with respect to the archive, and you need to use a current testbed or run apt-get update or use -U."));
    }

    #[test]
    fn test_deps_badpkg_empty_blame_unstripped() {
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "intltool             FAIL badpkg\n",
            "blame: \n",
            "badpkg: Test dependencies are unsatisfiable.\n",
        ];

        assert_autopkgtest_match(
            lines,
            vec![3],
            Some("intltool"),
            None,
            Some("Test intltool failed: Test dependencies are unsatisfiable."),
        );
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
        assert_autopkgtest_match(lines, vec![3], None, Some(Box::new(error)), Some("<VirtSubproc>: failure: ['chmod', '1777', '/tmp/autopkgtest.JLqPpH'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2-53ed-4f84-8f0d-2c53ebe71010: Failed to stat file: No such file or directory\n")
            );
    }

    #[test]
    fn test_stderr() {
        let error = AutopkgtestStderrFailure {
            stderr_line: "some output".to_string(),
        };
        let lines = vec![
            "intltool            FAIL stderr: some output",
            "autopkgtest [20:49:00]: test intltool:  - - - - - - - - - - stderr - - - - - - - - - -",
            "some output",
            "some more output",
            "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary",
            "intltool            FAIL stderr: some output",
        ];

        assert_autopkgtest_match(
            lines,
            vec![2],
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
        let error = MissingCommand {
            command: "ss".to_owned(),
        };
        assert_autopkgtest_match(
            lines,
            vec![1],
            Some("intltool"),
            Some(Box::new(error)),
            Some("/tmp/bla: 12: ss: not found"),
        );

        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            r#"command10            FAIL stderr: Can't exec "uptime": No such file or directory at /usr/lib/nagios/plugins/check_uptime line 529."#,
        ];

        let error = MissingCommand {
            command: "uptime".to_owned(),
        };
        assert_autopkgtest_match(
            lines,
            vec![1],
            Some("command10"),
            Some(Box::new(error)),
            Some(
                r#"Can't exec "uptime": No such file or directory at /usr/lib/nagios/plugins/check_uptime line 529."#,
            ),
        );
    }

    #[test]
    fn test_testbed_failure() {
        let error = AutopkgtestTestbedFailure { reason: "sent `copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ /tmp/autopkgtest.output.icg0g8e6/tests-tree/', got `timeout', expected `ok...'".to_owned() };
        let lines = vec![
                    "autopkgtest [12:46:18]: ERROR: testbed failure: sent `copyup /tmp/autopkgtest.9IStGJ/build.0Pm/src/ /tmp/autopkgtest.output.icg0g8e6/tests-tree/', got `timeout', expected `ok...'\n"
                ];

        assert_autopkgtest_match(lines, vec![0], None, Some(Box::new(error)), None);
    }

    #[test]
    fn test_testbed_failure_with_test() {
        let error = AutopkgtestTestbedFailure {
            reason: "testbed auxverb failed with exit code 255".to_owned(),
        };

        let lines = vec!["Removing autopkgtest-satdep (0) ...\n",
        "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
        "PHP Fatal error:  Declaration of Wicked_TestCase::setUp() must be compatible with PHPUnit\\Framework\\TestCase::setUp(): void in /tmp/autopkgtest.5ShOBp/build.ViG/src/wicked-2.0.8/test/Wicked/TestCase.php on line 31\n",
"autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n",
"Exiting with 16\n"
        ];
        assert_autopkgtest_match(lines, vec![3], Some("phpunit"), Some(Box::new(error)), None);
    }

    #[test]
    fn test_testbed_failure_in_summary() {
        let error = AutopkgtestTestbedFailure {
            reason: "testbed auxverb failed with exit code 255".to_owned(),
        };
        let lines = vec![
            "autopkgtest [06:59:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "phpunit              FAIL non-zero exit status 1\n",
            "autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n",
        ];
        assert_autopkgtest_match(lines, vec![2], None, Some(Box::new(error)), None);
    }

    #[test]
    fn test_error_cleaning_up_outside_test() {
        let lines = vec![
            "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
            "running tests\n",
            "autopkgtest [06:59:00]: test phpunit: -----------------------]\n",
            "timed out\n",
            "autopkgtest [06:59:01]: ERROR: autopkgtest\n",
            ": error cleaning up:\n",
        ];
        assert_autopkgtest_match(
            lines,
            vec![4],
            None,
            Some(Box::new(AutopkgtestTimedOut)),
            Some("timed out"),
        );
    }

    #[test]
    fn test_error_cleaning_up() {
        let lines = vec![
            "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
            "running tests\n",
            "timed out\n",
            "autopkgtest [06:59:01]: ERROR: autopkgtest\n",
            ": error cleaning up:\n",
        ];
        assert_autopkgtest_match(
            lines,
            vec![1],
            Some("phpunit"),
            Some(Box::new(AutopkgtestTimedOut)),
            Some("timed out"),
        );
    }

    #[test]
    fn test_error_autopkgtest_last_line() {
        let lines = vec![
            "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
            "running tests\n",
            "autopkgtest [06:59:01]: ERROR: autopkgtest\n",
        ];
        assert_autopkgtest_match(lines, vec![2], Some("phpunit"), None, Some("autopkgtest"));
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

        assert_autopkgtest_match(
            lines,
            vec![5],
            Some("command2"),
            Some(Box::new(error)),
            Some("Cannot open file \"/usr/share/php/Pimple/autoload.php\".\n"),
        );
    }

    #[test]
    fn test_test_command_failure_interleaved_message() {
        let lines = vec![
            "autopkgtest [01:30:11]: test command2: [-----------------------\n",
            "PHPUnit 8.5.2 by Sebastian Bergmann and contributors.\n",
            "autopkgtest [01:30:11]: some unrelated message\n",
            "Cannot open file \"/usr/share/php/Pimple/autoload.php\".\n",
            "autopkgtest [01:30:12]: test command2: -----------------------]\n",
            "autopkgtest [01:30:12]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "command2             FAIL non-zero exit status 1\n",
        ];

        assert_autopkgtest_match(
            lines,
            vec![3],
            Some("command2"),
            Some(Box::new(MissingFile::new(
                "/usr/share/php/Pimple/autoload.php".into(),
            ))),
            Some("Cannot open file \"/usr/share/php/Pimple/autoload.php\".\n"),
        );
    }

    #[test]
    fn test_summary_interleaved_message() {
        let lines = vec![
            "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
            "autopkgtest [07:58:03]: some unrelated message\n",
            "unit-tests           FAIL timed out\n",
        ];
        assert_autopkgtest_match(
            lines,
            vec![2],
            Some("unit-tests"),
            Some(Box::new(super::AutopkgtestTimedOut)),
            Some("timed out"),
        );
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
    fn test_multiline_quoted_error() {
        let lines = vec![
            "autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/x.deb\n",
            "first\n",
            "second\"\n",
        ];
        assert_autopkgtest_match(
            lines,
            vec![0],
            None,
            None,
            Some("\"dpkg --unpack /tmp/x.deb\nfirst\nsecond\""),
        );
    }

    #[test]
    fn test_multiline_quoted_error_trailing_line() {
        let lines = vec![
            "autopkgtest [10:00:00]: ERROR: \"some command",
            "continued here",
            "end\"",
            "trailing",
        ];
        assert_autopkgtest_match(
            lines,
            vec![0],
            None,
            None,
            Some("\"some command\ncontinued here\nend\""),
        );
    }

    #[test]
    fn test_unterminated_quoted_error() {
        let lines = vec![
            "autopkgtest [10:00:00]: ERROR: \"some command",
            "continued here",
        ];
        assert_autopkgtest_match(
            lines,
            vec![0],
            None,
            None,
            Some("\"some command\ncontinued here"),
        );
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

        assert_autopkgtest_match(
            lines,
            vec![10],
            Some("unmunge"),
            None,
            Some("Test unmunge failed: non-zero exit status 2"),
        );
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

        assert_autopkgtest_match(
            lines,
            vec![5],
            Some("unit-tests-3"),
            None,
            Some("builtins.OverflowError: mktime argument out of range"),
        );
    }

    mod parse_autopkgtest_summary {
        use super::*;

        #[test]
        fn test_empty() {
            assert_eq!(parse_autopkgtest_summary(&[]), vec![]);
        }

        #[test]
        fn test_single_pass() {
            assert_eq!(
                parse_autopkgtest_summary(&["python-bcolz PASS"]),
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
                parse_autopkgtest_summary(&["python-bcolz FAIL some error"]),
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
                parse_autopkgtest_summary(&["python-bcolz SKIP some reason"]),
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
                parse_autopkgtest_summary(&["python-bcolz FLAKY some reason"]),
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
                parse_autopkgtest_summary(&[
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
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ source "
                ),
                Some(("07:58:03", super::Packet::Source))
            );
        }

        #[test]
        fn test_summary() {
            assert_eq!(
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary"
                ),
                Some(("07:58:03", super::Packet::Summary))
            );
        }

        #[test]
        fn test_test_begin_output() {
            assert_eq!(
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: test unit-tests: [-----------------------"
                ),
                Some(("07:58:03", super::Packet::TestBeginOutput("unit-tests")))
            );
        }

        #[test]
        fn test_test_end_output() {
            assert_eq!(
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: test unit-tests: -----------------------]"
                ),
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
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: test unit-tests: preparing testbed"
                ),
                Some(("07:58:03", super::Packet::TestbedSetup("unit-tests")))
            );
        }

        #[test]
        fn test_test_output() {
            assert_eq!(
                super::parse_autopgktest_line(
                    "autopkgtest [07:58:03]: test unit-tests: some output"
                ),
                Some((
                    "07:58:03",
                    super::Packet::TestOutput("unit-tests", "some output")
                ))
            );
        }

        #[test]
        fn test_error() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: ERROR: some error"),
                Some(("07:58:03", super::Packet::Error("some error")))
            );
        }

        #[test]
        fn test_test_without_status() {
            assert_eq!(
                super::parse_autopgktest_line("autopkgtest [07:58:03]: test command1\n"),
                Some(("07:58:03", super::Packet::Other("test command1")))
            );
        }
    }

    fn describe(lines: &[&str]) -> serde_json::Value {
        match find_autopkgtest_failure_description(lines) {
            Ok(f) => f.map_or(serde_json::Value::Null, |f| {
                serde_json::json!({
                    "failure": crate::failure_json(Some(&f.failure)),
                    "testname": f.testname,
                    "description": f.description,
                })
            }),
            Err(e) => serde_json::json!({"error": e.to_string()}),
        }
    }

    #[test]
    fn test_describe_no_summary() {
        assert_eq!(describe(&[]), serde_json::json!(null));
        assert_eq!(
            describe(&["first\n", "last\n", " \n"]),
            serde_json::json!({"description":null,"failure":{"lines":["last\n"],"offsets":[1],"origin":"direct regex","problem":null},"testname":"last\n"}),
        );
    }

    #[test]
    fn test_describe_chroot_disappeared() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test runtestsuite: preparing testbed\n",
                "Get:1 file:/tmp/autopkgtest.hdIETy/binaries  InRelease\n",
                "autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/foo.deb\" failed with stderr \"W: /var/lib/schroot/session/unstable-amd64-sbuild-7fb1b836: Failed to stat file: No such file or directory\n",
            ]),
            serde_json::json!({"description":"W: /var/lib/schroot/session/unstable-amd64-sbuild-7fb1b836: Failed to stat file: No such file or directory","failure":{"lines":["autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/foo.deb\" failed with stderr \"W: /var/lib/schroot/session/unstable-amd64-sbuild-7fb1b836: Failed to stat file: No such file or directory\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{},"kind":"testbed-chroot-disappeared"}},"testname":"runtestsuite"}),
        );
    }

    #[test]
    fn test_describe_quoted_multiline_error() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test foo: preparing testbed\n",
                "autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/foo.deb\n",
                "more\n",
                "end\" failed\n",
                "trailing\n",
            ]),
            serde_json::json!({"description":"\"dpkg --unpack /tmp/foo.deb\nmore\nend\" failed","failure":{"lines":["autopkgtest [19:19:23]: ERROR: \"dpkg --unpack /tmp/foo.deb\n"],"offsets":[1],"origin":"direct regex","problem":null},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_auxverb_failure_with_output_problem() {
        assert_eq!(
            describe(&[
                "Removing autopkgtest-satdep (0) ...\n",
                "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
                "something\n",
                "/tmp/bla: 12: ss: not found\n",
                "autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[3],"origin":"direct regex","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":"phpunit"}),
        );
    }

    #[test]
    fn test_describe_auxverb_failure_without_output_problem() {
        assert_eq!(
            describe(&[
                "autopkgtest [06:59:00]: test phpunit: [-----------------------\n",
                "something\n",
                "autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["autopkgtest [06:59:01]: ERROR: testbed failure: testbed auxverb failed with exit code 255\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"reason":"testbed auxverb failed with exit code 255"},"kind":"testbed-failure"}},"testname":"phpunit"}),
        );
    }

    #[test]
    fn test_describe_copy_failed() {
        assert_eq!(
            describe(&[
                "/tmp/bla: 12: ss: not found\n",
                "autopkgtest [06:59:01]: ERROR: testbed failure: sent `auxverb_debug_fail', got `copy-failed', expected `ok...'\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[0],"origin":"direct regex (^.*: [0-9]+: (.*): not found)","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":null}),
        );
        assert_eq!(
            describe(&[
                "something\n",
                "autopkgtest [06:59:01]: ERROR: testbed failure: sent `auxverb_debug_fail', got `copy-failed', expected `ok...'\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["autopkgtest [06:59:01]: ERROR: testbed failure: sent `auxverb_debug_fail', got `copy-failed', expected `ok...'\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"reason":"sent `auxverb_debug_fail', got `copy-failed', expected `ok...'"},"kind":"testbed-failure"}},"testname":null}),
        );
    }

    #[test]
    fn test_describe_broken_pipe() {
        assert_eq!(
            describe(&[
                "[schroot -c foo] failed (exit status 1, stderr 'boom')\n",
                "autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n",
            ]),
            serde_json::json!({"description":"[schroot -c foo] failed (exit status 1, stderr 'boom')\n","failure":{"lines":["[schroot -c foo] failed (exit status 1, stderr 'boom')\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"command":"schroot -c foo","error":"boom","exit_status":1},"kind":"testbed-setup-failure"}},"testname":null}),
        );
        assert_eq!(
            describe(&[
                "something\n",
                "autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["autopkgtest [22:52:19]: ERROR: testbed failure: cannot send to testbed: [Errno 32] Broken pipe\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"reason":"cannot send to testbed: [Errno 32] Broken pipe"},"kind":"testbed-failure"}},"testname":null}),
        );
    }

    #[test]
    fn test_describe_apt_download_failure() {
        assert_eq!(
            describe(&[
                "E: Unable to locate package libfoo-dev\n",
                "autopkgtest [22:52:19]: ERROR: testbed failure: apt repeatedly failed to download packages\n",
            ]),
            serde_json::json!({"description":"E: Unable to locate package libfoo-dev\n","failure":{"lines":["E: Unable to locate package libfoo-dev\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"package":"libfoo-dev"},"kind":"apt-package-unknown"}},"testname":null}),
        );
        assert_eq!(
            describe(&[
                "something\n",
                "autopkgtest [22:52:19]: ERROR: testbed failure: apt repeatedly failed to download packages\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["autopkgtest [22:52:19]: ERROR: testbed failure: apt repeatedly failed to download packages\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"error":"apt repeatedly failed to download packages","url":null},"kind":"apt-file-fetch-failure"}},"testname":null}),
        );
    }

    #[test]
    fn test_describe_erroneous_package() {
        assert_eq!(
            describe(&[
                "/tmp/bla: 12: ss: not found\n",
                "autopkgtest [22:52:19]: ERROR: erroneous package: rules extract failed with exit code 1\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[0],"origin":"direct regex (^.*: [0-9]+: (.*): not found)","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":null}),
        );
        assert_eq!(
            describe(&[
                "something\n",
                "autopkgtest [22:52:19]: ERROR: erroneous package: rules extract failed with exit code 1\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["autopkgtest [22:52:19]: ERROR: erroneous package: rules extract failed with exit code 1\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"reason":"rules extract failed with exit code 1"},"kind":"erroneous-package"}},"testname":null}),
        );
    }

    #[test]
    fn test_describe_unexpected_error() {
        assert_eq!(
            describe(&[
                "autopkgtest [22:52:19]: ERROR: unexpected error:\n",
                "something\n",
                "/tmp/bla: 12: ss: not found\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[1],"origin":"direct regex (^.*: [0-9]+: (.*): not found)","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":null}),
        );
        assert_eq!(
            describe(&[
                "autopkgtest [22:52:19]: ERROR: unexpected error:\n",
                "something\n",
            ]),
            serde_json::json!({"description":"unexpected error:","failure":{"lines":["autopkgtest [22:52:19]: ERROR: unexpected error:\n"],"offsets":[0],"origin":"direct regex","problem":null},"testname":null}),
        );
    }

    #[test]
    fn test_describe_apt_failure_in_current_field() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test foo: preparing testbed\n",
                "Reading package lists...\n",
                "E: Unable to locate package libfoo-dev\n",
                "autopkgtest [19:19:23]: ERROR: something\n",
            ]),
            serde_json::json!({"description":"E: Unable to locate package libfoo-dev\n","failure":{"lines":["E: Unable to locate package libfoo-dev\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"package":"libfoo-dev"},"kind":"apt-package-unknown"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_error_cleaning_up() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test foo: [-----------------------\n",
                "some output\n",
                "more output\n",
                "autopkgtest [19:19:23]: ERROR: autopkgtest\n",
                ": error cleaning up:\n",
            ]),
            serde_json::json!({"description":"more output","failure":{"lines":["some output\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{},"kind":"timed-out"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_generic_error() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "autopkgtest [19:19:23]: ERROR: something broke\n",
            ]),
            serde_json::json!({"description":"something broke","failure":{"lines":["autopkgtest [19:19:23]: ERROR: something broke\n"],"offsets":[1],"origin":"direct regex","problem":null},"testname":null}),
        );
    }

    #[test]
    fn test_describe_timed_out() {
        assert_eq!(
            describe(&[
                "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  PASS\n",
                "unit-tests           FAIL timed out\n",
            ]),
            serde_json::json!({"description":"timed out","failure":{"lines":["unit-tests           FAIL timed out\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{},"kind":"timed-out"}},"testname":"unit-tests"}),
        );
    }

    #[test]
    fn test_describe_stderr_with_known_problem() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: test foo:  - - - - - - - - - - stderr - - - - - - - - - -\n",
                "some output\n",
                "/tmp/bla: 12: ss: not found\n",
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL stderr: some output\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_stderr_xdg_runtime_dir() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: test foo:  - - - - - - - - - - stderr - - - - - - - - - -\n",
                "QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to '/tmp/runtime-root'\n",
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL stderr: QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to '/tmp/runtime-root'\n",
            ]),
            serde_json::json!({"description":"QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to '/tmp/runtime-root'\n","failure":{"lines":["QStandardPaths: XDG_RUNTIME_DIR not set, defaulting to '/tmp/runtime-root'\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{},"kind":"xdg-runtime-dir-not-set"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_stderr_unknown() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: test foo:  - - - - - - - - - - stderr - - - - - - - - - -\n",
                "some output\n",
                "more output\n",
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL stderr: some output\n",
            ]),
            serde_json::json!({"description":"Test foo failed due to unauthorized stderr output: some output","failure":{"lines":["some output\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"stderr_line":"some output"},"kind":"stderr-output"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_stderr_without_stderr_section() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL stderr: /tmp/bla: 12: ss: not found\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found","failure":{"lines":["foo                  FAIL stderr: /tmp/bla: 12: ss: not found\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":"foo"}),
        );
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL stderr: some output\n",
            ]),
            serde_json::json!({"description":"Test foo failed due to unauthorized stderr output: some output","failure":{"lines":["foo                  FAIL stderr: some output\n"],"offsets":[1],"origin":"direct regex","problem":{"details":{"stderr_line":"some output"},"kind":"stderr-output"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_badpkg_with_apt_failure() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test foo: [-----------------------\n",
                "Reading package lists...\n",
                "E: Unable to locate package libfoo-dev\n",
                "autopkgtest [19:19:20]: test foo: -----------------------]\n",
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL badpkg\n",
                "blame: deb:libfoo-dev\n",
            ]),
            serde_json::json!({"description":null,"failure":{"lines":["E: Unable to locate package libfoo-dev\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"package":"libfoo-dev"},"kind":"apt-package-unknown"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_badpkg_without_extra() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL badpkg\n",
            ]),
            serde_json::json!({"description":"Test foo failed","failure":{"lines":["foo                  FAIL badpkg\n"],"offsets":[1],"origin":"direct regex","problem":null},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_failure_with_output() {
        assert_eq!(
            describe(&[
                "autopkgtest [19:19:20]: test foo: [-----------------------\n",
                "something\n",
                "/tmp/bla: 12: ss: not found\n",
                "autopkgtest [19:19:20]: test foo: -----------------------]\n",
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FAIL non-zero exit status 1\n",
            ]),
            serde_json::json!({"description":"/tmp/bla: 12: ss: not found\n","failure":{"lines":["/tmp/bla: 12: ss: not found\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"command":"ss"},"kind":"command-missing"}},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_failure_without_output() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  FLAKY non-zero exit status 1\n",
            ]),
            serde_json::json!({"description":"Test foo failed: non-zero exit status 1","failure":{"lines":["foo                  FLAKY non-zero exit status 1\n"],"offsets":[1],"origin":"direct regex","problem":null},"testname":"foo"}),
        );
    }

    #[test]
    fn test_describe_all_passed() {
        assert_eq!(
            describe(&[
                "autopkgtest [20:49:00]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "foo                  PASS\n",
                "bar                  SKIP no reason\n",
            ]),
            serde_json::json!(null),
        );
    }

    #[test]
    fn test_describe_match_error() {
        assert_eq!(
            describe(&[
                "autopkgtest [07:58:03]: @@@@@@@@@@@@@@@@@@@@ summary\n",
                "unit-tests           FAIL stderr: E: Build killed with signal TERM after 99999999999999999999 minutes of inactivity\n",
            ]),
            serde_json::json!({"error":"invalid integer at line 1 (direct regex (E: Build killed with signal TERM after ([0-9]+) minutes of inactivity))"}),
        );
    }

    fn testbed(lines: &[&str]) -> serde_json::Value {
        crate::failure_json(find_testbed_setup_failure(lines).unwrap().as_ref())
    }

    #[test]
    fn test_testbed_setup_chroot_not_found() {
        assert_eq!(
            testbed(&["[schroot -c unstable-amd64-sbuild] failed (exit status 1, stderr 'E: unstable-amd64-sbuild: Chroot not found\\n')\n"]),
            serde_json::json!({"lines":["[schroot -c unstable-amd64-sbuild] failed (exit status 1, stderr 'E: unstable-amd64-sbuild: Chroot not found\\n')\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"chroot":"unstable-amd64-sbuild"},"kind":"chroot-not-found"}}),
        );
    }

    #[test]
    fn test_testbed_setup_command_failed() {
        assert_eq!(
            testbed(&[
                "[first] failed (exit status 3, stderr 'one')\n",
                "foo\n",
                "[schroot -c foo] failed (exit status 2, stderr 'boom')\n",
                "bar\n",
            ]),
            serde_json::json!({"lines":["[schroot -c foo] failed (exit status 2, stderr 'boom')\n"],"offsets":[2],"origin":"direct regex","problem":{"details":{"command":"schroot -c foo","error":"boom","exit_status":2},"kind":"testbed-setup-failure"}}),
        );
    }

    #[test]
    fn test_testbed_setup_chroot_disappeared() {
        assert_eq!(
            testbed(&["<VirtSubproc>: failure: ['chmod', '1777', '/tmp/x'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2: Failed to stat file: No such file or directory\n"]),
            serde_json::json!({"lines":["<VirtSubproc>: failure: ['chmod', '1777', '/tmp/x'] unexpectedly produced stderr output `W: /var/lib/schroot/session/unstable-amd64-sbuild-dbcdb3f2: Failed to stat file: No such file or directory\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{},"kind":"testbed-chroot-disappeared"}}),
        );
    }

    #[test]
    fn test_testbed_setup_unexpected_stderr() {
        assert_eq!(
            testbed(&["<VirtSubproc>: failure: ['chmod', '1777', '/tmp/x'] unexpectedly produced stderr output `boom\n", "foo\n"]),
            serde_json::json!({"lines":["<VirtSubproc>: failure: ['chmod', '1777', '/tmp/x'] unexpectedly produced stderr output `boom\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"command":"chmod', '1777', '/tmp/x","error":"boom","exit_status":1},"kind":"testbed-setup-failure"}}),
        );
    }

    #[test]
    fn test_testbed_setup_first_pattern_wins() {
        assert_eq!(
            testbed(&["<VirtSubproc>: failure: ['a'] unexpectedly produced stderr output `[c] failed (exit status 1, stderr 'd')\n"]),
            serde_json::json!({"lines":["<VirtSubproc>: failure: ['a'] unexpectedly produced stderr output `[c] failed (exit status 1, stderr 'd')\n"],"offsets":[0],"origin":"direct regex","problem":{"details":{"command":"'a'] unexpectedly produced stderr output `[c","error":"d","exit_status":1},"kind":"testbed-setup-failure"}}),
        );
    }

    #[test]
    fn test_testbed_setup_no_match() {
        assert_eq!(testbed(&[]), serde_json::json!(null));
        assert_eq!(
            testbed(&["[schroot -c foo] failed (exit status 2, stderr 'boom')"]),
            serde_json::json!(null),
        );
    }
}
