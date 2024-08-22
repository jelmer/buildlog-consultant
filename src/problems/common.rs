use crate::Problem;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingFile {
    pub path: PathBuf,
}

impl MissingFile {
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
}

impl Display for MissingFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing file: {}", self.path.display())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MissingBuildFile {
    pub filename: String,
}

impl MissingBuildFile {
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
}

impl Display for MissingBuildFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing build file: {}", self.filename)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingCommandOrBuildFile {
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
}

impl Display for MissingCommandOrBuildFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing command or build file: {}", self.filename)
    }
}

impl MissingCommandOrBuildFile {
    pub fn command(&self) -> String {
        self.filename.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcsControlDirectoryNeeded {
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
}

impl VcsControlDirectoryNeeded {
    pub fn new(vcs: Vec<&str>) -> Self {
        Self {
            vcs: vcs.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MissingPythonModule {
    pub module: String,
    pub python_version: Option<i32>,
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
}

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
}

#[derive(Debug, Clone)]
pub struct MissingPythonDistribution {
    pub distribution: String,
    pub python_version: Option<i32>,
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
}

impl MissingPythonDistribution {
    pub fn from_requirement_str(
        text: &str,
        python_version: Option<i32>,
    ) -> PyResult<MissingPythonDistribution> {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let requirement = py
                .import_bound("requirements.requirement")?
                .getattr("Requirement")?
                .call_method1("parse", (text,))?;
            let distribution = requirement.getattr("name")?.extract::<String>()?;
            let specs = requirement
                .getattr("specs")?
                .extract::<Vec<(String, String)>>()?;

            Ok(if specs.len() == 1 && specs[0].0 == ">=" {
                MissingPythonDistribution {
                    distribution,
                    python_version,
                    minimum_version: Some(specs[0].1.clone()),
                }
            } else {
                MissingPythonDistribution {
                    distribution,
                    python_version,
                    minimum_version: None,
                }
            })
        })
    }

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

#[derive(Debug, Clone)]
pub struct MissingHaskellModule {
    pub module: String,
}

impl MissingHaskellModule {
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
}

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
}

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
}

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
}

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
}

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
}

#[derive(Debug, Clone)]
pub struct MissingRPackage {
    pub package: String,
    pub minimum_version: Option<String>,
}

impl MissingRPackage {
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
}

#[derive(Debug, Clone)]
pub struct MissingGoPackage {
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
}

impl Display for MissingGoPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Go package: {}", self.package)
    }
}

#[derive(Debug, Clone)]
pub struct MissingCHeader {
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
}

impl Display for MissingCHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C header: {}", self.header)
    }
}

impl MissingCHeader {
    pub fn new(header: String) -> Self {
        Self { header }
    }
}

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
}

impl Display for MissingNodeModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node module: {}", self.0)
    }
}

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
}

impl Display for MissingNodePackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Node package: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingConfigure;

impl Problem for MissingConfigure {
    fn kind(&self) -> Cow<str> {
        "missing-configure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingConfigure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing ./configure")
    }
}

#[derive(Debug, Clone)]
pub struct MissingVagueDependency {
    pub name: String,
    pub url: Option<String>,
    pub minimum_version: Option<String>,
    pub current_version: Option<String>,
}

impl MissingVagueDependency {
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
}

impl Display for MissingVagueDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing dependency: {}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct MissingQt;

impl Problem for MissingQt {
    fn kind(&self) -> Cow<str> {
        "missing-qt".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingQt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Qt")
    }
}

#[derive(Debug, Clone)]
pub struct MissingX11;

impl Problem for MissingX11 {
    fn kind(&self) -> Cow<str> {
        "missing-x11".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingX11 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing X11")
    }
}

#[derive(Debug, Clone)]
pub struct MissingAutoconfMacro {
    pub r#macro: String,
    pub need_rebuild: bool,
}

impl MissingAutoconfMacro {
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
}

impl Display for MissingAutoconfMacro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing autoconf macro: {}", self.r#macro)
    }
}

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
}

impl Display for DirectoryNonExistant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Directory does not exist: {}", self.0)
    }
}

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
}

impl Display for MissingValaPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Vala package: {}", self.0)
    }
}

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
}

impl Display for UpstartFilePresent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Upstart file present: {}", self.0)
    }
}

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
}

impl Display for MissingPostgresExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PostgreSQL extension: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingPkgConfig {
    pub module: String,
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
    pub fn new(module: String, minimum_version: Option<String>) -> Self {
        Self {
            module,
            minimum_version,
        }
    }

    pub fn simple(module: String) -> Self {
        Self {
            module,
            minimum_version: None,
        }
    }
}

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
}

impl Display for MissingHaskellDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Haskell dependencies: {:?}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct NoSpaceOnDevice;

