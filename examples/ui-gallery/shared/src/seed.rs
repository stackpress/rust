//! Pure native-tested admission before any browser root or fetch exists.
use serde::{Deserialize, Serialize};

/// Allowlisted app seed; dictionaries/settings arrive before the first render.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GallerySeed {
    /// Canonical selected locale, as decided by the host.
    pub locale: String,
    /// Checked mount shared by assets and fixed actions.
    pub base: String,
    /// Public current path/query for the preference action.
    pub location: String,
    /// Explicit initial presentation theme.
    pub theme: String,
    /// Initial owned text draft.
    pub name: String,
}

/// Validate a public preference acknowledgement before changing root state/history.
///
/// # Errors
/// Rejects oversized/malformed/unregistered or cross-mount acknowledgements.
pub fn decode_preference(
    text: &str,
    base: &str,
) -> Result<
    stackpress_framework_language::PreferenceReply,
    stackpress_framework_language::PreferenceError,
> {
    use stackpress_framework_language::{
        LocaleRegistry, LocaleSource, PreferenceError, PreferenceReply,
    };
    if text.len() > 1024 {
        return Err(PreferenceError::Limit);
    }
    let reply: PreferenceReply =
        serde_json::from_str(text).map_err(|_| PreferenceError::Request)?;
    let registry = LocaleRegistry::new(
        vec!["en".into(), "fr".into()],
        "en".into(),
        Default::default(),
    )?;
    let base =
        stackpress_framework_view::BasePath::new(base).map_err(|_| PreferenceError::Request)?;
    let selected = registry.select(&base, &reply.location, "")?;
    if selected.source != LocaleSource::Prefix || selected.locale != reply.locale {
        return Err(PreferenceError::Request);
    }
    Ok(reply)
}

/// Admit snapshot/settings and inert Resource replay before creating a reactive browser root.
///
/// # Errors
/// Rejects contract drift, unsafe mounts/settings, metadata disagreement and replay budgets.
pub fn decode_initial(
    page: &str,
    replay: &str,
) -> Result<
    (GallerySeed, stackpress_framework_ui::replay::ResourceReplay),
    stackpress_framework_view::ViewError,
> {
    use stackpress_framework_language::LocaleRegistry;
    use stackpress_framework_view::{BasePath, PageLimits, PageSnapshot, ViewError};
    let snapshot = PageSnapshot::<GallerySeed>::decode(page, 1, "s8-m2")?;
    let seed = snapshot.data;
    let base = BasePath::new(&seed.base)?;
    let registry = LocaleRegistry::new(
        vec!["en".into(), "fr".into()],
        "en".into(),
        std::collections::BTreeMap::from([("fr-FR".into(), "fr".into())]),
    )
    .map_err(|_| ViewError::InvalidSnapshot)?;
    let selected = registry
        .select(
            &base,
            &seed.location,
            &format!("stackpress-locale={}", seed.locale),
        )
        .map_err(|_| ViewError::InvalidSnapshot)?;
    if selected.locale != seed.locale
        || seed.locale != snapshot.locale
        || seed.theme != snapshot.theme
        || seed.location != snapshot.location
        || seed.name.len() > 4096
    {
        return Err(ViewError::ContractMismatch);
    }
    let replay = stackpress_framework_ui::replay::ResourceReplay::decode(
        replay,
        PageLimits::default().snapshot_bytes,
    )?;
    Ok((seed, replay))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus, ViewError};

    /// Produce one checked app envelope without trusting browser globals.
    fn page() -> PageSnapshot<GallerySeed> {
        PageSnapshot {
            revision: 1,
            build: "s8-m2".into(),
            page: PageId::try_from("gallery".to_owned()).unwrap(),
            location: "/demo/fr/gallery".into(),
            locale: "fr".into(),
            theme: "dark".into(),
            status: PublicStatus::ok(),
            data: GallerySeed {
                locale: "fr".into(),
                base: "/demo".into(),
                location: "/demo/fr/gallery".into(),
                theme: "dark".into(),
                name: "Ada".into(),
            },
        }
    }

    /// Reject contract/mount/settings drift and malformed Resource payloads before acquisition.
    #[test]
    fn seed_and_replay_admission() {
        let valid = page();
        let text = valid.script_json().unwrap();
        let (seed, replay) = decode_initial(&text, "[]").unwrap();
        assert_eq!(seed.locale, "fr");
        assert!(replay.entries().is_empty());
        assert!(decode_initial("bad", "[]").is_err());
        assert!(decode_initial(&text, "bad").is_err());
        // An old gallery asset/seed identity is rejected before creating controls.
        let mut old = page();
        old.build = "s8-m1".into();
        assert_eq!(
            decode_initial(&old.script_json().unwrap(), "[]").unwrap_err(),
            ViewError::ContractMismatch
        );
        let mut bad = page();
        bad.revision = 2;
        assert_eq!(
            decode_initial(&bad.script_json().unwrap(), "[]").unwrap_err(),
            ViewError::ContractMismatch
        );
        let mut bad = page();
        bad.data.locale = "en".into();
        assert_eq!(
            decode_initial(&bad.script_json().unwrap(), "[]").unwrap_err(),
            ViewError::ContractMismatch
        );
        let mut bad = page();
        bad.data.theme = "light".into();
        assert!(decode_initial(&bad.script_json().unwrap(), "[]").is_err());
        let mut bad = page();
        bad.data.base = "/elsewhere".into();
        assert!(decode_initial(&bad.script_json().unwrap(), "[]").is_err());
        let mut bad = page();
        bad.data.location = "//evil.invalid".into();
        assert!(decode_initial(&bad.script_json().unwrap(), "[]").is_err());
        let mut bad = page();
        bad.data.name = "x".repeat(4097);
        assert!(decode_initial(&bad.script_json().unwrap(), "[]").is_err());
    }

    /// Public acknowledgements cannot publish an outside path or conflicting locale.
    #[test]
    fn acknowledged_location_is_same_app() {
        assert_eq!(
            decode_preference(r#"{"locale":"fr","location":"/demo/fr/gallery"}"#, "/demo")
                .unwrap()
                .locale,
            "fr"
        );
        for text in [
            "bad",
            r#"{"locale":"de","location":"/demo/de/gallery"}"#,
            r#"{"locale":"fr","location":"/elsewhere/fr/gallery"}"#,
            r#"{"locale":"fr","location":"//evil.invalid"}"#,
            r#"{"locale":"en","location":"/demo/fr/gallery"}"#,
        ] {
            assert!(decode_preference(text, "/demo").is_err());
        }
        assert!(decode_preference(&"x".repeat(1025), "/demo").is_err());
    }
}
