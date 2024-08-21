use crate::Problem;
use debversion::Version;

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
}

impl std::fmt::Display for DpkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "dpkg error: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AptUpdateError;

impl Problem for AptUpdateError {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-update-error".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

impl std::fmt::Display for AptUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt update error")
    }
}

#[derive(Debug, Clone)]
pub struct AptFetchFailure {
    pub url: Option<String>,
    pub error: String,
}

impl Problem for AptFetchFailure {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-fetch-failure".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.url,
            "error": self.error,
        })
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

#[derive(Debug, Clone)]
pub struct AptMissingReleaseFile(pub String);

impl Problem for AptMissingReleaseFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "apt-missing-release-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "url": self.0,
        })
    }
}

impl std::fmt::Display for AptMissingReleaseFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt missing release file: {}", self.0)
    }
}

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
}

impl std::fmt::Display for AptPackageUnknown {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt package unknown: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct AptBrokenPackages {
    pub description: String,
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
}

impl std::fmt::Display for AptBrokenPackages {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "apt broken packages: {}", self.description)
    }
}

#[derive(Debug, Clone)]
pub struct UnableToFindUpstreamTarball {
    pub package: String,
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

#[derive(Debug, Clone)]
pub struct SourceFormatUnbuildable {
    pub source_format: String,
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
}

impl std::fmt::Display for SourceFormatUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source format {} is unsupported", self.0)
    }
}

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
}

impl std::fmt::Display for PatchFileMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Patch file missing: {}", self.0.display())
    }
}

#[derive(Debug, Clone)]
pub struct DpkgSourceLocalChanges {
    pub diff_file: Option<String>,
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

#[derive(Debug, Clone)]
pub struct DpkgSourceUnrepresentableChanges;

impl Problem for DpkgSourceUnrepresentableChanges {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unrepresentable-local-changes".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for DpkgSourceUnrepresentableChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unrepresentable changes")
    }
}

#[derive(Debug, Clone)]
pub struct DpkgUnwantedBinaryFiles;

impl Problem for DpkgUnwantedBinaryFiles {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unwanted-binary-files".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for DpkgUnwantedBinaryFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree has unwanted binary files")
    }
}

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
}

impl std::fmt::Display for DpkgBinaryFileChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binary file changed")
    }
}

#[derive(Debug, Clone)]
pub struct MissingControlFile(pub std::path::PathBuf);

impl Problem for MissingControlFile {
    fn kind(&self) -> std::borrow::Cow<str> {
        "missing-control-file".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for MissingControlFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing control file: {}", self.0.display())
    }
}

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
}

impl std::fmt::Display for UnknownMercurialExtraFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown Mercurial extra field: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct UpstreamPGPSignatureVerificationFailed;

impl Problem for UpstreamPGPSignatureVerificationFailed {
    fn kind(&self) -> std::borrow::Cow<str> {
        "upstream-pgp-signature-verification-failed".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

impl std::fmt::Display for UpstreamPGPSignatureVerificationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream PGP signature verification failed")
    }
}

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
}

impl std::fmt::Display for UScanRequestVersionMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan request version missing: {}", self.0)
    }
}

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
}

impl std::fmt::Display for DebcargoFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debcargo failure: {}", self.0)
    }
}

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
}

impl std::fmt::Display for ChangelogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Changelog parse error: {}", self.0)
    }
}

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
}

impl std::fmt::Display for UScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan error: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct UScanFailed {
    pub url: String,
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
}

impl std::fmt::Display for UScanFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan failed: {}", self.reason)
    }
}

#[derive(Debug, Clone)]
pub struct InconsistentSourceFormat {
    pub version: bool,
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
}

impl std::fmt::Display for InconsistentSourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Inconsistent source format between version and source format"
        )
    }
}

#[derive(Debug, Clone)]
pub struct UpstreamMetadataFileParseError {
    pub path: std::path::PathBuf,
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
}

impl std::fmt::Display for UpstreamMetadataFileParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Upstream metadata file parse error: {}", self.reason)
    }
}

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
}

impl std::fmt::Display for DpkgSourcePackFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dpkg source pack failed: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DpkgBadVersion {
    pub version: String,
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

#[derive(Debug, Clone)]
pub struct MissingDebcargoCrate {
    pub cratename: String,
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
}

impl std::fmt::Display for MissingDebcargoCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(version) = &self.version {
            write!(f, "Missing debcargo crate: {}={}", self.cratename, version)
        } else {
            write!(f, "Missing debcargo crate: {}", self.cratename)
        }
    }
}

impl MissingDebcargoCrate {
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
}

impl std::fmt::Display for PristineTarTreeMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pristine-tar tree missing: {}", self.0)
    }
}

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
}

impl std::fmt::Display for MissingRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing revision: {}", String::from_utf8_lossy(&self.0))
    }
}

#[derive(Debug)]
pub struct DebcargoUnacceptablePredicate {
    pub cratename: String,
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

#[derive(Debug)]
pub struct DebcargoUnacceptableComparator {
    pub cratename: String,
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
}

impl std::fmt::Display for UScanTooManyRequests {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UScan too many requests: {}", self.0)
    }
}

#[derive(Debug)]
pub struct UnsatisfiedAptConflicts(pub String);

impl Problem for UnsatisfiedAptConflicts {
    fn kind(&self) -> std::borrow::Cow<str> {
        "unsatisfied-apt-conflicts".into()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "relations": self.0
        })
    }
}

impl std::fmt::Display for UnsatisfiedAptConflicts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt conflicts: {}", self.0)
    }
}

impl std::error::Error for UnsatisfiedAptConflicts {}

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
}

impl std::fmt::Display for UnsatisfiedAptDependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unsatisfied apt dependencies: {}", self.0)
    }
}
