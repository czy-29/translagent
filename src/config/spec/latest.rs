use derive_more::{Display, FromStr};
use getset::Getters;
use hickory_proto::{ProtoError, rr::domain::Label};
use indexmap::{IndexMap, IndexSet, indexset};
use serde::Deserialize;
use serde_with::{DeserializeFromStr, MapPreventDuplicates, SetPreventDuplicates, serde_as};
use smart_default::SmartDefault;
use snafu::prelude::*;
use std::str::FromStr;
use toml::{Value, de::Error as TomlError};
use types::Subdir;
use url::Url;

#[cfg(test)]
mod tests;

pub mod types {
    use super::*;
    use relative_path::{Component, FromPathError, RelativePathBuf};

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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default, Getters)]
#[serde(default, deny_unknown_fields)]
#[getset(get = "pub")]
pub struct Spec {
    defaults: Defaults,
    runner: Runner,

    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    sites: IndexMap<SiteKey, SiteValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Snafu)]
pub enum SpecError {
    #[snafu(transparent)]
    TomlError { source: TomlError },

    #[snafu(display("`spec.defaults.target.langs` is empty"))]
    DefaultTargetLangsEmpty,

    #[snafu(display(
        "`spec.defaults.target.langs` is not allowed to contain `spec.defaults.source.lang`: `{src_lang}`"
    ))]
    DefaultTargetLangsContainsSource { src_lang: Lang },

    #[snafu(display("`spec.sites.{key}.target.langs` is empty"))]
    SiteTargetLangsEmpty { key: SiteKey },

    #[snafu(display(
        "`spec.sites.{key}.target.langs` is not allowed to contain `spec.sites.{key}.source.lang`: `{src_lang}`"
    ))]
    SiteTargetLangsContainsSource { key: SiteKey, src_lang: Lang },

    #[snafu(display("`spec.sites.{key}.translate.exts` is empty"))]
    SiteTranslateExtsEmpty { key: SiteKey },

    #[snafu(display("`spec.sites.{key}.translate.exts` contains empty extension"))]
    SiteTranslateExtsContainsEmptyExt { key: SiteKey },

    #[snafu(display("SiteKey `{key}` already exists"))]
    SiteKeyAlreadyExists { key: SiteKey },
}

impl Spec {
    pub fn resolve(value: Value) -> Result<Self, SpecError> {
        let mut spec: Self = value.try_into()?;
        let defaults = spec.defaults().clone();

        Self::ensure_defaults(&defaults)?;

        for (key, value) in spec.sites.iter_mut() {
            Self::resolve_site_value(&defaults, key, value)?;
        }

        Ok(spec)
    }

    pub fn new(defaults: Defaults, runner: Runner) -> Result<Self, SpecError> {
        Self::ensure_defaults(&defaults)?;
        let sites = Default::default();
        Ok(Self {
            defaults,
            runner,
            sites,
        })
    }

    pub fn add_site(&mut self, key: SiteKey, mut value: SiteValue) -> Result<(), SpecError> {
        ensure!(
            !self.sites().contains_key(&key),
            SiteKeyAlreadyExistsSnafu { key }
        );

        Self::resolve_site_value(self.defaults(), &key, &mut value)?;
        self.sites.insert(key, value);

        Ok(())
    }

    fn ensure_defaults(defaults: &Defaults) -> Result<(), SpecError> {
        let src_lang = defaults.source.lang;
        let tar_langs = &defaults.target.langs;

        ensure!(!tar_langs.is_empty(), DefaultTargetLangsEmptySnafu);
        ensure!(
            !tar_langs.contains(&src_lang),
            DefaultTargetLangsContainsSourceSnafu { src_lang }
        );

        Ok(())
    }

    fn resolve_site_value(
        defaults: &Defaults,
        key: &SiteKey,
        value: &mut SiteValue,
    ) -> Result<(), SpecError> {
        let key = key.clone();
        let src_lang = defaults.source.lang;
        let tar_langs = defaults.target.langs.clone();
        let target_use_github_token = defaults.target.use_github_token;
        let translate_model = defaults.translate.model;
        let translate_thinking = defaults.translate.thinking;
        let deploy_target = defaults.deploy.target;
        let deploy_src_lang = defaults.deploy.source_lang;

        let src_lang = *value.source.lang.get_or_insert(src_lang);
        let tar_langs = value.target.langs.get_or_insert(tar_langs);

        ensure!(!tar_langs.is_empty(), SiteTargetLangsEmptySnafu { key });
        ensure!(
            !tar_langs.contains(&src_lang),
            SiteTargetLangsContainsSourceSnafu { key, src_lang }
        );
        ensure!(
            !value.translate.exts.is_empty(),
            SiteTranslateExtsEmptySnafu { key }
        );
        ensure!(
            !value.translate.exts.contains(""),
            SiteTranslateExtsContainsEmptyExtSnafu { key }
        );

        value
            .target
            .use_github_token
            .get_or_insert(target_use_github_token);
        value.translate.model.get_or_insert(translate_model);
        value.translate.thinking.get_or_insert(translate_thinking);
        value.deploy.target.get_or_insert(deploy_target);
        value.deploy.source_lang.get_or_insert(deploy_src_lang);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Defaults {
    pub source: SourceDefaults,
    pub target: TargetDefaults,
    pub translate: TranslateDefaults,
    pub deploy: DeployDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, SmartDefault)]
#[serde(default, deny_unknown_fields)]
pub struct SourceDefaults {
    #[default(Lang::En)]
    pub lang: Lang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DeserializeFromStr, Display, FromStr)]