impl Problem for NoSpaceOnDevice {
    fn kind(&self) -> Cow<str> {
        "no-space-on-device".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for NoSpaceOnDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No space left on device")
    }
}

#[derive(Debug, Clone)]
pub struct MissingJRE;

impl Problem for MissingJRE {
    fn kind(&self) -> Cow<str> {
        "missing-jre".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingJRE {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JRE")
    }
}

#[derive(Debug, Clone)]
pub struct MissingJDK {
    pub jdk_path: String,
}

impl MissingJDK {
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
}

impl Display for MissingJDK {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK at {}", self.jdk_path)
    }
}

#[derive(Debug, Clone)]
pub struct MissingJDKFile {
    pub jdk_path: String,
    pub filename: String,
}

impl MissingJDKFile {
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
}

impl Display for MissingJDKFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JDK file {} at {}", self.filename, self.jdk_path)
    }
}

#[derive(Debug, Clone)]
pub struct MissingPerlFile {
    pub filename: String,
    pub inc: Option<Vec<String>>,
}

impl MissingPerlFile {
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

#[derive(Debug, Clone)]
pub struct MissingPerlModule {
    pub filename: Option<String>,
    pub module: String,
    pub inc: Option<Vec<String>>,
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
    pub fn simple(module: &str) -> Self {
        Self {
            filename: None,
            module: module.to_string(),
            inc: None,
            minimum_version: None,
        }
    }
}

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
}

impl Display for MissingSetupPyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing setup.py command: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingCSharpCompiler;

impl Problem for MissingCSharpCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-c#-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingCSharpCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing C# compiler")
    }
}

#[derive(Debug, Clone)]
pub struct MissingRustCompiler;

impl Problem for MissingRustCompiler {
    fn kind(&self) -> Cow<str> {
        "missing-rust-compiler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingRustCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Rust compiler")
    }
}

#[derive(Debug, Clone)]
pub struct MissingAssembler;

impl Problem for MissingAssembler {
    fn kind(&self) -> Cow<str> {
        "missing-assembler".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for MissingAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing assembler")
    }
}

#[derive(Debug, Clone)]
pub struct MissingCargoCrate {
    pub crate_name: String,
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
}

impl MissingCargoCrate {
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

#[derive(Debug, Clone)]
pub struct DhWithOrderIncorrect;

impl Problem for DhWithOrderIncorrect {
    fn kind(&self) -> Cow<str> {
        "debhelper-argument-order".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for DhWithOrderIncorrect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh argument order is incorrect")
    }
}

#[derive(Debug, Clone)]
pub struct UnsupportedDebhelperCompatLevel {
    pub oldest_supported: u32,
    pub requested: u32,
}

impl UnsupportedDebhelperCompatLevel {
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
}

impl Display for UnsupportedDebhelperCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Request debhlper compat level {} lower than supported {}",
            self.requested, self.oldest_supported
        )
    }
}

#[derive(Debug, Clone)]
pub struct SetuptoolScmVersionIssue;

impl Problem for SetuptoolScmVersionIssue {
    fn kind(&self) -> Cow<str> {
        "setuptools-scm-version-issue".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for SetuptoolScmVersionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "setuptools_scm was unable to find version")
    }
}

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
}

impl Display for MissingMavenArtifacts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Maven artifacts: {}", self.0.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct NotExecutableFile(pub String);

impl NotExecutableFile {
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
}

impl Display for NotExecutableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Command not executable: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DhMissingUninstalled(pub String);

impl DhMissingUninstalled {
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
}

impl Display for DhMissingUninstalled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh_missing file not installed: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DhLinkDestinationIsDirectory(pub String);

impl DhLinkDestinationIsDirectory {
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
}

impl Display for DhLinkDestinationIsDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Link destination {} is directory", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingXmlEntity {
    pub url: String,
}

impl MissingXmlEntity {
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
}

impl Display for MissingXmlEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing XML entity: {}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct CcacheError(pub String);

impl CcacheError {
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
}

impl Display for CcacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ccache error: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DebianVersionRejected {
    pub version: String,
}

impl DebianVersionRejected {
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
}

impl Display for DebianVersionRejected {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Debian Version Rejected; {}", self.version)
    }
}

#[derive(Debug, Clone)]
pub struct PatchApplicationFailed {
    pub patchname: String,
}

impl PatchApplicationFailed {
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
}

impl Display for PatchApplicationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Patch application failed: {}", self.patchname)
    }
}

#[derive(Debug, Clone)]
pub struct NeedPgBuildExtUpdateControl {
    pub generated_path: String,
    pub template_path: String,
}

