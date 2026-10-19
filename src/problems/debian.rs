use debversion::Version;
use serde::{Deserialize, Serialize};

/// Problem representing a generic dpkg error.
///
/// This struct is used for errors reported by the dpkg package manager
/// that don't fit into more specific categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgError {
    /// The error message from dpkg.
    pub msg: String,
}

crate::problem!(DpkgError, "dpkg-error", ["msg"]);

impl std::fmt::Display for DpkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dpkg error: {}", self.msg)
    }
}

/// Problem representing an error during apt-get update.
///
/// This struct is used when the apt package database update process
/// fails for any reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AptUpdateError;

crate::problem!(AptUpdateError, "apt-update-error", {});

impl std::fmt::Display for AptUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt update error")
    }
}

/// Problem representing a failure to fetch a package or repository data.
///
/// This struct is used when apt cannot download a package or repository
/// data from the specified URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AptFetchFailure {
    /// The URL that apt was trying to fetch from, if available.
    pub url: Option<String>,
    /// The error message from the fetch failure.
    pub error: String,
}

crate::problem!(AptFetchFailure, "apt-file-fetch-failure", ["url", "error"]);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AptMissingReleaseFile {
    /// The URL of the missing Release file.
    pub url: String,
}

crate::problem!(AptMissingReleaseFile, "missing-release-file", ["url"]);

impl std::fmt::Display for AptMissingReleaseFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt missing release file: {}", self.url)
    }
}

/// Problem representing a package that apt cannot find.
///
/// This struct is used when apt cannot find a requested package in any
/// of the configured repositories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "AptPackageUnknownDetails")]
pub struct AptPackageUnknown {
    /// The name of the unknown package.
    pub package: String,
}

/// The details may also be a bare string.
#[derive(Deserialize)]
#[serde(untagged)]
enum AptPackageUnknownDetails {
    Object { package: String },
    Bare(String),
}

impl From<AptPackageUnknownDetails> for AptPackageUnknown {
    fn from(details: AptPackageUnknownDetails) -> Self {
        match details {
            AptPackageUnknownDetails::Object { package }
            | AptPackageUnknownDetails::Bare(package) => Self { package },
        }
    }
}

crate::problem!(AptPackageUnknown, "apt-package-unknown", ["package"]);

impl std::fmt::Display for AptPackageUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt package unknown: {}", self.package)
    }
}

/// Problem representing broken package dependencies.
///
/// This struct is used when apt reports broken packages in the dependency
/// resolution process, which can occur when packages have incompatible dependencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AptBrokenPackages {
    /// A description of the broken package situation.
    pub description: String,
    /// List of packages that are broken, if available.
    pub broken: Option<Vec<String>>,
}

crate::problem!(
    AptBrokenPackages,
    "apt-broken-packages",
    ["description", "broken"]
);

impl std::fmt::Display for AptBrokenPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt broken packages: {}", self.description)
    }
}

/// Problem representing a missing upstream source tarball.
///
/// This struct is used when the build process cannot find the upstream
/// source tarball for a package, which is required for the build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnableToFindUpstreamTarball {
    /// The name of the package.
    pub package: String,
    /// The version of the package for which the tarball is missing.
    pub version: Version,
}

crate::problem!(
    UnableToFindUpstreamTarball,
    "unable-to-find-upstream-tarball",
    ["package", "version"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFormatUnbuildable {
    /// The source format that can't be built (e.g., "3.0 (quilt)").
    pub source_format: String,
    /// The reason why the source format cannot be built.
    pub reason: String,
}

crate::problem!(
    SourceFormatUnbuildable,
    "source-format-unbuildable",
    ["source_format", "reason"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFormatUnsupported {
    /// The unsupported source format.
    pub source_format: String,
}

crate::problem!(
    SourceFormatUnsupported,
    "source-format-unsupported",
    ["source_format"]
);

impl std::fmt::Display for SourceFormatUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source format {} is unsupported", self.source_format)
    }
}

/// Problem representing a missing patch file.
///
/// This struct is used when a build requires a patch file that is
/// referenced in the debian/patches directory but is not found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchFileMissing {
    /// The path to the missing patch file.
    pub path: std::path::PathBuf,
}

crate::problem!(PatchFileMissing, "patch-file-missing", ["path"]);

impl std::fmt::Display for PatchFileMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Patch file missing: {}", self.path.display())
    }
}

