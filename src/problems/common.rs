use crate::Problem;
use pep508_rs::pep440_rs;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::path::PathBuf;

/// Problem representing a file that was expected but not found.
///
/// This struct is used to report situations where a required file is missing,
/// which may cause build or execution failures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingFile {
    /// The path to the missing file.
    pub path: PathBuf,
}

impl MissingFile {
    /// Creates a new MissingFile instance.
    ///
    /// # Arguments
    /// * `path` - Path to the missing file
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Problem for MissingFile {
    fn kind(&self) -> Cow<str> {
        "missing-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path.to_string_lossy(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing file: {}", self.path.display())
    }
}

/// Problem representing a missing build system file.
///
/// This struct is used to report when a file required by the build system
/// (such as a Makefile, CMakeLists.txt, etc.) is missing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MissingBuildFile {
    /// The name of the missing build file.
    pub filename: String,
}

impl MissingBuildFile {
    /// Creates a new MissingBuildFile instance.
    ///
    /// # Arguments
    /// * `filename` - Name of the missing build file
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl Problem for MissingBuildFile {
    fn kind(&self) -> Cow<str> {
        "missing-build-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingBuildFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing build file: {}", self.filename)
    }
}

/// Problem representing something that could be either a missing command or build file.
///
/// This struct is used when it's not clear whether a missing entity is a
/// command (executable) or a build file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingCommandOrBuildFile {
    /// The name of the missing command or build file.
    pub filename: String,
}

impl Problem for MissingCommandOrBuildFile {
    fn kind(&self) -> Cow<str> {
        "missing-command-or-build-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingCommandOrBuildFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command or build file: {}", self.filename)
    }
}

impl MissingCommandOrBuildFile {
    /// Returns the command name, which is the same as the filename.
    ///
    /// # Returns
    /// The filename/command name as a String
    pub fn command(&self) -> String {
        self.filename.clone()
    }
}

/// Problem representing a need for a version control system directory.
///
/// This struct is used when a build process requires a version control
/// system directory (like .git, .bzr, .svn) to be present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcsControlDirectoryNeeded {
    /// List of version control systems that could provide the needed directory.
    pub vcs: Vec<String>,
}