impl NeedPgBuildExtUpdateControl {
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

#[derive(Debug, Clone)]
pub struct DhAddonLoadFailure {
    pub name: String,
    pub path: String,
}

impl DhAddonLoadFailure {
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
}

impl Display for DhAddonLoadFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh addon loading failed: {}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct DhUntilUnsupported;

impl Default for DhUntilUnsupported {
    fn default() -> Self {
        Self::new()
    }
}

impl DhUntilUnsupported {
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
}

impl Display for DhUntilUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dh --until is no longer supported")
    }
}

#[derive(Debug, Clone)]
pub struct DebhelperPatternNotFound {
    pub pattern: String,
    pub tool: String,
    pub directories: Vec<String>,
}

impl DebhelperPatternNotFound {
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

#[derive(Debug, Clone)]
pub struct MissingPerlManifest;

impl Default for MissingPerlManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingPerlManifest {
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
}

impl Display for MissingPerlManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Perl MANIFEST")
    }
}

#[derive(Debug, Clone)]
pub struct ImageMagickDelegateMissing {
    pub delegate: String,
}

impl ImageMagickDelegateMissing {
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
}

impl Display for ImageMagickDelegateMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Imagemagick missing delegate: {}", self.delegate)
    }
}

#[derive(Debug, Clone)]
pub struct Cancelled;

impl Default for Cancelled {
    fn default() -> Self {
        Self::new()
    }
}

impl Cancelled {
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
}

impl Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cancelled by runner or job manager")
    }
}

#[derive(Debug, Clone)]
pub struct DisappearedSymbols;

impl Default for DisappearedSymbols {
    fn default() -> Self {
        Self::new()
    }
}

impl DisappearedSymbols {
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
}

impl Display for DisappearedSymbols {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Disappeared symbols")
    }
}

#[derive(Debug, Clone)]
pub struct DuplicateDHCompatLevel {
    pub command: String,
}

impl DuplicateDHCompatLevel {
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

#[derive(Debug, Clone)]
pub struct MissingDHCompatLevel {
    pub command: String,
}

impl MissingDHCompatLevel {
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
}

impl Display for MissingDHCompatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing DH Compat Level (command: {})", self.command)
    }
}

#[derive(Debug, Clone)]
pub struct MissingJVM;

impl Default for MissingJVM {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingJVM {
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
}

impl Display for MissingJVM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing JVM")
    }
}

#[derive(Debug, Clone)]
pub struct MissingRubyGem {
    pub gem: String,
    pub version: Option<String>,
}

impl MissingRubyGem {
    pub fn new(gem: String, version: Option<String>) -> Self {
        Self { gem, version }
    }

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

#[derive(Debug, Clone)]
pub struct MissingJavaScriptRuntime;

impl Default for MissingJavaScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingJavaScriptRuntime {
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
}

impl Display for MissingJavaScriptRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing JavaScript Runtime")
    }
}

#[derive(Debug, Clone)]
pub struct MissingRubyFile {
    pub filename: String,
}

impl MissingRubyFile {
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
}

impl Display for MissingRubyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing ruby file: {}", self.filename)
    }
}

#[derive(Debug, Clone)]
pub struct MissingPhpClass {
    pub php_class: String,
}

impl MissingPhpClass {
    pub fn new(php_class: String) -> Self {
        Self { php_class }
    }

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
}

impl Display for MissingPhpClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing PHP class: {}", self.php_class)
    }
}

#[derive(Debug, Clone)]
pub struct MissingJavaClass {
    pub classname: String,
}

impl MissingJavaClass {
    pub fn new(classname: String) -> Self {
        Self { classname }
    }

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
}

impl Display for MissingJavaClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing Java class: {}", self.classname)
    }
}

#[derive(Debug, Clone)]
pub struct MissingSprocketsFile {
    pub name: String,
    pub content_type: String,
}

impl MissingSprocketsFile {
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
pub struct MissingXfceDependency {
    pub package: String,
}

impl MissingXfceDependency {
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
}

impl Display for MissingXfceDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing XFCE build dependency: {}", self.package)
    }
}

#[derive(Debug, Clone)]
pub struct GnomeCommonMissing;

impl Problem for GnomeCommonMissing {
    fn kind(&self) -> Cow<str> {
        "missing-gnome-common".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl Display for GnomeCommonMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "gnome-common is not installed")
    }
}

#[derive(Debug, Clone)]
pub struct MissingConfigStatusInput {
    pub path: String,
}

impl MissingConfigStatusInput {
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
}

impl Display for MissingConfigStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing config.status input {}", self.path)
    }
}

#[derive(Debug, Clone)]
pub struct MissingGnomeCommonDependency {
    pub package: String,
    pub minimum_version: Option<String>,
}