/// Problem representing unexpected local changes in the source package.
///
/// This struct is used when dpkg-source detects unexpected local changes
/// to upstream source code, which should be represented as patches instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgSourceLocalChanges {
    /// Path to the diff file showing the changes, if available.
    pub diff_file: Option<String>,
    /// List of files that have been changed locally, if available.
    pub files: Option<Vec<String>>,
}

crate::problem!(
    DpkgSourceLocalChanges,
    "unexpected-local-upstream-changes",
    ["diff_file", "files"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgSourceUnrepresentableChanges;

crate::problem!(
    DpkgSourceUnrepresentableChanges,
    "unrepresentable-local-changes",
    []
);

impl std::fmt::Display for DpkgSourceUnrepresentableChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unrepresentable changes")
    }
}

/// Problem representing unwanted binary files in the source package.
///
/// This struct is used when dpkg-source detects binary files in the source
/// package that are not allowed, which can happen when the source is dirty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgUnwantedBinaryFiles;

crate::problem!(DpkgUnwantedBinaryFiles, "unwanted-binary-files", []);

impl std::fmt::Display for DpkgUnwantedBinaryFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unwanted binary files")
    }
}

/// Problem representing changes to binary files in the source package.
///
/// This struct is used when dpkg-source detects that binary files have been
/// changed, which cannot be properly represented in source package formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgBinaryFileChanged {
    /// The binary files that changed.
    pub files: Vec<String>,
}

crate::problem!(DpkgBinaryFileChanged, "binary-file-changed", ["files"]);

impl std::fmt::Display for DpkgBinaryFileChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binary file changed")
    }
}

/// Problem representing a missing debian control file.
///
/// This struct is used when the debian/control file, which is required for
/// any Debian package, is missing from the source package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingControlFile {
    /// The path to the missing control file.
    pub path: std::path::PathBuf,
}

crate::problem!(
    MissingControlFile,
    "missing-control-file",
    [],
    json = |_| serde_json::Value::Null
);

impl std::fmt::Display for MissingControlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing control file: {}", self.path.display())
    }
}

/// Problem representing unknown Mercurial extra fields.
///
/// This struct is used when the build process encounters unknown extra fields
/// in Mercurial version control metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownMercurialExtraFields {
    /// The unknown extra field.
    pub field: String,
}

crate::problem!(
    UnknownMercurialExtraFields,
    "unknown-mercurial-extra-fields",
    ["field"]
);

impl std::fmt::Display for UnknownMercurialExtraFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown Mercurial extra field: {}", self.field)
    }
}

/// Problem representing a failure to verify an upstream PGP signature.
///
/// This struct is used when the build process cannot verify the PGP signature
/// of an upstream source tarball, which may indicate a security issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamPGPSignatureVerificationFailed;

crate::problem!(
    UpstreamPGPSignatureVerificationFailed,
    "upstream-pgp-signature-verification-failed",
    []
);

impl std::fmt::Display for UpstreamPGPSignatureVerificationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream PGP signature verification failed")
    }
}

/// Problem representing a missing requested version in uscan.
///
/// This struct is used when the uscan tool (which checks for upstream versions)
/// cannot find a specifically requested version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanRequestVersionMissing {
    /// The requested version.
    pub version: String,
}

crate::problem!(
    UScanRequestVersionMissing,
    "uscan-request-version-missing",
    ["version"]
);

impl std::fmt::Display for UScanRequestVersionMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan request version missing: {}", self.version)
    }
}

/// Problem representing a failure in the debcargo tool.
///
/// This struct is used when the debcargo tool, which is used to package
/// Rust crates as Debian packages, encounters an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebcargoFailure {
    /// The reason for the failure.
    pub reason: String,
}

crate::problem!(DebcargoFailure, "debcargo-failure", ["reason"]);

