//! Compiled route descriptions shared by native registration and client matching.

use crate::{BasePath, PageId, PublicLocation, ViewError};

/// Deliberately small path grammar for the initial shared page catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteSegment {
    /// Literal ASCII segment, supplied by compiled application source.
    Static(&'static str),
    /// One required named parameter; its URL value is encoded independently.
    Param(&'static str),
}

/// One compiled page identity and path; not a competing native router.
#[derive(Clone, Debug)]
pub struct PageRoute {
    /// Identity must remain unique in a catalog.
    id: PageId,
    /// Shared segments drive both native patterns and Leptos path descriptors.
    segments: &'static [RouteSegment],
}

impl PageRoute {
    /// Check a compiled route without evaluating input as a pattern or module path.
    ///
    /// # Errors
    /// Rejects invalid page/segment names and empty or excessive routes.
    pub fn new(id: &str, segments: &'static [RouteSegment]) -> Result<Self, ViewError> {
        let id = PageId::try_from(id.to_owned())?;
        if segments.is_empty()
            || segments.len() > 32
            || segments.iter().any(|segment| {
                let (RouteSegment::Static(value) | RouteSegment::Param(value)) = segment;
                value.is_empty()
                    || matches!(*value, "." | "..")
                    || value.len() > 64
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            })
        {
            return Err(ViewError::InvalidPolicy);
        }
        // Duplicate parameter names cannot be represented by an exact named input set.
        for (index, segment) in segments.iter().enumerate() {
            if let RouteSegment::Param(name) = segment
                && segments[..index].contains(&RouteSegment::Param(name))
            {
                return Err(ViewError::InvalidPolicy);
            }
        }
        Ok(Self { id, segments })
    }

    /// Borrow the checked public identity.
    pub fn id(&self) -> &PageId {
        &self.id
    }
    /// Borrow the immutable compiled grammar used by the Leptos adapter.
    pub fn segments(&self) -> &'static [RouteSegment] {
        self.segments
    }

    /// Produce an application-relative native Express-style route pattern.
    pub fn pattern(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                RouteSegment::Static(value) => format!("/{value}"),
                RouteSegment::Param(value) => format!("/:{value}"),
            })
            .collect()
    }

    /// Build an encoded public path from an exact named parameter set.
    ///
    /// # Errors
    /// Rejects missing/extra/duplicate parameter names and empty or oversized values.
    pub fn location(
        &self,
        base: &BasePath,
        params: &[(&str, &str)],
    ) -> Result<PublicLocation, ViewError> {
        let expected = self
            .segments
            .iter()
            .filter(|part| matches!(part, RouteSegment::Param(_)))
            .count();
        if params.len() != expected {
            return Err(ViewError::InvalidSnapshot);
        }
        let mut path = String::new();
        for segment in self.segments {
            path.push('/');
            match segment {
                RouteSegment::Static(value) => path.push_str(value),
                RouteSegment::Param(name) => {
                    let values = params
                        .iter()
                        .filter(|(key, _)| key == name)
                        .collect::<Vec<_>>();
                    if values.len() != 1
                        || values[0].1.is_empty()
                        || matches!(values[0].1, "." | "..")
                        || values[0].1.len() > 1024
                    {
                        return Err(ViewError::InvalidSnapshot);
                    }
                    path.push_str(&encode_uri_component(values[0].1));
                }
            }
        }
        base.join(&path)
    }
}

/// One mount and checked page inventory; native matching still executes all listeners.
#[derive(Clone, Debug)]
pub struct RouteCatalog {
    /// Public mount is trusted build metadata, never inferred from caller headers.
    base: BasePath,
    /// Catalog rows preserve application definition order.
    routes: Vec<PageRoute>,
}
impl RouteCatalog {
    /// Validate identity/path uniqueness before either adapter consumes this inventory.
    ///
    /// # Errors
    /// Rejects empty/excessive catalogs and duplicate identities or path definitions.
    pub fn new(base: BasePath, routes: Vec<PageRoute>) -> Result<Self, ViewError> {
        if routes.is_empty() || routes.len() > 128 {
            return Err(ViewError::InvalidPolicy);
        }
        for (index, route) in routes.iter().enumerate() {
            if routes[..index]
                .iter()
                .any(|old| old.id == route.id || old.pattern() == route.pattern())
            {
                return Err(ViewError::InvalidPolicy);
            }
        }
        Ok(Self { base, routes })
    }
    /// Borrow this application's trusted mount.
    pub fn base(&self) -> &BasePath {
        &self.base
    }
    /// Iterate the exact compiled definitions supplied to both adapters.
    pub fn routes(&self) -> impl Iterator<Item = &PageRoute> {
        self.routes.iter()
    }
    /// Resolve a page identity without accepting arbitrary dynamic event/module names.
    pub fn get(&self, id: &str) -> Option<&PageRoute> {
        self.routes.iter().find(|route| route.id.as_str() == id)
    }
    /// Return a checked mounted native pattern for one known catalog identity.
    ///
    /// # Errors
    /// Rejects unknown identities and an overlong mounted pattern.
    pub fn pattern(&self, id: &str) -> Result<PublicLocation, ViewError> {
        self.base
            .join(&self.get(id).ok_or(ViewError::InvalidPolicy)?.pattern())
    }
}

/// Percent-encode a UTF-8 value as one path/query component without truthiness loss.
pub fn encode_uri_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write;
            // String's fmt::Write implementation is infallible.
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
