//! Example-only, feature-selected owner review host; construction never opens a listener.
//! Normal examples do not depend on this crate. All review handlers require debug compilation.
#[cfg(all(
    feature = "review-preview",
    any(not(debug_assertions), stackpress_review_release)
))]
compile_error!(
    "STACKPRESS_REVIEW_PREVIEW requires a debug build; release review hosts are forbidden"
);

#[cfg(all(feature = "review-preview", debug_assertions))]
pub mod gateway;
#[cfg(all(feature = "review-preview", debug_assertions))]
pub mod options;
#[cfg(all(feature = "review-preview", debug_assertions))]
pub mod panel;
#[cfg(all(feature = "review-preview", debug_assertions))]
pub mod pipe;
#[cfg(all(feature = "review-preview", debug_assertions))]
pub mod setup;

#[cfg(all(test, feature = "review-preview", debug_assertions))]
mod tests;
