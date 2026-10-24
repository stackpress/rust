# API reference

Look up the public `stackpress-lib` surface by responsibility. These pages describe the current
checkout's API, with signatures, independent examples, parameters, returns, and error behavior. For
first use, begin with [Getting started](../getting-started.md); for the mental model, read
[Concepts](../concepts.md).

## Surfaces

The direct crate is `stackpress_lib`. Its root explicitly re-exports common types; public modules
provide the remaining operations. Rustdoc and [source exports](../../src/lib.rs) are the
authoritative declarations.

| Reference | Public surface | Typical use |
| --- | --- | --- |
| [Events](event.md) | EmitterBuilder, EventEmitter, Handler, handler, HandlerFuture, ListenerSpec/Id/Descriptor/Phase, EventMatch, Invocation, IntoRequest, Resolution, Flow, DispatchOutcome | Register and call operations |
| [Patterns](pattern.md) | EventPattern, MatchData, Method, MethodPattern, Route | Match literal/token/regex/route events |
| [Queues](queue.md) | queue::Item, ItemQueue, TaskHandler, task, TaskQueue, QueueOutcome | Ordered items and startup work |
| [Data](data.md) | Value, Timestamp, FileData, SharedNative, ParseLimits, Nest, DataPath, IntoPath, PathSegment, DataMap, DataSet; data::IntoNest, VisitOutcome, shape helpers | Owned values, paths, and explicit sharing |
| [Parsers](parsers.md) | data::parse_query, parse_args, parse_arg_string, parse_json_object, parse_multipart, parse_body | Buffered boundary conversion |
| [Request](request.md) | Request, RequestBuilder, RequestView, LoadResult, LoadState, with_unknown_host | Construct and load input |
| [Response](response.md) | Response, ResponseView, ResponseEnvelope, DeliveryState | Prepare, transfer, and finalize output |
| [Session](session.md) | Session, SessionValue, SessionRevision, CookieOptions, Priority, SameSite; session::cookie functions | Outgoing cookie revisions and codecs |
| [Headers and body](headers-and-body.md) | Headers, Body | Multi-value fields and buffered representations |
| [Status](status.md) | Status; status::CODES | Application code/message catalog |
| [Error](error.md) | Error, ErrorKind, Result, PublicError, SourceLocation, SourceSpan, InvocationFrame | Public failure data and developer diagnostics |

## Example conventions

Each page starts with imports and any shared fixture declarations. Read its setup sentence before
copying a method snippet: most omit the surrounding function returning `Result<()>`, and async calls
require an executor chosen by the caller. Named `async fn` callbacks are the primary form;
[handler](event.md#11-handler) and [task](queue.md#31-task) show the secondary boxed closure forms.

Signatures show the receiver, parameters, output, and relevant generic bounds; `Self` names the
enclosing type. Type parameters are not runtime parameters. Public fields, enum variants, constants,
and trait/conversion behavior are documented alongside their owners. Standard derived trait methods
are summarized with their types rather than repeated as framework operations. `DataMap` and
`DataSet`
are unwrapped collection aliases; the reference documents their role and representative operations
rather than copying the dependency's full method catalog.

Examples are plain Markdown and are not doctest inputs. Build/tests of the crate do not
automatically check these pages. Explicit source links allow reviewers to compare the documented
operation with its implementation.

## Integration paths

The crate is transport-neutral and does not choose an executor, provide native streams, start
servers, or orchestrate top-level lifecycle events. The
[request/response guide](../guides/requests-and-responses.md) describes adapter snapshots and
one-attempt boundaries; [startup queues](../guides/startup-with-task-queues.md) shows
construction-time registration.

Use the workspace facade as `stackpress::lib` when composing a wider application. Its selected
common root imports mirror the direct library's values; queue APIs, parser functions, and `IntoNest`
remain accessible through the module path. A re-export does not register plugins or prove an
untested platform adapter.
