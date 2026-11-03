//! Source SQL field codec fit, including consumed Date.toString challenge inputs.
//!
//! Schema helpers are shared with the API slice. Native date adaptation accepts the
//! host's actual offset/zone label explicitly because Rust has no JS ambient timezone.

pub use stackpress_framework_schema::providers::{decrypt, encrypt, hash};
pub use stackpress_framework_schema::source::SourceCodecs;

/// Source English Date.toString with the original session error shape.
pub fn date_string(
    milliseconds: i64,
    offset_minutes: i32,
    zone_name: &str,
) -> crate::SessionResult<String> {
    stackpress_framework_schema::source::date_string(milliseconds, offset_minutes, zone_name)
        .map_err(|error| crate::SessionError(error.0))
}
