use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Spec {
    defaults: Option<Defaults>,
    runner: Option<Runner>,
    sites: IndexMap<SiteKey, SiteValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Defaults {
    translate: Option<TranslateDefaults>,
    deploy: Option<DeployDefaults>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct TranslateDefaults {
    provider: Option<Provider>,
}

pub type DeployDefaults = Deploy;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Runner {
    exec_env: Option<ExecEnv>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ExecEnv {
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
    meta: Meta,
    source: Source,
    target: Target,
    framework: Framework,
    translate: Option<Translate>,
    deploy: Option<Deploy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Meta {
    desc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Source {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Target {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Framework {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Translate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Provider {
    Deepseek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Deploy {
    target: Option<DeployTarget>,
    source_lang: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DeployTarget {
    Target,
}
