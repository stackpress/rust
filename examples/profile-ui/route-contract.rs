//! Compiled public routes shared by native registration and Leptos path descriptors.

use stackpress_framework_view::{
    BasePath, PageRoute, PublicLocation, RouteCatalog, RouteSegment, ViewError,
    encode_uri_component,
};

/// The public search page; native and Leptos adapters consume these exact segments.
///
/// # Errors
/// Returns InvalidPolicy if the compiled page identity or fixed segments are invalid;
/// this is a definition error and queues no native registration.
pub fn search() -> Result<PageRoute, ViewError> {
    PageRoute::new(
        "profiles",
        &[RouteSegment::Static("ui"), RouteSegment::Static("profiles")],
    )
}

/// The public detail path, with an encoded single-segment identifier.
///
/// # Errors
/// Returns InvalidPolicy if the compiled identity/segments are invalid or parameter
/// names repeat. The current fixed definition has one unique parameter.
pub fn detail() -> Result<PageRoute, ViewError> {
    PageRoute::new(
        "profile-detail",
        &[
            RouteSegment::Static("ui"),
            RouteSegment::Static("profiles"),
            RouteSegment::Param("id"),
        ],
    )
}

/// Check the complete page inventory at the explicitly configured mount.
///
/// # Errors
/// Propagates invalid compiled page definitions, or rejects an empty/excessive
/// inventory or duplicate identity/path. No routes are registered here.
pub fn catalog(base: BasePath) -> Result<RouteCatalog, ViewError> {
    RouteCatalog::new(base, vec![search()?, detail()?])
}

/// Public search locations carry only the explicitly allowlisted prefix.
///
/// # Errors
/// Rejects an empty or >1024-byte prefix, invalid compiled route metadata or an
/// unsafe/oversized final location. Input and mount remain unchanged.
pub fn search_location(base: &BasePath, id: &str) -> Result<PublicLocation, ViewError> {
    valid_prefix(id)?;
    let path = search()?.location(base, &[])?;
    PublicLocation::new(format!("{}?id={}", path.as_str(), encode_uri_component(id)))
}

/// Canonical detail URL carries an independent, explicitly encoded return prefix.
///
/// # Errors
/// Rejects invalid IDs, prefixes or unsafe/oversized locations.
pub fn detail_location(
    base: &BasePath,
    detail_id: &str,
    search_prefix: &str,
) -> Result<PublicLocation, ViewError> {
    valid_prefix(search_prefix)?;
    let path = detail()?.location(base, &[("id", detail_id)])?;
    PublicLocation::new(format!(
        "{}?return={}",
        path.as_str(),
        encode_uri_component(search_prefix)
    ))
}

/// Decode a detail query exactly once; absence has an explicit caller-selected default.
///
/// # Errors
/// Rejects unknown/duplicate keys, empty values, raw plus aliases and malformed UTF-8/escapes.
pub fn decode_return_prefix(query: &str) -> Result<Option<String>, ViewError> {
    if query.is_empty() {
        return Ok(None);
    }
    let value = query
        .strip_prefix("return=")
        .ok_or(ViewError::InvalidSnapshot)?;
    if value.contains(['&', '+', '=']) {
        return Err(ViewError::InvalidSnapshot);
    }
    let value = decode_query_value(value)?;
    valid_prefix(&value)?;
    Ok(Some(value))
}

/// Checked navigation target, independent of browser history or DOM APIs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    /// Search at the exact public prefix.
    Search(String),
    /// Detail identity plus its independent return prefix.
    Detail {
        /// Exact decoded record ID.
        id: String,
        /// Independently decoded originating search prefix.
        search_prefix: String,
    },
}

/// Resolve only catalog locations; transport adapters never duplicate route parsing.
///
/// # Errors
/// Rejects foreign mounts, unknown paths and malformed/ambiguous queries.
pub fn resolve(base: &BasePath, location: &str) -> Result<Destination, ViewError> {
    let checked = PublicLocation::new(location)?;
    let query = location.split_once('?').map_or("", |(_, query)| query);
    if search()?.captures(base, &checked).is_some() {
        let value = query
            .strip_prefix("id=")
            .ok_or(ViewError::InvalidSnapshot)?;
        if value.contains(['&', '+', '=']) {
            return Err(ViewError::InvalidSnapshot);
        }
        return Ok(Destination::Search(decode_query_value(value)?));
    }
    let captures = detail()?
        .captures(base, &checked)
        .ok_or(ViewError::InvalidSnapshot)?;
    let encoded = captures
        .iter()
        .find(|(name, _)| *name == "id")
        .map(|(_, value)| *value)
        .ok_or(ViewError::InvalidSnapshot)?;
    Ok(Destination::Detail {
        id: decode_parameter(encoded)?,
        search_prefix: decode_return_prefix(query)?.unwrap_or_else(|| "abc".into()),
    })
}

/// Prefix bounds agree with SearchInput, without depending on a host crate.
fn valid_prefix(value: &str) -> Result<(), ViewError> {
    if value.is_empty() || value.len() > 1024 {
        Err(ViewError::InvalidSnapshot)
    } else {
        Ok(())
    }
}

/// The fixed operation endpoint cannot be selected by a caller field.
///
/// # Errors
/// Returns the mount join error if the combined endpoint exceeds PublicLocation
/// bounds or its fixed suffix becomes invalid. No endpoint is registered here.
pub fn search_api(base: &BasePath) -> Result<PublicLocation, ViewError> {
    base.join("/ui-api/v1/profile-search")
}

/// Decode exactly one native/Leptos path capture, preserving literal plus signs.
///
/// # Errors
/// Rejects malformed escapes, invalid UTF-8, dot aliases and unbounded identifiers.
pub fn decode_parameter(value: &str) -> Result<String, ViewError> {
    let decoded = decode_query_value(value)?;
    if matches!(decoded.as_str(), "." | "..") {
        return Err(ViewError::InvalidSnapshot);
    }
    Ok(decoded)
}

/// Decode a bounded encoded component once, preserving plus and Unicode as data.
fn decode_query_value(value: &str) -> Result<String, ViewError> {
    if value.len() > 4096 {
        return Err(ViewError::InvalidSnapshot);
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        // A percent escape consumes exactly two hex digits; never decode a second time.
        if bytes[index] == b'%' {
            let pair = bytes
                .get(index + 1..index + 3)
                .ok_or(ViewError::InvalidSnapshot)?;
            let high = char::from(pair[0])
                .to_digit(16)
                .ok_or(ViewError::InvalidSnapshot)?;
            let low = char::from(pair[1])
                .to_digit(16)
                .ok_or(ViewError::InvalidSnapshot)?;
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            // Path plus signs remain plus; only a form/query decoder treats them as spaces.
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let decoded = String::from_utf8(decoded).map_err(|_| ViewError::InvalidSnapshot)?;
    if decoded.is_empty() || decoded.len() > 1024 {
        return Err(ViewError::InvalidSnapshot);
    }
    Ok(decoded)
}
