use pep508_rs::pep440_rs;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::path::PathBuf;

/// Problem representing a file that was expected but not found.
///
/// This struct is used to report situations where a required file is missing,
/// which may cause build or execution failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingFile, "missing-file", ["path"]);

impl Display for MissingFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing file: {}", self.path.display())
    }
}

/// Problem representing a missing build system file.
///
/// This struct is used to report when a file required by the build system
/// (such as a Makefile, CMakeLists.txt, etc.) is missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingBuildFile, "missing-build-file", ["filename"]);

impl Display for MissingBuildFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing build file: {}", self.filename)
    }
}

/// Problem representing something that could be either a missing command or build file.
///
/// This struct is used when it's not clear whether a missing entity is a
/// command (executable) or a build file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCommandOrBuildFile {
    /// The name of the missing command or build file.
    pub filename: String,
}

crate::problem!(
    MissingCommandOrBuildFile,
    "missing-command-or-build-file",
    ["filename"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcsControlDirectoryNeeded {
    /// List of version control systems that could provide the needed directory.
    pub vcs: Vec<String>,
}

crate::problem!(
    VcsControlDirectoryNeeded,
    "vcs-control-directory-needed",
    ["vcs"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingPythonModule,
    "missing-python-module",
    ["module", "python_version", "minimum_version"]
);

/// Problem representing a missing command-line executable.
///
/// This struct is used when a required command is not available in the PATH.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCommand {
    /// The name of the missing command.
    pub command: String,
}

impl Display for MissingCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command: {}", self.command)
    }
}

crate::problem!(MissingCommand, "command-missing", ["command"]);

/// Problem representing a missing Python package distribution.
///
/// This struct is used when a required Python package is not available,
/// which may include version constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingPythonDistribution,
    "missing-python-distribution",
    ["distribution", "python_version", "minimum_version"]
);

fn find_python_version(marker: Vec<Vec<pep508_rs::MarkerExpression>>) -> Option<i32> {
    let mut major_version = None;
    for expr in marker.iter().flat_map(|x| x.iter()) {
        if let pep508_rs::MarkerExpression::Version {
            key: pep508_rs::MarkerValueVersion::PythonVersion,
            specifier,
        } = expr
        {
            let version = specifier.version();
            major_version = Some(version.release()[0] as i32);
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
    /// A new MissingPythonDistribution instance, or an error if `text` is not
    /// a valid requirement
    pub fn from_requirement_str(
        text: &str,
        python_version: Option<i32>,
    ) -> Result<MissingPythonDistribution, pep508_rs::Pep508Error> {
        use pep440_rs::Operator;
        use pep508_rs::{Requirement, VersionOrUrl};
        use std::str::FromStr;

        let depspec: Requirement = Requirement::from_str(text)?;

        let distribution = depspec.name.to_string();

        let python_version =
            python_version.or_else(|| find_python_version(depspec.marker.to_dnf()));
        let minimum_version =
            if let Some(VersionOrUrl::VersionSpecifier(vs)) = depspec.version_or_url {
                if vs.len() == 1 && *vs[0].operator() == Operator::GreaterThanEqual {
                    Some(vs[0].version().to_string())
                } else {
                    None
                }
            } else {
                None
            };

        Ok(MissingPythonDistribution {
            distribution,
            python_version,
            minimum_version,
        })
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingHaskellModule, "missing-haskell-module", ["module"]);

/// Problem representing a missing system library.
///
/// This struct is used when a required shared library (.so/.dll/.dylib) is not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingLibrary {
    /// The name of the missing library.
    pub library: String,
}

impl Display for MissingLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing library: {}", self.library)
    }
}

crate::problem!(MissingLibrary, "missing-library", ["library"]);

/// Problem representing a missing GObject Introspection typelib.
///
/// This struct is used when a required GObject Introspection typelib file is not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingIntrospectionTypelib {
    /// The name of the missing typelib.
    pub library: String,
}

impl Display for MissingIntrospectionTypelib {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing introspection typelib: {}", self.library)
    }
}

crate::problem!(
    MissingIntrospectionTypelib,
    "missing-introspection-typelib",
    ["library"]
);

/// Problem representing a missing pytest fixture.
///
/// This struct is used when a pytest test requires a fixture that is not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPytestFixture {
    /// The name of the missing fixture.
    pub fixture: String,
}

impl Display for MissingPytestFixture {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing pytest fixture: {}", self.fixture)
    }
}

crate::problem!(MissingPytestFixture, "missing-pytest-fixture", ["fixture"]);

/// Problem representing an unsupported pytest configuration option.
///
/// This struct is used when a pytest configuration specifies an option
/// that is not supported in the current environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedPytestConfigOption {
    /// The name of the unsupported option.
    pub name: String,
}

impl Display for UnsupportedPytestConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest config option: {}", self.name)
    }
}

crate::problem!(
    UnsupportedPytestConfigOption,
    "unsupported-pytest-config-option",
    ["name"]
);

/// Problem representing unsupported pytest command-line arguments.
///
/// This struct is used when pytest is invoked with command-line arguments
/// that are not supported in the current environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsupportedPytestArguments {
    /// The unsupported arguments.
    pub args: Vec<String>,
}

impl Display for UnsupportedPytestArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported pytest arguments: {:?}", self.args)
    }
}

crate::problem!(
    UnsupportedPytestArguments,
    "unsupported-pytest-arguments",
    ["args"]
);

/// Problem representing a missing R package.
///
/// This struct is used when a required R package is not installed
/// or not available in the environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingRPackage,
    "missing-r-package",
    ["package", "minimum_version"]
);

/// Problem representing a missing Go package.
///
/// This struct is used when a required Go package is not installed
/// or not available in the environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingGoPackage {
    /// The import path of the missing Go package.
    pub package: String,
}

crate::problem!(MissingGoPackage, "missing-go-package", ["package"]);

impl Display for MissingGoPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Go package: {}", self.package)
    }
}

/// Problem representing a missing C header file.
///
/// This struct is used when a required C header file (.h) is not available
/// during compilation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCHeader {
    /// The name of the missing C header file.
    pub header: String,
}

crate::problem!(MissingCHeader, "missing-c-header", ["header"]);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingNodeModule {
    /// The name of the missing module.
    pub module: String,
}

crate::problem!(MissingNodeModule, "missing-node-module", ["module"]);

impl Display for MissingNodeModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node module: {}", self.module)
    }
}

/// Problem representing a missing Node.js package.
///
/// This struct is used when a required Node.js package is not installed
/// via npm/yarn/pnpm or cannot be found in node_modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingNodePackage {
    /// The name of the missing package.
    pub package: String,
}

crate::problem!(MissingNodePackage, "missing-node-package", ["package"]);

impl Display for MissingNodePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node package: {}", self.package)
    }
}

/// Problem representing a missing configure script.
///
/// This struct is used when a build expects to find a configure script
/// (typically from autotools) but it doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingConfigure;

crate::problem!(MissingConfigure, "missing-configure", {});

impl Display for MissingConfigure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing ./configure")
    }
}

/// Problem representing a vague or unspecified dependency.
///
/// This struct is used when a build requires a dependency that
/// cannot be clearly categorized as a specific type of dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingVagueDependency,
    "missing-vague-dependency",
    ["name", "url", "minimum_version", "current_version"]
);

impl Display for MissingVagueDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing dependency: {}", self.name)
    }
}

/// Problem representing missing Qt framework.
///
/// This struct is used when a build requires the Qt framework
/// but it is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingQt;

crate::problem!(MissingQt, "missing-qt", {});

impl Display for MissingQt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Qt")
    }
}

/// Problem representing missing X11 libraries or headers.
///
/// This struct is used when a build requires X11 (X Window System)
/// components but they are not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingX11;

crate::problem!(MissingX11, "missing-x11", {});

impl Display for MissingX11 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing X11")
    }
}

/// Problem representing a missing autoconf macro.
///
/// This struct is used when a build using autoconf requires a macro
/// that is not available in the build environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingAutoconfMacro {
    /// The name of the missing autoconf macro.
    pub r#macro: String,
    /// Whether the build system needs to be rebuilt after adding the macro.
    #[serde(default)]
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

crate::problem!(
    MissingAutoconfMacro,
    "missing-autoconf-macro",
    ["macro", "need_rebuild"]
);

impl Display for MissingAutoconfMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing autoconf macro: {}", self.r#macro)
    }
}

/// Problem representing a directory that does not exist.
///
/// This struct is used when a build process expects a directory to exist
/// but it cannot be found in the filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryNonExistant {
    /// The path of the missing directory.
    pub path: String,
}

crate::problem!(
    DirectoryNonExistant,
    "local-directory-not-existing",
    ["path"]
);

impl Display for DirectoryNonExistant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Directory does not exist: {}", self.path)
    }
}

/// Problem representing a missing Vala package.
///
/// This struct is used when a build requires a Vala package
/// that is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingValaPackage {
    /// The name of the missing package.
    pub package: String,
}

crate::problem!(MissingValaPackage, "missing-vala-package", ["package"]);

