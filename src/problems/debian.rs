use crate::Problem;
use debversion::Version;

/// Problem representing a generic dpkg error.
///
/// This struct is used for errors reported by the dpkg package manager
/// that don't fit into more specific categories.
#[derive(Debug)]
pub struct DpkgError(pub String);

impl Problem for DpkgError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "msg": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dpkg error: {}", self.0)
    }
}

/// Problem representing an error during apt-get update.
///
/// This struct is used when the apt package database update process
/// fails for any reason.
#[derive(Debug, Clone)]
pub struct AptUpdateError;

impl Problem for AptUpdateError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-update-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AptUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt update error")
    }
}

/// Problem representing a failure to fetch a package or repository data.
///
/// This struct is used when apt cannot download a package or repository
/// data from the specified URL.
#[derive(Debug, Clone)]
pub struct AptFetchFailure {
    /// The URL that apt was trying to fetch from, if available.
    pub url: Option<String>,
    /// The error message from the fetch failure.
    pub error: String,
}

impl Problem for AptFetchFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-file-fetch-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "error": self.error,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AptFetchFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(url) = &self.url {
            write!(f, "apt fetch failure: {} ({})", url, self.error)
        } else {
            write!(f, "apt fetch failure: {}", self.error)
        }
    }
}

/// Problem representing a missing Release file for a repository.
///
/// This struct is used when apt cannot find the Release file for a repository,
/// which typically indicates a misconfigured or unavailable repository.
#[derive(Debug, Clone)]
pub struct AptMissingReleaseFile(pub String);

impl Problem for AptMissingReleaseFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-release-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AptMissingReleaseFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt missing release file: {}", self.0)
    }
}

/// Problem representing a package that apt cannot find.
///
/// This struct is used when apt cannot find a requested package in any
/// of the configured repositories.
#[derive(Debug, Clone)]
pub struct AptPackageUnknown(pub String);

impl Problem for AptPackageUnknown {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-package-unknown".into()
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

impl std::fmt::Display for AptPackageUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt package unknown: {}", self.0)
    }
}

/// Problem representing broken package dependencies.
///
/// This struct is used when apt reports broken packages in the dependency
/// resolution process, which can occur when packages have incompatible dependencies.
#[derive(Debug, Clone)]
pub struct AptBrokenPackages {
    /// A description of the broken package situation.
    pub description: String,
    /// List of packages that are broken, if available.
    pub broken: Option<Vec<String>>,
}

impl Problem for AptBrokenPackages {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-broken-packages".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "description": self.description,
            "broken": self.broken,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for AptBrokenPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt broken packages: {}", self.description)
    }
}

/// Problem representing a missing upstream source tarball.
///
/// This struct is used when the build process cannot find the upstream
/// source tarball for a package, which is required for the build.
#[derive(Debug, Clone)]
pub struct UnableToFindUpstreamTarball {
    /// The name of the package.
    pub package: String,
    /// The version of the package for which the tarball is missing.
    pub version: Version,
}

impl Problem for UnableToFindUpstreamTarball {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unable-to-find-upstream-tarball".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "package": self.package,
            "version": self.version.to_string(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UnableToFindUpstreamTarball {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to find upstream tarball for {} {}",
            self.package, self.version
        )
    }
}

/// Problem representing a source format that cannot be built.
///
/// This struct is used when the source package format specified in
/// debian/source/format cannot be built for some reason.
#[derive(Debug, Clone)]
pub struct SourceFormatUnbuildable {
    /// The source format that can't be built (e.g., "3.0 (quilt)").
    pub source_format: String,
    /// The reason why the source format cannot be built.
    pub reason: String,
}

impl Problem for SourceFormatUnbuildable {
    fn kind(&self) -> std::borrow::Cow<str> {
        "source-format-unbuildable".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "source_format": self.source_format,
            "reason": self.reason,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for SourceFormatUnbuildable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Source format {} is unbuildable: {}",
            self.source_format, self.reason
        )
    }
}