impl MissingGnomeCommonDependency {
    pub fn new(package: String, minimum_version: Option<String>) -> Self {
        Self {
            package,
            minimum_version,
        }
    }

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
pub struct MissingAutomakeInput {
    pub path: String,
}

impl MissingAutomakeInput {
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
}

impl Display for MissingAutomakeInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "automake input file {} missing", self.path)
    }
}

#[derive(Debug, Clone)]
pub struct ChrootNotFound {
    pub chroot: String,
}

impl ChrootNotFound {
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
}

impl Display for ChrootNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "chroot not found: {}", self.chroot)
    }
}

#[derive(Debug, Clone)]
pub struct MissingLibtool;

impl Default for MissingLibtool {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingLibtool {
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
}

impl Display for MissingLibtool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Libtool is missing")
    }
}

#[derive(Debug, Clone)]
pub struct CMakeFilesMissing {
    pub filenames: Vec<String>,
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
}

impl std::fmt::Display for CMakeFilesMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CMake files missing: {:?}", self.filenames)
    }
}

#[derive(Debug, Clone)]
pub struct MissingCMakeComponents {
    pub name: String,
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
}

impl std::fmt::Display for MissingCMakeComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing CMake components: {:?}", self.components)
    }
}

#[derive(Debug, Clone)]
pub struct MissingCMakeConfig {
    pub name: String,
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
pub struct CMakeNeedExactVersion {
    pub package: String,
    pub version_found: String,
    pub exact_version_needed: String,
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
pub struct MissingStaticLibrary {
    pub library: String,
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
}

impl std::fmt::Display for MissingStaticLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing static library: {}", self.library)
    }
}

#[derive(Debug, Clone)]
pub struct MissingGoRuntime;

impl Problem for MissingGoRuntime {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-go-runtime".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingGoRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go runtime is missing")
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for UnknownCertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unknown Certificate Authority for {}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingPerlPredeclared {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "missing predeclared function: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingGitIdentity;

impl Problem for MissingGitIdentity {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-git-identity".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingGitIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Git Identity")
    }
}

#[derive(Debug, Clone)]
pub struct MissingSecretGpgKey;

impl Problem for MissingSecretGpgKey {
    fn kind(&self) -> std::borrow::Cow<str> {
        "no-secret-gpg-key".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingSecretGpgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No secret GPG key is present")
    }
}

#[derive(Debug, Clone)]
pub struct MissingVcVersionerVersion;

impl Problem for MissingVcVersionerVersion {
    fn kind(&self) -> std::borrow::Cow<str> {
        "no-vcversioner-version".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
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
pub struct MissingLatexFile(pub String);

impl MissingLatexFile {
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
}

impl std::fmt::Display for MissingLatexFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing LaTeX file: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingXDisplay;

impl Problem for MissingXDisplay {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-x-display".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingXDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No X Display")
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingFontspec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing font spec: {}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for InactiveKilled {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Killed due to inactivity after {} minutes", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingPauseCredentials;

impl Problem for MissingPauseCredentials {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-pause-credentials".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingPauseCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing credentials for PAUSE")
    }
}

#[derive(Debug, Clone)]
pub struct MismatchGettextVersions {
    pub makefile_version: String,
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
}

impl std::fmt::Display for InvalidCurrentUser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not run as {}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingGnulibDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing gnulib directory: {}", self.0.display())
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingLuaModule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Lua Module: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingGoModFile;

impl Problem for MissingGoModFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-go.mod-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for MissingGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is missing")
    }
}

#[derive(Debug, Clone)]
pub struct OutdatedGoModFile;

impl Problem for OutdatedGoModFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "outdated-go.mod-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for OutdatedGoModFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "go.mod file is outdated")
    }
}

#[derive(Debug, Clone)]
pub struct CodeCoverageTooLow {
    pub actual: f64,
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
}

impl std::fmt::Display for ESModuleMustUseImport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ESM-only module {} must use import()", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingPHPExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing PHP Extension: {}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingPerlDistributionFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing perl distribution file: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct MissingGoSumEntry {
    pub package: String,
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
}

impl std::fmt::Display for MissingGoSumEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing go.sum entry: {}@{}", self.package, self.version)
    }
}

#[derive(Debug, Clone)]
pub struct ValaCompilerCannotCompile;

impl Problem for ValaCompilerCannotCompile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "valac-cannot-compile".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for ValaCompilerCannotCompile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "valac can not compile")
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingDebianBuildDep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing Debian Build-Depends: {}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingQtModules {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing QT modules: {:?}", self.0)
    }
}

#[derive(Debug, Clone)]
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
}

impl std::fmt::Display for MissingOCamlPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing OCaml package: {}", self.0)
    }
}
