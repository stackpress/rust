//! A separate Cargo consumer of generic Leptos UI, without any application host.
//!
//! The same root receives explicit values in SSR, hydration and CSR. Its shell,
//! bootstrap and fixture identities belong here; no Stackpress page protocol is used.

#[cfg(any(
    all(feature = "ssr", feature = "hydrate"),
    all(feature = "ssr", feature = "csr"),
    all(feature = "hydrate", feature = "csr")
))]
compile_error!("independent consumer: select one render mode per build");

mod app;
#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
mod browser;
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod native;

pub use app::{IndependentRoot, IndependentRootProps};
use stackpress_leptos::{language::Language, state::Theme};
use std::collections::BTreeMap;

/// Explicit root inputs. No cookie, OS sample, database or application contract is read.
#[derive(Clone, Debug)]
pub struct Initial {
    /// One concrete presentation choice, identical in server and client markup.
    pub theme: Theme,
    /// Checked dictionary supplied by this consumer.
    pub language: Language,
    /// Native draft adopted before mounting, including edits made before hydration.
    pub draft: String,
}
impl Initial {
    /// Admit this fixture's closed public choice vocabulary and bounded draft.
    /// # Errors
    /// Rejects unknown themes/locales and drafts over 1024 UTF-8 bytes.
    pub fn new(theme: &str, locale: &str, draft: String) -> Result<Self, &'static str> {
        let theme = match theme {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => return Err("Unknown explicit theme"),
        };
        if draft.len() > 1024 {
            return Err("Draft exceeds fixture budget");
        }
        Ok(Self {
            theme,
            language: dictionary(locale)?,
            draft,
        })
    }
}

/// Construct the checked English/French dictionary without fetching or persistence.
/// # Errors
/// Only the two fixture locales are available.
pub fn dictionary(locale: &str) -> Result<Language, &'static str> {
    if !matches!(locale, "en" | "fr") {
        return Err("Unknown explicit locale");
    }
    let phrases = if locale == "fr" {
        BTreeMap::from([("Welcome".into(), "Bienvenue".into())])
    } else {
        BTreeMap::new()
    };
    Language::new(locale.into(), phrases, "{}".into())
}

/// Exact hostile Resource text shared by native and browser execution.
pub const RESOURCE_TEXT: &str =
    "Resolved </script><script>globalThis.independentPwned=1</script>&雪";