impl std::fmt::Display for DebcargoFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debcargo failure: {}", self.reason)
    }
}

/// Problem representing an error parsing a debian/changelog file.
///
/// This struct is used when the build process encounters a syntax error
/// or other issue when parsing the debian/changelog file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangelogParseError {
    /// The reason the changelog could not be parsed.
    pub reason: String,
}

crate::problem!(ChangelogParseError, "changelog-parse-error", ["reason"]);

impl std::fmt::Display for ChangelogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Changelog parse error: {}", self.reason)
    }
}

/// Problem representing a generic error in the uscan tool.
///
/// This struct is used when the uscan tool, which checks for upstream versions,
/// encounters an error that doesn't fit into more specific categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanError {
    /// The error reported by uscan.
    pub reason: String,
}

crate::problem!(UScanError, "uscan-error", ["reason"]);

impl std::fmt::Display for UScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan error: {}", self.reason)
    }
}

/// Problem representing a failure in the uscan tool.
///
/// This struct is used when the uscan tool fails to find or process
/// upstream versions from a specific URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanFailed {
    /// The URL that uscan was trying to process.
    pub url: String,
    /// The reason for the failure.
    pub reason: String,
}

crate::problem!(UScanFailed, "uscan-failed", ["url", "reason"]);

impl std::fmt::Display for UScanFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan failed: {}", self.reason)
    }
}

/// Problem representing inconsistency between source format and version.
///
/// This struct is used when there's an inconsistency between the source format
/// specified in debian/source/format and the version numbering scheme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InconsistentSourceFormat {
    /// Whether the version is inconsistent with the source format.
    pub version: bool,
    /// Whether the source format is inconsistent with the version.
    pub source_format: bool,
}

crate::problem!(
    InconsistentSourceFormat,
    "inconsistent-source-format",
    ["version", "source_format"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamMetadataFileParseError {
    /// The path to the metadata file.
    pub path: std::path::PathBuf,
    /// The reason for the parsing failure.
    pub reason: String,
}

crate::problem!(
    UpstreamMetadataFileParseError,
    "debian-upstream-metadata-invalid",
    ["path", "reason"]
);

impl std::fmt::Display for UpstreamMetadataFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream metadata file parse error: {}", self.reason)
    }
}

/// Problem representing a failure in dpkg-source when packaging source files.
///
/// This struct is used when dpkg-source cannot package the source files
/// into a source package for various reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgSourcePackFailed {
    /// The reason packing the source failed.
    pub reason: String,
}

crate::problem!(DpkgSourcePackFailed, "dpkg-source-pack-failed", ["reason"]);

impl std::fmt::Display for DpkgSourcePackFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dpkg source pack failed: {}", self.reason)
    }
}

/// Problem representing an invalid version string in a package.
///
/// This struct is used when dpkg encounters a version string that
/// doesn't follow the Debian version format rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpkgBadVersion {
    /// The invalid version string.
    pub version: String,
    /// The reason why the version is invalid, if available.
    pub reason: Option<String>,
}

crate::problem!(DpkgBadVersion, "dpkg-bad-version", ["version", "reason"]);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingDebcargoCrate {
    /// The name of the missing Rust crate.
    #[serde(rename = "crate")]
    pub cratename: String,
    /// The version of the crate that is required, if specified.
    pub version: Option<String>,
}

crate::problem!(
    MissingDebcargoCrate,
    "debcargo-missing-crate",
    ["crate", "version"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PristineTarTreeMissing {
    /// The tree-ish that could not be found.
    pub treeish: String,
}

crate::problem!(
    PristineTarTreeMissing,
    "pristine-tar-missing-tree",
    ["treeish"]
);

impl std::fmt::Display for PristineTarTreeMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pristine-tar tree missing: {}", self.treeish)
    }
}

/// Problem representing a missing revision in version control.
///
/// This struct is used when a build process references a revision
/// in version control that does not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingRevision {
    /// The missing revision identifier.
    #[serde(with = "lossy_utf8")]
    pub revision: Vec<u8>,
}

