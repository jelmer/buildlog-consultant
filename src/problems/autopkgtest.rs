use crate::Problem;

/// Problem representing unsatisfiable dependencies in autopkgtest.
#[derive(Debug, Clone)]
pub struct AutopkgtestDepsUnsatisfiable(pub Vec<(Option<String>, String)>);

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
        Self(args)
    }
}

impl Problem for AutopkgtestDepsUnsatisfiable {
    fn kind(&self) -> std::borrow::Cow<str> {
        "badpkg".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "args": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestDepsUnsatisfiable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependencies unsatisfiable: {:?}", self.0)
    }
}

/// Problem representing an autopkgtest test that timed out during execution.
#[derive(Debug, Clone)]
pub struct AutopkgtestTimedOut;

impl Problem for AutopkgtestTimedOut {
    fn kind(&self) -> std::borrow::Cow<str> {
        "timed-out".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestTimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest timed out")
    }
}

/// Problem representing a missing XDG_RUNTIME_DIR environment variable.
///
/// This issue typically occurs when running GUI tests in autopkgtest.
#[derive(Debug, Clone)]
pub struct XDGRunTimeNotSet;

impl Problem for XDGRunTimeNotSet {
    fn kind(&self) -> std::borrow::Cow<str> {
        "xdg-runtime-dir-not-set".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for XDGRunTimeNotSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "XDG_RUNTIME_DIR not set")
    }
}

/// Problem representing a failure in the autopkgtest testbed.
///
/// Contains a string describing the specific reason for the testbed failure.
#[derive(Debug, Clone)]
pub struct AutopkgtestTestbedFailure(pub String);

impl Problem for AutopkgtestTestbedFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "testbed-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestTestbedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest testbed failure: {}", self.0)
    }
}

/// Problem representing an autopkgtest dependency chroot that disappeared.
///
/// This occurs when the chroot environment used for dependency resolution
/// becomes unavailable during testing.
#[derive(Debug, Clone)]
pub struct AutopkgtestDepChrootDisappeared;

impl Problem for AutopkgtestDepChrootDisappeared {
    fn kind(&self) -> std::borrow::Cow<str> {
        "testbed-chroot-disappeared".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestDepChrootDisappeared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest dependency chroot disappeared")
    }
}

/// Problem representing an erroneous package in autopkgtest.
///
/// Contains a string describing the specific package error encountered.
#[derive(Debug, Clone)]
pub struct AutopkgtestErroneousPackage(pub String);

impl Problem for AutopkgtestErroneousPackage {
    fn kind(&self) -> std::borrow::Cow<str> {
        "erroneous-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestErroneousPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest erroneous package: {}", self.0)
    }
}

/// Problem representing a failure detected from stderr output in autopkgtest.
///
/// Contains the stderr line that indicates the failure.
#[derive(Debug, Clone)]
pub struct AutopkgtestStderrFailure(pub String);

impl Problem for AutopkgtestStderrFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "stderr-output".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "stderr_line": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AutopkgtestStderrFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "autopkgtest output on stderr: {}", self.0)
    }
}

/// Problem representing a failure during autopkgtest testbed setup.
///
/// Contains details about the command that failed, its exit status,
/// and the error message.
#[derive(Debug, Clone)]
pub struct AutopkgtestTestbedSetupFailure {
    /// The command that failed to execute properly.
    pub command: String,
    /// The exit status code of the failed command.
    pub exit_status: i32,
    /// The error message provided by the command.
    pub error: String,
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
