pub mod versions {
    pub mod latest;
}

use semver::Version;
use serde::Deserialize;
use toml::Value;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VersionSpec {
    version: Option<Version>,
    spec: Value,
}
