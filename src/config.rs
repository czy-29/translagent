pub mod v1_0;

use semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    version: Option<Version>,
    spec: v1_0::Spec,
}