impl Display for MissingValaPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Vala package: {}", self.package)
    }
}

/// Problem representing the presence of an upstart configuration file.
///
/// This struct is used to indicate that a package includes an upstart file,
/// which may be problematic in environments that have moved to systemd.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstartFilePresent {
    /// The name of the upstart file.
    pub filename: String,
}

crate::problem!(UpstartFilePresent, "upstart-file-present", ["filename"]);

impl Display for UpstartFilePresent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Upstart file present: {}", self.filename)
    }
}

/// Problem representing a missing PostgreSQL extension.
///
/// This struct is used when a build or runtime requires a PostgreSQL extension
/// that is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPostgresExtension {
    /// The name of the missing extension.
    pub extension: String,
}

crate::problem!(
    MissingPostgresExtension,
    "missing-postgresql-extension",
    ["extension"]
);

impl Display for MissingPostgresExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PostgreSQL extension: {}", self.extension)
    }
}

/// Problem representing a missing pkg-config module.
///
/// This struct is used when a build requires a package found via pkg-config
/// that is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPkgConfig {
    /// The name of the missing pkg-config module.
    pub module: String,
    /// The minimum required version of the module, if specified.
    pub minimum_version: Option<String>,
}

crate::problem!(
    MissingPkgConfig,
    "missing-pkg-config-package",
    ["module", "minimum_version"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "MissingHaskellDependenciesDetails")]
pub struct MissingHaskellDependencies {
    /// The missing dependencies.
    pub deps: Vec<String>,
}

/// The details may also be a bare array.
#[derive(Deserialize)]
#[serde(untagged)]
enum MissingHaskellDependenciesDetails {
    Object { deps: Vec<String> },
    Bare(Vec<String>),
}

impl From<MissingHaskellDependenciesDetails> for MissingHaskellDependencies {
    fn from(details: MissingHaskellDependenciesDetails) -> Self {
        match details {
            MissingHaskellDependenciesDetails::Object { deps }
            | MissingHaskellDependenciesDetails::Bare(deps) => Self { deps },
        }
    }
}

crate::problem!(
    MissingHaskellDependencies,
    "missing-haskell-dependencies",
    ["deps"]
);

impl Display for MissingHaskellDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Haskell dependencies: {:?}", self.deps)
    }
}

/// Problem representing lack of disk space.
///
/// This struct is used when a build fails because there is no space
/// left on the device/filesystem where the build is running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSpaceOnDevice;

crate::problem!(NoSpaceOnDevice, "no-space-on-device", {}, universal);

impl Display for NoSpaceOnDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No space left on device")
    }
}

/// Problem representing a missing Java Runtime Environment.
///
/// This struct is used when a build or runtime requires a Java Runtime
/// Environment (JRE) that is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingJRE;

crate::problem!(MissingJRE, "missing-jre", {});

impl Display for MissingJRE {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JRE")
    }
}

/// Problem representing a missing Java Development Kit.
///
/// This struct is used when a build requires a Java Development Kit (JDK)
/// at a specific path but it cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingJDK, "missing-jdk", ["jdk_path"]);

impl Display for MissingJDK {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK at {}", self.jdk_path)
    }
}

/// Problem representing a missing file in the Java Development Kit.
///
/// This struct is used when a build requires a specific file from the JDK
/// but it cannot be found in the JDK directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingJDKFile, "missing-jdk-file", ["jdk_path", "filename"]);

impl Display for MissingJDKFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK file {} at {}", self.filename, self.jdk_path)
    }
}

/// Problem representing a missing Perl file.
///
/// This struct is used when a Perl script attempts to load a file
/// but it cannot be found in any of the include paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingPerlFile, "missing-perl-file", ["filename", "inc"]);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingPerlModule,
    "missing-perl-module",
    ["filename", "module", "inc", "minimum_version"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingSetupPyCommand {
    /// The name of the missing command.
    pub command: String,
}

crate::problem!(
    MissingSetupPyCommand,
    "missing-setup.py-command",
    ["command"]
);

impl Display for MissingSetupPyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing setup.py command: {}", self.command)
    }
}

/// Problem representing a missing C# compiler.
///
/// This struct is used when a build requires a C# compiler
/// but none is available in the build environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingCSharpCompiler;

crate::problem!(MissingCSharpCompiler, "missing-c#-compiler", {});

impl Display for MissingCSharpCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C# compiler")
    }
}

/// Problem representing a missing Rust compiler.
///
/// This struct is used when a build requires a Rust compiler (rustc)
/// but none is available in the build environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingRustCompiler;

crate::problem!(MissingRustCompiler, "missing-rust-compiler", {});

impl Display for MissingRustCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Rust compiler")
    }
}

/// Problem representing a missing assembler.
///
/// This struct is used when a build requires an assembler (like as or nasm)
/// but none is available in the build environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingAssembler;

crate::problem!(MissingAssembler, "missing-assembler", {});

impl Display for MissingAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing assembler")
    }
}

/// Problem representing a missing Rust crate for Cargo.
///
/// This struct is used when a Cargo build requires a Rust crate
/// that is not available or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCargoCrate {
    /// The name of the missing Rust crate.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// The requirement or dependency that needs this crate, if known.
    pub requirement: Option<String>,
}

crate::problem!(
    MissingCargoCrate,
    "missing-cargo-crate",
    ["crate", "requirement"]
);

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DhWithOrderIncorrect;

crate::problem!(DhWithOrderIncorrect, "debhelper-argument-order", {});

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    UnsupportedDebhelperCompatLevel,
    "unsupported-debhelper-compat-level",
    ["oldest_supported", "requested"]
);

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetuptoolScmVersionIssue;

crate::problem!(SetuptoolScmVersionIssue, "setuptools-scm-version-issue", {});

impl Display for SetuptoolScmVersionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "setuptools_scm was unable to find version")
    }
}

/// Problem representing missing Maven artifacts.
///
/// This struct is used when a Java build process that uses Maven
/// is missing required artifacts from Maven repositories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingMavenArtifacts {
    /// The missing artifacts.
    pub artifacts: Vec<String>,
}

crate::problem!(
    MissingMavenArtifacts,
    "missing-maven-artifacts",
    ["artifacts"]
);

impl Display for MissingMavenArtifacts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Maven artifacts: {}", self.artifacts.join(", "))
    }
}

/// Problem representing a file that is not executable.
///
/// This struct is used when a command or script file that needs to be
/// executed does not have the executable permission bit set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotExecutableFile {
    /// The path of the file that is not executable.
    pub path: String,
}

impl NotExecutableFile {
    /// Creates a new NotExecutableFile instance.
    ///
    /// # Arguments
    /// * `path` - Path to the non-executable file
    ///
    /// # Returns
    /// A new NotExecutableFile instance
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

crate::problem!(NotExecutableFile, "not-executable-file", ["path"]);

impl Display for NotExecutableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Command not executable: {}", self.path)
    }
}

/// Problem representing a debhelper script attempting to access an uninstalled file.
///
/// This struct is used when debhelper tries to access a file that has been
/// removed or was never installed in the build environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DhMissingUninstalled {
    /// The file that was not installed.
    pub missing_file: String,
}

impl DhMissingUninstalled {
    /// Creates a new DhMissingUninstalled instance.
    ///
    /// # Arguments
    /// * `missing_file` - Path to the missing file
    ///
    /// # Returns
    /// A new DhMissingUninstalled instance
    pub fn new(missing_file: String) -> Self {
        Self { missing_file }
    }
}

crate::problem!(
    DhMissingUninstalled,
    "dh-missing-uninstalled",
    ["missing_file"]
);

impl Display for DhMissingUninstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh_missing file not installed: {}", self.missing_file)
    }
}

/// Problem representing a debhelper link whose destination is a directory.
///
/// This struct is used when debhelper's dh_link attempts to create a symlink
/// to a path that is a directory, which can cause issues in package builds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DhLinkDestinationIsDirectory {
    /// The link destination that is a directory.
    pub path: String,
}

impl DhLinkDestinationIsDirectory {
    /// Creates a new DhLinkDestinationIsDirectory instance.
    ///
    /// # Arguments
    /// * `path` - Path to the directory that was incorrectly specified as a link destination
    ///
    /// # Returns
    /// A new DhLinkDestinationIsDirectory instance
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

crate::problem!(
    DhLinkDestinationIsDirectory,
    "dh-link-destination-is-directory",
    ["path"]
);

impl Display for DhLinkDestinationIsDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Link destination {} is directory", self.path)
    }
}

/// Problem representing a missing XML entity.
///
/// This struct is used when an XML parser attempts to resolve an external
/// entity reference but the referenced entity cannot be found at the given URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingXmlEntity, "missing-xml-entity", ["url"]);

impl Display for MissingXmlEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing XML entity: {}", self.url)
    }
}

/// Problem representing an error from the ccache compiler cache.
///
/// This struct is used when the ccache tool, which accelerates repeated compilations,
/// encounters an error during its operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CcacheError {
    /// The error reported by ccache.
    pub error: String,
}

