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