pub enum Lang {
    En,
    Zh,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, SmartDefault)]
#[serde(default, deny_unknown_fields)]
pub struct TargetDefaults {
    #[serde_as(as = "SetPreventDuplicates<_>")]
    #[default(default_target_langs())]
    pub langs: IndexSet<Lang>,

    #[default(true)]
    pub use_github_token: bool,
}

fn default_target_langs() -> IndexSet<Lang> {
    indexset! { Lang::Zh }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TranslateDefaults {
    pub model: Model,
    pub thinking: Thinking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum Model {
    #[default]
    DeepseekV4Pro,
    DeepseekV4Flash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum Thinking {
    #[default]
    Max,
    High,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DeployDefaults {
    pub target: DeployTarget,
    pub source_lang: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum DeployTarget {
    #[default]
    Target,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Runner {
    pub exec_env: ExecEnv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
pub enum ExecEnv {
    #[default]
    GithubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, DeserializeFromStr, Display)]
#[display("{_0:?}")]
pub struct SiteKey(Label);

#[derive(Debug, Clone, Snafu)]
pub enum SiteKeyError {
    #[snafu(transparent)]
    FromAscii { source: ProtoError },

    #[snafu(display("SiteKey does not support the use of wildcard `*`"))]
    Wildcard,
}

impl FromStr for SiteKey {
    type Err = SiteKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let label = Label::from_ascii(s)?;
        ensure!(!label.is_wildcard(), WildcardSnafu);
        Ok(Self(label.to_lowercase()))
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
        fn from_ascii_err(k: &str) {
            assert!(matches!(
                SiteKey::from_str(k),
                Err(SiteKeyError::FromAscii { source: _ })
            ));
        }

        fn wildcard_err(k: &str) {
            assert!(matches!(SiteKey::from_str(k), Err(SiteKeyError::Wildcard)));
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

        from_ascii_err("");
        from_ascii_err("test?");
        from_ascii_err("测试");
        from_ascii_err("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        wildcard_err("*");

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
#[serde(deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
pub struct Meta {
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub git: Url,

    #[serde(default)]
    pub dir: Subdir,

    pub lang: Option<Lang>,
}

impl Source {
    pub fn unwrap_lang(&self) -> Lang {
        self.lang.unwrap()
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub git: Url,

    #[serde(default)]
    pub dir: Subdir,

    #[serde_as(as = "Option<SetPreventDuplicates<_>>")]
    pub langs: Option<IndexSet<Lang>>,

    pub use_github_token: Option<bool>,
}

impl Target {
    pub fn unwrap_langs(&self) -> &IndexSet<Lang> {
        self.langs.as_ref().unwrap()
    }

    pub fn unwrap_use_github_token(&self) -> bool {
        self.use_github_token.unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Framework {
    pub preset: Preset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Preset {
    Zola,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, SmartDefault)]
#[serde(default, deny_unknown_fields)]
pub struct Translate {
    #[serde_as(as = "SetPreventDuplicates<_>")]
    #[default(default_translate_exts())]
    pub exts: IndexSet<String>,

    pub model: Option<Model>,
    pub thinking: Option<Thinking>,
}

fn default_translate_exts() -> IndexSet<String> {
    indexset! { "md".into() }
}

impl Translate {
    pub fn unwrap_model(&self) -> Model {
        self.model.unwrap()
    }

    pub fn unwrap_thinking(&self) -> Thinking {
        self.thinking.unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Deploy {
    pub target: Option<DeployTarget>,
    pub source_lang: Option<bool>,
}

impl Deploy {
    pub fn unwrap_target(&self) -> DeployTarget {
        self.target.unwrap()
    }

    pub fn unwrap_source_lang(&self) -> bool {
        self.source_lang.unwrap()
    }
}
