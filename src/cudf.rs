//! Module for parsing CUDF (Common Upgradeability Description Format) files.
//!
//! CUDF is a format used for representing package dependency information,
//! particularly in the context of dependency resolution problems in package managers.
//! This module provides structures and deserialization logic for CUDF files.

use debversion::Version;
use serde::Deserialize;

/// Deserializes a string representation of a version number into a (major, minor) tuple.
///
/// This function is used with serde's custom deserializer to convert version strings
/// like "1.2" into (1, 2).
///
/// # Arguments
/// * `deserializer` - The deserializer to use
///
/// # Returns
/// A result containing either the parsed (major, minor) tuple or an error
fn deserialize_output_version<'de, D>(deserializer: D) -> Result<(u8, u8), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let (major, minor) = s
        .split_once('.')
        .ok_or(serde::de::Error::custom("invalid version string"))?;
    let major = major.parse().map_err(serde::de::Error::custom)?;
    let minor = minor.parse().map_err(serde::de::Error::custom)?;
    Ok((major, minor))
}

/// Root structure representing a CUDF document.
///
/// This structure represents the top-level of a CUDF document, containing
/// version information, architecture, and a list of reports.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cudf {
    /// The version of the CUDF output format.
    #[serde(
        rename = "output-version",
        deserialize_with = "deserialize_output_version"
    )]
    pub output_version: (u8, u8),

    /// The native architecture for the reported packages.
    #[serde(rename = "native-architecture")]
    pub native_architecture: String,

    /// A list of reports about package issues.
    pub report: Vec<Report>,
}

/// Status of a package as reported in CUDF.
///
/// This enum represents the possible status values for packages in a CUDF report.
#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    /// Indicates that a package is broken due to dependency or conflict issues.
    #[serde(rename = "broken")]
    Broken,
}

/// A report about a package in a CUDF document.
///
/// This structure represents a report about a specific package,
/// including its status and the reasons for any issues.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    /// The name of the package.
    pub package: String,

    /// The version of the package.
    pub version: Version,

    /// The architecture of the package.
    pub architecture: String,

    /// The status of the package (e.g., broken).
    pub status: Status,

    /// The reasons why the package has the given status.
    pub reasons: Vec<Reason>,
}

/// A reason for a package's status in a CUDF report.
///
/// This structure represents a reason why a package has a particular status,
/// such as missing dependencies or conflicts with other packages.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Reason {
    /// Information about missing dependencies, if applicable.
    pub missing: Option<Missing>,

    /// Information about package conflicts, if applicable.
    pub conflict: Option<Conflict>,
}

/// Information about a missing dependency.
///
/// This structure contains information about a package with a missing dependency.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Missing {
    /// The package with the missing dependency.
    pub pkg: Pkg,
}

/// Information about a package conflict.
///
/// This structure contains information about two packages that conflict with each other.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Conflict {
    /// The first package in the conflict.
    pub pkg1: Pkg,

    /// The second package in the conflict.
    pub pkg2: Pkg,
}

/// Information about a package in a CUDF report.
///
/// This structure represents a package, including information about
/// its unsatisfied dependencies or conflicts.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pkg {
    /// The name of the package.
    pub package: String,

    /// The version of the package.
    pub version: Version,

    /// The architecture of the package.
    pub architecture: String,

    /// The unsatisfied dependency, if any.
    #[serde(rename = "unsat-dependency")]
    pub unsat_dependency: Option<String>,

    /// The unsatisfied conflict, if any.
    #[serde(rename = "unsat-conflict)")]
    pub unsat_conflict: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_output_version() {
        let result: Result<(u8, u8), _> =
            deserialize_output_version(&mut serde_json::Deserializer::from_str("\"2.0\""));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (2, 0));
    }

    #[test]
    fn test_deserialize_output_version_invalid() {
        let result: Result<(u8, u8), _> =
            deserialize_output_version(&mut serde_json::Deserializer::from_str("\"invalid\""));
        assert!(result.is_err());
    }

    #[test]
    fn test_cudf_deserialization() {
        let json = json!({
            "output-version": "2.0",
            "native-architecture": "amd64",
            "report": [
                {
                    "package": "libfoo",
                    "version": "1.0-1",
                    "architecture": "amd64",
                    "status": "broken",
                    "reasons": [
                        {
                            "missing": {
                                "pkg": {
                                    "package": "libbar",
                                    "version": "2.0-1",
                                    "architecture": "amd64",
                                    "unsat-dependency": "libzlib (>= 3.0)"
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let cudf: Cudf = serde_json::from_value(json).unwrap();
        assert_eq!(cudf.output_version, (2, 0));
        assert_eq!(cudf.native_architecture, "amd64");
        assert_eq!(cudf.report.len(), 1);

        let report = &cudf.report[0];
        assert_eq!(report.package, "libfoo");
        assert_eq!(report.version.to_string(), "1.0-1");
        assert_eq!(report.architecture, "amd64");
        assert_eq!(report.status, Status::Broken);
        assert_eq!(report.reasons.len(), 1);

        let reason = &report.reasons[0];
        assert!(reason.missing.is_some());
        assert!(reason.conflict.is_none());

        let missing = reason.missing.as_ref().unwrap();
        assert_eq!(missing.pkg.package, "libbar");
        assert_eq!(missing.pkg.version.to_string(), "2.0-1");
        assert_eq!(missing.pkg.architecture, "amd64");
        assert_eq!(
            missing.pkg.unsat_dependency,
            Some("libzlib (>= 3.0)".to_string())
        );
        assert_eq!(missing.pkg.unsat_conflict, None);
    }

    #[test]
    fn test_cudf_with_conflict() {
        let json = json!({
            "output-version": "2.0",
            "native-architecture": "amd64",
            "report": [
                {
                    "package": "libfoo",
                    "version": "1.0-1",
                    "architecture": "amd64",
                    "status": "broken",
                    "reasons": [
                        {
                            "conflict": {
                                "pkg1": {
                                    "package": "libbar",
                                    "version": "2.0-1",
                                    "architecture": "amd64"
                                },
                                "pkg2": {
                                    "package": "libbaz",
                                    "version": "3.0-1",
                                    "architecture": "amd64"
                                }
                            }
                        }
                    ]
                }
            ]
        });

        let cudf: Cudf = serde_json::from_value(json).unwrap();
        assert_eq!(cudf.output_version, (2, 0));

        let report = &cudf.report[0];
        let reason = &report.reasons[0];
        assert!(reason.missing.is_none());
        assert!(reason.conflict.is_some());

        let conflict = reason.conflict.as_ref().unwrap();
        assert_eq!(conflict.pkg1.package, "libbar");
        assert_eq!(conflict.pkg1.version.to_string(), "2.0-1");
        assert_eq!(conflict.pkg2.package, "libbaz");
        assert_eq!(conflict.pkg2.version.to_string(), "3.0-1");
    }
}
