//! Compiled public routes shared by native registration and Leptos path descriptors.

use stackpress_framework_view::{
    BasePath, PageRoute, PublicLocation, RouteCatalog, RouteSegment, ViewError,
    encode_uri_component,
};

/// The public search page; native and Leptos adapters consume these exact segments.
pub fn search() -> Result<PageRoute, ViewError> {
    PageRoute::new(
        "profiles",
        &[RouteSegment::Static("ui"), RouteSegment::Static("profiles")],
    )
}

/// The public detail path, with an encoded single-segment identifier.
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
pub fn catalog(base: BasePath) -> Result<RouteCatalog, ViewError> {
    RouteCatalog::new(base, vec![search()?, detail()?])
}

/// Public search locations carry only the explicitly allowlisted prefix.
pub fn search_location(base: &BasePath, id: &str) -> Result<PublicLocation, ViewError> {
    let path = search()?.location(base, &[])?;
    PublicLocation::new(format!("{}?id={}", path.as_str(), encode_uri_component(id)))
}

/// The fixed operation endpoint cannot be selected by a caller field.
pub fn search_api(base: &BasePath) -> Result<PublicLocation, ViewError> {
    base.join("/ui-api/v1/profile-search")
}

/// Decode exactly one native/Leptos path capture, preserving literal plus signs.
///
/// # Errors
/// Rejects malformed escapes, invalid UTF-8, dot aliases and unbounded identifiers.
pub fn decode_parameter(value: &str) -> Result<String, ViewError> {
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
    if decoded.is_empty() || decoded.len() > 1024 || matches!(decoded.as_str(), "." | "..") {
        return Err(ViewError::InvalidSnapshot);
    }
    Ok(decoded)
}
