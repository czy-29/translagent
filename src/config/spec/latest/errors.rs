use super::{Lang, SiteKey};
use hickory_proto::ProtoError;
use relative_path::{FromPathError, RelativePathBuf};
use snafu::prelude::*;
use toml::de::Error as TomlError;

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SubdirError {
    #[snafu(transparent)]
    FromPath { source: FromPathError },

    #[snafu(display("normalized path `{normalized}` escapes to parent directory"))]
    EscapedToParent { normalized: RelativePathBuf },
}

#[derive(Debug, Clone, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SiteKeyError {
    #[snafu(transparent)]
    FromAscii { source: ProtoError },

    #[snafu(display("SiteKey does not support the use of wildcard `*`"))]
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Snafu)]
#[snafu(visibility(pub(crate)))]
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
