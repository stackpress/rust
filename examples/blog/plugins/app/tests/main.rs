//! Native portable UI and server lifecycle contracts.
#[cfg(feature = "server")]
#[path = "cases/lifecycle.rs"]
mod lifecycle;
#[cfg(feature = "ui")]
#[path = "cases/portable.rs"]
mod portable;