impl CcacheError {
    /// Creates a new CcacheError instance.
    ///
    /// # Arguments
    /// * `error` - The error message from ccache
    ///
    /// # Returns
    /// A new CcacheError instance
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

crate::problem!(CcacheError, "ccache-error", ["error"]);

impl Display for CcacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ccache error: {}", self.error)
    }
}

/// Problem representing a rejected Debian package version string.
///
/// This struct is used when a version string for a Debian package is rejected
/// by Debian tools as invalid or incompatible with policy requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    DebianVersionRejected,
    "debian-version-rejected",
    ["version"]
);

impl Display for DebianVersionRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Debian Version Rejected; {}", self.version)
    }
}

/// Problem representing a failure to apply a patch.
///
/// This struct is used when a build process fails because a patch
/// cannot be successfully applied to the source code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    PatchApplicationFailed,
    "patch-application-failed",
    ["patchname"]
);

impl Display for PatchApplicationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Patch application failed: {}", self.patchname)
    }
}

/// Problem representing a need to update PostgreSQL build extension control files.
///
/// This struct is used when PostgreSQL extension build files need to be updated
/// using the pg_buildext updatecontrol command to generate control files from templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    NeedPgBuildExtUpdateControl,
    "need-pg-buildext-updatecontrol",
    ["generated_path", "template_path"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    DhAddonLoadFailure,
    "dh-addon-load-failure",
    ["name", "path"]
);

impl Display for DhAddonLoadFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh addon loading failed: {}", self.name)
    }
}

/// Problem representing an unsupported usage of the --until flag in debhelper.
///
/// This struct is used when the --until flag is used with debhelper (dh)
/// but the version of debhelper in use does not support this option.
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(DhUntilUnsupported, "dh-until-unsupported", {});

impl Display for DhUntilUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh --until is no longer supported")
    }
}

/// Problem representing a debhelper file pattern that was not found.
///
/// This struct is used when a debhelper tool is looking for files matching
/// a specific pattern but cannot find any matches in the searched directories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    DebhelperPatternNotFound,
    "debhelper-pattern-not-found",
    ["pattern", "tool", "directories"]
);

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
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(MissingPerlManifest, "missing-perl-manifest", {});

impl Display for MissingPerlManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Perl MANIFEST")
    }
}

/// Problem representing a missing ImageMagick delegate.
///
/// This struct is used when ImageMagick requires a delegate library
/// to handle a specific file format or operation, but the delegate is not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    ImageMagickDelegateMissing,
    "imagemagick-delegate-missing",
    ["delegate"]
);

impl Display for ImageMagickDelegateMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Imagemagick missing delegate: {}", self.delegate)
    }
}

/// Problem representing a cancelled build or operation.
///
/// This struct is used when a build process or operation was cancelled
/// before completion, typically by user intervention or a timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(Cancelled, "cancelled", {});

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cancelled by runner or job manager")
    }
}

/// Problem representing symbols that have disappeared from a library.
///
/// This struct is used when symbols (functions or variables) that were previously
/// exported by a library are no longer present, which can break API compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(DisappearedSymbols, "disappeared-symbols", {});

impl Display for DisappearedSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Disappeared symbols")
    }
}

/// Problem representing duplicate debhelper compatibility level specifications.
///
/// This struct is used when the debhelper compatibility level is specified
/// multiple times in different places, which can lead to conflicts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    DuplicateDHCompatLevel,
    "duplicate-dh-compat-level",
    ["command"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingDHCompatLevel, "missing-dh-compat-level", ["command"]);

impl Display for MissingDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing DH Compat Level (command: {})", self.command)
    }
}

/// Problem representing a missing Java Virtual Machine (JVM).
///
/// This struct is used when a build process requires a Java Virtual Machine
/// but cannot find one installed or properly configured in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(MissingJVM, "missing-jvm", {});

impl Display for MissingJVM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing JVM")
    }
}

/// Problem representing a missing Ruby gem.
///
/// This struct is used when a build process requires a Ruby gem
/// that is not installed or available in the current environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingRubyGem, "missing-ruby-gem", ["gem", "version"]);

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
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(MissingJavaScriptRuntime, "javascript-runtime-missing", {});

impl Display for MissingJavaScriptRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JavaScript Runtime")
    }
}

/// Problem representing a missing Ruby source file.
///
/// This struct is used when a Ruby application or library tries to
/// load or require a Ruby file that does not exist or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingRubyFile, "missing-ruby-file", ["filename"]);

impl Display for MissingRubyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing ruby file: {}", self.filename)
    }
}

/// Problem representing a missing PHP class.
///
/// This struct is used when a PHP application tries to use a class
/// that has not been defined or cannot be autoloaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingPhpClass, "missing-php-class", ["php_class"]);

impl Display for MissingPhpClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing PHP class: {}", self.php_class)
    }
}

/// Problem representing a missing Java class.
///
/// This struct is used when a Java application or build process
/// requires a Java class that cannot be found in the classpath.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingJavaClass, "missing-java-class", ["classname"]);

impl Display for MissingJavaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Java class: {}", self.classname)
    }
}

/// Problem representing a missing Sprockets asset file.
///
/// This struct is used when a Ruby on Rails application using the Sprockets
/// asset pipeline is missing a required asset file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingSprocketsFile,
    "missing-sprockets-file",
    ["name", "content_type"]
);

impl Display for MissingSprocketsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "missing sprockets file: {} (type: {})",
            self.name, self.content_type
        )
    }
}

/// Problem representing a missing Xfce desktop environment dependency.
///
/// This struct is used when a package build requires an Xfce-specific
/// dependency package that is not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingXfceDependency,
    "missing-xfce-dependency",
    ["package"]
);

impl Display for MissingXfceDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing XFCE build dependency: {}", self.package)
    }
}

/// Problem representing missing GNOME common build tools and macros.
///
/// This struct is used when a GNOME-related package build requires the
/// gnome-common package, which provides common build tools and macros for GNOME projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GnomeCommonMissing;

crate::problem!(GnomeCommonMissing, "missing-gnome-common", {});

impl Display for GnomeCommonMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "gnome-common is not installed")
    }
}

/// Problem representing a missing input file for GNU config.status.
///
/// This struct is used when the GNU autotools config.status script
/// is missing one of its required input files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingConfigStatusInput,
    "missing-config.status-input",
    ["path"]
);

impl Display for MissingConfigStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing config.status input {}", self.path)
    }
}

/// Problem representing a missing GNOME common dependency.
///
/// This struct is used when a GNOME-related package build requires a dependency
/// that is typically provided by or related to the gnome-common infrastructure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    MissingGnomeCommonDependency,
    "missing-gnome-common-dependency",
    ["package", "minimum_version"]
);

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

/// Problem representing a missing input file for GNU Automake.
///
/// This struct is used when GNU Automake cannot find a required input
/// file that it needs to generate build files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(MissingAutomakeInput, "missing-automake-input", ["path"]);

impl Display for MissingAutomakeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "automake input file {} missing", self.path)
    }
}

/// Problem representing a chroot environment that could not be found.
///
/// This struct is used when a build process tries to use a chroot environment
/// (a root directory that appears as the system root to enclosed processes),
/// but the specified chroot does not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(ChrootNotFound, "chroot-not-found", ["chroot"]);

impl Display for ChrootNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "chroot not found: {}", self.chroot)
    }
}

/// Problem representing missing GNU Libtool.
///
/// This struct is used when a build process requires the GNU Libtool
/// utility for creating portable shared libraries, but it is not installed.
#[derive(Debug, Clone, PartialEq, Eq)]
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

crate::problem!(MissingLibtool, "missing-libtool", {});

impl Display for MissingLibtool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Libtool is missing")
    }
}

/// Problem representing missing CMake files.
///
/// This struct is used when a CMake-based build process cannot find
/// required CMake module or configuration files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CMakeFilesMissing {
    /// The names of the missing CMake files.
    pub filenames: Vec<String>,
    /// The version of CMake that was requested, if specified.
    pub version: Option<String>,
}

crate::problem!(
    CMakeFilesMissing,
    "missing-cmake-files",
    ["filenames", "version"]
);

impl std::fmt::Display for CMakeFilesMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CMake files missing: {:?}", self.filenames)
    }
}

/// Problem representing missing CMake package components.
///
/// This struct is used when a CMake-based build process requires specific
/// components of a package, but they cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCMakeComponents {
    /// The name of the CMake package.
    pub name: String,
    /// The names of the missing components.
    pub components: Vec<String>,
}

crate::problem!(
    MissingCMakeComponents,
    "missing-cmake-components",
    ["name", "components"]
);

impl std::fmt::Display for MissingCMakeComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing CMake components: {:?}", self.components)
    }
}

/// Problem representing a missing CMake package configuration.
///
/// This struct is used when a CMake-based build process cannot find
/// a required package configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingCMakeConfig {
    /// The name of the CMake package.
    pub name: String,
    /// The version of the package that was requested, if specified.
    pub version: Option<String>,
}

crate::problem!(
    MissingCMakeConfig,
    "missing-cmake-config",
    ["name", "version"]
);

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

