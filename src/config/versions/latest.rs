use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use smart_default::SmartDefault;
use std::str::FromStr;
use url::Url;

pub mod types {
    use super::*;
    use relative_path::RelativePathBuf;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, Default)]
    pub struct Subdir(pub RelativePathBuf);

    impl FromStr for Subdir {
        type Err = String;

        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            todo!()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct Spec {
    pub defaults: Defaults,
    pub runner: Runner,
    pub sites: IndexMap<SiteKey, SiteValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Defaults {
    pub target: TargetDefaults,
    pub translate: TranslateDefaults,
    pub deploy: DeployDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, SmartDefault)]
#[serde(default)]
pub struct TargetDefaults {
    #[default = true]
    pub use_github_token: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct TranslateDefaults {
    pub provider: Provider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct DeployDefaults {
    pub target: DeployTarget,
    pub source_lang: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Runner {
    pub exec_env: ExecEnv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum ExecEnv {
    #[default]
    GithubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr)]
pub struct SiteKey(pub Label);

impl FromStr for SiteKey {
    type Err = ProtoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Label::from_ascii(s)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct SiteValue {
    #[serde(default)]
    pub meta: Meta,

    pub source: Source,
    pub target: Target,
    pub framework: Framework,

    #[serde(default)]
    pub translate: Translate,

    #[serde(default)]
    pub deploy: Deploy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Meta {
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Source {
    pub git: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Target {
    pub git: Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Framework {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Translate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum Provider {
    #[default]
    Deepseek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Deploy {
    pub target: Option<DeployTarget>,
    pub source_lang: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum DeployTarget {
    #[default]
    Target,
}
