use debversion::Version;
use serde::Deserialize;

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

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cudf {
    #[serde(
        rename = "output-version",
        deserialize_with = "deserialize_output_version"
    )]
    pub output_version: (u8, u8),
    #[serde(rename = "native-architecture")]
    pub native_architecture: String,
    pub report: Vec<Report>,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    #[serde(rename = "broken")]
    Broken,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Report {
    pub package: String,
    pub version: Version,
    pub architecture: String,
    pub status: Status,
    pub reasons: Vec<Reason>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Reason {
    pub missing: Option<Missing>,
    pub conflict: Option<Conflict>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Missing {
    pub pkg: Pkg,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Conflict {
    pub pkg1: Pkg,
    pub pkg2: Pkg,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pkg {
    pub package: String,
    pub version: Version,
    pub architecture: String,
    #[serde(rename = "unsat-dependency")]
    pub unsat_dependency: Option<String>,
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
