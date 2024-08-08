use crate::{Match, Problem, SingleLineMatch};
use crate::lines::Lines;
use std::collections::HashMap;

pub struct AutopkgtestDepsUnsatisfiable(pub Vec<(Option<String>, String)>);

impl Problem for AutopkgtestDepsUnsatisfiable {
    fn kind(&self) -> std::borrow::Cow<str> {
        "badpkg".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "args": self.0,
        })
    }
}

impl AutopkgtestDepsUnsatisfiable {
    fn from_blame_line(line: &str) -> Self {
        let mut args = vec![];
        for entry in line.strip_prefix("blame: ").unwrap().split_whitespace() {
            let (kind, arg) = match entry.split_once(':') {
                Some((kind, arg)) => (Some(kind), arg),
                None => (None, entry),
            };
            args.push((kind.map(|x| x.to_string()), arg.to_string()));
            match kind {
                Some("deb") | Some("arg") | Some("dsc") | None => {}
                Some(entry) => {
                    log::warn!("unknown entry {} on badpkg line", entry);
                }
            }
        }
        Self(args)
    }
}

impl std::fmt::Display for AutopkgtestDepsUnsatisfiable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependencies unsatisfiable: {:?}", self.0)
    }
}

pub struct AutopkgtestTimedOut;

impl Problem for AutopkgtestTimedOut {
    fn kind(&self) -> std::borrow::Cow<str> {
        "timed-out".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for AutopkgtestTimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest timed out")
    }
}

pub struct XDGRunTimeNotSet;

impl Problem for XDGRunTimeNotSet {
    fn kind(&self) -> std::borrow::Cow<str> {
        "xdg-runtime-dir-not-set".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for XDGRunTimeNotSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "XDG_RUNTIME_DIR not set")
    }
}

pub struct AutopkgtestTestbedFailure(String);

impl Problem for AutopkgtestTestbedFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "testbed-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }
}

impl std::fmt::Display for AutopkgtestTestbedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest testbed failure: {}", self.0)
    }
}

pub struct AutopkgtestDepChrootDisappeared;

impl Problem for AutopkgtestDepChrootDisappeared {
    fn kind(&self) -> std::borrow::Cow<str> {
        "testbed-chroot-disappeared".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for AutopkgtestDepChrootDisappeared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependency chroot disappeared")
    }
}

pub struct AutopkgtestErroneousPackage(String);

impl Problem for AutopkgtestErroneousPackage {
    fn kind(&self) -> std::borrow::Cow<str> {
        "erroneous-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }
}

impl std::fmt::Display for AutopkgtestErroneousPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest erroneous package: {}", self.0)
    }
}

pub struct AutopkgtestStderrFailure(String);

impl Problem for AutopkgtestStderrFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "stderr-output".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "stderr_line": self.0,
        })
    }
}

impl std::fmt::Display for AutopkgtestStderrFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest output on stderr: {}", self.0)
    }
}

pub struct AutopkgtestTestbedSetupFailure {
    command: String,
    exit_status: i32,
    error: String,
}

impl Problem for AutopkgtestTestbedSetupFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "testbed-setup-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.command,
            "exit_status": self.exit_status,
            "error": self.error,
        })
    }
}

impl std::fmt::Display for AutopkgtestTestbedSetupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "autopkgtest testbed setup failure: {} exited with status {}: {}",
            self.command, self.exit_status, self.error
        )
    }
}

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
    } else if let Some(message) = message.strip_prefix("ERROR:") {
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
///
/// # Returns
/// tuple with (line offset, testname, error, description)
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
    let mut i = 0;
    while i < lines.len() - 1 {
        i += 1;
        let line = lines[i];
        match parse_autopgktest_line(line) {
            Some((_, Packet::Source)) => {}
            Some((_, Packet::Other(_))) => {}
            Some((_, Packet::Error(msg))) => {
                let msg = if msg.starts_with('"') && msg.chars().filter(|x| *x == '"').count() == 1
                {
                    let mut sublines = vec![msg];
                    while i < lines.len() {
                        i += 1;
                        sublines.push(lines[i]);
                        if lines[i].chars().filter(|x| x == &'"').count() == 1 {
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
                            Some(Box::new(crate::apt::AptFetchFailure {
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
                            .get(&current_field)
                            .unwrap()
                            .0
                            .iter()
                            .map(|x| x.as_str())
                            .collect(),
                    );
                    if error.is_some()
                        && r#match.is_some()
                        && test_output.contains_key(&current_field)
                    {
                        let description = r#match.as_ref().unwrap().line();
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines,
                                test_output.get(&current_field).unwrap().1
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
                            test_output.get(&current_field.as_ref().unwrap()).unwrap().1,
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
                    summary_offset + packet.lineno(),
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
                    offset = stderr_offset.clone();
                } else {
                    if let Some(stderr_offset) = stderr_offset {
                        offset = Some(stderr_offset.clone());
                    }
                    description = None;
                }
            } else {
                (r#match, error) = crate::common::find_build_failure_description(vec![output]);

                (offset, description) = if let Some(r#match) = r#match.as_ref() {
                    (
                        Some(summary_offset + packet.lineno() + r#match.offset()),
                        Some(r#match.line()),
                    )
                } else {
                    (None, None)
                };
            }
            let offset = offset.unwrap_or_else(|| summary_offset + packet.lineno());
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

            let error = if let Some(blame) = blame {
                Some(
                    Box::new(AutopkgtestDepsUnsatisfiable::from_blame_line(blame))
                        as Box<dyn Problem>,
                )
            } else {
                None
            };
            return (
                Some(Box::new(SingleLineMatch::from_lines(
                    &lines,
                    summary_offset + packet.lineno() + blame_offset.unwrap(),
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
                summary_offset + packet.lineno()
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

    return (None, None, None, None);
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
                    Some(Box::new(crate::common::ChrootNotFound {
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
            r"\<VirtSubproc\>: failure: \[\'(.*)\'\] unexpectedly produced stderr output `(.*)",
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
