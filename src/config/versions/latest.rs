use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct Spec {
    defaults: Defaults,
    runner: Runner,
    sites: IndexMap<SiteKey, SiteValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Defaults {
    translate: TranslateDefaults,
    deploy: DeployDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct TranslateDefaults {
    provider: Provider,
}

pub type DeployDefaults = Deploy;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Runner {
    exec_env: ExecEnv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum ExecEnv {
    #[default]
    GithubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr)]
pub struct SiteKey(Label);

impl FromStr for SiteKey {
    type Err = ProtoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Label::from_ascii(s)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct SiteValue {
    #[serde(default)]
    meta: Meta,

    source: Source,
    target: Target,
    framework: Framework,

    #[serde(default)]
    translate: Translate,

    #[serde(default)]
    deploy: Deploy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Meta {
    desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Source {
    git: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Target {
    git: Url,
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
    target: DeployTarget,
    source_lang: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum DeployTarget {
    #[default]
    Target,
}
