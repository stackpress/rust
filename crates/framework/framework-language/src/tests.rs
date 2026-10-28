//! Pure registry invariants without view or serialization dependencies.
use super::*;
#[test]
fn registry_aliases_and_borrowed_enumeration() {
    let registry = LocaleRegistry::new(
        vec!["fr".into(), "en".into()],
        "en".into(),
        BTreeMap::from([("fr_FR".into(), "fr".into())]),
    )
    .unwrap();
    assert_eq!(registry.canonical("fr_FR"), Some("fr"));
    assert_eq!(registry.canonical("FR"), None);
    assert_eq!(registry.locales().collect::<Vec<_>>(), ["en", "fr"]);
    assert_eq!(registry.aliases().collect::<Vec<_>>(), [("fr_FR", "fr")]);
    assert_eq!(registry.default_locale(), "en");
}
#[test]
fn invalid_registry_configuration() {
    for locales in [
        vec![],
        vec!["en".into(), "en".into()],
        vec!["en\r\n".into()],
    ] {
        assert!(LocaleRegistry::new(locales, "en".into(), BTreeMap::new()).is_err());
    }
    assert!(LocaleRegistry::new(vec!["en".into()], "fr".into(), BTreeMap::new()).is_err());
    for aliases in [
        BTreeMap::from([("en".into(), "en".into())]),
        BTreeMap::from([("fr".into(), "missing".into())]),
    ] {
        assert!(LocaleRegistry::new(vec!["en".into()], "en".into(), aliases).is_err());
    }
}
