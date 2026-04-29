use super::*;

#[test]
fn resolve() {
    use toml::from_str;

    fn value_error(toml: &str) {
        assert!(from_str::<Value>(toml).is_err());
    }

    fn toml_error(toml: &str) {
        assert!(matches!(
            Spec::resolve(from_str(toml).unwrap()),
            Err(SpecError::TomlError { source: _ })
        ))
    }

    fn eq(toml: &str, spec: Spec) {
        assert_eq!(Spec::resolve(from_str(toml).unwrap()).unwrap(), spec);
    }

    value_error("fuck");
    toml_error("[fuck]");
    //toml_error("defaults.translate.provider = \"DeepseekV4Flash\"");
    toml_error(include_str!("tests/fuck.toml"));

    eq("", Default::default());
    eq(
        "",
        Spec {
            defaults: Defaults {
                source: SourceDefaults { lang: Lang::En },
                target: TargetDefaults {
                    langs: indexset! { Lang::Zh },
                    use_github_token: true,
                },
                translate: TranslateDefaults {
                    provider: Provider::DeepseekV4Pro,
                },
                deploy: DeployDefaults {
                    target: DeployTarget::Target,
                    source_lang: false,
                },
            },
            ..Default::default()
        },
    );
    eq(
        //"defaults.translate.provider = \"deepseek-v4-flash\"",
        "defaults.translate.provider = \"DeepseekV4Flash\"",
        Spec {
            defaults: Defaults {
                translate: TranslateDefaults {
                    provider: Provider::DeepseekV4Flash,
                },
                ..Default::default()
            },
            ..Default::default()
        },
    );
}

#[test]
fn add_site() {}
