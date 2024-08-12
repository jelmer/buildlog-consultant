use crate::Problem;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::path::PathBuf;
use pyo3::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingFile {
    pub path: PathBuf,
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
    pub fn simple(package: String) -> Self {
        Self {
            package,
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


