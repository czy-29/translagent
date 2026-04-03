use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Spec {
    defaults: Option<Defaults>,
    runner: Option<Runner>,
    sites: Vec<Site>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Defaults {
    deploy_target: Option<DeployTarget>,
    llm_provider: Option<LLMProvider>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DeployTarget {
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum LLMProvider {
    Deepseek,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Runner {
    exec_env: Option<ExecEnv>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ExecEnv {
    GithubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Site {
    meta: Meta,
    source: Source,
    target: Target,
    framework: Framework,
    translate: Option<Translate>,
    deploy: Option<Deploy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Meta {
    name: String, // 限制ASCII
    desc: Option<String>,
    // entry: Option<String>, // 考虑用URL安全的类型
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
pub struct Deploy {}