/// Problem representing a source format that is not supported.
///
/// This struct is used when the source package format specified in
/// debian/source/format is not supported by the build environment.
#[derive(Debug, Clone)]
pub struct SourceFormatUnsupported(pub String);

impl Problem for SourceFormatUnsupported {
    fn kind(&self) -> std::borrow::Cow<str> {
        "source-format-unsupported".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "source_format": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for SourceFormatUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source format {} is unsupported", self.0)
    }
}

/// Problem representing a missing patch file.
///
/// This struct is used when a build requires a patch file that is
/// referenced in the debian/patches directory but is not found.
#[derive(Debug, Clone)]
pub struct PatchFileMissing(pub std::path::PathBuf);

impl Problem for PatchFileMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "patch-file-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.0.display().to_string(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for PatchFileMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Patch file missing: {}", self.0.display())
    }
}

/// Problem representing unexpected local changes in the source package.
///
/// This struct is used when dpkg-source detects unexpected local changes
/// to upstream source code, which should be represented as patches instead.
#[derive(Debug, Clone)]
pub struct DpkgSourceLocalChanges {
    /// Path to the diff file showing the changes, if available.
    pub diff_file: Option<String>,
    /// List of files that have been changed locally, if available.
    pub files: Option<Vec<String>>,
}

impl Problem for DpkgSourceLocalChanges {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unexpected-local-upstream-changes".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "diff_file": self.diff_file,
            "files": self.files,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgSourceLocalChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(files) = self.files.as_ref() {
            if files.len() < 5 {
                write!(f, "Tree has local changes: {:?}", files)?;
                return Ok(());
            }

            write!(f, "Tree has local changes: {} files", files.len())?;
        } else {
            write!(f, "Tree has local changes")?;
        }
        Ok(())
    }
}

/// Problem representing changes that cannot be represented in the source package.
///
/// This struct is used when dpkg-source detects changes that cannot be
/// represented in the chosen source format, such as mode changes in some formats.
#[derive(Debug, Clone)]
pub struct DpkgSourceUnrepresentableChanges;

impl Problem for DpkgSourceUnrepresentableChanges {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unrepresentable-local-changes".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgSourceUnrepresentableChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unrepresentable changes")
    }
}

/// Problem representing unwanted binary files in the source package.
///
/// This struct is used when dpkg-source detects binary files in the source
/// package that are not allowed, which can happen when the source is dirty.
#[derive(Debug, Clone)]
pub struct DpkgUnwantedBinaryFiles;

impl Problem for DpkgUnwantedBinaryFiles {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unwanted-binary-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgUnwantedBinaryFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unwanted binary files")
    }
}

/// Problem representing changes to binary files in the source package.
///
/// This struct is used when dpkg-source detects that binary files have been
/// changed, which cannot be properly represented in source package formats.
#[derive(Debug, Clone)]
pub struct DpkgBinaryFileChanged(pub Vec<String>);

impl Problem for DpkgBinaryFileChanged {
    fn kind(&self) -> std::borrow::Cow<str> {
        "binary-file-changed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "files": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgBinaryFileChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binary file changed")
    }
}

/// Problem representing a missing debian control file.
///
/// This struct is used when the debian/control file, which is required for
/// any Debian package, is missing from the source package.
#[derive(Debug, Clone)]
pub struct MissingControlFile(pub std::path::PathBuf);

impl Problem for MissingControlFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-control-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingControlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing control file: {}", self.0.display())
    }
}

/// Problem representing unknown Mercurial extra fields.
///
/// This struct is used when the build process encounters unknown extra fields
/// in Mercurial version control metadata.
#[derive(Debug, Clone)]
pub struct UnknownMercurialExtraFields(pub String);

impl Problem for UnknownMercurialExtraFields {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unknown-mercurial-extra-fields".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "field": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UnknownMercurialExtraFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown Mercurial extra field: {}", self.0)
    }
}

/// Problem representing a failure to verify an upstream PGP signature.
///
/// This struct is used when the build process cannot verify the PGP signature
/// of an upstream source tarball, which may indicate a security issue.
#[derive(Debug, Clone)]
pub struct UpstreamPGPSignatureVerificationFailed;

impl Problem for UpstreamPGPSignatureVerificationFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "upstream-pgp-signature-verification-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UpstreamPGPSignatureVerificationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream PGP signature verification failed")
    }
}

/// Problem representing a missing requested version in uscan.
///
/// This struct is used when the uscan tool (which checks for upstream versions)
/// cannot find a specifically requested version.
#[derive(Debug, Clone)]
pub struct UScanRequestVersionMissing(pub String);

impl Problem for UScanRequestVersionMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-request-version-missing".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UScanRequestVersionMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan request version missing: {}", self.0)
    }
}

/// Problem representing a failure in the debcargo tool.
///
/// This struct is used when the debcargo tool, which is used to package
/// Rust crates as Debian packages, encounters an error.
#[derive(Debug, Clone)]
pub struct DebcargoFailure(pub String);

impl Problem for DebcargoFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-failure".into()
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

impl std::fmt::Display for DebcargoFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debcargo failure: {}", self.0)
    }
}

/// Problem representing an error parsing a debian/changelog file.
///
/// This struct is used when the build process encounters a syntax error
/// or other issue when parsing the debian/changelog file.
#[derive(Debug, Clone)]
pub struct ChangelogParseError(pub String);

impl Problem for ChangelogParseError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "changelog-parse-error".into()
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

impl std::fmt::Display for ChangelogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Changelog parse error: {}", self.0)
    }
}

/// Problem representing a generic error in the uscan tool.
///
/// This struct is used when the uscan tool, which checks for upstream versions,
/// encounters an error that doesn't fit into more specific categories.
#[derive(Debug, Clone)]
pub struct UScanError(pub String);

impl Problem for UScanError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-error".into()
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

impl std::fmt::Display for UScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan error: {}", self.0)
    }
}

/// Problem representing a failure in the uscan tool.
///
/// This struct is used when the uscan tool fails to find or process
/// upstream versions from a specific URL.
#[derive(Debug, Clone)]
pub struct UScanFailed {
    /// The URL that uscan was trying to process.
    pub url: String,
    /// The reason for the failure.
    pub reason: String,
}

impl Problem for UScanFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "reason": self.reason,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UScanFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan failed: {}", self.reason)
    }
}

/// Problem representing inconsistency between source format and version.
///
/// This struct is used when there's an inconsistency between the source format
/// specified in debian/source/format and the version numbering scheme.
#[derive(Debug, Clone)]
pub struct InconsistentSourceFormat {
    /// Whether the version is inconsistent with the source format.
    pub version: bool,
    /// Whether the source format is inconsistent with the version.
    pub source_format: bool,
}

impl Problem for InconsistentSourceFormat {
    fn kind(&self) -> std::borrow::Cow<str> {
        "inconsistent-source-format".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "source_format": self.source_format,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for InconsistentSourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inconsistent source format between version and source format"
        )
    }
}

/// Problem representing an error parsing the debian/upstream/metadata file.
///
/// This struct is used when the build process cannot parse the
/// debian/upstream/metadata file, which contains information about the upstream project.
#[derive(Debug, Clone)]
pub struct UpstreamMetadataFileParseError {
    /// The path to the metadata file.
    pub path: std::path::PathBuf,
    /// The reason for the parsing failure.
    pub reason: String,
}

impl Problem for UpstreamMetadataFileParseError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debian-upstream-metadata-invalid".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path.display().to_string(),
            "reason": self.reason,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UpstreamMetadataFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream metadata file parse error: {}", self.reason)
    }
}

/// Problem representing a failure in dpkg-source when packaging source files.
///
/// This struct is used when dpkg-source cannot package the source files
/// into a source package for various reasons.
#[derive(Debug, Clone)]
pub struct DpkgSourcePackFailed(pub String);

impl Problem for DpkgSourcePackFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-source-pack-failed".into()
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

impl std::fmt::Display for DpkgSourcePackFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dpkg source pack failed: {}", self.0)
    }
}

