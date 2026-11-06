//! Shared source-compatible controls, separate from the opt-in Typed admission policy.
//!
//! These surfaces preserve Frui's permissive values and observable presentation.
//! Application code owns content, destinations, upload behavior and validation.

//!
//! Markdown rendering/editing additionally requires the `source` feature. The source
//! code/Markdown/mask/formula differences are the owner-approved G55 contract; see
//! `docs/api/source-roles.md`. No JS library or network upload is bundled.
//!
//! # Examples
//! ```
//! use leptos::prelude::*;
//! use stackpress_leptos::components::source::{SourceCodeEditor, source_formula};
//! let owner = Owner::new();
//! owner.with(|| {
//!     let value = RwSignal::new("let answer = 42;".to_owned());
//!     let _control = view! { <SourceCodeEditor id="example-code" name="code" value/> };
//! });
//! owner.cleanup();
//! assert_eq!(source_formula("{this} * (1 + {tax})", 100.0, &[("tax".into(), 0.2)]), 120.0);
//! ```

mod display;
pub use display::{
    SourceColor, SourceEmailLink, SourceList, SourceRating, SourceSpread, SourceTable,
    source_overflow,
};

/// Caller-authored classes and serialized CSS, without an additional style policy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceAppearance {
    /// Additional class names, retained verbatim as an escaped attribute.
    pub class_name: String,
    /// Native CSS declarations; source object conversion belongs to the metadata adapter.
    pub style: String,
}

/// Optional neutral styles for the source-compatible role family.
pub const SOURCE_STYLES: &str = include_str!("styles.css");

#[cfg(all(test, feature = "ssr"))]
mod tests;

mod relative;
pub use relative::source_relative;
mod small;
pub use small::{SourceSmallInput, source_small};
mod file_state;
mod files;
pub use files::{
    SourceFile, SourceFileInput, SourceFileState, SourceUpload, SourceUploadCompletion,
};
mod lists;
pub use lists::{SourceFieldset, SourceFieldsetDisplay, SourceRows, SourceTextList};

mod suggest;
pub use suggest::SourceSuggestInput;
mod formula;
pub use formula::source_formula;
mod data;
pub use data::{
    COLORS, COLORS_SHA256, COUNTRIES, COUNTRIES_SHA256, CURRENCIES, CURRENCIES_SHA256, CountryData,
    CurrencyData,
};
mod select;
pub use select::{SourceChoice, SourceSelect};
mod countries;
pub use countries::{SourceCountry, SourceCountrySelect, SourceCurrencySelect};
mod mask;
pub use mask::{SourceMaskToken, source_mask, source_mask_delete, source_mask_value};
mod code;
#[cfg(feature = "source")]
mod markdown;
mod mask_input;
pub use code::{SourceCode, SourceCodeEditor, source_code_tab};
#[cfg(feature = "source")]
pub use markdown::{SourceMarkdown, SourceMarkdownEditor, source_markdown};
pub use mask_input::SourceMaskInput;
mod color_input;
pub use color_input::{SourceColorInput, source_color_hex};
mod media;
pub use media::{SourceCarouselState, SourceImageCarousel, SourceImageFilm};

mod radio;
pub use radio::SourceRadio;
