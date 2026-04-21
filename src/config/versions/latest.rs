use derive_more::{Display, FromStr};
use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::{IndexMap, IndexSet, indexset};
use serde::Deserialize;
use serde_with::{DeserializeFromStr, MapPreventDuplicates, SetPreventDuplicates, serde_as};
use smart_default::SmartDefault;
use std::str::FromStr;
use types::Subdir;
use url::Url;

pub mod types {
    use super::*;
    use relative_path::{Component, FromPathError, RelativePathBuf};
    use snafu::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, Default, Display)]
    pub struct Subdir(RelativePathBuf);

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

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct Spec {
    defaults: Defaults,
    runner: Runner,
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    sites: IndexMap<SiteKey, SiteValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default)]
pub struct Defaults {
    source: SourceDefaults,
    target: TargetDefaults,
    translate: TranslateDefaults,
    deploy: DeployDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, SmartDefault)]
#[serde(default)]
pub struct SourceDefaults {
    #[default(Lang::En)]
    lang: Lang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeserializeFromStr, Display, FromStr)]
pub enum Lang {
    En,
    Zh,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, SmartDefault)]
#[serde(default)]
pub struct TargetDefaults {
    #[serde_as(as = "SetPreventDuplicates<_>")]
    #[default(default_target_langs())]
    langs: IndexSet<Lang>,
    #[default(true)]
    use_github_token: bool,
}

fn default_target_langs() -> IndexSet<Lang> {
    indexset! { Lang::Zh }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct TranslateDefaults {
    provider: Provider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum Provider {
    #[default]
    Deepseek,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct DeployDefaults {
    target: DeployTarget,
    source_lang: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum DeployTarget {
    #[default]
    Target,
}

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
        Ok(Self(Label::from_ascii(s)?.to_lowercase()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
    #[serde(default)]
    dir: Subdir,
    lang: Option<Lang>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Target {
    git: Url,
    #[serde(default)]
    dir: Subdir,
    #[serde_as(as = "Option<SetPreventDuplicates<_>>")]
    langs: Option<IndexSet<Lang>>,
    use_github_token: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Framework {
    preset: Preset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Preset {
    Zola,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, SmartDefault)]
#[serde(default)]
pub struct Translate {
    #[serde_as(as = "SetPreventDuplicates<_>")]
    #[default(default_translate_exts())]
    exts: IndexSet<String>,
    provider: Option<Provider>,
}

fn default_translate_exts() -> IndexSet<String> {
    indexset! { "md".into() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default)]
pub struct Deploy {
    target: Option<DeployTarget>,
    source_lang: Option<bool>,
}