/// Problem representing an invalid version string in a package.
///
/// This struct is used when dpkg encounters a version string that
/// doesn't follow the Debian version format rules.
#[derive(Debug, Clone)]
pub struct DpkgBadVersion {
    /// The invalid version string.
    pub version: String,
    /// The reason why the version is invalid, if available.
    pub reason: Option<String>,
}

impl Problem for DpkgBadVersion {
    fn kind(&self) -> std::borrow::Cow<str> {
        "dpkg-bad-version".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "reason": self.reason,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DpkgBadVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(reason) = &self.reason {
            write!(f, "Version {} is invalid: {}", self.version, reason)
        } else {
            write!(f, "Version {} is invalid", self.version)
        }
    }
}

/// Problem representing a missing Rust crate in debcargo.
///
/// This struct is used when debcargo cannot find a Rust crate
/// that is required for the build.
#[derive(Debug, Clone)]
pub struct MissingDebcargoCrate {
    /// The name of the missing Rust crate.
    pub cratename: String,
    /// The version of the crate that is required, if specified.
    pub version: Option<String>,
}

impl Problem for MissingDebcargoCrate {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-missing-crate".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.cratename,
            "version": self.version,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingDebcargoCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(
                f,
                "debcargo can't find crate {} (version: {})",
                self.cratename, version
            )
        } else {
            write!(f, "debcargo can't find crate {}", self.cratename)
        }
    }
}

impl MissingDebcargoCrate {
    /// Creates a MissingDebcargoCrate instance from a string.
    ///
    /// Parses a string in the format "cratename=version" or just "cratename"
    /// to create a new instance.
    ///
    /// # Arguments
    /// * `text` - The string to parse
    ///
    /// # Returns
    /// A new MissingDebcargoCrate instance
    pub fn from_string(text: &str) -> Self {
        let text = text.trim();
        if let Some((cratename, version)) = text.split_once('=') {
            Self {
                cratename: cratename.trim().to_string(),
                version: Some(version.trim().to_string()),
            }
        } else {
            Self {
                cratename: text.to_string(),
                version: None,
            }
        }
    }
}

/// Problem representing a missing pristine-tar tree reference.
///
/// This struct is used when a pristine-tar operation cannot find
/// a referenced tree in the git repository.
#[derive(Debug, Clone)]
pub struct PristineTarTreeMissing(pub String);

impl Problem for PristineTarTreeMissing {
    fn kind(&self) -> std::borrow::Cow<str> {
        "pristine-tar-missing-tree".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "treeish": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for PristineTarTreeMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pristine-tar tree missing: {}", self.0)
    }
}

/// Problem representing a missing revision in version control.
///
/// This struct is used when a build process references a revision
/// in version control that does not exist.
#[derive(Debug, Clone)]
pub struct MissingRevision(pub Vec<u8>);

impl Problem for MissingRevision {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-revision".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "revision": String::from_utf8_lossy(&self.0),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for MissingRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing revision: {}", String::from_utf8_lossy(&self.0))
    }
}

/// Problem representing a Rust crate dependency predicate that debcargo cannot handle.
///
/// This struct is used when debcargo cannot represent a predicate in a Rust
/// crate dependency, such as certain prerelease version constraints.
#[derive(Debug)]
pub struct DebcargoUnacceptablePredicate {
    /// The name of the crate with the unacceptable predicate.
    pub cratename: String,
    /// The predicate that cannot be represented.
    pub predicate: String,
}

impl Problem for DebcargoUnacceptablePredicate {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-unacceptable-predicate".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.cratename,
            "predicate": self.predicate,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DebcargoUnacceptablePredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot represent prerelease part of dependency: {}",
            self.predicate
        )
    }
}

/// Problem representing a Rust crate dependency comparator that debcargo cannot handle.
///
/// This struct is used when debcargo cannot represent a version comparison operator
/// in a Rust crate dependency, such as certain complex version constraints.
#[derive(Debug)]
pub struct DebcargoUnacceptableComparator {
    /// The name of the crate with the unacceptable comparator.
    pub cratename: String,
    /// The comparator that cannot be represented.
    pub comparator: String,
}