/// Problem representing a CMake package with mismatched version requirements.
///
/// This struct is used when a CMake-based build process found a package,
/// but it requires an exact version that doesn't match the found version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

crate::problem!(
    CMakeNeedExactVersion,
    "cmake-exact-version-missing",
    ["package", "version_found", "exact_version_needed", "path"]
);

impl std::fmt::Display for CMakeNeedExactVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "CMake needs exact package {}, version {}",
            self.package, self.exact_version_needed
        )
    }
}

/// Problem representing a missing static library file.
///
/// This struct is used when a build process requires a static library
/// (typically a .a or .lib file) but it cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingStaticLibrary {
    /// The name of the library (without file extension).
    pub library: String,
    /// The expected filename of the static library.
    pub filename: String,
}

crate::problem!(
    MissingStaticLibrary,
    "missing-static-library",
    ["library", "filename"]
);

impl std::fmt::Display for MissingStaticLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing static library: {}", self.library)
    }
}

/// Problem representing a missing Go runtime.
///
/// This struct is used when a build process requires the Go language runtime
/// but it is not installed or cannot be found in the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGoRuntime;

crate::problem!(MissingGoRuntime, "missing-go-runtime", {});

impl std::fmt::Display for MissingGoRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go runtime is missing")
    }
}

/// Problem representing an unknown SSL/TLS certificate authority.
///
/// This struct is used when a build process fails to establish a secure connection
/// because it cannot verify the certificate authority of a remote server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownCertificateAuthority {
    /// The URL that was being accessed.
    pub url: String,
}

crate::problem!(
    UnknownCertificateAuthority,
    "unknown-certificate-authority",
    ["url"]
);

impl std::fmt::Display for UnknownCertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unknown Certificate Authority for {}", self.url)
    }
}

/// Problem representing a missing predeclared function in Perl.
///
/// This struct is used when a Perl script tries to use a predeclared function
/// that is not available, often because a required module is not loaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPerlPredeclared {
    /// The name of the missing predeclared function.
    pub name: String,
}

crate::problem!(MissingPerlPredeclared, "missing-perl-predeclared", ["name"]);

impl std::fmt::Display for MissingPerlPredeclared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing predeclared function: {}", self.name)
    }
}

/// Problem representing missing Git user identity configuration.
///
/// This struct is used when Git operations that require user identity
/// (like commits) fail because the user.name and user.email are not configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGitIdentity;

crate::problem!(MissingGitIdentity, "missing-git-identity", {});

impl std::fmt::Display for MissingGitIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Git Identity")
    }
}

/// Problem representing a missing GPG secret key.
///
/// This struct is used when an operation requires a GPG secret key
/// (such as signing packages or commits) but no secret key is available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingSecretGpgKey;

crate::problem!(MissingSecretGpgKey, "no-secret-gpg-key", {});

impl std::fmt::Display for MissingSecretGpgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No secret GPG key is present")
    }
}

/// Problem representing missing version information for vcversioner.
///
/// This struct is used when the vcversioner Python package cannot determine
/// the version from either a Git directory or a version.txt file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingVcVersionerVersion;

crate::problem!(MissingVcVersionerVersion, "no-vcversioner-version", {});

impl std::fmt::Display for MissingVcVersionerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "vcversion could not find a git directory or version.txt file"
        )
    }
}

/// Problem representing a missing LaTeX file.
///
/// This struct is used when a LaTeX build process requires a file
/// (such as a class file, style file, or content file) but it cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingLatexFile {
    /// The name of the missing file.
    pub filename: String,
}

impl MissingLatexFile {
    /// Creates a new MissingLatexFile instance.
    ///
    /// # Arguments
    /// * `filename` - Name of the missing LaTeX file
    ///
    /// # Returns
    /// A new MissingLatexFile instance
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

crate::problem!(MissingLatexFile, "missing-latex-file", ["filename"]);

impl std::fmt::Display for MissingLatexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing LaTeX file: {}", self.filename)
    }
}

/// Problem representing a missing X Window System display.
///
/// This struct is used when a program requires an X11 display connection
/// but no display server is available (such as in headless environments).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingXDisplay;

crate::problem!(MissingXDisplay, "missing-x-display", {});

impl std::fmt::Display for MissingXDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No X Display")
    }
}

/// Problem representing a missing font specification in LaTeX.
///
/// This struct is used when a LaTeX document requires a specific font
/// but the fontspec package cannot find it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingFontspec {
    /// The missing font specification.
    pub fontspec: String,
}

crate::problem!(MissingFontspec, "missing-fontspec", ["fontspec"]);

impl std::fmt::Display for MissingFontspec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing font spec: {}", self.fontspec)
    }
}

/// Problem representing a process killed due to inactivity.
///
/// This struct is used when a build process was killed by the system
/// because it was inactive for too long.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InactiveKilled {
    /// The number of minutes of inactivity.
    pub minutes: i64,
}

crate::problem!(InactiveKilled, "inactive-killed", ["minutes"]);

impl std::fmt::Display for InactiveKilled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Killed due to inactivity after {} minutes", self.minutes)
    }
}

/// Problem representing missing PAUSE credentials for Perl module upload.
///
/// This struct is used when attempting to upload a Perl module to PAUSE
/// (Perl Authors Upload Server) without proper authentication credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingPauseCredentials;

crate::problem!(MissingPauseCredentials, "missing-pause-credentials", {});

impl std::fmt::Display for MissingPauseCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing credentials for PAUSE")
    }
}

/// Problem representing mismatched gettext versions.
///
/// This struct is used when there's a version mismatch between gettext versions
/// referenced in Makefile.in.in and the autoconf macros.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MismatchGettextVersions {
    /// The gettext version specified in the Makefile.
    pub makefile_version: String,
    /// The gettext version specified in autoconf macros.
    pub autoconf_version: String,
}

crate::problem!(
    MismatchGettextVersions,
    "mismatch-gettext-versions",
    ["makefile_version", "autoconf_version"]
);

impl std::fmt::Display for MismatchGettextVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Mismatch versions ({}, {})",
            self.makefile_version, self.autoconf_version
        )
    }
}

/// Problem representing an invalid current user for a build operation.
///
/// This struct is used when a build process encounters issues because
/// it's running under an unexpected or inappropriate user account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidCurrentUser {
    /// The invalid user.
    pub user: String,
}

crate::problem!(InvalidCurrentUser, "invalid-current-user", ["user"]);

impl std::fmt::Display for InvalidCurrentUser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not run as {}", self.user)
    }
}

/// Problem representing a missing GNU lib directory.
///
/// This struct is used when a build process requires a gnulib directory
/// (a collection of portable GNU utility functions) but it cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingGnulibDirectory {
    /// The path of the missing gnulib directory.
    pub directory: PathBuf,
}

crate::problem!(
    MissingGnulibDirectory,
    "missing-gnulib-directory",
    ["directory"]
);

impl std::fmt::Display for MissingGnulibDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing gnulib directory: {}", self.directory.display())
    }
}

/// Problem representing a missing Lua module.
///
/// This struct is used when a Lua script or application attempts to
/// load a Lua module that is not installed or cannot be found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingLuaModule {
    /// The name of the missing module.
    pub module: String,
}

crate::problem!(MissingLuaModule, "missing-lua-module", ["module"]);

impl std::fmt::Display for MissingLuaModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Lua Module: {}", self.module)
    }
}

/// Problem representing a missing Go module file.
///
/// This struct is used when a Go project requires a go.mod file for
/// module and dependency management, but the file is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGoModFile;

crate::problem!(MissingGoModFile, "missing-go.mod-file", {});

impl std::fmt::Display for MissingGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is missing")
    }
}

/// Problem representing an outdated Go module file.
///
/// This struct is used when a Go project's go.mod file needs to be
/// updated due to changes in dependencies or Go version requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutdatedGoModFile;

crate::problem!(OutdatedGoModFile, "outdated-go.mod-file", {});

impl std::fmt::Display for OutdatedGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is outdated")
    }
}

/// Problem representing insufficient code test coverage.
///
/// This struct is used when a build process requires a minimum level of
/// code test coverage, but the actual coverage is below the required threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeCoverageTooLow {
    /// The actual code coverage percentage achieved.
    pub actual: f64,
    /// The minimum code coverage percentage required.
    pub required: f64,
}

crate::problem!(
    CodeCoverageTooLow,
    "code-coverage-too-low",
    ["actual", "required"]
);

impl std::fmt::Display for CodeCoverageTooLow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Code coverage too low: {:.2} < {:.2}",
            self.actual, self.required
        )
    }
}

/// Problem representing improper usage of ES modules.
///
/// This struct is used when a JavaScript module is using CommonJS require()
/// syntax to load an ES module, which must be loaded with import() instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ESModuleMustUseImport {
    /// The ES module that must be loaded with import().
    pub path: String,
}

crate::problem!(ESModuleMustUseImport, "esmodule-must-use-import", ["path"]);

impl std::fmt::Display for ESModuleMustUseImport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ESM-only module {} must use import()", self.path)
    }
}

