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

    value_error("fuck");
    toml_error("[fuck]");
    toml_error(include_str!("tests/fuck.toml"));
}

#[test]
fn add_site() {}
