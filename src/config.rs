pub mod versions {
    pub mod latest;
}

use semver::Version;
use serde::Deserialize;
use versions::latest;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    version: Option<Version>,
    spec: latest::Spec,
}
