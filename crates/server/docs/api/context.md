# Host context and Invocation API

HostContext is the frozen typed config/service view. ServerInvocationExt adds shortcuts to Lib
Invocation; handler authors import the trait. Source: [context.rs](../../src/context.rs).

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new("ready".to_owned()))?;
    start.register_named("primary", Arc::new("named".to_owned()))?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
app.config_section("limit", 3_u64, |_| Ok(()))?;
app.on_config(configure)?;
let server = app.build().await?;
let host = server.host();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. HostContext](#1-hostcontext) · [2. ServerInvocationExt](#2-serverinvocationext)

## 1. HostContext

Lookup types are `T: Any + Send + Sync` (static concrete type). Missing and wrong-typed values
return Error; registry access holds no lock across later application awaits. A borrowed service can
still contain the interior mutability chosen by its author.

### 1.1. config

`config<T: Any + Send + Sync>(&self) -> Result<&T>`

Borrow decoded config.

```rust
assert_eq!(*host.config::<u64>()?, 3);
```

**Returns** A borrowed section or missing-type error.

### 1.2. service

`service<T: Any + Send + Sync>(&self) -> Result<&T>`

Borrow a typed singleton.

```rust
assert_eq!(host.service::<String>()?, "ready");
```

**Returns** Borrowed T or missing-type error; never clones the payload.

### 1.3. service_named

`service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T>`

Borrow a named instance.

```rust
assert_eq!(host.service_named::<String>("primary")?, "named");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact named identity, independent from typed singleton slots. |

**Returns** Borrowed T or missing/wrong-type error.

### 1.4. shared_service

`shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>>`

Retain the exact singleton allocation.

```rust
let retained = host.shared_service::<String>()?;
```

**Returns** A cloned Arc handle or missing-type error. T itself is never cloned.

### 1.5. service_sites

`service_sites<T: Any + Send + Sync>(&self) -> Result<(&SourceLocation, Option<&SourceLocation>)>`

Inspect singleton contribution provenance.

```rust
let (original, replaced) = host.service_sites::<String>()?;
assert!(replaced.is_none());
```

**Returns** Original and latest replacement sites, borrowed, or missing-type error; service contents
remain private.

## 2. ServerInvocationExt

The snippets here define independent handlers; register them with a Server to supply host context.
Plain Lib invocations lack it. Nested emits/resolves inherit the exact context from their parent.

### 2.1. host

`host(&self) -> Result<&HostContext>`

Access the complete frozen registry.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _host = inv.host()?;
    Ok(Flow::Continue)
}
```

**Returns** Borrowed host, or missing-host Error for a non-host Lib invocation.

### 2.2. config

`config<T: Any + Send + Sync>(&self) -> Result<&T>`

Borrow typed config from a handler.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _limit = inv.config::<u64>()?;
    Ok(Flow::Continue)
}
```

**Returns** Borrowed section, or missing host/type Error.

### 2.3. service

`service<T: Any + Send + Sync>(&self) -> Result<&T>`

Borrow a singleton from a handler.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _service = inv.service::<String>()?;
    Ok(Flow::Continue)
}
```

**Returns** Borrowed resource, or missing host/type Error.

### 2.4. service_named

`service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T>`

Read a named resource.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _service = inv.service_named::<String>("primary")?;
    Ok(Flow::Continue)
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact named slot. |

**Returns** Borrowed T or missing host/name/type Error.

### 2.5. shared_service

`shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>>`

Explicitly retain a singleton Arc.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _retained = inv.shared_service::<String>()?;
    Ok(Flow::Continue)
}
```

**Returns** Arc sharing the existing allocation, or missing host/type Error.

### 2.6. stage

`stage(&self) -> Result<LifecycleStage>`

Inspect the root host stage, separate from Lib listener phases.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _stage = inv.stage()?;
    Ok(Flow::Continue)
}
```

**Returns** Active stage or missing-host Error. Raw host roots report Dispatch; nested calls inherit
their parent stage.

### 2.7. correlation_id

`correlation_id(&self) -> Result<Option<&str>>`

Read local top-level correlation.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _id = inv.correlation_id()?;
    Ok(Flow::Continue)
}
```

**Returns** Some borrowed ID for top-level work, None for raw host roots, or missing-host Error.

### 2.8. failure

`failure(&self) -> Result<Option<&StageFailure>>`

Inspect the original failure during an Error offer.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _original = inv.failure()?;
    Ok(Flow::Continue)
}
```

**Returns** Some retained failure in Error and its nested calls, None otherwise, or missing-host
Error.

### 2.9. adapter_context

`adapter_context<T: Any + Send + Sync>(&self) -> Option<&T>`

Borrow trusted typed adapter metadata.

```rust
async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _metadata = inv.adapter_context::<String>();
    Ok(Flow::Continue)
}
```

**Returns** Some matching adapter slot, otherwise None, including plain Lib/raw-host calls. No
request-data fallback.

### 2.10. delegate

`delegate<'a>(&'a self, event: &'a str, req: &'a mut Request, res: &'a mut Response) -> HandlerFuture<'a, Result<Flow>>`

Call a required capability with the same mutable pair and propagate its execution decision.

```rust
async fn expose(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("profile-search", req, res).await
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Required child selector. |
| `req` | `&'a mut Request` | Exact parent request; no load/lifecycle. |
| `res` | `&'a mut Response` | Exact parent response; no merge. |

**Returns** A borrowed Send future: Completed maps to Continue, Aborted to Abort, NoListener to
Error sourced by DelegationError::NoListener { event: String } with code 404/public Not Found. Other
dispatch errors preserve their causes/frames. DelegationError implements
Debug/Display/std::error::Error. Unlike the host lookup methods, delegate uses ordinary inv.emit and
can also run in a plain Lib invocation.

Related: [services guide](../guides/services.md), [Lib Invocation](../../../lib/docs/api/event.md).
