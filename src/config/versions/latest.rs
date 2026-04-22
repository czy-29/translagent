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

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, Display)]
#[display("{_0:?}")]
pub struct SiteKey(Label);

impl FromStr for SiteKey {
    type Err = ProtoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Label::from_ascii(s)?.to_lowercase()))
    }
}

impl SiteKey {
    pub fn as_ascii(&self) -> String {
        self.to_string()
    }

    pub fn to_unicode(&self) -> String {
        self.0.to_utf8()
    }
}

#[cfg(test)]
mod site_key {
    use super::*;

    #[test]
    fn site_key() {
        fn invalid(k: &str) {
            assert!(SiteKey::from_str(k).is_err())
        }

        fn valid(k: &str) {
            assert_eq!(SiteKey::from_str(k).unwrap().to_string(), k.to_lowercase());
        }

        fn as_ascii(k: &str) {
            assert_eq!(SiteKey::from_str(k).unwrap().as_ascii(), k.to_lowercase());
        }

        fn ascii_unicode(k: &str) {
            assert_eq!(SiteKey::from_str(k).unwrap().to_unicode(), k.to_lowercase());
        }

        fn to_unicode(k: &str, unicode: &str) {
            assert_eq!(SiteKey::from_str(k).unwrap().to_unicode(), unicode);
        }

        fn eq(left: &str, right: &str) {
            assert_eq!(
                SiteKey::from_str(left).unwrap(),
                SiteKey::from_str(right).unwrap()
            );
        }

        fn hash(left: &str, right: &str) {
            let mut hash_set = std::collections::HashSet::new();
            hash_set.insert(SiteKey::from_str(left).unwrap());
            assert!(hash_set.contains(&SiteKey::from_str(right).unwrap()));
        }

        invalid("");
        invalid("test?");
        invalid("测试");
        invalid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        valid("test");
        valid("Test");
        valid("xn--0zwm56d");
        valid("xn--0zwm56");
        valid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        as_ascii("test");
        as_ascii("Test");
        as_ascii("xn--0zwm56d");
        as_ascii("xn--0zwm56");

        ascii_unicode("test");
        ascii_unicode("Test");
        ascii_unicode("xn--0zwm56");
        to_unicode("xn--0zwm56d", "测试");

        eq("test", "test");
        eq("test", "Test");
        hash("test", "test");
        hash("test", "Test");
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
