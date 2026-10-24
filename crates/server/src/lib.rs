//! Transport-neutral hosting over Stackpress's immutable capability table.
//!
//! M0 supplies the proven host/context/startup/wrapper boundary. Top-level
//! request/operation/response/error handling, config section decoding and full
//! adapters are later milestones; [`Server::emit`] remains raw dispatch.
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
mod output;

#[cfg(feature = "http")]
pub mod http;

pub use builder::{BuildError, BuildResult, Server, ServerBuilder, StartupContext, StartupStage};
pub use config::ConfigLayers;
pub use context::{DelegationError, HostContext, LifecycleStage, ServerInvocationExt};
pub use output::OutputGuard;

#[cfg(test)]
mod m0;