impl Problem for VcsControlDirectoryNeeded {
    fn kind(&self) -> Cow<str> {
        "vcs-control-directory-needed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "vcs": self.vcs,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VcsControlDirectoryNeeded {
    /// Creates a new VcsControlDirectoryNeeded instance.
    ///
    /// # Arguments
    /// * `vcs` - List of version control system names
    pub fn new(vcs: Vec<&str>) -> Self {
        Self {
            vcs: vcs.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Problem representing a missing Python module.
///
/// This struct is used when a required Python module is not available,
/// which may include version constraints.
#[derive(Debug, Clone)]
pub struct MissingPythonModule {
    /// The name of the missing Python module.
    pub module: String,
    /// The Python major version (e.g., 2 or 3) if specific.
    pub python_version: Option<i32>,
    /// The minimum required version of the module if specified.
    pub minimum_version: Option<String>,
}

impl Display for MissingPythonModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(python_version) = self.python_version {
            write!(
                f,
                "Missing {} Python module: {}",
                python_version, self.module
            )?;
        } else {
            write!(f, "Missing Python module: {}", self.module)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl MissingPythonModule {
    /// Creates a simple MissingPythonModule instance without version constraints.
    ///
    /// # Arguments
    /// * `module` - Name of the missing Python module
    ///
    /// # Returns
    /// A new MissingPythonModule with no version requirements
    pub fn simple(module: String) -> MissingPythonModule {
        MissingPythonModule {
            module,
            python_version: None,
            minimum_version: None,
        }
    }
}

impl Problem for MissingPythonModule {
    fn kind(&self) -> Cow<str> {
        "missing-python-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.module,
            "python_version": self.python_version,
            "minimum_version": self.minimum_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing command-line executable.
///
/// This struct is used when a required command is not available in the PATH.
#[derive(Debug, Clone)]
pub struct MissingCommand(pub String);

impl Display for MissingCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command: {}", self.0)
    }
}

impl Problem for MissingCommand {
    fn kind(&self) -> Cow<str> {
        "command-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing Python package distribution.
///
/// This struct is used when a required Python package is not available,
/// which may include version constraints.
#[derive(Debug, Clone)]
pub struct MissingPythonDistribution {
    /// The name of the missing Python distribution.
    pub distribution: String,
    /// The Python major version (e.g., 2 or 3) if specific.
    pub python_version: Option<i32>,
    /// The minimum required version of the distribution if specified.
    pub minimum_version: Option<String>,
}

impl Display for MissingPythonDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(python_version) = self.python_version {
            write!(
                f,
                "Missing {} Python distribution: {}",
                python_version, self.distribution
            )?;
        } else {
            write!(f, "Missing Python distribution: {}", self.distribution)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl Problem for MissingPythonDistribution {
    fn kind(&self) -> Cow<str> {
        "missing-python-distribution".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "distribution": self.distribution,
            "python_version": self.python_version,
            "minimum_version": self.minimum_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn find_python_version(marker: Vec<Vec<pep508_rs::MarkerExpression>>) -> Option<i32> {
    let mut major_version = None;
    for expr in marker.iter().flat_map(|x| x.iter()) {
        match expr {
            pep508_rs::MarkerExpression::Version {
                key: pep508_rs::MarkerValueVersion::PythonVersion,
                specifier,
            } => {
                let version = specifier.version();
                major_version = Some(version.release()[0] as i32);
            }
            _ => {}
        }
    }

    major_version
}

impl MissingPythonDistribution {
    /// Creates a MissingPythonDistribution from a PEP508 requirement string.
    ///
    /// Parses a Python package requirement string (in PEP508 format) to extract
    /// the package name and version constraints.
    ///
    /// # Arguments
    /// * `text` - The requirement string in PEP508 format
    /// * `python_version` - Optional Python version to override detected version
    ///
    /// # Returns
    /// A new MissingPythonDistribution instance
    pub fn from_requirement_str(
        text: &str,
        python_version: Option<i32>,
    ) -> MissingPythonDistribution {
        use pep440_rs::Operator;
        use pep508_rs::{Requirement, VersionOrUrl};
        use std::str::FromStr;

        let depspec: Requirement = Requirement::from_str(text).unwrap();

        let distribution = depspec.name.to_string();

        let python_version =
            python_version.or_else(|| find_python_version(depspec.marker.to_dnf()));
        let minimum_version = if let Some(v_u) = depspec.version_or_url {
            if let VersionOrUrl::VersionSpecifier(vs) = v_u {
                if vs.len() == 1 {
                    if *vs[0].operator() == Operator::GreaterThanEqual {
                        Some(vs[0].version().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        MissingPythonDistribution {
            distribution,
            python_version,
            minimum_version,
        }
    }

    /// Creates a simple MissingPythonDistribution without version constraints.
    ///
    /// # Arguments
    /// * `distribution` - Name of the missing Python distribution
    ///
    /// # Returns
    /// A new MissingPythonDistribution with no version requirements
    pub fn simple(distribution: &str) -> MissingPythonDistribution {
        MissingPythonDistribution {
            distribution: distribution.to_string(),
            python_version: None,
            minimum_version: None,
        }
    }
}

impl Display for VcsControlDirectoryNeeded {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "VCS control directory needed: {}", self.vcs.join(", "))
    }
}

/// Problem representing a missing Haskell module.
///
/// This struct is used when a required Haskell module is not available.
#[derive(Debug, Clone)]
pub struct MissingHaskellModule {
    /// The name of the missing Haskell module.
    pub module: String,
}

impl MissingHaskellModule {
    /// Creates a new MissingHaskellModule instance.
    ///
    /// # Arguments
    /// * `module` - Name of the missing Haskell module
    ///
    /// # Returns
    /// A new MissingHaskellModule instance
    pub fn new(module: String) -> MissingHaskellModule {
        MissingHaskellModule { module }
    }
}

impl Display for MissingHaskellModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Haskell module: {}", self.module)
    }
}

impl Problem for MissingHaskellModule {
    fn kind(&self) -> Cow<str> {
        "missing-haskell-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.module,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing system library.
///
/// This struct is used when a required shared library (.so/.dll/.dylib) is not available.
#[derive(Debug, Clone)]
pub struct MissingLibrary(pub String);

impl Display for MissingLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing library: {}", self.0)
    }
}

impl Problem for MissingLibrary {
    fn kind(&self) -> Cow<str> {
        "missing-library".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "library": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing GObject Introspection typelib.
///
/// This struct is used when a required GObject Introspection typelib file is not available.
#[derive(Debug, Clone)]
pub struct MissingIntrospectionTypelib(pub String);

impl Display for MissingIntrospectionTypelib {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing introspection typelib: {}", self.0)
    }
}

impl Problem for MissingIntrospectionTypelib {
    fn kind(&self) -> Cow<str> {
        "missing-introspection-typelib".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "library": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing pytest fixture.
///
/// This struct is used when a pytest test requires a fixture that is not available.
#[derive(Debug, Clone)]
pub struct MissingPytestFixture(pub String);

impl Display for MissingPytestFixture {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing pytest fixture: {}", self.0)
    }
}

impl Problem for MissingPytestFixture {
    fn kind(&self) -> Cow<str> {
        "missing-pytest-fixture".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "fixture": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing an unsupported pytest configuration option.
///
/// This struct is used when a pytest configuration specifies an option
/// that is not supported in the current environment.
#[derive(Debug, Clone)]
pub struct UnsupportedPytestConfigOption(pub String);

impl Display for UnsupportedPytestConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest config option: {}", self.0)
    }
}

impl Problem for UnsupportedPytestConfigOption {
    fn kind(&self) -> Cow<str> {
        "unsupported-pytest-config-option".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing unsupported pytest command-line arguments.
///
/// This struct is used when pytest is invoked with command-line arguments
/// that are not supported in the current environment.
#[derive(Debug, Clone)]
pub struct UnsupportedPytestArguments(pub Vec<String>);

impl Display for UnsupportedPytestArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest arguments: {:?}", self.0)
    }
}

impl Problem for UnsupportedPytestArguments {
    fn kind(&self) -> Cow<str> {
        "unsupported-pytest-arguments".into()
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

/// Problem representing a missing R package.
///
/// This struct is used when a required R package is not installed
/// or not available in the environment.
#[derive(Debug, Clone)]
pub struct MissingRPackage {
    /// The name of the missing R package.
    pub package: String,
    /// The minimum required version of the package, if specified.
    pub minimum_version: Option<String>,
}

impl MissingRPackage {
    /// Creates a simple MissingRPackage instance without version constraints.
    ///
    /// # Arguments
    /// * `package` - Name of the missing R package
    ///
    /// # Returns
    /// A new MissingRPackage with no version requirements
    pub fn simple(package: &str) -> Self {
        Self {
            package: package.to_string(),
            minimum_version: None,
        }
    }
}

impl Display for MissingRPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing R package: {}", self.package)?;
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " (>= {})", minimum_version)?;
        }
        Ok(())
    }
}

impl Problem for MissingRPackage {
    fn kind(&self) -> Cow<str> {
        "missing-r-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "minimum_version": self.minimum_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Problem representing a missing Go package.
///
/// This struct is used when a required Go package is not installed
/// or not available in the environment.
#[derive(Debug, Clone)]
pub struct MissingGoPackage {
    /// The import path of the missing Go package.
    pub package: String,
}

impl Problem for MissingGoPackage {
    fn kind(&self) -> Cow<str> {
        "missing-go-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingGoPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Go package: {}", self.package)
    }
}

/// Problem representing a missing C header file.
///
/// This struct is used when a required C header file (.h) is not available
/// during compilation.
#[derive(Debug, Clone)]
pub struct MissingCHeader {
    /// The name of the missing C header file.
    pub header: String,
}

impl Problem for MissingCHeader {
    fn kind(&self) -> Cow<str> {
        "missing-c-header".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "header": self.header,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingCHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C header: {}", self.header)
    }
}

impl MissingCHeader {
    /// Creates a new MissingCHeader instance.
    ///
    /// # Arguments
    /// * `header` - Name of the missing C header file
    ///
    /// # Returns
    /// A new MissingCHeader instance
    pub fn new(header: String) -> Self {
        Self { header }
    }
}

/// Problem representing a missing Node.js module.
///
/// This struct is used when a required Node.js module is not installed
/// or cannot be imported.
#[derive(Debug, Clone)]
pub struct MissingNodeModule(pub String);

impl Problem for MissingNodeModule {
    fn kind(&self) -> Cow<str> {
        "missing-node-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingNodeModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node module: {}", self.0)
    }
}

/// Problem representing a missing Node.js package.
///
/// This struct is used when a required Node.js package is not installed
/// via npm/yarn/pnpm or cannot be found in node_modules.
#[derive(Debug, Clone)]
pub struct MissingNodePackage(pub String);

impl Problem for MissingNodePackage {
    fn kind(&self) -> Cow<str> {
        "missing-node-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingNodePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node package: {}", self.0)
    }
}

/// Problem representing a missing configure script.
///
/// This struct is used when a build expects to find a configure script
/// (typically from autotools) but it doesn't exist.
#[derive(Debug, Clone)]
pub struct MissingConfigure;

impl Problem for MissingConfigure {
    fn kind(&self) -> Cow<str> {
        "missing-configure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingConfigure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing ./configure")
    }
}

/// Problem representing a vague or unspecified dependency.
///
/// This struct is used when a build requires a dependency that
/// cannot be clearly categorized as a specific type of dependency.
#[derive(Debug, Clone)]
pub struct MissingVagueDependency {
    /// The name of the missing dependency.
    pub name: String,
    /// An optional URL where the dependency might be found.
    pub url: Option<String>,
    /// The minimum required version of the dependency, if specified.
    pub minimum_version: Option<String>,
    /// The current version of the dependency, if known.
    pub current_version: Option<String>,
}

impl MissingVagueDependency {
    /// Creates a simple MissingVagueDependency instance with just a name.
    ///
    /// # Arguments
    /// * `name` - Name of the missing dependency
    ///
    /// # Returns
    /// A new MissingVagueDependency with no additional information
    pub fn simple(name: &str) -> Self {
        Self {
            name: name.to_string(),
            url: None,
            minimum_version: None,
            current_version: None,
        }
    }
}

impl Problem for MissingVagueDependency {
    fn kind(&self) -> Cow<str> {
        "missing-vague-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "url": self.url,
            "minimum_version": self.minimum_version,
            "current_version": self.current_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingVagueDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing dependency: {}", self.name)
    }
}

/// Problem representing missing Qt framework.
///
/// This struct is used when a build requires the Qt framework
/// but it is not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingQt;

impl Problem for MissingQt {
    fn kind(&self) -> Cow<str> {
        "missing-qt".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingQt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Qt")
    }
}

/// Problem representing missing X11 libraries or headers.
///
/// This struct is used when a build requires X11 (X Window System)
/// components but they are not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingX11;

impl Problem for MissingX11 {
    fn kind(&self) -> Cow<str> {
        "missing-x11".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingX11 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing X11")
    }
}

/// Problem representing a missing autoconf macro.
///
/// This struct is used when a build using autoconf requires a macro
/// that is not available in the build environment.
#[derive(Debug, Clone)]
pub struct MissingAutoconfMacro {
    /// The name of the missing autoconf macro.
    pub r#macro: String,
    /// Whether the build system needs to be rebuilt after adding the macro.
    pub need_rebuild: bool,
}

impl MissingAutoconfMacro {
    /// Creates a new MissingAutoconfMacro instance.
    ///
    /// # Arguments
    /// * `macro` - Name of the missing autoconf macro
    ///
    /// # Returns
    /// A new MissingAutoconfMacro instance with need_rebuild set to false
    pub fn new(r#macro: String) -> Self {
        Self {
            r#macro,
            need_rebuild: false,
        }
    }
}

impl Problem for MissingAutoconfMacro {
    fn kind(&self) -> Cow<str> {
        "missing-autoconf-macro".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "macro": self.r#macro,
            "need_rebuild": self.need_rebuild,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingAutoconfMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing autoconf macro: {}", self.r#macro)
    }
}

/// Problem representing a directory that does not exist.
///
/// This struct is used when a build process expects a directory to exist
/// but it cannot be found in the filesystem.
#[derive(Debug, Clone)]
pub struct DirectoryNonExistant(pub String);

impl Problem for DirectoryNonExistant {
    fn kind(&self) -> Cow<str> {
        "local-directory-not-existing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DirectoryNonExistant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Directory does not exist: {}", self.0)
    }
}

/// Problem representing a missing Vala package.
///
/// This struct is used when a build requires a Vala package
/// that is not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingValaPackage(pub String);

impl Problem for MissingValaPackage {
    fn kind(&self) -> Cow<str> {
        "missing-vala-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingValaPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Vala package: {}", self.0)
    }
}

/// Problem representing the presence of an upstart configuration file.
///
/// This struct is used to indicate that a package includes an upstart file,
/// which may be problematic in environments that have moved to systemd.
#[derive(Debug, Clone)]
pub struct UpstartFilePresent(pub String);

impl Problem for UpstartFilePresent {
    fn kind(&self) -> Cow<str> {
        "upstart-file-present".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for UpstartFilePresent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Upstart file present: {}", self.0)
    }
}

/// Problem representing a missing PostgreSQL extension.
///
/// This struct is used when a build or runtime requires a PostgreSQL extension
/// that is not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingPostgresExtension(pub String);

impl Problem for MissingPostgresExtension {
    fn kind(&self) -> Cow<str> {
        "missing-postgresql-extension".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "extension": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPostgresExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PostgreSQL extension: {}", self.0)
    }
}

/// Problem representing a missing pkg-config module.
///
/// This struct is used when a build requires a package found via pkg-config
/// that is not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingPkgConfig {
    /// The name of the missing pkg-config module.
    pub module: String,
    /// The minimum required version of the module, if specified.
    pub minimum_version: Option<String>,
}

impl Problem for MissingPkgConfig {
    fn kind(&self) -> Cow<str> {
        "missing-pkg-config-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.module,
            "minimum_version": self.minimum_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPkgConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(minimum_version) = &self.minimum_version {
            write!(
                f,
                "Missing pkg-config module: {} >= {}",
                self.module, minimum_version
            )
        } else {
            write!(f, "Missing pkg-config module: {}", self.module)
        }
    }
}

impl MissingPkgConfig {
    /// Creates a new MissingPkgConfig instance with optional version constraint.
    ///
    /// # Arguments
    /// * `module` - Name of the missing pkg-config module
    /// * `minimum_version` - Optional minimum version requirement
    ///
    /// # Returns
    /// A new MissingPkgConfig instance
    pub fn new(module: String, minimum_version: Option<String>) -> Self {
        Self {
            module,
            minimum_version,
        }
    }

    /// Creates a simple MissingPkgConfig instance without version constraint.
    ///
    /// # Arguments
    /// * `module` - Name of the missing pkg-config module
    ///
    /// # Returns
    /// A new MissingPkgConfig with no version requirements
    pub fn simple(module: String) -> Self {
        Self {
            module,
            minimum_version: None,
        }
    }
}

/// Problem representing multiple missing Haskell dependencies.
///
/// This struct is used when a build requires multiple Haskell packages
/// that are not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingHaskellDependencies(pub Vec<String>);

impl Problem for MissingHaskellDependencies {
    fn kind(&self) -> Cow<str> {
        "missing-haskell-dependencies".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "deps": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingHaskellDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Haskell dependencies: {:?}", self.0)
    }
}

/// Problem representing lack of disk space.
///
/// This struct is used when a build fails because there is no space
/// left on the device/filesystem where the build is running.
#[derive(Debug, Clone)]
pub struct NoSpaceOnDevice;

impl Problem for NoSpaceOnDevice {
    fn kind(&self) -> Cow<str> {
        "no-space-on-device".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Indicates that this problem is universal across all build steps.
    ///
    /// No space on device is considered a universal problem because it can
    /// affect any stage of the build process and is not specific to particular
    /// build steps.
    fn is_universal(&self) -> bool {
        true
    }
}

impl Display for NoSpaceOnDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No space left on device")
    }
}

/// Problem representing a missing Java Runtime Environment.
///
/// This struct is used when a build or runtime requires a Java Runtime
/// Environment (JRE) that is not installed or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingJRE;

impl Problem for MissingJRE {
    fn kind(&self) -> Cow<str> {
        "missing-jre".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJRE {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JRE")
    }
}

/// Problem representing a missing Java Development Kit.
///
/// This struct is used when a build requires a Java Development Kit (JDK)
/// at a specific path but it cannot be found.
#[derive(Debug, Clone)]
pub struct MissingJDK {
    /// The path where the JDK was expected to be found.
    pub jdk_path: String,
}

impl MissingJDK {
    /// Creates a new MissingJDK instance.
    ///
    /// # Arguments
    /// * `jdk_path` - Path where the JDK was expected to be found
    ///
    /// # Returns
    /// A new MissingJDK instance
    pub fn new(jdk_path: String) -> Self {
        Self { jdk_path }
    }
}

impl Problem for MissingJDK {
    fn kind(&self) -> Cow<str> {
        "missing-jdk".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "jdk_path": self.jdk_path
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJDK {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK at {}", self.jdk_path)
    }
}

/// Problem representing a missing file in the Java Development Kit.
///
/// This struct is used when a build requires a specific file from the JDK
/// but it cannot be found in the JDK directory.
#[derive(Debug, Clone)]
pub struct MissingJDKFile {
    /// The path to the JDK directory.
    pub jdk_path: String,
    /// The name of the file that is missing from the JDK.
    pub filename: String,
}

impl MissingJDKFile {
    /// Creates a new MissingJDKFile instance.
    ///
    /// # Arguments
    /// * `jdk_path` - Path to the JDK directory
    /// * `filename` - Name of the file that is missing from the JDK
    ///
    /// # Returns
    /// A new MissingJDKFile instance
    pub fn new(jdk_path: String, filename: String) -> Self {
        Self { jdk_path, filename }
    }
}

impl Problem for MissingJDKFile {
    fn kind(&self) -> Cow<str> {
        "missing-jdk-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "jdk_path": self.jdk_path,
            "filename": self.filename
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJDKFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK file {} at {}", self.filename, self.jdk_path)
    }
}

/// Problem representing a missing Perl file.
///
/// This struct is used when a Perl script attempts to load a file
/// but it cannot be found in any of the include paths.
#[derive(Debug, Clone)]
pub struct MissingPerlFile {
    /// The name of the missing Perl file.
    pub filename: String,
    /// The include paths that were searched, if available.
    pub inc: Option<Vec<String>>,
}

impl MissingPerlFile {
    /// Creates a new MissingPerlFile instance.
    ///
    /// # Arguments
    /// * `filename` - Name of the missing Perl file
    /// * `inc` - List of include paths that were searched, if known
    ///
    /// # Returns
    /// A new MissingPerlFile instance
    pub fn new(filename: String, inc: Option<Vec<String>>) -> Self {
        Self { filename, inc }
    }
}

impl Problem for MissingPerlFile {
    fn kind(&self) -> Cow<str> {
        "missing-perl-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
            "inc": self.inc
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPerlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(inc) = self.inc.as_ref() {
            write!(
                f,
                "Missing Perl file {} (INC: {})",
                self.filename,
                inc.join(":")
            )
        } else {
            write!(f, "Missing Perl file {}", self.filename)
        }
    }
}

/// Problem representing a missing Perl module.
///
/// This struct is used when a Perl script requires a module
/// that is not installed or cannot be found in the include paths.
#[derive(Debug, Clone)]
pub struct MissingPerlModule {
    /// The name of the file where the module is required, if known.
    pub filename: Option<String>,
    /// The name of the missing Perl module.
    pub module: String,
    /// The include paths that were searched, if available.
    pub inc: Option<Vec<String>>,
    /// The minimum version of the module required, if specified.
    pub minimum_version: Option<String>,
}

impl Problem for MissingPerlModule {
    fn kind(&self) -> Cow<str> {
        "missing-perl-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename,
            "module": self.module,
            "inc": self.inc,
            "minimum_version": self.minimum_version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPerlModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(filename) = &self.filename {
            write!(
                f,
                "Missing Perl module: {} (from {})",
                self.module, filename
            )?;
        } else {
            write!(f, "Missing Perl module: {}", self.module)?;
        }
        if let Some(minimum_version) = &self.minimum_version {
            write!(f, " >= {}", minimum_version)?;
        }
        if let Some(inc) = &self.inc {
            write!(f, " (INC: {})", inc.join(", "))?;
        }
        Ok(())
    }
}

impl MissingPerlModule {
    /// Creates a simple MissingPerlModule instance with just a module name.
    ///
    /// # Arguments
    /// * `module` - Name of the missing Perl module
    ///
    /// # Returns
    /// A new MissingPerlModule with no additional information
    pub fn simple(module: &str) -> Self {
        Self {
            filename: None,
            module: module.to_string(),
            inc: None,
            minimum_version: None,
        }
    }
}

/// Problem representing a missing command in a Python setup.py script.
///
/// This struct is used when a Python setup.py script is called with a command
/// that it does not support or recognize.
#[derive(Debug, Clone)]
pub struct MissingSetupPyCommand(pub String);

impl Problem for MissingSetupPyCommand {
    fn kind(&self) -> Cow<str> {
        "missing-setup.py-command".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingSetupPyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing setup.py command: {}", self.0)
    }
}

/// Problem representing a missing C# compiler.
///
/// This struct is used when a build requires a C# compiler
/// but none is available in the build environment.
#[derive(Debug, Clone)]
pub struct MissingCSharpCompiler;

impl Problem for MissingCSharpCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-c#-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingCSharpCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C# compiler")
    }
}

/// Problem representing a missing Rust compiler.
///
/// This struct is used when a build requires a Rust compiler (rustc)
/// but none is available in the build environment.
#[derive(Debug, Clone)]
pub struct MissingRustCompiler;

impl Problem for MissingRustCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-rust-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingRustCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Rust compiler")
    }
}

/// Problem representing a missing assembler.
///
/// This struct is used when a build requires an assembler (like as or nasm)
/// but none is available in the build environment.
#[derive(Debug, Clone)]
pub struct MissingAssembler;

impl Problem for MissingAssembler {
    fn kind(&self) -> Cow<str> {
        "missing-assembler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing assembler")
    }
}

/// Problem representing a missing Rust crate for Cargo.
///
/// This struct is used when a Cargo build requires a Rust crate
/// that is not available or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingCargoCrate {
    /// The name of the missing Rust crate.
    pub crate_name: String,
    /// The requirement or dependency that needs this crate, if known.
    pub requirement: Option<String>,
}

impl Problem for MissingCargoCrate {
    fn kind(&self) -> Cow<str> {
        "missing-cargo-crate".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.crate_name,
            "requirement": self.requirement
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MissingCargoCrate {
    /// Creates a simple MissingCargoCrate instance with just a crate name.
    ///
    /// # Arguments
    /// * `crate_name` - Name of the missing Rust crate
    ///
    /// # Returns
    /// A new MissingCargoCrate with no requirement information
    pub fn simple(crate_name: String) -> Self {
        Self {
            crate_name,
            requirement: None,
        }
    }
}

impl Display for MissingCargoCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(requirement) = self.requirement.as_ref() {
            write!(
                f,
                "Missing Cargo crate {} (required by {})",
                self.crate_name, requirement
            )
        } else {
            write!(f, "Missing Cargo crate {}", self.crate_name)
        }
    }
}

/// Problem representing incorrect debhelper (dh) command argument order.
///
/// This struct is used when the debhelper command is used with arguments
/// in an incorrect order, which can cause build issues in Debian packaging.
#[derive(Debug, Clone)]
pub struct DhWithOrderIncorrect;

impl Problem for DhWithOrderIncorrect {
    fn kind(&self) -> Cow<str> {
        "debhelper-argument-order".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DhWithOrderIncorrect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh argument order is incorrect")
    }
}

/// Problem representing an unsupported debhelper compatibility level.
///
/// This struct is used when a Debian package build specifies a debhelper
/// compatibility level that is lower than the minimum supported level
/// in the current environment.
#[derive(Debug, Clone)]
pub struct UnsupportedDebhelperCompatLevel {
    /// The oldest (minimum) compatibility level supported by the current debhelper.
    pub oldest_supported: u32,
    /// The compatibility level requested by the package.
    pub requested: u32,
}

impl UnsupportedDebhelperCompatLevel {
    /// Creates a new UnsupportedDebhelperCompatLevel instance.
    ///
    /// # Arguments
    /// * `oldest_supported` - The oldest (minimum) compatibility level supported
    /// * `requested` - The compatibility level requested by the package
    ///
    /// # Returns
    /// A new UnsupportedDebhelperCompatLevel instance
    pub fn new(oldest_supported: u32, requested: u32) -> Self {
        Self {
            oldest_supported,
            requested,
        }
    }
}

impl Problem for UnsupportedDebhelperCompatLevel {
    fn kind(&self) -> Cow<str> {
        "unsupported-debhelper-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "oldest_supported": self.oldest_supported,
            "requested": self.requested
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for UnsupportedDebhelperCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Request debhelper compat level {} lower than supported {}",
            self.requested, self.oldest_supported
        )
    }
}