impl Problem for DebcargoUnacceptableComparator {
    fn kind(&self) -> std::borrow::Cow<str> {
        "debcargo-unacceptable-comparator".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "crate": self.cratename,
            "comparator": self.comparator,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for DebcargoUnacceptableComparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot represent prerelease part of dependency: {}",
            self.comparator
        )
    }
}

/// Problem representing a "too many requests" error from uscan.
///
/// This struct is used when uscan receives a rate limiting response
/// from a server it is checking for upstream versions.
#[derive(Debug)]
pub struct UScanTooManyRequests(pub String);

impl Problem for UScanTooManyRequests {
    fn kind(&self) -> std::borrow::Cow<str> {
        "uscan-too-many-requests".into()
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

impl std::fmt::Display for UScanTooManyRequests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan too many requests: {}", self.0)
    }
}

/// Problem representing unsatisfied conflicts in apt dependencies.
///
/// This struct is used when apt cannot resolve package conflicts
/// during the dependency resolution process.
#[derive(Debug)]
pub struct UnsatisfiedAptConflicts(pub String);

impl Problem for UnsatisfiedAptConflicts {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unsatisfied-apt-conflicts".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "relations": self.0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UnsatisfiedAptConflicts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt conflicts: {}", self.0)
    }
}

impl std::error::Error for UnsatisfiedAptConflicts {}

/// Problem representing an architecture not in the supported architecture list.
///
/// This struct is used when a build is attempted for an architecture that
/// is not in the list of architectures supported by the package.
#[derive(Debug, Clone)]
pub struct ArchitectureNotInList {
    /// The architecture being built for.
    pub arch: String,
    /// The list of supported architectures.
    pub arch_list: Vec<String>,
}

impl Problem for ArchitectureNotInList {
    fn kind(&self) -> std::borrow::Cow<str> {
        "arch-not-in-list".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "arch": self.arch,
            "arch_list": self.arch_list,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for ArchitectureNotInList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Architecture {} not a build arch", self.arch)
    }
}

/// Problem representing unsatisfied dependencies in apt.
///
/// This struct is used when apt cannot satisfy the dependencies
/// required for a package installation.
#[derive(Debug)]
pub struct UnsatisfiedAptDependencies(pub String);

impl Problem for UnsatisfiedAptDependencies {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unsatisfied-apt-dependencies".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "relations": self.0
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for UnsatisfiedAptDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt dependencies: {}", self.0)
    }
}

/// Problem representing insufficient disk space for a build.
///
/// This struct is used when a build process determines that there is
/// not enough disk space available to complete the build.
#[derive(Debug)]
pub struct InsufficientDiskSpace {
    /// The amount of disk space needed for the build in KiB.
    pub needed: i64,
    /// The amount of free disk space available in KiB.
    pub free: i64,
}

impl Problem for InsufficientDiskSpace {
    fn kind(&self) -> std::borrow::Cow<str> {
        "insufficient-disk-space".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "needed": self.needed,
            "free": self.free,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for InsufficientDiskSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient disk space for build. Need: {} KiB, free: {} KiB",
            self.needed, self.free
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Problem;

    #[test]
    fn test_dpkg_source_local_changes_trait() {
        let problem = DpkgSourceLocalChanges {
            diff_file: Some("/tmp/diff.patch".to_string()),
            files: Some(vec!["file1.txt".to_string(), "file2.txt".to_string()]),
        };
        let json = problem.json();
        assert_eq!(json["diff_file"], "/tmp/diff.patch");
        assert_eq!(json["files"], serde_json::json!(["file1.txt", "file2.txt"]));
    }

    #[test]
    fn test_uscan_too_many_requests_trait() {
        let problem = UScanTooManyRequests("rate limit exceeded".to_string());
        let json = problem.json();
        assert_eq!(json["reason"], "rate limit exceeded");
    }
}
