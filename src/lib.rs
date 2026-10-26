//! Convenient imports for the independently usable Stackpress libraries.
//!
//! The foundation provides values, parsing and ordered capability execution.
//! Native hosts/UI/database libraries are introduced only as implemented.
//! Re-exporting a library does not register a plugin or start an executor.
//!
//! Idea build-host tools are opt-in: `idea-parser` exposes `idea_parser` for pure parsing,
//! `idea-transformer` exposes the native transformer, and `idea` exposes their convenience
//! wrapper. None is selected by default. The wrapper also re-exports both component crates.
//! The transformer and wrapper are native build tools; browser targets are not verified.
//!
//! `sql` exposes the runtime-independent SQL contracts. `sql-sqlite` also selects the
//! native Tokio/SQLx SQLite adapter. `sql-postgres` and `sql-mysql` select the optional
//! native network adapters. All database features are disabled by default; a re-export
//! does not open a database or expose a query capability.
//!
//! UI is opt-in with `framework-ui-ssr`, `framework-ui-hydrate` or `framework-ui-csr`.
//! Build these mutually exclusive render modes separately. Native buffered SSR requires
//! `framework_ui::render::initialize_ssr()` during startup inside an active Tokio runtime.
//!
//! `server-http` retains the buffered adapter only. `server-http-host` separately
//! exposes the supplied-IO HTTP/1 host as `server_http_host`; `server-http-host-tls`
//! also selects its Rustls integration. None is selected by default.
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

#[cfg(feature = "framework-ui")]
pub use stackpress_framework_ui as framework_ui;
#[cfg(feature = "framework-view")]
pub use stackpress_framework_view as framework_view;
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
#[cfg(feature = "server-http-host")]
pub use stackpress_server_http as server_http_host;
#[cfg(feature = "sql")]
pub use stackpress_sql as sql;
#[cfg(feature = "sql-mysql")]
pub use stackpress_sql_mysql as sql_mysql;
#[cfg(feature = "sql-postgres")]
pub use stackpress_sql_postgres as sql_postgres;
#[cfg(feature = "sql-sqlite")]
pub use stackpress_sql_sqlite as sql_sqlite;

/// Checked locale selection and preference action policy.
#[cfg(feature = "framework-language")]
pub use stackpress_framework_language as framework_language;

/// Host-managed presentation settings, public flash and fail-closed layout policy.
#[cfg(feature = "framework-preferences")]
pub use stackpress_framework_preferences as framework_preferences;
