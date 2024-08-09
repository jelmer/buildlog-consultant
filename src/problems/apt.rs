use crate::Problem;

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
