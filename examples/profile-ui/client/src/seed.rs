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
    let language = profile_language(&snapshot.locale).map_err(str::to_owned)?;
    let theme = match snapshot.theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => return Err("Page theme invalid. Refresh to continue.".into()),
    };
    Ok(ClientSeed {
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
        base: BasePath::default(),
        page: SearchPage {
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

#[cfg(test)]
mod tests;