crate::problem!(MissingRevision, "missing-revision", ["revision"]);

mod lossy_utf8 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&String::from_utf8_lossy(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        Ok(String::deserialize(d)?.into_bytes())
    }
}

impl std::fmt::Display for MissingRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Missing revision: {}",
            String::from_utf8_lossy(&self.revision)
        )
    }
}

/// Problem representing a Rust crate dependency predicate that debcargo cannot handle.
///
/// This struct is used when debcargo cannot represent a predicate in a Rust
/// crate dependency, such as certain prerelease version constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebcargoUnacceptablePredicate {
    /// The name of the crate with the unacceptable predicate.
    #[serde(rename = "crate")]
    pub cratename: String,
    /// The predicate that cannot be represented.
    pub predicate: String,
}

crate::problem!(
    DebcargoUnacceptablePredicate,
    "debcargo-unacceptable-predicate",
    ["crate", "predicate"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebcargoUnacceptableComparator {
    /// The name of the crate with the unacceptable comparator.
    #[serde(rename = "crate")]
    pub cratename: String,
    /// The comparator that cannot be represented.
    pub comparator: String,
}

crate::problem!(
    DebcargoUnacceptableComparator,
    "debcargo-unacceptable-comparator",
    ["crate", "comparator"]
);

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanTooManyRequests {
    /// The error reported by uscan.
    pub reason: String,
}

crate::problem!(UScanTooManyRequests, "uscan-too-many-requests", ["reason"]);

impl std::fmt::Display for UScanTooManyRequests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan too many requests: {}", self.reason)
    }
}

/// Problem representing unsatisfied conflicts in apt dependencies.
///
/// This struct is used when apt cannot resolve package conflicts
/// during the dependency resolution process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsatisfiedAptConflicts {
    /// The conflicting relations.
    pub relations: String,
}

crate::problem!(
    UnsatisfiedAptConflicts,
    "unsatisfied-apt-conflicts",
    ["relations"]
);

impl std::fmt::Display for UnsatisfiedAptConflicts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt conflicts: {}", self.relations)
    }
}

impl std::error::Error for UnsatisfiedAptConflicts {}

/// Problem representing an architecture not in the supported architecture list.
///
/// This struct is used when a build is attempted for an architecture that
/// is not in the list of architectures supported by the package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureNotInList {
    /// The architecture being built for.
    pub arch: String,
    /// The list of supported architectures.
    pub arch_list: Vec<String>,
}

crate::problem!(
    ArchitectureNotInList,
    "arch-not-in-list",
    ["arch", "arch_list"]
);

impl std::fmt::Display for ArchitectureNotInList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Architecture {} not a build arch", self.arch)
    }
}

/// Problem representing unsatisfied dependencies in apt.
///
/// This struct is used when apt cannot satisfy the dependencies
/// required for a package installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsatisfiedAptDependencies {
    /// The unsatisfied relations.
    pub relations: String,
}

crate::problem!(
    UnsatisfiedAptDependencies,
    "unsatisfied-apt-dependencies",
    ["relations"]
);

impl std::fmt::Display for UnsatisfiedAptDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt dependencies: {}", self.relations)
    }
}

/// Problem representing insufficient disk space for a build.
///
/// This struct is used when a build process determines that there is
/// not enough disk space available to complete the build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsufficientDiskSpace {
    /// The amount of disk space needed for the build in KiB.
    pub needed: i64,
    /// The amount of free disk space available in KiB.
    pub free: i64,
}

crate::problem!(
    InsufficientDiskSpace,
    "insufficient-disk-space",
    ["needed", "free"]
);

impl std::fmt::Display for InsufficientDiskSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient disk space for build. Need: {} KiB, free: {} KiB",
            self.needed, self.free
        )
    }
}

/// Problem representing brz failing because the workspace contains a
/// nested tree (a `bzr-revtree` containing another working tree, e.g.
/// vendored upstream source). brz emits
/// `brz: ERROR: The nested tree for <name> can not be resolved.` and
/// stops; without a matcher this surfaced as `unidentified` in
/// janitor.debian.net.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NestedTreeUnresolvable {
    /// The name of the nested tree brz couldn't resolve.
    pub name: String,
}

