//! Native document lifetime and theme bootstrap shared by consent and management adapters.
//! Templates stay with their source pages, including their distinct head/seed ordering.
use leptos::prelude::Owner;
use stackpress_framework_view::preferences::ThemePreference;
use stackpress_lib::{Error, Result};

/// Dispose all reactive state synchronously before handing owned HTML to a Send future.
/// Errors retain the same cleanup order as successful rendering; no work is detached.
/// # Errors
/// Propagate the page builder's error after disposing its reactive owner.
pub(super) fn render(build: impl FnOnce() -> Result<String>) -> Result<String> {
    let owner = Owner::new();
    let result = owner.with(build);
    owner.cleanup();
    result
}

/// Only an unset acknowledged preference selects the existing one-shot theme bootstrap.
/// # Errors
/// Propagate shared bootstrap asset/location errors without substituting a theme policy.
pub(super) fn bootstrap(theme: Option<ThemePreference>) -> Result<String> {
    if theme.is_some() {
        return Ok(String::new());
    }
    let location =
        stackpress_framework_view::theme::bootstrap_location().map_err(Error::from_source)?;
    stackpress_framework_view::theme::bootstrap_tag(&Default::default(), Some(&location), None)
        .map_err(Error::from_source)
}
