# Server API

Immutable runtime hosting with explicit raw and top-level entry points. Server is non-Clone; use an
application-owned Arc to share it. Sources: [server.rs](../../src/server.rs),
[lifecycle.rs](../../src/lifecycle.rs).

```rust
use stackpress_lib::{Flow, Invocation, Method, Request, Response, Result};
use stackpress_server::{HandleOptions, ServerBuilder};

async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
app.on("greet", greet)?;
app.get("/hello", greet)?;
let server = app.build().await?;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Inspection](#1-inspection) · [2. Top level](#2-top-level) · [3. Raw
dispatch](#3-raw-dispatch)

## 1. Inspection

These methods borrow frozen descriptors or registries; none reopens construction.

### 1.1. plugins

`plugins(&self) -> impl Iterator<Item = &PluginDescriptor>`

Inspect selected order/provenance.

```rust
assert_eq!(server.plugins().count(), 0);
```

**Returns** A borrowing iterator of compiled selection metadata; direct handler contributions do not
create a plugin descriptor.

### 1.2. listeners

`listeners(&self) -> impl Iterator<Item = &ListenerDescriptor>`

Inspect matching and listener provenance.

```rust
assert_eq!(server.listeners().count(), 2);
```

**Returns** Borrowed Lib descriptors with IDs, sites, and matching metadata.

### 1.3. host

`host(&self) -> &HostContext`

Access frozen typed config/services outside dispatch.

```rust
let host = server.host();
```

**Returns** A borrowed HostContext, without cloning or mutation authority.

## 2. Top level

Normal stage/input failures are returned in HandleReport with chosen public response. Outer Result
errors mean handling could not start/report, such as correlation exhaustion. These methods never
perform native output; panic unwinding propagates. See [lifecycle rules](../concepts.md).

HandleOptions is Default with public `adapter_context: Option<Arc<dyn Any + Send + Sync>>`,
initially None. It holds trusted typed metadata, kept separate from caller-controlled Request.data.

### 2.1. handle

`async fn handle(&self, event: &str, req: &mut Request, res: &mut Response) -> Result<HandleReport>`

Run one lifecycle on the caller's exact buffered values.

```rust
let mut req = Request::new();
let mut res = Response::new();
let report = server.handle("greet", &mut req, &mut res).await?;
assert!(report.is_success(&res));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Nonempty selector, ≤8,192 bytes without controls. |
| `req` | `&mut Request` | Normalized caller input; load attempt is finalized once. |
| `res` | `&mut Response` | Caller response; its delivery state/limits govern handling. |

**Returns** HandleReport or inability-to-start Error. Invalid events become Input failure; finalized
entry skips load and callbacks. Open output stays buffered.

### 2.2. handle_with_options

`async fn handle_with_options(&self, event: &str, req: &mut Request, res: &mut Response, options: HandleOptions) -> Result<HandleReport>`

Carry trusted per-call adapter context through one lifecycle.

```rust
let mut req = Request::new();
let mut res = Response::new();
let report = server
    .handle_with_options("greet", &mut req, &mut res, HandleOptions::default())
    .await?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Same checked selector as handle. |
| `req` | `&mut Request` | Exact normalized request. |
| `res` | `&mut Response` | Exact response. |
| `options` | `HandleOptions` | Owned trusted metadata inherited by nested dispatch. |

**Returns** Report or outer start Error, with the same stage/error/Abort rules as handle. handle is
this method with default options.

### 2.3. call

`async fn call<I: IntoRequest>(&self, event: &str, input: I) -> Result<HostReply>`

Convert owned input inside the reporting boundary and own a fresh response.

```rust
let reply = server.call("greet", serde_json::json!({})).await?;
assert!(reply.report.is_success(&reply.response));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Checked selected event. |
| `input` | `I: IntoRequest` | Object-compatible input or normalized Request; see Lib request conversions. |

**Returns** HostReply { response, report } or outer start Error. Pre-normalization conversion
failure returns public 400/Input failure and no callbacks. A valid request enters the normal
lifecycle. This preserves Response limits/private state rather than returning only an envelope.

### 2.4. handle_route

`async fn handle_route(&self, req: &mut Request, res: &mut Response) -> Result<HandleReport>`

Derive the event from normalized method and pathname.

```rust
let mut req = Request::builder().url("/hello?event=ignored").build()?;
let mut res = Response::new();
let report = server.handle_route(&mut req, &mut res).await?;
assert_eq!(report.event, "GET /hello");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `req` | `&mut Request` | Checked method/URL provide selector; query/data/headers do not. |
| `res` | `&mut Response` | Exact output value. |

**Returns** The same handled report/start Error as handle for METHOD pathname. No listener or
network request is started.

## 3. Raw dispatch

Raw roots supply host access with stage Dispatch and no correlation. They perform no automatic load,
status normalization, Error offer, or native output. Invocation nesting inherits its current parent
context instead. Returned HandlerFuture values borrow all inputs for the shared lifetime.

### 3.1. emit

`emit<'a>(&'a self, event: &'a str, req: &'a mut Request, res: &'a mut Response) -> HandlerFuture<'a, Result<DispatchOutcome>>`

Dispatch with the exact mutable pair.

```rust
let mut req = Request::new();
let mut res = Response::new();
let outcome = server.emit("greet", &mut req, &mut res).await?;
assert!(!req.loaded());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Nonempty ≤8,192-byte selector without controls. |
| `req` | `&'a mut Request` | Exact mutable input. |
| `res` | `&'a mut Response` | Exact mutable output. |

**Returns** A Send future yielding Completed, Aborted, or NoListener, or validation/matching/handler
Error. NoListener stays an execution outcome.

### 3.2. resolve

`resolve<'a, I: IntoRequest + 'a>(&'a self, event: &'a str, input: I) -> HandlerFuture<'a, Result<Resolution>>`

Resolve raw work with fresh owned values.

```rust
let resolved = server.resolve("greet", serde_json::json!({})).await?;
assert!(resolved.response.results.is_some());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Same raw selector checks. |
| `input` | `I: IntoRequest + 'a` | Consumed input for a fresh request/response pair. |

**Returns** A Send future yielding Lib Resolution (outcome and envelope), or input/selector/dispatch
Error. No top-level lifecycle/report is added.

### 3.3. resolve_route

`resolve_route<'a, I: IntoRequest + 'a>(&'a self, method: Method, path: &'a str, input: I) -> HandlerFuture<'a, Result<Resolution>>`

Form METHOD path and resolve it raw.

```rust
let resolved = server
    .resolve_route(Method::GET, "/hello", serde_json::json!({}))
    .await?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `Method` | Checked concrete method. |
| `path` | `&'a str` | Caller-supplied pathname, not a full URL/query. |
| `input` | `I: IntoRequest + 'a` | Owned raw input conversion. |

**Returns** A borrowed Send future with raw Resolution or selector/conversion/dispatch Error; no URL
parsing or lifecycle policy is added.

Related: [testing guide](../guides/testing-in-process.md), [reports](reports.md),
[context](context.md).
