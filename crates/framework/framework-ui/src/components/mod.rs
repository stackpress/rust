//! Native-semantic controls, explicit layout slots and root-scoped providers.
//!
//! All strings render as escaped text. Styles are small, neutral and identical
//! across SSR/hydration/CSR; application-specific branding remains with callers.

mod controls;
mod layout;

pub use controls::{
    Alert, Button, ButtonType, FieldControl, Input, Link, List, ListItem, Loader, LoaderContainer,
    NotifierContainer, safe_href,
};
pub use layout::{
    LayoutBlank, LayoutHead, LayoutLeft, LayoutMain, LayoutMenu, LayoutPanel, LayoutProvider,
    LayoutRight,
};

#[cfg(all(test, feature = "ssr"))]
mod tests;