/// Problem representing an issue with setuptools_scm version detection.
///
/// This struct is used when the setuptools_scm Python package is unable
/// to automatically determine the package version from version control
/// metadata, which typically happens when building from a source archive
/// rather than a git repository.
#[derive(Debug, Clone)]
pub struct SetuptoolScmVersionIssue;

impl Problem for SetuptoolScmVersionIssue {
    fn kind(&self) -> Cow<str> {
        "setuptools-scm-version-issue".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for SetuptoolScmVersionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "setuptools_scm was unable to find version")
    }
}

/// Problem representing missing Maven artifacts.
///
/// This struct is used when a Java build process that uses Maven
/// is missing required artifacts from Maven repositories.
#[derive(Debug, Clone)]
pub struct MissingMavenArtifacts(pub Vec<String>);

impl Problem for MissingMavenArtifacts {
    fn kind(&self) -> Cow<str> {
        "missing-maven-artifacts".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "artifacts": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingMavenArtifacts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Maven artifacts: {}", self.0.join(", "))
    }
}

/// Problem representing a file that is not executable.
///
/// This struct is used when a command or script file that needs to be
/// executed does not have the executable permission bit set.
#[derive(Debug, Clone)]
pub struct NotExecutableFile(pub String);

impl NotExecutableFile {
    /// Creates a new NotExecutableFile instance.
    ///
    /// # Arguments
    /// * `path` - Path to the non-executable file
    ///
    /// # Returns
    /// A new NotExecutableFile instance
    pub fn new(path: String) -> Self {
        Self(path)
    }
}

