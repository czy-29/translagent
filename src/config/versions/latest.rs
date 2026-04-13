use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::DeserializeFromStr;
use smart_default::SmartDefault;
use std::str::FromStr;
use url::Url;

pub mod types {
    use super::*;
    use relative_path::{Component, FromPathError, RelativePathBuf};
    use snafu::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, Default)]
    pub struct Subdir(pub RelativePathBuf);

    impl FromStr for Subdir {
        type Err = SubdirError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let normalized = RelativePathBuf::from_path(s)?.normalize();

            ensure!(
                !matches!(normalized.components().next(), Some(Component::ParentDir)),
                EscapedToParentSnafu { normalized }
            );

            Ok(Self(normalized))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Snafu)]
    pub enum SubdirError {
        #[snafu(transparent)]
        FromPath { source: FromPathError },
        #[snafu(display("normalized path `{normalized}` escapes to parent directory"))]
        EscapedToParent { normalized: RelativePathBuf },
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn subdir() {
            use relative_path::RelativePath;

            fn exact(path: &str) {
                normalized(path, path);
            }

            fn normalized(before: &str, after: &str) {
                assert_eq!(
                    Subdir::from_str(before).unwrap().0.as_relative_path(),
                    RelativePath::new(after)
                );
            }

            fn non_relative(path: &str) {
                assert!(matches!(
                    Subdir::from_str(path),
                    Err(SubdirError::FromPath { source: _ })
                ));
            }

            fn escaped(path: &str) {
                assert!(matches!(
                    Subdir::from_str(path),
                    Err(SubdirError::EscapedToParent { normalized: _ })
                ));
            }

            exact("");
            exact("test");
            exact("test1/test2");

            normalized(".", "");
            normalized("./.", "");
            normalized("./test/..", "");
            normalized("test/..", "");
            normalized("test/../.", "");
            normalized("test/./..", "");

            normalized("./test1", "test1");
            normalized("test1/.", "test1");
            normalized("././test1", "test1");
            normalized("./test1/.", "test1");
            normalized("test1/./.", "test1");
            normalized("test1/../test1", "test1");
            normalized("test1/test2/..", "test1");

            non_relative("/test");
            non_relative("C:/test");
            non_relative("C:\\test");

            escaped("..");
            escaped("./..");
            escaped("../.");
            escaped("../test");

            escaped("././..");
            escaped("./../.");
            escaped("./../..");
            escaped("./../test");

            escaped(".././.");
            escaped(".././..");
            escaped(".././test");
            escaped("../../.");
            escaped("../../..");
            escaped("../../test");
            escaped("../test/.");
            escaped("../test/..");
            escaped("../test1/test2");

            escaped("test/../..");
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
