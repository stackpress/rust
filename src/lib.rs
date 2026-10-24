//! Convenient imports for the independently usable Stackpress libraries.
//!
//! The foundation provides values, parsing and ordered capability execution.
//! Native hosts/UI/database libraries are introduced only as implemented.
//! Re-exporting a library does not register a plugin or start an executor.
//!
//! # Examples
//!
//! ```
//! use stackpress::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
//!
//! async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
//!     res.results(&"ready")?;
//!     Ok(Flow::Continue)
//! }
//! # async fn example() -> Result<()> {
//! let mut builder = EmitterBuilder::new();
//! builder.on("health", health)?;
//! // Building freezes registrations; capability calls create their own req/res pair.
//! let reply = builder
//!     .build()
//!     .resolve("health", stackpress::Nest::new())
//!     .await?;
//! assert_eq!(reply.response.results.unwrap().as_str(), Some("ready"));
//! #     Ok(())
//! # }
//! ```

#[cfg(feature = "idea")]
pub use stackpress_idea as idea;
#[cfg(feature = "idea-parser")]
pub use stackpress_idea_parser as idea_parser;
#[cfg(feature = "idea-transformer")]
pub use stackpress_idea_transformer as idea_transformer;
pub use stackpress_lib as lib;
pub use stackpress_lib::{
    Body, CookieOptions, DataMap, DataPath, DataSet, DeliveryState, DispatchOutcome,
    EmitterBuilder, Error, ErrorKind, EventEmitter, EventMatch, EventPattern, FileData, Flow,
    Handler, HandlerFuture, Headers, IntoPath, IntoRequest, Invocation, InvocationFrame,
    ListenerDescriptor, ListenerId, ListenerPhase, ListenerSpec, LoadResult, LoadState, MatchData,
    Method, MethodPattern, Nest, ParseLimits, PathSegment, Priority, PublicError, Request,
    RequestBuilder, RequestView, Resolution, Response, ResponseEnvelope, ResponseView, Result,
    Route, SameSite, Session, SessionRevision, SessionValue, SharedNative, SourceLocation,
    SourceSpan, Status, Timestamp, Value, handler, with_unknown_host,
};
#[cfg(feature = "server")]
pub use stackpress_server as server;
