//! Transport-neutral primitives for Stackpress capability execution.
//!
//! The foundation does not choose an executor or depend on a native host.
//! Named async handlers run sequentially against caller-owned requests and
//! responses. Nested capability calls share immutable listener registrations;
//! rich values and explicit boundary parsers remain usable without a server.
//!
//! # Examples
//!
//! ```
//! use stackpress_lib::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
//!
//! async fn greet(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
//!     let name = req
//!         .data
//!         .path_as::<String>("profile.name")?
//!         .unwrap_or_else(|| "world".into());
//!     res.results(&format!("Hello, {name}"))?;
//!     Ok(Flow::Continue)
//! }
//! # async fn example() -> Result<()> {
//! let mut builder = EmitterBuilder::new();
//! builder.on_priority("greet", 10, greet)?;
//! // Plain data becomes a fresh request; resolve returns its response envelope.
//! let resolution = builder
//!     .build()
//!     .resolve("greet", serde_json::json!({"profile": {"name": "Ada"}}))
//!     .await?;
//! assert_eq!(
//!     resolution.response.results.unwrap().as_str(),
//!     Some("Hello, Ada")
//! );
//! #     Ok(())
//! # }
//! ```

mod body;
pub mod data;
pub mod error;
pub mod event;
mod flow;
mod future;
pub mod headers;
pub mod pattern;
pub mod queue;
pub mod request;
pub mod response;
pub mod session;
pub mod status;

pub use body::Body;
pub use data::{
    DataMap, DataPath, DataSet, FileData, IntoPath, Nest, ParseLimits, PathSegment, SharedNative,
    Timestamp, Value,
};
pub use error::{
    Error, ErrorKind, InvocationFrame, PublicError, Result, SourceLocation, SourceSpan,
};
pub use event::{
    EmitterBuilder, EventEmitter, EventMatch, Handler, IntoRequest, Invocation, ListenerDescriptor,
    ListenerId, ListenerPhase, ListenerSpec, Resolution, handler,
};
pub use flow::{DispatchOutcome, Flow};
pub use future::HandlerFuture;
pub use headers::Headers;
pub use pattern::{EventPattern, MatchData, Method, MethodPattern, Route};
pub use request::{LoadResult, LoadState, Request, RequestBuilder, RequestView, with_unknown_host};
pub use response::{DeliveryState, Response, ResponseEnvelope, ResponseView};
pub use session::{CookieOptions, Priority, SameSite, Session, SessionRevision, SessionValue};
pub use status::Status;