impl Problem for NotExecutableFile {
    fn kind(&self) -> Cow<str> {
        "not-executable-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for NotExecutableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Command not executable: {}", self.0)
    }
}

/// Problem representing a debhelper script attempting to access an uninstalled file.
///
/// This struct is used when debhelper tries to access a file that has been
/// removed or was never installed in the build environment.
#[derive(Debug, Clone)]
pub struct DhMissingUninstalled(pub String);

impl DhMissingUninstalled {
    /// Creates a new DhMissingUninstalled instance.
    ///
    /// # Arguments
    /// * `missing_file` - Path to the missing file
    ///
    /// # Returns
    /// A new DhMissingUninstalled instance
    pub fn new(missing_file: String) -> Self {
        Self(missing_file)
    }
}

impl Problem for DhMissingUninstalled {
    fn kind(&self) -> Cow<str> {
        "dh-missing-uninstalled".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "missing_file": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DhMissingUninstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh_missing file not installed: {}", self.0)
    }
}

/// Problem representing a debhelper link whose destination is a directory.
///
/// This struct is used when debhelper's dh_link attempts to create a symlink
/// to a path that is a directory, which can cause issues in package builds.
#[derive(Debug, Clone)]
pub struct DhLinkDestinationIsDirectory(pub String);

impl DhLinkDestinationIsDirectory {
    /// Creates a new DhLinkDestinationIsDirectory instance.
    ///
    /// # Arguments
    /// * `path` - Path to the directory that was incorrectly specified as a link destination
    ///
    /// # Returns
    /// A new DhLinkDestinationIsDirectory instance
    pub fn new(path: String) -> Self {
        Self(path)
    }
}

impl Problem for DhLinkDestinationIsDirectory {
    fn kind(&self) -> Cow<str> {
        "dh-link-destination-is-directory".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DhLinkDestinationIsDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Link destination {} is directory", self.0)
    }
}

/// Problem representing a missing XML entity.
///
/// This struct is used when an XML parser attempts to resolve an external
/// entity reference but the referenced entity cannot be found at the given URL.
#[derive(Debug, Clone)]
pub struct MissingXmlEntity {
    /// The URL where the XML entity was expected to be found.
    pub url: String,
}

impl MissingXmlEntity {
    /// Creates a new MissingXmlEntity instance.
    ///
    /// # Arguments
    /// * `url` - URL where the XML entity was expected to be found
    ///
    /// # Returns
    /// A new MissingXmlEntity instance
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Problem for MissingXmlEntity {
    fn kind(&self) -> Cow<str> {
        "missing-xml-entity".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingXmlEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing XML entity: {}", self.url)
    }
}

/// Problem representing an error from the ccache compiler cache.
///
/// This struct is used when the ccache tool, which accelerates repeated compilations,
/// encounters an error during its operation.
#[derive(Debug, Clone)]
pub struct CcacheError(pub String);

impl CcacheError {
    /// Creates a new CcacheError instance.
    ///
    /// # Arguments
    /// * `error` - The error message from ccache
    ///
    /// # Returns
    /// A new CcacheError instance
    pub fn new(error: String) -> Self {
        Self(error)
    }
}

impl Problem for CcacheError {
    fn kind(&self) -> Cow<str> {
        "ccache-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for CcacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ccache error: {}", self.0)
    }
}

/// Problem representing a rejected Debian package version string.
///
/// This struct is used when a version string for a Debian package is rejected
/// by Debian tools as invalid or incompatible with policy requirements.
#[derive(Debug, Clone)]
pub struct DebianVersionRejected {
    /// The version string that was rejected.
    pub version: String,
}

impl DebianVersionRejected {
    /// Creates a new DebianVersionRejected instance.
    ///
    /// # Arguments
    /// * `version` - The version string that was rejected
    ///
    /// # Returns
    /// A new DebianVersionRejected instance
    pub fn new(version: String) -> Self {
        Self { version }
    }
}

impl Problem for DebianVersionRejected {
    fn kind(&self) -> Cow<str> {
        "debian-version-rejected".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DebianVersionRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Debian Version Rejected; {}", self.version)
    }
}

/// Problem representing a failure to apply a patch.
///
/// This struct is used when a build process fails because a patch
/// cannot be successfully applied to the source code.
#[derive(Debug, Clone)]
pub struct PatchApplicationFailed {
    /// The name of the patch file that could not be applied.
    pub patchname: String,
}

impl PatchApplicationFailed {
    /// Creates a new PatchApplicationFailed instance.
    ///
    /// # Arguments
    /// * `patchname` - Name of the patch file that failed to apply
    ///
    /// # Returns
    /// A new PatchApplicationFailed instance
    pub fn new(patchname: String) -> Self {
        Self { patchname }
    }
}

impl Problem for PatchApplicationFailed {
    fn kind(&self) -> Cow<str> {
        "patch-application-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "patchname": self.patchname
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for PatchApplicationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Patch application failed: {}", self.patchname)
    }
}

