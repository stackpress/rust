//! One checked mount and Leptos path adapter over client-safe compiled definitions.

pub use crate::{BasePath, PageRoute, RouteCatalog, RouteSegment};
use leptos::prelude::*;
use leptos_router::{
    ParamSegment, PartialPathMatch, PathSegment, PossibleRouteMatch, StaticSegment,
};

/// Root-local routing metadata, provided explicitly by the native/client bootstrap.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RouteContext {
    /// Canonical trusted application mount; root is empty.
    pub base: BasePath,
}

/// Provide the same checked mount before evaluating a shared component tree.
pub fn provide_route_context(base: BasePath) {
    provide_context(RouteContext { base });
}

/// Read the root mount; simple packaged/root-only applications use the explicit root default.
pub fn use_route_context() -> RouteContext {
    use_context::<RouteContext>().unwrap_or_default()
}

/// Leptos path descriptor derived directly from the compiled native page definition.
#[derive(Clone, Debug)]
pub struct CatalogPath(PageRoute);
impl CatalogPath {
    /// Retain a checked compiled route; caller data never becomes a pattern.
    pub fn new(route: PageRoute) -> Self {
        Self(route)
    }
}
impl PossibleRouteMatch for CatalogPath {
    /// The initial catalog supports required parameters only.
    fn optional(&self) -> bool {
        false
    }
    /// Delegate segment semantics to Leptos rather than implementing a second matcher.
    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let mut remaining = path;
        let mut params = Vec::new();
        for segment in self.0.segments() {
            let matched = match segment {
                RouteSegment::Static(value) => StaticSegment(*value).test(remaining),
                RouteSegment::Param(name) => ParamSegment(name).test(remaining),
            }?;
            remaining = matched.remaining();
            params.extend(matched.params());
        }
        Some(PartialPathMatch::new(
            remaining,
            params,
            &path[..path.len() - remaining.len()],
        ))
    }
    /// Supply the same compiled segments to Leptos route generation/introspection.
    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        for segment in self.0.segments() {
            path.push(match segment {
                RouteSegment::Static(value) => PathSegment::Static((*value).into()),
                RouteSegment::Param(name) => PathSegment::Param((*name).into()),
            });
        }
    }
}
