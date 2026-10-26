//! TEMPORARY M1 compatibility shim, authorized by the coordinator.
//!
//! At lane integration replace imports with framework_ui::routing and remove
//! this file. It mirrors the checked mount/context signatures already written
//! by lane A; it is not an alternative public routing contract.

use leptos::prelude::*;
use stackpress_framework_ui::leptos;

/// Canonical trusted application mount; root is the empty string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BasePath(String);

/// Owned checked location returned by a mount join, matching lane A's accessor.
pub struct PublicLocation(String);

impl PublicLocation {
    /// Borrow this checked location for a native form, anchor or HTTP adapter.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl BasePath {
    /// Check a mount before applying it to form, operation or asset locations.
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() || value == "/" {
            return Ok(Self::default());
        }
        if value.len() > 1024
            || !value.starts_with('/')
            || value.ends_with('/')
            || value[1..].split('/').any(|part| {
                part.is_empty()
                    || matches!(part, "." | "..")
                    || !part.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
                    })
            })
        {
            return Err("Invalid application mount");
        }
        Ok(Self(value))
    }

    /// Borrow the exact canonical prefix; no trailing slash is stored.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Join one fixed application-relative path under the checked mount.
    pub fn join(&self, suffix: &str) -> Result<PublicLocation, &'static str> {
        if !suffix.starts_with('/')
            || suffix.starts_with("//")
            || suffix.contains('\\')
            || suffix.chars().any(char::is_control)
            || self.0.len() + suffix.len() > 4096
        {
            return Err("Invalid application location");
        }
        Ok(PublicLocation(format!("{}{suffix}", self.0)))
    }

    /// Strip only this exact mount on a path-segment boundary, matching lane A.
    pub fn strip<'a>(&self, location: &'a str) -> Option<&'a str> {
        let rest = location.strip_prefix(&self.0)?;
        if rest.starts_with('/') && !rest.starts_with("//") {
            Some(rest)
        } else {
            None
        }
    }
}

/// Root-local mount context, matching lane A's existing field name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteContext {
    /// Checked canonical application mount.
    pub base: BasePath,
}

/// Provide the trusted mount before evaluating shared UI descendants.
pub fn provide_route_context(base: BasePath) {
    provide_context(RouteContext { base });
}

/// Root-only fixtures retain the explicit root default until M1 integration.
pub fn use_route_context() -> RouteContext {
    use_context::<RouteContext>().unwrap_or_default()
}

#[cfg(test)]
#[path = "m1_compat/tests.rs"]
mod tests;