/// Problem representing a need to update PostgreSQL build extension control files.
///
/// This struct is used when PostgreSQL extension build files need to be updated
/// using the pg_buildext updatecontrol command to generate control files from templates.
#[derive(Debug, Clone)]
pub struct NeedPgBuildExtUpdateControl {
    /// The path to the generated control file.
    pub generated_path: String,
    /// The path to the template file to use for generation.
    pub template_path: String,
}

impl NeedPgBuildExtUpdateControl {
    /// Creates a new NeedPgBuildExtUpdateControl instance.
    ///
    /// # Arguments
    /// * `generated_path` - Path to the generated control file
    /// * `template_path` - Path to the template file
    ///
    /// # Returns
    /// A new NeedPgBuildExtUpdateControl instance
    pub fn new(generated_path: String, template_path: String) -> Self {
        Self {
            generated_path,
            template_path,
        }
    }
}

impl Problem for NeedPgBuildExtUpdateControl {
    fn kind(&self) -> Cow<str> {
        "need-pg-buildext-updatecontrol".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "generated_path": self.generated_path,
            "template_path": self.template_path
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for NeedPgBuildExtUpdateControl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Need to run 'pg_buildext updatecontrol' to update {}",
            self.generated_path
        )
    }
}

/// Problem representing a failure to load a debhelper addon.
///
/// This struct is used when debhelper fails to load an addon module,
/// which typically provides additional functionality to the debhelper tools.
#[derive(Debug, Clone)]
pub struct DhAddonLoadFailure {
    /// The name of the addon that failed to load.
    pub name: String,
    /// The path where the addon was expected to be found.
    pub path: String,
}

impl DhAddonLoadFailure {
    /// Creates a new DhAddonLoadFailure instance.
    ///
    /// # Arguments
    /// * `name` - Name of the addon that failed to load
    /// * `path` - Path where the addon was expected to be found
    ///
    /// # Returns
    /// A new DhAddonLoadFailure instance
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }
}

impl Problem for DhAddonLoadFailure {
    fn kind(&self) -> Cow<str> {
        "dh-addon-load-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "path": self.path
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DhAddonLoadFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh addon loading failed: {}", self.name)
    }
}

/// Problem representing an unsupported usage of the --until flag in debhelper.
///
/// This struct is used when the --until flag is used with debhelper (dh)
/// but the version of debhelper in use does not support this option.
#[derive(Debug, Clone)]
pub struct DhUntilUnsupported;

impl Default for DhUntilUnsupported {
    /// Provides a default instance of DhUntilUnsupported.
    ///
    /// # Returns
    /// A new DhUntilUnsupported instance
    fn default() -> Self {
        Self::new()
    }
}

impl DhUntilUnsupported {
    /// Creates a new DhUntilUnsupported instance.
    ///
    /// # Returns
    /// A new DhUntilUnsupported instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for DhUntilUnsupported {
    fn kind(&self) -> Cow<str> {
        "dh-until-unsupported".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DhUntilUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh --until is no longer supported")
    }
}

/// Problem representing a debhelper file pattern that was not found.
///
/// This struct is used when a debhelper tool is looking for files matching
/// a specific pattern but cannot find any matches in the searched directories.
#[derive(Debug, Clone)]
pub struct DebhelperPatternNotFound {
    /// The file pattern that was being searched for.
    pub pattern: String,
    /// The name of the debhelper tool that was performing the search.
    pub tool: String,
    /// The list of directories that were searched.
    pub directories: Vec<String>,
}

impl DebhelperPatternNotFound {
    /// Creates a new DebhelperPatternNotFound instance.
    ///
    /// # Arguments
    /// * `pattern` - The file pattern that was being searched for
    /// * `tool` - The name of the debhelper tool
    /// * `directories` - The list of directories that were searched
    ///
    /// # Returns
    /// A new DebhelperPatternNotFound instance
    pub fn new(pattern: String, tool: String, directories: Vec<String>) -> Self {
        Self {
            pattern,
            tool,
            directories,
        }
    }
}

impl Problem for DebhelperPatternNotFound {
    fn kind(&self) -> Cow<str> {
        "debhelper-pattern-not-found".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "pattern": self.pattern,
            "tool": self.tool,
            "directories": self.directories
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DebhelperPatternNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "debhelper ({}) expansion failed for {:?} (directories: {:?})",
            self.tool, self.pattern, self.directories
        )
    }
}

/// Problem representing a missing Perl MANIFEST file.
///
/// This struct is used when a Perl module build expects to find a MANIFEST
/// file listing all files in the distribution, but it doesn't exist.
#[derive(Debug, Clone)]
pub struct MissingPerlManifest;

impl Default for MissingPerlManifest {
    /// Provides a default instance of MissingPerlManifest.
    ///
    /// # Returns
    /// A new MissingPerlManifest instance
    fn default() -> Self {
        Self::new()
    }
}

impl MissingPerlManifest {
    /// Creates a new MissingPerlManifest instance.
    ///
    /// # Returns
    /// A new MissingPerlManifest instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingPerlManifest {
    fn kind(&self) -> Cow<str> {
        "missing-perl-manifest".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPerlManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Perl MANIFEST")
    }
}

/// Problem representing a missing ImageMagick delegate.
///
/// This struct is used when ImageMagick requires a delegate library
/// to handle a specific file format or operation, but the delegate is not available.
#[derive(Debug, Clone)]
pub struct ImageMagickDelegateMissing {
    /// The name of the missing ImageMagick delegate.
    pub delegate: String,
}

impl ImageMagickDelegateMissing {
    /// Creates a new ImageMagickDelegateMissing instance.
    ///
    /// # Arguments
    /// * `delegate` - Name of the missing ImageMagick delegate
    ///
    /// # Returns
    /// A new ImageMagickDelegateMissing instance
    pub fn new(delegate: String) -> Self {
        Self { delegate }
    }
}

impl Problem for ImageMagickDelegateMissing {
    fn kind(&self) -> Cow<str> {
        "imagemagick-delegate-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "delegate": self.delegate
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for ImageMagickDelegateMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Imagemagick missing delegate: {}", self.delegate)
    }
}

/// Problem representing a cancelled build or operation.
///
/// This struct is used when a build process or operation was cancelled
/// before completion, typically by user intervention or a timeout.
#[derive(Debug, Clone)]
pub struct Cancelled;

impl Default for Cancelled {
    /// Provides a default instance of Cancelled.
    ///
    /// # Returns
    /// A new Cancelled instance
    fn default() -> Self {
        Self::new()
    }
}

impl Cancelled {
    /// Creates a new Cancelled instance.
    ///
    /// # Returns
    /// A new Cancelled instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for Cancelled {
    fn kind(&self) -> Cow<str> {
        "cancelled".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cancelled by runner or job manager")
    }
}

/// Problem representing symbols that have disappeared from a library.
///
/// This struct is used when symbols (functions or variables) that were previously
/// exported by a library are no longer present, which can break API compatibility.
#[derive(Debug, Clone)]
pub struct DisappearedSymbols;

impl Default for DisappearedSymbols {
    /// Provides a default instance of DisappearedSymbols.
    ///
    /// # Returns
    /// A new DisappearedSymbols instance
    fn default() -> Self {
        Self::new()
    }
}

impl DisappearedSymbols {
    /// Creates a new DisappearedSymbols instance.
    ///
    /// # Returns
    /// A new DisappearedSymbols instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for DisappearedSymbols {
    fn kind(&self) -> Cow<str> {
        "disappeared-symbols".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DisappearedSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Disappeared symbols")
    }
}

/// Problem representing duplicate debhelper compatibility level specifications.
///
/// This struct is used when the debhelper compatibility level is specified
/// multiple times in different places, which can lead to conflicts.
#[derive(Debug, Clone)]
pub struct DuplicateDHCompatLevel {
    /// The command or file where the duplicate compatibility level was found.
    pub command: String,
}

impl DuplicateDHCompatLevel {
    /// Creates a new DuplicateDHCompatLevel instance.
    ///
    /// # Arguments
    /// * `command` - The command or file with the duplicate compatibility level
    ///
    /// # Returns
    /// A new DuplicateDHCompatLevel instance
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl Problem for DuplicateDHCompatLevel {
    fn kind(&self) -> Cow<str> {
        "duplicate-dh-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.command
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for DuplicateDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "DH Compat Level specified twice (command: {})",
            self.command
        )
    }
}

/// Problem representing a missing debhelper compatibility level specification.
///
/// This struct is used when debhelper requires a compatibility level to be
/// specified, but none was found in the expected locations.
#[derive(Debug, Clone)]
pub struct MissingDHCompatLevel {
    /// The command that reported the missing compatibility level.
    pub command: String,
}

impl MissingDHCompatLevel {
    /// Creates a new MissingDHCompatLevel instance.
    ///
    /// # Arguments
    /// * `command` - The command that reported the missing compatibility level
    ///
    /// # Returns
    /// A new MissingDHCompatLevel instance
    pub fn new(command: String) -> Self {
        Self { command }
    }
}

