use serde::{Deserialize, Serialize};

/// Problem representing unsatisfiable dependencies in autopkgtest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopkgtestDepsUnsatisfiable {
    /// The unsatisfiable dependencies, as (package, reason) pairs.
    pub args: Vec<(Option<String>, String)>,
}

impl AutopkgtestDepsUnsatisfiable {
    /// Creates a new instance from a blame line string.
    ///
    /// Parses a blame line from autopkgtest output to extract dependency issues.
    ///
    /// # Arguments
    /// * `line` - The blame line string to parse
    pub fn from_blame_line(line: &str) -> Self {
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
        Self { args }
    }
}

crate::problem!(AutopkgtestDepsUnsatisfiable, "badpkg", ["args"]);

impl std::fmt::Display for AutopkgtestDepsUnsatisfiable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependencies unsatisfiable: {:?}", self.args)
    }
}

/// Problem representing an autopkgtest test that timed out during execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopkgtestTimedOut;

crate::problem!(AutopkgtestTimedOut, "timed-out", {});

impl std::fmt::Display for AutopkgtestTimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest timed out")
    }
}

/// Problem representing a missing XDG_RUNTIME_DIR environment variable.
///
/// This issue typically occurs when running GUI tests in autopkgtest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XDGRunTimeNotSet;

crate::problem!(XDGRunTimeNotSet, "xdg-runtime-dir-not-set", {});

impl std::fmt::Display for XDGRunTimeNotSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "XDG_RUNTIME_DIR not set")
    }
}

/// Problem representing a failure in the autopkgtest testbed.
///
/// Contains a string describing the specific reason for the testbed failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopkgtestTestbedFailure {
    /// The reason for the testbed failure.
    pub reason: String,
}

crate::problem!(AutopkgtestTestbedFailure, "testbed-failure", ["reason"]);

impl std::fmt::Display for AutopkgtestTestbedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest testbed failure: {}", self.reason)
    }
}

/// Problem representing an autopkgtest dependency chroot that disappeared.
///
/// This occurs when the chroot environment used for dependency resolution
/// becomes unavailable during testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopkgtestDepChrootDisappeared;

crate::problem!(
    AutopkgtestDepChrootDisappeared,
    "testbed-chroot-disappeared",
    {}
);

impl std::fmt::Display for AutopkgtestDepChrootDisappeared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependency chroot disappeared")
    }
}

/// Problem representing an erroneous package in autopkgtest.
///
/// Contains a string describing the specific package error encountered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopkgtestErroneousPackage {
    /// The reason the package is erroneous.
    pub reason: String,
}

crate::problem!(AutopkgtestErroneousPackage, "erroneous-package", ["reason"]);

impl std::fmt::Display for AutopkgtestErroneousPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest erroneous package: {}", self.reason)
    }
}

/// Problem representing a failure detected from stderr output in autopkgtest.
///
/// Contains the stderr line that indicates the failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopkgtestStderrFailure {
    /// The stderr line that caused the failure.
    pub stderr_line: String,
}

crate::problem!(AutopkgtestStderrFailure, "stderr-output", ["stderr_line"]);

impl std::fmt::Display for AutopkgtestStderrFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest output on stderr: {}", self.stderr_line)
    }
}

/// Problem representing a failure during autopkgtest testbed setup.
///
/// Contains details about the command that failed, its exit status,
/// and the error message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopkgtestTestbedSetupFailure {
    /// The command that failed to execute properly.
    pub command: String,
    /// The exit status code of the failed command.
    pub exit_status: i32,
    /// The error message provided by the command.
    pub error: String,
}

crate::problem!(
    AutopkgtestTestbedSetupFailure,
    "testbed-setup-failure",
    ["command", "exit_status", "error"]
);

impl std::fmt::Display for AutopkgtestTestbedSetupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "autopkgtest testbed setup failure: {} exited with status {}: {}",
            self.command, self.exit_status, self.error
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::assert_wire_format;
    use crate::Problem;
    use serde_json::json;

    #[test]
    fn test_autopkgtest_testbed_setup_failure_trait() {
        let problem = AutopkgtestTestbedSetupFailure {
            command: "apt-get update".to_string(),
            exit_status: 100,
            error: "Failed to fetch".to_string(),
        };
        let json = problem.json();
        assert_eq!(json["command"], "apt-get update");
        assert_eq!(json["exit_status"], 100);
        assert_eq!(json["error"], "Failed to fetch");
    }

    #[test]
    fn wire_format() {
        assert_wire_format(
            &AutopkgtestDepsUnsatisfiable {
                args: vec![
                    (Some("a".to_string()), "x".to_string()),
                    (None, "y".to_string()),
                ],
            },
            "badpkg",
            json!({"args": [["a", "x"], [null, "y"]]}),
            "autopkgtest dependencies unsatisfiable: [(Some(\"a\"), \"x\"), (None, \"y\")]",
        );
        assert_wire_format(
            &AutopkgtestTimedOut,
            "timed-out",
            json!({}),
            "autopkgtest timed out",
        );
        assert_wire_format(
            &XDGRunTimeNotSet,
            "xdg-runtime-dir-not-set",
            json!({}),
            "XDG_RUNTIME_DIR not set",
        );
        assert_wire_format(
            &AutopkgtestTestbedFailure {
                reason: "f0-value".to_string(),
            },
            "testbed-failure",
            json!({"reason": "f0-value"}),
            "autopkgtest testbed failure: f0-value",
        );
        assert_wire_format(
            &AutopkgtestDepChrootDisappeared,
            "testbed-chroot-disappeared",
            json!({}),
            "autopkgtest dependency chroot disappeared",
        );
        assert_wire_format(
            &AutopkgtestErroneousPackage {
                reason: "f0-value".to_string(),
            },
            "erroneous-package",
            json!({"reason": "f0-value"}),
            "autopkgtest erroneous package: f0-value",
        );
        assert_wire_format(
            &AutopkgtestStderrFailure {
                stderr_line: "f0-value".to_string(),
            },
            "stderr-output",
            json!({"stderr_line": "f0-value"}),
            "autopkgtest output on stderr: f0-value",
        );
        assert_wire_format(
            &AutopkgtestTestbedSetupFailure {
                command: "command-value".to_string(),
                exit_status: 3,
                error: "error-value".to_string(),
            },
            "testbed-setup-failure",
            json!({"command": "command-value", "error": "error-value", "exit_status": 3}),
            "autopkgtest testbed setup failure: command-value exited with status 3: error-value",
        );
    }
}