/// Problem representing a missing PHP extension.
///
/// This struct is used when a PHP application requires an extension
/// (like mysqli, gd, intl, etc.) that is not installed or enabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPHPExtension {
    /// The name of the missing extension.
    pub extension: String,
}

crate::problem!(MissingPHPExtension, "missing-php-extension", ["extension"]);

impl std::fmt::Display for MissingPHPExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PHP Extension: {}", self.extension)
    }
}

/// Problem representing an outdated minimum autoconf version requirement.
///
/// This struct is used when a project's configure script specifies a minimum autoconf
/// version that is considered too old for modern builds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimumAutoconfTooOld {
    /// The minimum autoconf version that was specified.
    pub minimum_version: String,
}

crate::problem!(
    MinimumAutoconfTooOld,
    "minimum-autoconf-too-old",
    ["minimum_version"]
);

impl std::fmt::Display for MinimumAutoconfTooOld {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "configure.{{ac,in}} should require newer autoconf {}",
            self.minimum_version
        )
    }
}

/// Problem representing a missing file in a Perl distribution.
///
/// This struct is used when a Perl module build or installation process
/// cannot find a required file that should be part of the distribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingPerlDistributionFile {
    /// The name of the missing file.
    pub filename: String,
}

crate::problem!(
    MissingPerlDistributionFile,
    "missing-perl-distribution-file",
    ["filename"]
);

impl std::fmt::Display for MissingPerlDistributionFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing perl distribution file: {}", self.filename)
    }
}

/// Problem representing a missing entry in the Go checksum file.
///
/// This struct is used when a Go project requires an entry in the go.sum file
/// for a specific package version, but the entry is missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingGoSumEntry {
    /// The package import path.
    pub package: String,
    /// The version of the package.
    pub version: String,
}

crate::problem!(
    MissingGoSumEntry,
    "missing-go.sum-entry",
    ["package", "version"]
);

impl std::fmt::Display for MissingGoSumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing go.sum entry: {}@{}", self.package, self.version)
    }
}

/// Problem representing an issue with the Vala compiler.
///
/// This struct is used when the Vala compiler (valac) encounters
/// an error that prevents it from compiling Vala source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValaCompilerCannotCompile;

crate::problem!(ValaCompilerCannotCompile, "valac-cannot-compile", {});

impl std::fmt::Display for ValaCompilerCannotCompile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "valac can not compile")
    }
}

/// Problem representing a missing Debian build dependency.
///
/// This struct is used when a Debian package build requires a dependency
/// that is listed in Build-Depends but is not installed in the build environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingDebianBuildDep {
    /// The missing build dependency.
    pub dep: String,
}

crate::problem!(MissingDebianBuildDep, "missing-debian-build-dep", ["dep"]);

impl std::fmt::Display for MissingDebianBuildDep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Debian Build-Depends: {}", self.dep)
    }
}

/// Problem representing missing Qt modules.
///
/// This struct is used when a build process requires specific Qt modules
/// (like QtCore, QtGui, QtWidgets, etc.) that are not available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingQtModules {
    /// The missing Qt modules.
    pub modules: Vec<String>,
}

crate::problem!(MissingQtModules, "missing-qt-modules", ["modules"]);

impl std::fmt::Display for MissingQtModules {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing QT modules: {:?}", self.modules)
    }
}

/// Problem representing a missing OCaml package.
///
/// This struct is used when an OCaml project requires a package
/// that is not installed or cannot be found in the OCaml environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingOCamlPackage {
    /// The name of the missing package.
    pub package: String,
}

crate::problem!(MissingOCamlPackage, "missing-ocaml-package", ["package"]);

impl std::fmt::Display for MissingOCamlPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing OCaml package: {}", self.package)
    }
}

/// Problem representing a "too many open files" error.
///
/// This struct is used when a process hits the system limit for the number
/// of files that can be opened simultaneously, often due to a resource leak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooManyOpenFiles;

crate::problem!(TooManyOpenFiles, "too-many-open-files", {});

impl std::fmt::Display for TooManyOpenFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Too many open files")
    }
}

/// Problem representing a missing Makefile target.
///
/// This struct is used when a build process tries to run a make target
/// that doesn't exist in the Makefile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingMakeTarget {
    /// The missing target.
    pub target: String,
    /// The target that requires it, if known.
    pub required_by: Option<String>,
}

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
        Self {
            target: target.to_string(),
            required_by: required_by.map(String::from),
        }
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
        write!(f, "Unknown make target: {}", self.target)
    }
}

