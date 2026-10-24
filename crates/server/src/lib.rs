//! Transport-neutral hosting over Stackpress's immutable capability table.
//!
//! Selected compiled plugins configure typed sections/services and startup phases before
//! immutable publication. [`Server::handle`] runs one top-level lifecycle; nested dispatch
//! and [`Server::emit`] remain raw. Native output belongs to an explicit adapter.
//! Nothing in this crate starts a listener, executor or background task.
//!
//! ```
//! use stackpress_lib::{Flow, Invocation, Request, Response, Result};
//! use stackpress_server::{ServerBuilder, BuildResult};
//! async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
//!     res.results(&"hello")?;
//!     Ok(Flow::Continue)
//! }
//! fn plugin(app: &mut ServerBuilder) -> Result<()> {
//!     app.on("hello", hello)?;
//!     Ok(())
//! }
//! # async fn example() -> BuildResult<()> {
//! let mut app = ServerBuilder::new();
//! app.plugin("hello", plugin)?;
//! let server = app.build().await?;
//! # let _ = server;
//! # Ok(())
//! # }
//! ```

mod builder;
mod config;
mod context;
mod lifecycle;
mod output;
mod plugin;
mod report;
mod router;
mod services;

#[cfg(feature = "http")]
pub mod http;

pub use builder::{
    BuildError, BuildResult, Server, ServerBuilder, StartupContext, StartupSpec, StartupStage,
};
pub use config::ConfigLayers;
pub use context::{DelegationError, HostContext, ServerInvocationExt};
pub use output::OutputGuard;
pub use plugin::{PluginDescriptor, PluginSpec};
pub use router::{CapturePolicy, RouteSpec};

#[cfg(test)]
mod core_tests;

pub use lifecycle::HandleOptions;
pub use report::{
    CorrelationId, DeveloperReport, ErrorHandling, ErrorOffer, HandleReport, HostReply,
    LifecycleOutcome, LifecycleStage, StageFailure, StageOutcome, StageRecord,
};