crate::problem!(NestedTreeUnresolvable, "nested-tree-unresolvable", ["name"]);

impl std::fmt::Display for NestedTreeUnresolvable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nested tree for {} cannot be resolved", self.name)
    }
}

/// Problem representing dulwich refusing to traverse a git submodule
/// during a brz operation. brz emits
/// `brz: ERROR: dulwich.objects.SubmoduleEncountered: (path, sha)`;
/// the package needs `--include-submodules` or the submodule needs
/// resolving manually before the run can proceed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmoduleEncountered {
    /// Path of the submodule inside the working tree.
    pub path: String,
    /// Submodule commit SHA brz reported.
    pub sha: String,
}

crate::problem!(
    SubmoduleEncountered,
    "submodule-encountered",
    ["path", "sha"]
);

impl std::fmt::Display for SubmoduleEncountered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Submodule encountered at {} ({})", self.path, self.sha)
    }
}

/// Problem representing UScan finding the upstream watchfile lists a
/// version that doesn't match the version from `debian/changelog`.
/// The brz wrapper formats this as
/// `UScan failed to run: Newest version of <pkg> on remote site is
/// <remote>, specified download version is <wanted>.`. Distinct from
/// `UScanRequestVersionMissing` (matches "no matching files for
/// version"); this fires when uscan *did* find files but for a
/// different version label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanRemoteVersionMismatch {
    /// Source package name uscan reported.
    pub package: String,
    /// Newest version on the upstream watchfile target.
    pub remote_version: String,
    /// Version brz/uscan was asked to download (from changelog).
    pub wanted_version: String,
}

crate::problem!(
    UScanRemoteVersionMismatch,
    "uscan-remote-version-mismatch",
    ["package", "remote_version", "wanted_version"]
);

impl std::fmt::Display for UScanRemoteVersionMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "uscan: {} latest is {}, wanted {}",
            self.package, self.remote_version, self.wanted_version
        )
    }
}

/// Problem representing UScan refusing to parse a malformed paragraph
/// in `debian/watch`. brz emits
/// `UScan failed to run: The following paragraph isn't well formatted,
/// skipping it: <text>`; surface it explicitly so the failure is
/// classified as a watchfile-format issue (actionable: fix
/// debian/watch) rather than `unidentified`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UScanWatchfileMalformed {
    /// The raw paragraph text uscan rejected (may be multi-line).
    pub paragraph: String,
}

crate::problem!(
    UScanWatchfileMalformed,
    "uscan-watchfile-malformed",
    ["paragraph"]
);

impl std::fmt::Display for UScanWatchfileMalformed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uscan rejected malformed debian/watch paragraph")
    }
}

/// Problem representing brz crashing with an uncaught exception
/// (typically a Python AssertionError or similar deep stack
/// trace). The brz crash banner ("Breezy has encountered an internal
/// error") is what we want to surface; treating these as
/// `unidentified` masks real upstream brz bugs that need filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrzInternalError {
    /// The exception type (e.g. `AssertionError`, `AttributeError`).
    pub exception_type: String,
    /// The exception message; may be empty (assertion errors often are).
    pub exception_message: String,
}

crate::problem!(
    BrzInternalError,
    "brz-internal-error",
    ["exception_type", "exception_message"]
);

