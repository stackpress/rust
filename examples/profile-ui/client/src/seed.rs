//! Natively tested wire admission, separated from thin browser/DOM acquisition.

#[cfg(any(feature = "csr", test))]
use stackpress_example_profile_ui_contracts::SearchInput;
#[cfg(any(feature = "hydrate", test))]
use stackpress_example_profile_ui_contracts::{BUILD, REVISION};
use stackpress_example_profile_ui_contracts::{
    SearchFailure, SearchOutcome, SearchPage, SearchReply, decode_search_reply,
};
use stackpress_example_profile_ui_shared::profile_language;
use stackpress_framework_ui::routing::BasePath;
use stackpress_framework_ui::{language::Language, state::Theme};
#[cfg(any(feature = "hydrate", test))]
use stackpress_framework_view::PageSnapshot;

/// Checked public page and initial presentation settings admitted before mounting.
pub(crate) struct ClientSeed {
    /// Checked page identity, used by the shared page root.
    pub(crate) page_id: String,
    /// Exact public native status, including not-found.
    pub(crate) status: stackpress_example_profile_ui_contracts::PublicStatus,
    /// Asset-declared trusted mount, supplied to both routing and fixed HTTP calls.
    pub(crate) base: BasePath,
    /// Exact native-loaded result, consumed without refetch during hydration.
    pub(crate) page: SearchPage,
    /// Explicit checked snapshot locale/map.
    pub(crate) language: Language,
    /// Explicit checked snapshot theme.
    pub(crate) theme: Theme,
}

/// Decode a bounded revision/build-checked public seed before trusting server markup.
#[cfg(any(feature = "hydrate", test))]
pub(crate) fn decode(text: &str, assets: &str) -> Result<ClientSeed, String> {
    let assets = stackpress_framework_view::PageAssets::decode(assets, BUILD)
        .map_err(|error| error.to_string())?;
    let snapshot = PageSnapshot::<SearchPage>::decode(text, REVISION, BUILD)
        .map_err(|error| error.to_string())?;
    if assets.base.strip(&snapshot.location).is_none() {
        return Err("Page location invalid. Refresh to continue.".into());
    }
    snapshot.data.input.validate().map_err(str::to_owned)?;
    snapshot.data.reply.validate().map_err(str::to_owned)?;
    stackpress_example_profile_ui_contracts::validate_page(
        snapshot.page.as_str(),
        &snapshot.status,
        &snapshot.data,
    )
    .map_err(str::to_owned)?;
    let canonical = match &snapshot.data.detail {
        Some(detail) => stackpress_example_profile_ui_contracts::routes::detail_location(
            &assets.base,
            &detail.id,
            &detail.search_prefix,
        ),
        None => stackpress_example_profile_ui_contracts::routes::search_location(
            &assets.base,
            &snapshot.data.input.id,
        ),
    }
    .map_err(|_| "Page location invalid. Refresh to continue.")?;
    if canonical.as_str() != snapshot.location {
        return Err("Page location invalid. Refresh to continue.".into());
    }
    let language = profile_language(&snapshot.locale).map_err(str::to_owned)?;
    let theme = match snapshot.theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => return Err("Page theme invalid. Refresh to continue.".into()),
    };
    Ok(ClientSeed {
        page_id: snapshot.page.as_str().into(),
        status: snapshot.status,
        base: assets.base,
        page: snapshot.data,
        language,
        theme,
    })
}

/// Create explicit empty packaged state; the first local read is a separate action.
#[cfg(any(feature = "csr", test))]
pub(crate) fn packaged() -> Result<ClientSeed, String> {
    Ok(ClientSeed {
        page_id: "profiles".into(),
        status: Default::default(),
        base: BasePath::default(),
        page: SearchPage {
            detail: None,
            input: SearchInput::new("abc".into()).map_err(str::to_owned)?,
            reply: SearchReply {
                revision: stackpress_example_profile_ui_contracts::REVISION,
                rows: Vec::new(),
                total: 0,
            },
        },
        language: profile_language("en").map_err(str::to_owned)?,
        theme: Theme::Light,
    })
}

/// Decode and validate a complete public reply before invoking the shared controller.
pub(crate) fn reply(text: &str, status: Option<u16>) -> Result<SearchReply, SearchFailure> {
    // Both HTTP and IPC use the same decoder; a valid host rejection is never discarded.
    match decode_search_reply(text, status).map_err(SearchFailure::Protocol)? {
        SearchOutcome::Success(reply) => Ok(reply),
        SearchOutcome::Rejected(error) => Err(SearchFailure::Host(error)),
    }
}

/// Admit the separate fixed native preference command DTO before mounting/publishing.
///
/// # Errors
/// Rejects size/revision/locale/theme/unknown-field drift without touching UI state.
#[cfg(any(feature = "csr", test))]
pub(crate) fn native_preferences(
    text: &str,
) -> Result<stackpress_framework_preferences::LoadedPreferences, &'static str> {
    if text.len() > 1024 {
        return Err("Preferences unavailable");
    }
    let settings: stackpress_framework_preferences::LoadedPreferences =
        serde_json::from_str(text).map_err(|_| "Preferences unavailable")?;
    if settings.revision != 1 || !matches!(settings.preferences.locale.as_str(), "en" | "fr") {
        return Err("Preferences unavailable");
    }
    Ok(settings)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod native_settings_tests {
    /// Native settings are a separate closed contract, not an extension to search wire data.
    #[test]
    fn admit_native_preferences_before_mount() {
        let valid =
            r#"{"revision":1,"preferences":{"locale":"fr","theme":"dark"},"recovered":false}"#;
        assert_eq!(
            super::native_preferences(valid).unwrap().preferences.locale,
            "fr"
        );
        for invalid in [
            "{}".into(),
            valid.replace("\"fr\"", "\"de\""),
            valid.replace("\"dark\"", "\"unknown\""),
            valid.replace("\"revision\":1", "\"revision\":2"),
            "x".repeat(1025),
        ] {
            assert!(super::native_preferences(&invalid).is_err());
        }
    }
}