impl Problem for MissingDHCompatLevel {
    fn kind(&self) -> Cow<str> {
        "missing-dh-compat-level".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "command": self.command
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing DH Compat Level (command: {})", self.command)
    }
}

/// Problem representing a missing Java Virtual Machine (JVM).
///
/// This struct is used when a build process requires a Java Virtual Machine
/// but cannot find one installed or properly configured in the system.
#[derive(Debug, Clone)]
pub struct MissingJVM;

impl Default for MissingJVM {
    /// Provides a default instance of MissingJVM.
    ///
    /// # Returns
    /// A new MissingJVM instance
    fn default() -> Self {
        Self::new()
    }
}

impl MissingJVM {
    /// Creates a new MissingJVM instance.
    ///
    /// # Returns
    /// A new MissingJVM instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingJVM {
    fn kind(&self) -> Cow<str> {
        "missing-jvm".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJVM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing JVM")
    }
}

/// Problem representing a missing Ruby gem.
///
/// This struct is used when a build process requires a Ruby gem
/// that is not installed or available in the current environment.
#[derive(Debug, Clone)]
pub struct MissingRubyGem {
    /// The name of the missing Ruby gem.
    pub gem: String,
    /// The required version of the gem, if specified.
    pub version: Option<String>,
}

impl MissingRubyGem {
    /// Creates a new MissingRubyGem instance.
    ///
    /// # Arguments
    /// * `gem` - Name of the missing Ruby gem
    /// * `version` - Optional version requirement for the gem
    ///
    /// # Returns
    /// A new MissingRubyGem instance
    pub fn new(gem: String, version: Option<String>) -> Self {
        Self { gem, version }
    }

    /// Creates a simple MissingRubyGem instance without version requirements.
    ///
    /// # Arguments
    /// * `gem` - Name of the missing Ruby gem
    ///
    /// # Returns
    /// A new MissingRubyGem instance with no version requirements
    pub fn simple(gem: String) -> Self {
        Self::new(gem, None)
    }
}

impl Problem for MissingRubyGem {
    fn kind(&self) -> Cow<str> {
        "missing-ruby-gem".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "gem": self.gem,
            "version": self.version
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingRubyGem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "missing ruby gem: {} (>= {})", self.gem, version)
        } else {
            write!(f, "missing ruby gem: {}", self.gem)
        }
    }
}

/// Problem representing a missing JavaScript runtime environment.
///
/// This struct is used when a build process requires a JavaScript runtime
/// (like Node.js, Deno, or a browser JavaScript engine) but none is available.
#[derive(Debug, Clone)]
pub struct MissingJavaScriptRuntime;

impl Default for MissingJavaScriptRuntime {
    /// Provides a default instance of MissingJavaScriptRuntime.
    ///
    /// # Returns
    /// A new MissingJavaScriptRuntime instance
    fn default() -> Self {
        Self::new()
    }
}

impl MissingJavaScriptRuntime {
    /// Creates a new MissingJavaScriptRuntime instance.
    ///
    /// # Returns
    /// A new MissingJavaScriptRuntime instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingJavaScriptRuntime {
    fn kind(&self) -> Cow<str> {
        "javascript-runtime-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJavaScriptRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JavaScript Runtime")
    }
}

/// Problem representing a missing Ruby source file.
///
/// This struct is used when a Ruby application or library tries to
/// load or require a Ruby file that does not exist or cannot be found.
#[derive(Debug, Clone)]
pub struct MissingRubyFile {
    /// The name or path of the missing Ruby file.
    pub filename: String,
}

impl MissingRubyFile {
    /// Creates a new MissingRubyFile instance.
    ///
    /// # Arguments
    /// * `filename` - Name or path of the missing Ruby file
    ///
    /// # Returns
    /// A new MissingRubyFile instance
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl Problem for MissingRubyFile {
    fn kind(&self) -> Cow<str> {
        "missing-ruby-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.filename
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingRubyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing ruby file: {}", self.filename)
    }
}

/// Problem representing a missing PHP class.
///
/// This struct is used when a PHP application tries to use a class
/// that has not been defined or cannot be autoloaded.
#[derive(Debug, Clone)]
pub struct MissingPhpClass {
    /// The name of the missing PHP class.
    pub php_class: String,
}

impl MissingPhpClass {
    /// Creates a new MissingPhpClass instance.
    ///
    /// # Arguments
    /// * `php_class` - Name of the missing PHP class
    ///
    /// # Returns
    /// A new MissingPhpClass instance
    pub fn new(php_class: String) -> Self {
        Self { php_class }
    }