impl std::fmt::Display for BrzInternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.exception_message.is_empty() {
            write!(f, "brz internal error: {}", self.exception_type)
        } else {
            write!(
                f,
                "brz internal error: {}: {}",
                self.exception_type, self.exception_message
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::assert_wire_format;
    use crate::Problem;
    use serde_json::json;
    use std::path::PathBuf;

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
        let problem = UScanTooManyRequests {
            reason: "rate limit exceeded".to_string(),
        };
        let json = problem.json();
        assert_eq!(json["reason"], "rate limit exceeded");
    }

    #[test]
    fn wire_format() {
        assert_wire_format(
            &DpkgError {
                msg: "f0-value".to_string(),
            },
            "dpkg-error",
            json!({"msg": "f0-value"}),
            "dpkg error: f0-value",
        );
        assert_wire_format(
            &AptUpdateError,
            "apt-update-error",
            json!({}),
            "apt update error",
        );
        assert_wire_format(
            &AptFetchFailure {
                url: Some("url-value".to_string()),
                error: "error-value".to_string(),
            },
            "apt-file-fetch-failure",
            json!({"error": "error-value", "url": "url-value"}),
            "apt fetch failure: url-value (error-value)",
        );
        assert_wire_format(
            &AptFetchFailure {
                url: None,
                error: "error-value".to_string(),
            },
            "apt-file-fetch-failure",
            json!({"error": "error-value", "url": null}),
            "apt fetch failure: error-value",
        );
        assert_wire_format(
            &AptMissingReleaseFile {
                url: "f0-value".to_string(),
            },
            "missing-release-file",
            json!({"url": "f0-value"}),
            "apt missing release file: f0-value",
        );
        assert_wire_format(
            &AptPackageUnknown {
                package: "f0-value".to_string(),
            },
            "apt-package-unknown",
            json!({"package": "f0-value"}),
            "apt package unknown: f0-value",
        );
        assert_wire_format(
            &AptBrokenPackages {
                description: "description-value".to_string(),
                broken: Some(vec!["broken-a".to_string(), "broken-b".to_string()]),
            },
            "apt-broken-packages",
            json!({"broken": ["broken-a", "broken-b"], "description": "description-value"}),
            "apt broken packages: description-value",
        );
        assert_wire_format(
            &AptBrokenPackages {
                description: "description-value".to_string(),
                broken: None,
            },
            "apt-broken-packages",
            json!({"broken": null, "description": "description-value"}),
            "apt broken packages: description-value",
        );
        assert_wire_format(
            &UnableToFindUpstreamTarball {
                package: "package-value".to_string(),
                version: "1.2-3".parse().unwrap(),
            },
            "unable-to-find-upstream-tarball",
            json!({"package": "package-value", "version": "1.2-3"}),
            "Unable to find upstream tarball for package-value 1.2-3",
        );
        assert_wire_format(
            &SourceFormatUnbuildable {
                source_format: "source_format-value".to_string(),
                reason: "reason-value".to_string(),
            },
            "source-format-unbuildable",
            json!({"reason": "reason-value", "source_format": "source_format-value"}),
            "Source format source_format-value is unbuildable: reason-value",
        );
        assert_wire_format(
            &SourceFormatUnsupported {
                source_format: "f0-value".to_string(),
            },
            "source-format-unsupported",
            json!({"source_format": "f0-value"}),
            "Source format f0-value is unsupported",
        );
        assert_wire_format(
            &PatchFileMissing {
                path: PathBuf::from("/srv/f0"),
            },
            "patch-file-missing",
            json!({"path": "/srv/f0"}),
            "Patch file missing: /srv/f0",
        );
        assert_wire_format(
            &DpkgSourceLocalChanges {
                diff_file: Some("diff_file-value".to_string()),
                files: Some(vec!["files-a".to_string(), "files-b".to_string()]),
            },
            "unexpected-local-upstream-changes",
            json!({"diff_file": "diff_file-value", "files": ["files-a", "files-b"]}),
            "Tree has local changes: [\"files-a\", \"files-b\"]",
        );
        assert_wire_format(
            &DpkgSourceLocalChanges {
                diff_file: None,
                files: None,
            },
            "unexpected-local-upstream-changes",
            json!({"diff_file": null, "files": null}),
            "Tree has local changes",
        );
        assert_wire_format(
            &DpkgSourceUnrepresentableChanges,
            "unrepresentable-local-changes",
            json!(null),
            "Tree has unrepresentable changes",
        );
        assert_wire_format(
            &DpkgUnwantedBinaryFiles,
            "unwanted-binary-files",
            json!(null),
            "Tree has unwanted binary files",
        );
        assert_wire_format(
            &DpkgBinaryFileChanged {
                files: vec!["f0-a".to_string(), "f0-b".to_string()],
            },
            "binary-file-changed",
            json!({"files": ["f0-a", "f0-b"]}),
            "Binary file changed",
        );
        assert_wire_format(
            &MissingControlFile {
                path: PathBuf::from("/srv/f0"),
            },
            "missing-control-file",
            json!(null),
            "Missing control file: /srv/f0",
        );
        assert_wire_format(
            &UnknownMercurialExtraFields {
                field: "f0-value".to_string(),
            },
            "unknown-mercurial-extra-fields",
            json!({"field": "f0-value"}),
            "Unknown Mercurial extra field: f0-value",
        );
        assert_wire_format(
            &UpstreamPGPSignatureVerificationFailed,
            "upstream-pgp-signature-verification-failed",
            json!(null),
            "Upstream PGP signature verification failed",
        );
        assert_wire_format(
            &UScanRequestVersionMissing {
                version: "f0-value".to_string(),
            },
            "uscan-request-version-missing",
            json!({"version": "f0-value"}),
            "UScan request version missing: f0-value",
        );
        assert_wire_format(
            &DebcargoFailure {
                reason: "f0-value".to_string(),
            },
            "debcargo-failure",
            json!({"reason": "f0-value"}),
            "Debcargo failure: f0-value",
        );
        assert_wire_format(
            &ChangelogParseError {
                reason: "f0-value".to_string(),
            },
            "changelog-parse-error",
            json!({"reason": "f0-value"}),
            "Changelog parse error: f0-value",
        );
        assert_wire_format(
            &UScanError {
                reason: "f0-value".to_string(),
            },
            "uscan-error",
            json!({"reason": "f0-value"}),
            "UScan error: f0-value",
        );
        assert_wire_format(
            &UScanFailed {
                url: "url-value".to_string(),
                reason: "reason-value".to_string(),
            },
            "uscan-failed",
            json!({"reason": "reason-value", "url": "url-value"}),
            "UScan failed: reason-value",
        );
        assert_wire_format(
            &InconsistentSourceFormat {
                version: true,
                source_format: true,
            },
            "inconsistent-source-format",
            json!({"source_format": true, "version": true}),
            "Inconsistent source format between version and source format",
        );
        assert_wire_format(
            &UpstreamMetadataFileParseError {
                path: PathBuf::from("/srv/path"),
                reason: "reason-value".to_string(),
            },
            "debian-upstream-metadata-invalid",
            json!({"path": "/srv/path", "reason": "reason-value"}),
            "Upstream metadata file parse error: reason-value",
        );
        assert_wire_format(
            &DpkgSourcePackFailed {
                reason: "f0-value".to_string(),
            },
            "dpkg-source-pack-failed",
            json!({"reason": "f0-value"}),
            "Dpkg source pack failed: f0-value",
        );
        assert_wire_format(
            &DpkgBadVersion {
                version: "version-value".to_string(),
                reason: Some("reason-value".to_string()),
            },
            "dpkg-bad-version",
            json!({"reason": "reason-value", "version": "version-value"}),
            "Version version-value is invalid: reason-value",
        );
        assert_wire_format(
            &DpkgBadVersion {
                version: "version-value".to_string(),
                reason: None,
            },
            "dpkg-bad-version",
            json!({"reason": null, "version": "version-value"}),
            "Version version-value is invalid",
        );
        assert_wire_format(
            &MissingDebcargoCrate {
                cratename: "cratename-value".to_string(),
                version: Some("version-value".to_string()),
            },
            "debcargo-missing-crate",
            json!({"crate": "cratename-value", "version": "version-value"}),
            "debcargo can't find crate cratename-value (version: version-value)",
        );
        assert_wire_format(
            &MissingDebcargoCrate {
                cratename: "cratename-value".to_string(),
                version: None,
            },
            "debcargo-missing-crate",
            json!({"crate": "cratename-value", "version": null}),
            "debcargo can't find crate cratename-value",
        );
        assert_wire_format(
            &PristineTarTreeMissing {
                treeish: "f0-value".to_string(),
            },
            "pristine-tar-missing-tree",
            json!({"treeish": "f0-value"}),
            "Pristine-tar tree missing: f0-value",
        );
        assert_wire_format(
            &MissingRevision {
                revision: b"deadbeef".to_vec(),
            },
            "missing-revision",
            json!({"revision": "deadbeef"}),
            "Missing revision: deadbeef",
        );
        assert_wire_format(
            &DebcargoUnacceptablePredicate {
                cratename: "cratename-value".to_string(),
                predicate: "predicate-value".to_string(),
            },
            "debcargo-unacceptable-predicate",
            json!({"crate": "cratename-value", "predicate": "predicate-value"}),
            "Cannot represent prerelease part of dependency: predicate-value",
        );
        assert_wire_format(
            &DebcargoUnacceptableComparator {
                cratename: "cratename-value".to_string(),
                comparator: "comparator-value".to_string(),
            },
            "debcargo-unacceptable-comparator",
            json!({"comparator": "comparator-value", "crate": "cratename-value"}),
            "Cannot represent prerelease part of dependency: comparator-value",
        );
        assert_wire_format(
            &UScanTooManyRequests {
                reason: "f0-value".to_string(),
            },
            "uscan-too-many-requests",
            json!({"reason": "f0-value"}),
            "UScan too many requests: f0-value",
        );
        assert_wire_format(
            &UnsatisfiedAptConflicts {
                relations: "f0-value".to_string(),
            },
            "unsatisfied-apt-conflicts",
            json!({"relations": "f0-value"}),
            "unsatisfied apt conflicts: f0-value",
        );
        assert_wire_format(
            &ArchitectureNotInList {
                arch: "arch-value".to_string(),
                arch_list: vec!["arch_list-a".to_string(), "arch_list-b".to_string()],
            },
            "arch-not-in-list",
            json!({"arch": "arch-value", "arch_list": ["arch_list-a", "arch_list-b"]}),
            "Architecture arch-value not a build arch",
        );
        assert_wire_format(
            &UnsatisfiedAptDependencies {
                relations: "f0-value".to_string(),
            },
            "unsatisfied-apt-dependencies",
            json!({"relations": "f0-value"}),
            "unsatisfied apt dependencies: f0-value",
        );
        assert_wire_format(
            &InsufficientDiskSpace {
                needed: 42,
                free: 42,
            },
            "insufficient-disk-space",
            json!({"free": 42, "needed": 42}),
            "Insufficient disk space for build. Need: 42 KiB, free: 42 KiB",
        );
        assert_wire_format(
            &NestedTreeUnresolvable {
                name: "name-value".to_string(),
            },
            "nested-tree-unresolvable",
            json!({"name": "name-value"}),
            "Nested tree for name-value cannot be resolved",
        );
        assert_wire_format(
            &SubmoduleEncountered {
                path: "path-value".to_string(),
                sha: "sha-value".to_string(),
            },
            "submodule-encountered",
            json!({"path": "path-value", "sha": "sha-value"}),
            "Submodule encountered at path-value (sha-value)",
        );
        assert_wire_format(
            &UScanRemoteVersionMismatch {
                package: "package-value".to_string(),
                remote_version: "remote_version-value".to_string(),
                wanted_version: "wanted_version-value".to_string(),
            },
            "uscan-remote-version-mismatch",
            json!({
                "package": "package-value",
                "remote_version": "remote_version-value",
                "wanted_version": "wanted_version-value",
            }),
            "uscan: package-value latest is remote_version-value, wanted wanted_version-value",
        );
        assert_wire_format(
            &UScanWatchfileMalformed {
                paragraph: "paragraph-value".to_string(),
            },
            "uscan-watchfile-malformed",
            json!({"paragraph": "paragraph-value"}),
            "uscan rejected malformed debian/watch paragraph",
        );
        assert_wire_format(
            &BrzInternalError {
                exception_type: "exception_type-value".to_string(),
                exception_message: "exception_message-value".to_string(),
            },
            "brz-internal-error",
            json!({
                "exception_message": "exception_message-value",
                "exception_type": "exception_type-value",
            }),
            "brz internal error: exception_type-value: exception_message-value",
        );
    }
}