crate::problem!(
    MissingMakeTarget,
    "missing-make-target",
    ["target", "required_by"]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::assert_wire_format;
    use crate::Problem;
    use serde_json::json;

    fn assert_registered(problem: &dyn Problem) {
        let kind = problem.kind();
        let info = inventory::iter::<crate::ProblemKindInfo>()
            .find(|info| info.kind == kind)
            .unwrap_or_else(|| panic!("no ProblemKindInfo registered for {:?}", kind));
        let mut expected: Vec<String> = problem
            .json()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        expected.sort();
        let mut actual: Vec<&str> = info.detail_fields.to_vec();
        actual.sort();
        assert_eq!(actual, expected, "detail_fields mismatch for {:?}", kind);
    }

    #[test]
    fn test_kinds_registered() {
        assert_registered(&MissingSetupPyCommand {
            command: "test".into(),
        });
        assert_registered(&MissingCSharpCompiler);
        assert_registered(&MissingConfigStatusInput::new("Makefile.in".into()));
        assert_registered(&MissingGoModFile);
        assert_registered(&OutdatedGoModFile);
        assert_registered(&MissingGoSumEntry {
            package: "github.com/foo/bar".into(),
            version: "v1.0.0".into(),
        });
    }

    #[test]
    fn wire_format() {
        assert_wire_format(
            &MissingFile {
                path: PathBuf::from("/srv/path"),
            },
            "missing-file",
            json!({"path": "/srv/path"}),
            "Missing file: /srv/path",
        );
        assert_wire_format(
            &MissingBuildFile {
                filename: "filename-value".to_string(),
            },
            "missing-build-file",
            json!({"filename": "filename-value"}),
            "Missing build file: filename-value",
        );
        assert_wire_format(
            &MissingCommandOrBuildFile {
                filename: "filename-value".to_string(),
            },
            "missing-command-or-build-file",
            json!({"filename": "filename-value"}),
            "Missing command or build file: filename-value",
        );
        assert_wire_format(
            &VcsControlDirectoryNeeded {
                vcs: vec!["vcs-a".to_string(), "vcs-b".to_string()],
            },
            "vcs-control-directory-needed",
            json!({"vcs": ["vcs-a", "vcs-b"]}),
            "VCS control directory needed: vcs-a, vcs-b",
        );
        assert_wire_format(
            &MissingPythonModule {
                module: "module-value".to_string(),
                python_version: Some(3),
                minimum_version: Some("minimum_version-value".to_string()),
            },
            "missing-python-module",
            json!({
                "minimum_version": "minimum_version-value",
                "module": "module-value",
                "python_version": 3,
            }),
            "Missing 3 Python module: module-value (>= minimum_version-value)",
        );
        assert_wire_format(
            &MissingPythonModule {
                module: "module-value".to_string(),
                python_version: None,
                minimum_version: None,
            },
            "missing-python-module",
            json!({"minimum_version": null, "module": "module-value", "python_version": null}),
            "Missing Python module: module-value",
        );
        assert_wire_format(
            &MissingCommand {
                command: "f0-value".to_string(),
            },
            "command-missing",
            json!({"command": "f0-value"}),
            "Missing command: f0-value",
        );
        assert_wire_format(
            &MissingPythonDistribution {
                distribution: "distribution-value".to_string(),
                python_version: Some(3),
                minimum_version: Some("minimum_version-value".to_string()),
            },
            "missing-python-distribution",
            json!({
                "distribution": "distribution-value",
                "minimum_version": "minimum_version-value",
                "python_version": 3,
            }),
            "Missing 3 Python distribution: distribution-value (>= minimum_version-value)",
        );
        assert_wire_format(
            &MissingPythonDistribution {
                distribution: "distribution-value".to_string(),
                python_version: None,
                minimum_version: None,
            },
            "missing-python-distribution",
            json!({
                "distribution": "distribution-value",
                "minimum_version": null,
                "python_version": null,
            }),
            "Missing Python distribution: distribution-value",
        );
        assert_wire_format(
            &MissingHaskellModule {
                module: "module-value".to_string(),
            },
            "missing-haskell-module",
            json!({"module": "module-value"}),
            "Missing Haskell module: module-value",
        );
        assert_wire_format(
            &MissingLibrary {
                library: "f0-value".to_string(),
            },
            "missing-library",
            json!({"library": "f0-value"}),
            "Missing library: f0-value",
        );
        assert_wire_format(
            &MissingIntrospectionTypelib {
                library: "f0-value".to_string(),
            },
            "missing-introspection-typelib",
            json!({"library": "f0-value"}),
            "Missing introspection typelib: f0-value",
        );
        assert_wire_format(
            &MissingPytestFixture {
                fixture: "f0-value".to_string(),
            },
            "missing-pytest-fixture",
            json!({"fixture": "f0-value"}),
            "Missing pytest fixture: f0-value",
        );
        assert_wire_format(
            &UnsupportedPytestConfigOption {
                name: "f0-value".to_string(),
            },
            "unsupported-pytest-config-option",
            json!({"name": "f0-value"}),
            "Unsupported pytest config option: f0-value",
        );
        assert_wire_format(
            &UnsupportedPytestArguments {
                args: vec!["f0-a".to_string(), "f0-b".to_string()],
            },
            "unsupported-pytest-arguments",
            json!({"args": ["f0-a", "f0-b"]}),
            "Unsupported pytest arguments: [\"f0-a\", \"f0-b\"]",
        );
        assert_wire_format(
            &MissingRPackage {
                package: "package-value".to_string(),
                minimum_version: Some("minimum_version-value".to_string()),
            },
            "missing-r-package",
            json!({"minimum_version": "minimum_version-value", "package": "package-value"}),
            "Missing R package: package-value (>= minimum_version-value)",
        );
        assert_wire_format(
            &MissingRPackage {
                package: "package-value".to_string(),
                minimum_version: None,
            },
            "missing-r-package",
            json!({"minimum_version": null, "package": "package-value"}),
            "Missing R package: package-value",
        );
        assert_wire_format(
            &MissingGoPackage {
                package: "package-value".to_string(),
            },
            "missing-go-package",
            json!({"package": "package-value"}),
            "Missing Go package: package-value",
        );
        assert_wire_format(
            &MissingCHeader {
                header: "header-value".to_string(),
            },
            "missing-c-header",
            json!({"header": "header-value"}),
            "Missing C header: header-value",
        );
        assert_wire_format(
            &MissingNodeModule {
                module: "f0-value".to_string(),
            },
            "missing-node-module",
            json!({"module": "f0-value"}),
            "Missing Node module: f0-value",
        );
        assert_wire_format(
            &MissingNodePackage {
                package: "f0-value".to_string(),
            },
            "missing-node-package",
            json!({"package": "f0-value"}),
            "Missing Node package: f0-value",
        );
        assert_wire_format(
            &MissingConfigure,
            "missing-configure",
            json!({}),
            "Missing ./configure",
        );
        assert_wire_format(
            &MissingVagueDependency {
                name: "name-value".to_string(),
                url: Some("url-value".to_string()),
                minimum_version: Some("minimum_version-value".to_string()),
                current_version: Some("current_version-value".to_string()),
            },
            "missing-vague-dependency",
            json!({
                "current_version": "current_version-value",
                "minimum_version": "minimum_version-value",
                "name": "name-value",
                "url": "url-value",
            }),
            "Missing dependency: name-value",
        );
        assert_wire_format(
            &MissingVagueDependency {
                name: "name-value".to_string(),
                url: None,
                minimum_version: None,
                current_version: None,
            },
            "missing-vague-dependency",
            json!({
                "current_version": null,
                "minimum_version": null,
                "name": "name-value",
                "url": null,
            }),
            "Missing dependency: name-value",
        );
        assert_wire_format(&MissingQt, "missing-qt", json!({}), "Missing Qt");
        assert_wire_format(&MissingX11, "missing-x11", json!({}), "Missing X11");
        assert_wire_format(
            &MissingAutoconfMacro {
                r#macro: "macro-value".to_string(),
                need_rebuild: true,
            },
            "missing-autoconf-macro",
            json!({"macro": "macro-value", "need_rebuild": true}),
            "Missing autoconf macro: macro-value",
        );
        assert_wire_format(
            &DirectoryNonExistant {
                path: "f0-value".to_string(),
            },
            "local-directory-not-existing",
            json!({"path": "f0-value"}),
            "Directory does not exist: f0-value",
        );
        assert_wire_format(
            &MissingValaPackage {
                package: "f0-value".to_string(),
            },
            "missing-vala-package",
            json!({"package": "f0-value"}),
            "Missing Vala package: f0-value",
        );
        assert_wire_format(
            &UpstartFilePresent {
                filename: "f0-value".to_string(),
            },
            "upstart-file-present",
            json!({"filename": "f0-value"}),
            "Upstart file present: f0-value",
        );
        assert_wire_format(
            &MissingPostgresExtension {
                extension: "f0-value".to_string(),
            },
            "missing-postgresql-extension",
            json!({"extension": "f0-value"}),
            "Missing PostgreSQL extension: f0-value",
        );
        assert_wire_format(
            &MissingPkgConfig {
                module: "module-value".to_string(),
                minimum_version: Some("minimum_version-value".to_string()),
            },
            "missing-pkg-config-package",
            json!({"minimum_version": "minimum_version-value", "module": "module-value"}),
            "Missing pkg-config module: module-value >= minimum_version-value",
        );
        assert_wire_format(
            &MissingPkgConfig {
                module: "module-value".to_string(),
                minimum_version: None,
            },
            "missing-pkg-config-package",
            json!({"minimum_version": null, "module": "module-value"}),
            "Missing pkg-config module: module-value",
        );
        assert_wire_format(
            &MissingHaskellDependencies {
                deps: vec!["f0-a".to_string(), "f0-b".to_string()],
            },
            "missing-haskell-dependencies",
            json!({"deps": ["f0-a", "f0-b"]}),
            "Missing Haskell dependencies: [\"f0-a\", \"f0-b\"]",
        );
        assert_wire_format(
            &NoSpaceOnDevice,
            "no-space-on-device",
            json!({}),
            "No space left on device",
        );
        assert_wire_format(&MissingJRE, "missing-jre", json!({}), "Missing JRE");
        assert_wire_format(
            &MissingJDK {
                jdk_path: "jdk_path-value".to_string(),
            },
            "missing-jdk",
            json!({"jdk_path": "jdk_path-value"}),
            "Missing JDK at jdk_path-value",
        );
        assert_wire_format(
            &MissingJDKFile {
                jdk_path: "jdk_path-value".to_string(),
                filename: "filename-value".to_string(),
            },
            "missing-jdk-file",
            json!({"filename": "filename-value", "jdk_path": "jdk_path-value"}),
            "Missing JDK file filename-value at jdk_path-value",
        );
        assert_wire_format(
            &MissingPerlFile {
                filename: "filename-value".to_string(),
                inc: Some(vec!["inc-a".to_string(), "inc-b".to_string()]),
            },
            "missing-perl-file",
            json!({"filename": "filename-value", "inc": ["inc-a", "inc-b"]}),
            "Missing Perl file filename-value (INC: inc-a:inc-b)",
        );
        assert_wire_format(
            &MissingPerlFile {
                filename: "filename-value".to_string(),
                inc: None,
            },
            "missing-perl-file",
            json!({"filename": "filename-value", "inc": null}),
            "Missing Perl file filename-value",
        );
        assert_wire_format(
            &MissingPerlModule {
                filename: Some("Foo/Bar.pm".to_string()),
                module: "Foo::Bar".to_string(),
                inc: Some(vec!["/usr/lib".to_string(), "/etc".to_string()]),
                minimum_version: Some("1.2".to_string()),
            },
            "missing-perl-module",
            json!({
                "filename": "Foo/Bar.pm",
                "inc": ["/usr/lib", "/etc"],
                "minimum_version": "1.2",
                "module": "Foo::Bar",
            }),
            "Missing Perl module: Foo::Bar (from Foo/Bar.pm) >= 1.2 (INC: /usr/lib, /etc)",
        );
        assert_wire_format(
            &MissingPerlModule {
                filename: None,
                module: "module-value".to_string(),
                inc: None,
                minimum_version: None,
            },
            "missing-perl-module",
            json!({
                "filename": null,
                "inc": null,
                "minimum_version": null,
                "module": "module-value",
            }),
            "Missing Perl module: module-value",
        );
        assert_wire_format(
            &MissingSetupPyCommand {
                command: "f0-value".to_string(),
            },
            "missing-setup.py-command",
            json!({"command": "f0-value"}),
            "Missing setup.py command: f0-value",
        );
        assert_wire_format(
            &MissingCSharpCompiler,
            "missing-c#-compiler",
            json!({}),
            "Missing C# compiler",
        );
        assert_wire_format(
            &MissingRustCompiler,
            "missing-rust-compiler",
            json!({}),
            "Missing Rust compiler",
        );
        assert_wire_format(
            &MissingAssembler,
            "missing-assembler",
            json!({}),
            "Missing assembler",
        );
        assert_wire_format(
            &MissingCargoCrate {
                crate_name: "crate_name-value".to_string(),
                requirement: Some("requirement-value".to_string()),
            },
            "missing-cargo-crate",
            json!({"crate": "crate_name-value", "requirement": "requirement-value"}),
            "Missing Cargo crate crate_name-value (required by requirement-value)",
        );
        assert_wire_format(
            &MissingCargoCrate {
                crate_name: "crate_name-value".to_string(),
                requirement: None,
            },
            "missing-cargo-crate",
            json!({"crate": "crate_name-value", "requirement": null}),
            "Missing Cargo crate crate_name-value",
        );
        assert_wire_format(
            &DhWithOrderIncorrect,
            "debhelper-argument-order",
            json!({}),
            "dh argument order is incorrect",
        );
        assert_wire_format(
            &UnsupportedDebhelperCompatLevel {
                oldest_supported: 12,
                requested: 12,
            },
            "unsupported-debhelper-compat-level",
            json!({"oldest_supported": 12, "requested": 12}),
            "Request debhelper compat level 12 lower than supported 12",
        );
        assert_wire_format(
            &SetuptoolScmVersionIssue,
            "setuptools-scm-version-issue",
            json!({}),
            "setuptools_scm was unable to find version",
        );
        assert_wire_format(
            &MissingMavenArtifacts {
                artifacts: vec!["f0-a".to_string(), "f0-b".to_string()],
            },
            "missing-maven-artifacts",
            json!({"artifacts": ["f0-a", "f0-b"]}),
            "Missing Maven artifacts: f0-a, f0-b",
        );
        assert_wire_format(
            &NotExecutableFile {
                path: "f0-value".to_string(),
            },
            "not-executable-file",
            json!({"path": "f0-value"}),
            "Command not executable: f0-value",
        );
        assert_wire_format(
            &DhMissingUninstalled {
                missing_file: "f0-value".to_string(),
            },
            "dh-missing-uninstalled",
            json!({"missing_file": "f0-value"}),
            "dh_missing file not installed: f0-value",
        );
        assert_wire_format(
            &DhLinkDestinationIsDirectory {
                path: "f0-value".to_string(),
            },
            "dh-link-destination-is-directory",
            json!({"path": "f0-value"}),
            "Link destination f0-value is directory",
        );
        assert_wire_format(
            &MissingXmlEntity {
                url: "url-value".to_string(),
            },
            "missing-xml-entity",
            json!({"url": "url-value"}),
            "Missing XML entity: url-value",
        );
        assert_wire_format(
            &CcacheError {
                error: "f0-value".to_string(),
            },
            "ccache-error",
            json!({"error": "f0-value"}),
            "ccache error: f0-value",
        );
        assert_wire_format(
            &DebianVersionRejected {
                version: "version-value".to_string(),
            },
            "debian-version-rejected",
            json!({"version": "version-value"}),
            "Debian Version Rejected; version-value",
        );
        assert_wire_format(
            &PatchApplicationFailed {
                patchname: "patchname-value".to_string(),
            },
            "patch-application-failed",
            json!({"patchname": "patchname-value"}),
            "Patch application failed: patchname-value",
        );
        assert_wire_format(
            &NeedPgBuildExtUpdateControl {
                generated_path: "generated_path-value".to_string(),
                template_path: "template_path-value".to_string(),
            },
            "need-pg-buildext-updatecontrol",
            json!({
                "generated_path": "generated_path-value",
                "template_path": "template_path-value",
            }),
            "Need to run 'pg_buildext updatecontrol' to update generated_path-value",
        );
        assert_wire_format(
            &DhAddonLoadFailure {
                name: "name-value".to_string(),
                path: "path-value".to_string(),
            },
            "dh-addon-load-failure",
            json!({"name": "name-value", "path": "path-value"}),
            "dh addon loading failed: name-value",
        );
        assert_wire_format(
            &DhUntilUnsupported,
            "dh-until-unsupported",
            json!({}),
            "dh --until is no longer supported",
        );
        assert_wire_format(
            &DebhelperPatternNotFound {
                pattern: "*.so".to_string(),
                tool: "install".to_string(),
                directories: vec!["usr".to_string(), "lib".to_string()],
            },
            "debhelper-pattern-not-found",
            json!({"directories": ["usr", "lib"], "pattern": "*.so", "tool": "install"}),
            "debhelper (install) expansion failed for \"*.so\" (directories: [\"usr\", \"lib\"])",
        );
        assert_wire_format(
            &MissingPerlManifest,
            "missing-perl-manifest",
            json!({}),
            "missing Perl MANIFEST",
        );
        assert_wire_format(
            &ImageMagickDelegateMissing {
                delegate: "delegate-value".to_string(),
            },
            "imagemagick-delegate-missing",
            json!({"delegate": "delegate-value"}),
            "Imagemagick missing delegate: delegate-value",
        );
        assert_wire_format(
            &Cancelled,
            "cancelled",
            json!({}),
            "Cancelled by runner or job manager",
        );
        assert_wire_format(
            &DisappearedSymbols,
            "disappeared-symbols",
            json!({}),
            "Disappeared symbols",
        );
        assert_wire_format(
            &DuplicateDHCompatLevel {
                command: "command-value".to_string(),
            },
            "duplicate-dh-compat-level",
            json!({"command": "command-value"}),
            "DH Compat Level specified twice (command: command-value)",
        );
        assert_wire_format(
            &MissingDHCompatLevel {
                command: "command-value".to_string(),
            },
            "missing-dh-compat-level",
            json!({"command": "command-value"}),
            "Missing DH Compat Level (command: command-value)",
        );
        assert_wire_format(&MissingJVM, "missing-jvm", json!({}), "missing JVM");
        assert_wire_format(
            &MissingRubyGem {
                gem: "gem-value".to_string(),
                version: Some("version-value".to_string()),
            },
            "missing-ruby-gem",
            json!({"gem": "gem-value", "version": "version-value"}),
            "missing ruby gem: gem-value (>= version-value)",
        );
        assert_wire_format(
            &MissingRubyGem {
                gem: "gem-value".to_string(),
                version: None,
            },
            "missing-ruby-gem",
            json!({"gem": "gem-value", "version": null}),
            "missing ruby gem: gem-value",
        );
        assert_wire_format(
            &MissingJavaScriptRuntime,
            "javascript-runtime-missing",
            json!({}),
            "Missing JavaScript Runtime",
        );
        assert_wire_format(
            &MissingRubyFile {
                filename: "filename-value".to_string(),
            },
            "missing-ruby-file",
            json!({"filename": "filename-value"}),
            "missing ruby file: filename-value",
        );
        assert_wire_format(
            &MissingPhpClass {
                php_class: "php_class-value".to_string(),
            },
            "missing-php-class",
            json!({"php_class": "php_class-value"}),
            "missing PHP class: php_class-value",
        );
        assert_wire_format(
            &MissingJavaClass {
                classname: "classname-value".to_string(),
            },
            "missing-java-class",
            json!({"classname": "classname-value"}),
            "missing Java class: classname-value",
        );
        assert_wire_format(
            &MissingSprocketsFile {
                name: "name-value".to_string(),
                content_type: "content_type-value".to_string(),
            },
            "missing-sprockets-file",
            json!({"content_type": "content_type-value", "name": "name-value"}),
            "missing sprockets file: name-value (type: content_type-value)",
        );
        assert_wire_format(
            &MissingXfceDependency {
                package: "package-value".to_string(),
            },
            "missing-xfce-dependency",
            json!({"package": "package-value"}),
            "missing XFCE build dependency: package-value",
        );
        assert_wire_format(
            &GnomeCommonMissing,
            "missing-gnome-common",
            json!({}),
            "gnome-common is not installed",
        );
        assert_wire_format(
            &MissingConfigStatusInput {
                path: "path-value".to_string(),
            },
            "missing-config.status-input",
            json!({"path": "path-value"}),
            "missing config.status input path-value",
        );
        assert_wire_format(
            &MissingGnomeCommonDependency {
                package: "package-value".to_string(),
                minimum_version: Some("minimum_version-value".to_string()),
            },
            "missing-gnome-common-dependency",
            json!({"minimum_version": "minimum_version-value", "package": "package-value"}),
            "missing gnome-common dependency: package-value: (>= minimum_version-value)",
        );
        assert_wire_format(
            &MissingGnomeCommonDependency {
                package: "package-value".to_string(),
                minimum_version: None,
            },
            "missing-gnome-common-dependency",
            json!({"minimum_version": null, "package": "package-value"}),
            "missing gnome-common dependency: package-value: (>= any)",
        );
        assert_wire_format(
            &MissingAutomakeInput {
                path: "path-value".to_string(),
            },
            "missing-automake-input",
            json!({"path": "path-value"}),
            "automake input file path-value missing",
        );
        assert_wire_format(
            &ChrootNotFound {
                chroot: "chroot-value".to_string(),
            },
            "chroot-not-found",
            json!({"chroot": "chroot-value"}),
            "chroot not found: chroot-value",
        );
        assert_wire_format(
            &MissingLibtool,
            "missing-libtool",
            json!({}),
            "Libtool is missing",
        );
        assert_wire_format(
            &CMakeFilesMissing {
                filenames: vec!["filenames-a".to_string(), "filenames-b".to_string()],
                version: Some("version-value".to_string()),
            },
            "missing-cmake-files",
            json!({"filenames": ["filenames-a", "filenames-b"], "version": "version-value"}),
            "CMake files missing: [\"filenames-a\", \"filenames-b\"]",
        );
        assert_wire_format(
            &CMakeFilesMissing {
                filenames: vec!["filenames-a".to_string(), "filenames-b".to_string()],
                version: None,
            },
            "missing-cmake-files",
            json!({"filenames": ["filenames-a", "filenames-b"], "version": null}),
            "CMake files missing: [\"filenames-a\", \"filenames-b\"]",
        );
        assert_wire_format(
            &MissingCMakeComponents {
                name: "name-value".to_string(),
                components: vec!["components-a".to_string(), "components-b".to_string()],
            },
            "missing-cmake-components",
            json!({"components": ["components-a", "components-b"], "name": "name-value"}),
            "Missing CMake components: [\"components-a\", \"components-b\"]",
        );
        assert_wire_format(
            &MissingCMakeConfig {
                name: "name-value".to_string(),
                version: Some("version-value".to_string()),
            },
            "missing-cmake-config",
            json!({"name": "name-value", "version": "version-value"}),
            "Missing CMake package configuration for name-value (version version-value)",
        );
        assert_wire_format(
            &MissingCMakeConfig {
                name: "name-value".to_string(),
                version: None,
            },
            "missing-cmake-config",
            json!({"name": "name-value", "version": null}),
            "Missing CMake package configuration for name-value",
        );
        assert_wire_format(
            &CMakeNeedExactVersion {
                package: "package-value".to_string(),
                version_found: "version_found-value".to_string(),
                exact_version_needed: "exact_version_needed-value".to_string(),
                path: PathBuf::from("/srv/path"),
            },
            "cmake-exact-version-missing",
            json!({
                "exact_version_needed": "exact_version_needed-value",
                "package": "package-value",
                "path": "/srv/path",
                "version_found": "version_found-value",
            }),
            "CMake needs exact package package-value, version exact_version_needed-value",
        );
        assert_wire_format(
            &MissingStaticLibrary {
                library: "library-value".to_string(),
                filename: "filename-value".to_string(),
            },
            "missing-static-library",
            json!({"filename": "filename-value", "library": "library-value"}),
            "missing static library: library-value",
        );
        assert_wire_format(
            &MissingGoRuntime,
            "missing-go-runtime",
            json!({}),
            "go runtime is missing",
        );
        assert_wire_format(
            &UnknownCertificateAuthority {
                url: "f0-value".to_string(),
            },
            "unknown-certificate-authority",
            json!({"url": "f0-value"}),
            "Unknown Certificate Authority for f0-value",
        );
        assert_wire_format(
            &MissingPerlPredeclared {
                name: "f0-value".to_string(),
            },
            "missing-perl-predeclared",
            json!({"name": "f0-value"}),
            "missing predeclared function: f0-value",
        );
        assert_wire_format(
            &MissingGitIdentity,
            "missing-git-identity",
            json!({}),
            "Missing Git Identity",
        );
        assert_wire_format(
            &MissingSecretGpgKey,
            "no-secret-gpg-key",
            json!({}),
            "No secret GPG key is present",
        );
        assert_wire_format(
            &MissingVcVersionerVersion,
            "no-vcversioner-version",
            json!({}),
            "vcversion could not find a git directory or version.txt file",
        );
        assert_wire_format(
            &MissingLatexFile {
                filename: "f0-value".to_string(),
            },
            "missing-latex-file",
            json!({"filename": "f0-value"}),
            "Missing LaTeX file: f0-value",
        );
        assert_wire_format(
            &MissingXDisplay,
            "missing-x-display",
            json!({}),
            "No X Display",
        );
        assert_wire_format(
            &MissingFontspec {
                fontspec: "f0-value".to_string(),
            },
            "missing-fontspec",
            json!({"fontspec": "f0-value"}),
            "Missing font spec: f0-value",
        );
        assert_wire_format(
            &InactiveKilled { minutes: 42 },
            "inactive-killed",
            json!({"minutes": 42}),
            "Killed due to inactivity after 42 minutes",
        );
        assert_wire_format(
            &MissingPauseCredentials,
            "missing-pause-credentials",
            json!({}),
            "Missing credentials for PAUSE",
        );
        assert_wire_format(
            &MismatchGettextVersions {
                makefile_version: "makefile_version-value".to_string(),
                autoconf_version: "autoconf_version-value".to_string(),
            },
            "mismatch-gettext-versions",
            json!({
                "autoconf_version": "autoconf_version-value",
                "makefile_version": "makefile_version-value",
            }),
            "Mismatch versions (makefile_version-value, autoconf_version-value)",
        );
        assert_wire_format(
            &InvalidCurrentUser {
                user: "f0-value".to_string(),
            },
            "invalid-current-user",
            json!({"user": "f0-value"}),
            "Can not run as f0-value",
        );
        assert_wire_format(
            &MissingGnulibDirectory {
                directory: PathBuf::from("/srv/f0"),
            },
            "missing-gnulib-directory",
            json!({"directory": "/srv/f0"}),
            "Missing gnulib directory: /srv/f0",
        );
        assert_wire_format(
            &MissingLuaModule {
                module: "f0-value".to_string(),
            },
            "missing-lua-module",
            json!({"module": "f0-value"}),
            "Missing Lua Module: f0-value",
        );
        assert_wire_format(
            &MissingGoModFile,
            "missing-go.mod-file",
            json!({}),
            "go.mod file is missing",
        );
        assert_wire_format(
            &OutdatedGoModFile,
            "outdated-go.mod-file",
            json!({}),
            "go.mod file is outdated",
        );
        assert_wire_format(
            &CodeCoverageTooLow {
                actual: 61.5,
                required: 61.5,
            },
            "code-coverage-too-low",
            json!({"actual": 61.5, "required": 61.5}),
            "Code coverage too low: 61.50 < 61.50",
        );
        assert_wire_format(
            &ESModuleMustUseImport {
                path: "f0-value".to_string(),
            },
            "esmodule-must-use-import",
            json!({"path": "f0-value"}),
            "ESM-only module f0-value must use import()",
        );
        assert_wire_format(
            &MissingPHPExtension {
                extension: "f0-value".to_string(),
            },
            "missing-php-extension",
            json!({"extension": "f0-value"}),
            "Missing PHP Extension: f0-value",
        );
        assert_wire_format(
            &MinimumAutoconfTooOld {
                minimum_version: "f0-value".to_string(),
            },
            "minimum-autoconf-too-old",
            json!({"minimum_version": "f0-value"}),
            "configure.{ac,in} should require newer autoconf f0-value",
        );
        assert_wire_format(
            &MissingPerlDistributionFile {
                filename: "f0-value".to_string(),
            },
            "missing-perl-distribution-file",
            json!({"filename": "f0-value"}),
            "Missing perl distribution file: f0-value",
        );
        assert_wire_format(
            &MissingGoSumEntry {
                package: "package-value".to_string(),
                version: "version-value".to_string(),
            },
            "missing-go.sum-entry",
            json!({"package": "package-value", "version": "version-value"}),
            "Missing go.sum entry: package-value@version-value",
        );
        assert_wire_format(
            &ValaCompilerCannotCompile,
            "valac-cannot-compile",
            json!({}),
            "valac can not compile",
        );
        assert_wire_format(
            &MissingDebianBuildDep {
                dep: "f0-value".to_string(),
            },
            "missing-debian-build-dep",
            json!({"dep": "f0-value"}),
            "Missing Debian Build-Depends: f0-value",
        );
        assert_wire_format(
            &MissingQtModules {
                modules: vec!["f0-a".to_string(), "f0-b".to_string()],
            },
            "missing-qt-modules",
            json!({"modules": ["f0-a", "f0-b"]}),
            "Missing QT modules: [\"f0-a\", \"f0-b\"]",
        );
        assert_wire_format(
            &MissingOCamlPackage {
                package: "f0-value".to_string(),
            },
            "missing-ocaml-package",
            json!({"package": "f0-value"}),
            "Missing OCaml package: f0-value",
        );
        assert_wire_format(
            &TooManyOpenFiles,
            "too-many-open-files",
            json!({}),
            "Too many open files",
        );
        assert_wire_format(
            &MissingMakeTarget {
                target: "f0-value".to_string(),
                required_by: Some("f1-value".to_string()),
            },
            "missing-make-target",
            json!({"required_by": "f1-value", "target": "f0-value"}),
            "Unknown make target: f0-value",
        );
        assert_wire_format(
            &MissingMakeTarget {
                target: "f0-value".to_string(),
                required_by: None,
            },
            "missing-make-target",
            json!({"required_by": null, "target": "f0-value"}),
            "Unknown make target: f0-value",
        );
    }

    #[test]
    fn test_missing_python_distribution_from_requirement_str() {
        assert_eq!(
            MissingPythonDistribution::from_requirement_str("foo>=1.0", None).unwrap(),
            MissingPythonDistribution {
                distribution: "foo".to_string(),
                python_version: None,
                minimum_version: Some("1.0".to_string()),
            }
        );
    }

    #[test]
    fn test_missing_python_distribution_from_invalid_requirement_str() {
        assert_eq!(
            MissingPythonDistribution::from_requirement_str("foo>=", None)
                .unwrap_err()
                .to_string(),
            "Unexpected end of version specifier, expected version\nfoo>=\n   ^^"
        );
    }
}