    /// Creates a simple MissingPhpClass instance.
    ///
    /// This is an alias for new() for API consistency with other similar types.
    ///
    /// # Arguments
    /// * `php_class` - Name of the missing PHP class
    ///
    /// # Returns
    /// A new MissingPhpClass instance
    pub fn simple(php_class: String) -> Self {
        Self::new(php_class)
    }
}

impl Problem for MissingPhpClass {
    fn kind(&self) -> Cow<str> {
        "missing-php-class".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "php_class": self.php_class
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingPhpClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing PHP class: {}", self.php_class)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Java class.
///
/// This struct is used when a Java application or build process
/// requires a Java class that cannot be found in the classpath.
pub struct MissingJavaClass {
    /// The name of the missing Java class.
    pub classname: String,
}

impl MissingJavaClass {
    /// Creates a new MissingJavaClass instance.
    ///
    /// # Arguments
    /// * `classname` - Name of the missing Java class
    ///
    /// # Returns
    /// A new MissingJavaClass instance
    pub fn new(classname: String) -> Self {
        Self { classname }
    }

    /// Creates a simple MissingJavaClass instance.
    ///
    /// This is an alias for new() for API consistency with other similar types.
    ///
    /// # Arguments
    /// * `classname` - Name of the missing Java class
    ///
    /// # Returns
    /// A new MissingJavaClass instance
    pub fn simple(classname: String) -> Self {
        Self::new(classname)
    }
}

impl Problem for MissingJavaClass {
    fn kind(&self) -> Cow<str> {
        "missing-java-class".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "classname": self.classname
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingJavaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Java class: {}", self.classname)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Sprockets asset file.
///
/// This struct is used when a Ruby on Rails application using the Sprockets
/// asset pipeline is missing a required asset file.
pub struct MissingSprocketsFile {
    /// The name of the missing Sprockets asset file.
    pub name: String,
    /// The content type of the missing asset file.
    pub content_type: String,
}

impl MissingSprocketsFile {
    /// Creates a new MissingSprocketsFile instance.
    ///
    /// # Arguments
    /// * `name` - Name of the missing Sprockets asset file
    /// * `content_type` - Content type of the missing asset file
    ///
    /// # Returns
    /// A new MissingSprocketsFile instance
    pub fn new(name: String, content_type: String) -> Self {
        Self { name, content_type }
    }
}

impl Problem for MissingSprocketsFile {
    fn kind(&self) -> Cow<str> {
        "missing-sprockets-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "content_type": self.content_type
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingSprocketsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "missing sprockets file: {} (type: {})",
            self.name, self.content_type
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Xfce desktop environment dependency.
///
/// This struct is used when a package build requires an Xfce-specific
/// dependency package that is not available.
pub struct MissingXfceDependency {
    /// The name of the missing Xfce dependency package.
    pub package: String,
}

impl MissingXfceDependency {
    /// Creates a new MissingXfceDependency instance.
    ///
    /// # Arguments
    /// * `package` - Name of the missing Xfce dependency package
    ///
    /// # Returns
    /// A new MissingXfceDependency instance
    pub fn new(package: String) -> Self {
        Self { package }
    }
}

impl Problem for MissingXfceDependency {
    fn kind(&self) -> Cow<str> {
        "missing-xfce-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingXfceDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing XFCE build dependency: {}", self.package)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing GNOME common build tools and macros.
///
/// This struct is used when a GNOME-related package build requires the
/// gnome-common package, which provides common build tools and macros for GNOME projects.
pub struct GnomeCommonMissing;

impl Problem for GnomeCommonMissing {
    fn kind(&self) -> Cow<str> {
        "missing-gnome-common".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for GnomeCommonMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "gnome-common is not installed")
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing input file for GNU config.status.
///
/// This struct is used when the GNU autotools config.status script
/// is missing one of its required input files.
pub struct MissingConfigStatusInput {
    /// The path to the missing input file.
    pub path: String,
}

impl MissingConfigStatusInput {
    /// Creates a new MissingConfigStatusInput instance.
    ///
    /// # Arguments
    /// * `path` - Path to the missing config.status input file
    ///
    /// # Returns
    /// A new MissingConfigStatusInput instance
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Problem for MissingConfigStatusInput {
    fn kind(&self) -> Cow<str> {
        "missing-config.status-input".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingConfigStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing config.status input {}", self.path)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing GNOME common dependency.
///
/// This struct is used when a GNOME-related package build requires a dependency
/// that is typically provided by or related to the gnome-common infrastructure.
pub struct MissingGnomeCommonDependency {
    /// The name of the missing GNOME common dependency package.
    pub package: String,
    /// The minimum required version of the dependency, if specified.
    pub minimum_version: Option<String>,
}

impl MissingGnomeCommonDependency {
    /// Creates a new MissingGnomeCommonDependency instance.
    ///
    /// # Arguments
    /// * `package` - Name of the missing GNOME common dependency
    /// * `minimum_version` - Optional minimum version requirement
    ///
    /// # Returns
    /// A new MissingGnomeCommonDependency instance
    pub fn new(package: String, minimum_version: Option<String>) -> Self {
        Self {
            package,
            minimum_version,
        }
    }

    /// Creates a simple MissingGnomeCommonDependency instance without version constraints.
    ///
    /// # Arguments
    /// * `package` - Name of the missing GNOME common dependency
    ///
    /// # Returns
    /// A new MissingGnomeCommonDependency instance with no version requirements
    pub fn simple(package: String) -> Self {
        Self::new(package, None)
    }
}

impl Problem for MissingGnomeCommonDependency {
    fn kind(&self) -> Cow<str> {
        "missing-gnome-common-dependency".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "minimum_version": self.minimum_version
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingGnomeCommonDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "missing gnome-common dependency: {}: (>= {})",
            self.package,
            self.minimum_version.as_deref().unwrap_or("any")
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing input file for GNU Automake.
///
/// This struct is used when GNU Automake cannot find a required input
/// file that it needs to generate build files.
pub struct MissingAutomakeInput {
    /// The path to the missing input file.
    pub path: String,
}

impl MissingAutomakeInput {
    /// Creates a new MissingAutomakeInput instance.
    ///
    /// # Arguments
    /// * `path` - Path to the missing Automake input file
    ///
    /// # Returns
    /// A new MissingAutomakeInput instance
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Problem for MissingAutomakeInput {
    fn kind(&self) -> Cow<str> {
        "missing-automake-input".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingAutomakeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "automake input file {} missing", self.path)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a chroot environment that could not be found.
///
/// This struct is used when a build process tries to use a chroot environment
/// (a root directory that appears as the system root to enclosed processes),
/// but the specified chroot does not exist.
pub struct ChrootNotFound {
    /// The path or name of the chroot that could not be found.
    pub chroot: String,
}

impl ChrootNotFound {
    /// Creates a new ChrootNotFound instance.
    ///
    /// # Arguments
    /// * `chroot` - Path or name of the chroot that could not be found
    ///
    /// # Returns
    /// A new ChrootNotFound instance
    pub fn new(chroot: String) -> Self {
        Self { chroot }
    }
}

impl Problem for ChrootNotFound {
    fn kind(&self) -> Cow<str> {
        "chroot-not-found".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "chroot": self.chroot
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for ChrootNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "chroot not found: {}", self.chroot)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing GNU Libtool.
///
/// This struct is used when a build process requires the GNU Libtool
/// utility for creating portable shared libraries, but it is not installed.
pub struct MissingLibtool;

impl Default for MissingLibtool {
    /// Provides a default instance of MissingLibtool.
    ///
    /// # Returns
    /// A new MissingLibtool instance
    fn default() -> Self {
        Self::new()
    }
}

impl MissingLibtool {
    /// Creates a new MissingLibtool instance.
    ///
    /// # Returns
    /// A new MissingLibtool instance
    pub fn new() -> Self {
        Self
    }
}

impl Problem for MissingLibtool {
    fn kind(&self) -> Cow<str> {
        "missing-libtool".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for MissingLibtool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Libtool is missing")
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing CMake files.
///
/// This struct is used when a CMake-based build process cannot find
/// required CMake module or configuration files.
pub struct CMakeFilesMissing {
    /// The names of the missing CMake files.
    pub filenames: Vec<String>,
    /// The version of CMake that was requested, if specified.
    pub version: Option<String>,
}

impl Problem for CMakeFilesMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-cmake-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filenames": self.filenames,
            "version": self.version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for CMakeFilesMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CMake files missing: {:?}", self.filenames)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing CMake package components.
///
/// This struct is used when a CMake-based build process requires specific
/// components of a package, but they cannot be found.
pub struct MissingCMakeComponents {
    /// The name of the CMake package.
    pub name: String,
    /// The names of the missing components.
    pub components: Vec<String>,
}

impl Problem for MissingCMakeComponents {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-cmake-components".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "components": self.components,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingCMakeComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing CMake components: {:?}", self.components)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing CMake package configuration.
///
/// This struct is used when a CMake-based build process cannot find
/// a required package configuration file.
pub struct MissingCMakeConfig {
    /// The name of the CMake package.
    pub name: String,
    /// The version of the package that was requested, if specified.
    pub version: Option<String>,
}

impl Problem for MissingCMakeConfig {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-cmake-config".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "version": self.version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingCMakeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(
                f,
                "Missing CMake package configuration for {} (version {})",
                self.name, version
            )
        } else {
            write!(f, "Missing CMake package configuration for {}", self.name)
        }
    }
}

#[derive(Debug, Clone)]
/// Problem representing a CMake package with mismatched version requirements.
///
/// This struct is used when a CMake-based build process found a package,
/// but it requires an exact version that doesn't match the found version.
pub struct CMakeNeedExactVersion {
    /// The name of the CMake package.
    pub package: String,
    /// The version of the package that was found.
    pub version_found: String,
    /// The exact version required by the build.
    pub exact_version_needed: String,
    /// The path to the CMake package configuration file.
    pub path: PathBuf,
}

impl Problem for CMakeNeedExactVersion {
    fn kind(&self) -> std::borrow::Cow<str> {
        "cmake-exact-version-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "version_found": self.version_found,
            "exact_version_needed": self.exact_version_needed,
            "path": self.path,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for CMakeNeedExactVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CMake needs exact package {}, version {}",
            self.package, self.exact_version_needed
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing static library file.
///
/// This struct is used when a build process requires a static library
/// (typically a .a or .lib file) but it cannot be found.
pub struct MissingStaticLibrary {
    /// The name of the library (without file extension).
    pub library: String,
    /// The expected filename of the static library.
    pub filename: String,
}

impl Problem for MissingStaticLibrary {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-static-library".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "library": self.library,
            "filename": self.filename,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingStaticLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing static library: {}", self.library)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Go runtime.
///
/// This struct is used when a build process requires the Go language runtime
/// but it is not installed or cannot be found in the system.
pub struct MissingGoRuntime;

impl Problem for MissingGoRuntime {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-go-runtime".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingGoRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go runtime is missing")
    }
}

#[derive(Debug, Clone)]
/// Problem representing an unknown SSL/TLS certificate authority.
///
/// This struct is used when a build process fails to establish a secure connection
/// because it cannot verify the certificate authority of a remote server.
pub struct UnknownCertificateAuthority(pub String);

impl Problem for UnknownCertificateAuthority {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unknown-certificate-authority".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UnknownCertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unknown Certificate Authority for {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing predeclared function in Perl.
///
/// This struct is used when a Perl script tries to use a predeclared function
/// that is not available, often because a required module is not loaded.
pub struct MissingPerlPredeclared(pub String);

impl Problem for MissingPerlPredeclared {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-perl-predeclared".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingPerlPredeclared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing predeclared function: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing Git user identity configuration.
///
/// This struct is used when Git operations that require user identity
/// (like commits) fail because the user.name and user.email are not configured.
pub struct MissingGitIdentity;

impl Problem for MissingGitIdentity {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-git-identity".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingGitIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Git Identity")
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing GPG secret key.
///
/// This struct is used when an operation requires a GPG secret key
/// (such as signing packages or commits) but no secret key is available.
pub struct MissingSecretGpgKey;

impl Problem for MissingSecretGpgKey {
    fn kind(&self) -> std::borrow::Cow<str> {
        "no-secret-gpg-key".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingSecretGpgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No secret GPG key is present")
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing version information for vcversioner.
///
/// This struct is used when the vcversioner Python package cannot determine
/// the version from either a Git directory or a version.txt file.
pub struct MissingVcVersionerVersion;

impl Problem for MissingVcVersionerVersion {
    fn kind(&self) -> std::borrow::Cow<str> {
        "no-vcversioner-version".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingVcVersionerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "vcversion could not find a git directory or version.txt file"
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing LaTeX file.
///
/// This struct is used when a LaTeX build process requires a file
/// (such as a class file, style file, or content file) but it cannot be found.
pub struct MissingLatexFile(pub String);

impl MissingLatexFile {
    /// Creates a new MissingLatexFile instance.
    ///
    /// # Arguments
    /// * `filename` - Name of the missing LaTeX file
    ///
    /// # Returns
    /// A new MissingLatexFile instance
    pub fn new(filename: String) -> Self {
        Self(filename)
    }
}

impl Problem for MissingLatexFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-latex-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingLatexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing LaTeX file: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing X Window System display.
///
/// This struct is used when a program requires an X11 display connection
/// but no display server is available (such as in headless environments).
pub struct MissingXDisplay;

impl Problem for MissingXDisplay {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-x-display".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingXDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No X Display")
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing font specification in LaTeX.
///
/// This struct is used when a LaTeX document requires a specific font
/// but the fontspec package cannot find it.
pub struct MissingFontspec(pub String);

impl Problem for MissingFontspec {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-fontspec".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "fontspec": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingFontspec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing font spec: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a process killed due to inactivity.
///
/// This struct is used when a build process was killed by the system
/// because it was inactive for too long.
pub struct InactiveKilled(pub i64);

impl Problem for InactiveKilled {
    fn kind(&self) -> std::borrow::Cow<str> {
        "inactive-killed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "minutes": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for InactiveKilled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Killed due to inactivity after {} minutes", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing PAUSE credentials for Perl module upload.
///
/// This struct is used when attempting to upload a Perl module to PAUSE
/// (Perl Authors Upload Server) without proper authentication credentials.
pub struct MissingPauseCredentials;

impl Problem for MissingPauseCredentials {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-pause-credentials".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingPauseCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing credentials for PAUSE")
    }
}

#[derive(Debug, Clone)]
/// Problem representing mismatched gettext versions.
///
/// This struct is used when there's a version mismatch between gettext versions
/// referenced in Makefile.in.in and the autoconf macros.
pub struct MismatchGettextVersions {
    /// The gettext version specified in the Makefile.
    pub makefile_version: String,
    /// The gettext version specified in autoconf macros.
    pub autoconf_version: String,
}

impl Problem for MismatchGettextVersions {
    fn kind(&self) -> std::borrow::Cow<str> {
        "mismatch-gettext-versions".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "makefile_version": self.makefile_version,
            "autoconf_version": self.autoconf_version
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MismatchGettextVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Mismatch versions ({}, {})",
            self.makefile_version, self.autoconf_version
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing an invalid current user for a build operation.
///
/// This struct is used when a build process encounters issues because
/// it's running under an unexpected or inappropriate user account.
pub struct InvalidCurrentUser(pub String);

impl Problem for InvalidCurrentUser {
    fn kind(&self) -> std::borrow::Cow<str> {
        "invalid-current-user".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "user": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for InvalidCurrentUser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not run as {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing GNU lib directory.
///
/// This struct is used when a build process requires a gnulib directory
/// (a collection of portable GNU utility functions) but it cannot be found.
pub struct MissingGnulibDirectory(pub PathBuf);

impl Problem for MissingGnulibDirectory {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-gnulib-directory".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "directory": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingGnulibDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing gnulib directory: {}", self.0.display())
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Lua module.
///
/// This struct is used when a Lua script or application attempts to
/// load a Lua module that is not installed or cannot be found.
pub struct MissingLuaModule(pub String);

impl Problem for MissingLuaModule {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-lua-module".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "module": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingLuaModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Lua Module: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Go module file.
///
/// This struct is used when a Go project requires a go.mod file for
/// module and dependency management, but the file is missing.
pub struct MissingGoModFile;

impl Problem for MissingGoModFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-go.mod-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is missing")
    }
}

#[derive(Debug, Clone)]
/// Problem representing an outdated Go module file.
///
/// This struct is used when a Go project's go.mod file needs to be
/// updated due to changes in dependencies or Go version requirements.
pub struct OutdatedGoModFile;

impl Problem for OutdatedGoModFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "outdated-go.mod-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for OutdatedGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is outdated")
    }
}

#[derive(Debug, Clone)]
/// Problem representing insufficient code test coverage.
///
/// This struct is used when a build process requires a minimum level of
/// code test coverage, but the actual coverage is below the required threshold.
pub struct CodeCoverageTooLow {
    /// The actual code coverage percentage achieved.
    pub actual: f64,
    /// The minimum code coverage percentage required.
    pub required: f64,
}

impl Problem for CodeCoverageTooLow {
    fn kind(&self) -> std::borrow::Cow<str> {
        "code-coverage-too-low".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "actual": self.actual,
            "required": self.required
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for CodeCoverageTooLow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Code coverage too low: {:.2} < {:.2}",
            self.actual, self.required
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing improper usage of ES modules.
///
/// This struct is used when a JavaScript module is using CommonJS require()
/// syntax to load an ES module, which must be loaded with import() instead.
pub struct ESModuleMustUseImport(pub String);

impl Problem for ESModuleMustUseImport {
    fn kind(&self) -> std::borrow::Cow<str> {
        "esmodule-must-use-import".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for ESModuleMustUseImport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ESM-only module {} must use import()", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing PHP extension.
///
/// This struct is used when a PHP application requires an extension
/// (like mysqli, gd, intl, etc.) that is not installed or enabled.
pub struct MissingPHPExtension(pub String);

impl Problem for MissingPHPExtension {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-php-extension".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "extension": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingPHPExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PHP Extension: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing an outdated minimum autoconf version requirement.
///
/// This struct is used when a project's configure script specifies a minimum autoconf
/// version that is considered too old for modern builds.
pub struct MinimumAutoconfTooOld(pub String);

impl Problem for MinimumAutoconfTooOld {
    fn kind(&self) -> std::borrow::Cow<str> {
        "minimum-autoconf-too-old".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "minimum_version": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MinimumAutoconfTooOld {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "configure.{{ac,in}} should require newer autoconf {}",
            self.0
        )
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing file in a Perl distribution.
///
/// This struct is used when a Perl module build or installation process
/// cannot find a required file that should be part of the distribution.
pub struct MissingPerlDistributionFile(pub String);

impl Problem for MissingPerlDistributionFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-perl-distribution-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "filename": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingPerlDistributionFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing perl distribution file: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing entry in the Go checksum file.
///
/// This struct is used when a Go project requires an entry in the go.sum file
/// for a specific package version, but the entry is missing.
pub struct MissingGoSumEntry {
    /// The package import path.
    pub package: String,
    /// The version of the package.
    pub version: String,
}

impl Problem for MissingGoSumEntry {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-go.sum-entry".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "version": self.version
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingGoSumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing go.sum entry: {}@{}", self.package, self.version)
    }
}

#[derive(Debug, Clone)]
/// Problem representing an issue with the Vala compiler.
///
/// This struct is used when the Vala compiler (valac) encounters
/// an error that prevents it from compiling Vala source code.
pub struct ValaCompilerCannotCompile;

impl Problem for ValaCompilerCannotCompile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "valac-cannot-compile".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for ValaCompilerCannotCompile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "valac can not compile")
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Debian build dependency.
///
/// This struct is used when a Debian package build requires a dependency
/// that is listed in Build-Depends but is not installed in the build environment.
pub struct MissingDebianBuildDep(pub String);

impl Problem for MissingDebianBuildDep {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-debian-build-dep".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "dep": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingDebianBuildDep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Debian Build-Depends: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing missing Qt modules.
///
/// This struct is used when a build process requires specific Qt modules
/// (like QtCore, QtGui, QtWidgets, etc.) that are not available.
pub struct MissingQtModules(pub Vec<String>);

impl Problem for MissingQtModules {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-qt-modules".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "modules": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingQtModules {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing QT modules: {:?}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing OCaml package.
///
/// This struct is used when an OCaml project requires a package
/// that is not installed or cannot be found in the OCaml environment.
pub struct MissingOCamlPackage(pub String);

impl Problem for MissingOCamlPackage {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-ocaml-package".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingOCamlPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing OCaml package: {}", self.0)
    }
}

#[derive(Debug, Clone)]
/// Problem representing a "too many open files" error.
///
/// This struct is used when a process hits the system limit for the number
/// of files that can be opened simultaneously, often due to a resource leak.
pub struct TooManyOpenFiles;

impl Problem for TooManyOpenFiles {
    fn kind(&self) -> std::borrow::Cow<str> {
        "too-many-open-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for TooManyOpenFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Too many open files")
    }
}

#[derive(Debug, Clone)]
/// Problem representing a missing Makefile target.
///
/// This struct is used when a build process tries to run a make target
/// that doesn't exist in the Makefile.
pub struct MissingMakeTarget(pub String, pub Option<String>);

impl MissingMakeTarget {
    /// Creates a new MissingMakeTarget instance.
    ///
    /// # Arguments
    /// * `target` - The name of the missing make target
    /// * `required_by` - Optional name of the entity that requires this target
    ///
    /// # Returns
    /// A new MissingMakeTarget instance
    pub fn new(target: &str, required_by: Option<&str>) -> Self {
        Self(target.to_string(), required_by.map(String::from))
    }

    /// Creates a simple MissingMakeTarget instance without specifying what requires it.
    ///
    /// # Arguments
    /// * `target` - The name of the missing make target
    ///
    /// # Returns
    /// A new MissingMakeTarget instance with no requirer information
    pub fn simple(target: &str) -> Self {
        Self::new(target, None)
    }
}

impl std::fmt::Display for MissingMakeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unknown make target: {}", self.0)
    }
}

impl Problem for MissingMakeTarget {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-make-target".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "target": self.0,
            "required_by": self.1
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
