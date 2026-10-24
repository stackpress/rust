# Event API

Construction-time registration and immutable, all-matching capability dispatch. Source:
[event.rs](../../src/event.rs). Imports below also orient the independent examples on this page.

```rust
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, EventPattern, Flow, HandlerFuture, IntoRequest,
    Invocation, ListenerSpec, MethodPattern, Nest, Request, Response, Result, handler,
};

async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"Hello")?;
    Ok(Flow::Continue)
}
```

Registration/emitter snippets run independently inside an async function returning `Result<()>`,
with `let mut builder = EmitterBuilder::new();`. Invocation snippets show statements inside a
compatible named handler with `req`, `res`, and `inv` arguments.
[Events and routing](../guides/events-and-routing.md) supplies complete integration examples.

Contents: [Handler contract](#1-handler-contract) ·
[Specification and metadata](#2-specification-and-metadata) ·
[EmitterBuilder operations](#3-emitterbuilder-operations) ·
[EventEmitter operations](#4-eventemitter-operations) ·
[Invocation and input adapter](#5-invocation-and-input-adapter)

## 1. Handler contract

`Handler` is the public registration bound, automatically implemented for compatible callbacks. Use
named `async fn` handlers with the signature above. The shared callable must be
`Send + Sync + 'static`; for every input-borrow lifetime its returned future must be `Send`. Authors
do not implement private adaptation traits.

`Flow::{Continue, Abort}` is the handler decision. A before/action/after phase error skips all later
work. Abort returns `DispatchOutcome::Aborted`; no matches returns `NoListener`; all continuing
matches return `Completed`. None automatically changes response status. Hooks run per matched
listener, and after is not cleanup. Cancellation preserves completed mutations and prevents
unstarted phases; panics propagate.

`HandlerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>` is the public erased future
alias. `Send` does not imply `'static`; the caller retains ownership of borrowed inputs.

### 1.1. handler

`handler<F>(callback: F) -> F where F: for<'a> Fn(&'a mut Request, &'a mut Response, Invocation<'a>) -> HandlerFuture<'a, Result<Flow>> + Send + Sync + 'static`

Give a borrowing closure its higher-ranked expected type. Use this secondary form when capturing
owned configuration.

```rust
let greeting = String::from("Hello");
builder.on(
    "greet",
    handler(move |_, res, _| {
        // Copy configuration for this future; the stored Fn can be called again.
        let greeting = greeting.clone();
        Box::pin(async move {
            res.results(&greeting)?;
            Ok(Flow::Continue)
        })
    }),
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `F` | Shared borrowing closure producing a boxed Send future. |

**Returns** The same callback unchanged. It does not register or run it. `Box::pin` supplies the
required pinned erased future, preserving its borrow lifetime.

## 2. Specification and metadata

`ListenerSpec` owns a pattern and optional diagnostics. Labels need not be unique;
replacement/removal uses opaque registry-qualified `ListenerId` handles. IDs implement
Copy/Clone/Debug/Eq/PartialEq/Hash/Display; do not parse their display text as a stable persistence
format.

| Public metadata | Fields |
| --- | --- |
| `ListenerDescriptor` | `id: ListenerId`, `pattern: String`, `label: String`, `plugin: Option<String>`, `priority: i64`, `registered: SourceLocation`, `replaced: Option<SourceLocation>`. |
| `EventMatch` | `listener: ListenerId`, `event: String`, `pattern: String`, `data: MatchData`. |
| `Resolution` | `outcome: DispatchOutcome`, `response: ResponseEnvelope`. |
| `ListenerPhase` | `Before`, `Action`, `After`; diagnostic frames use lowercase phase strings. |

Descriptors and matches are `Clone + Debug`; `Resolution` is also `Clone + Debug` with
[rich-value clone semantics](data.md). Descriptor `pattern` is the original display source; `label`
defaults to the ID text. `registered` retains the original registration site; `replaced` records the
latest replacement site. `plugin` is metadata, not dynamic loading. Captures and
[InvocationFrame](error.md#1-public-error-and-diagnostic-values) remain separate.

### 2.1. ListenerSpec::exact

`ListenerSpec::exact(event: &str) -> Self`

Start a literal contribution at priority 0 with no label/plugin.

```rust
let spec = ListenerSpec::exact("greet");
builder.listen(spec, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Literal event text; validated at registration. |

**Returns** An owned specification; construction itself is infallible.

### 2.2. ListenerSpec::pattern

`ListenerSpec::pattern(pattern: EventPattern) -> Self`

Start an explicitly compiled matcher contribution at priority 0.

```rust
let spec = ListenerSpec::pattern(EventPattern::express("greet/:name", '/')?);
builder.listen(spec, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pattern` | `EventPattern` | Owned exact/Express/regex/route matcher. |

**Returns** An owned specification with absent label/plugin.

### 2.3. ListenerSpec::priority

`priority(self, priority: i64) -> Self`

Choose scheduling priority; higher values run first.

```rust
let spec = ListenerSpec::exact("greet").priority(10);
builder.listen(spec, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `priority` | `i64` | Selected priority. |

**Returns** The moved specification with the new priority; default is 0.

### 2.4. ListenerSpec::label

`label(self, label: impl Into<String>) -> Self`

Choose a human-readable diagnostic label; it is not a unique lookup key.

```rust
let spec = ListenerSpec::exact("greet").label("greeting");
builder.listen(spec, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `label` | `impl Into<String>` | Selected label. |

**Returns** The moved specification with an explicit label; registration checks at most 8,192 bytes.

### 2.5. ListenerSpec::plugin

`plugin(self, plugin: impl Into<String>) -> Self`

Record the contributing plugin name for diagnostics.

```rust
let spec = ListenerSpec::exact("greet").plugin("profiles");
builder.listen(spec, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `plugin` | `impl Into<String>` | Selected plugin. |

**Returns** The moved specification with explicit plugin metadata; registration checks at most 8,192
bytes.

## 3. EmitterBuilder operations

An empty builder has no hooks and depth limit 64. Registration uses exclusive `&mut self`, records
the synchronous call site, and finishes before `build` consumes the builder. Priority ties retain
global contribution order across matcher kinds. Builders and finished emitters have no public Clone.

### 3.1. EmitterBuilder::new

`EmitterBuilder::new() -> Self`

Create an empty mutable registry; `Default` is equivalent.

```rust
let builder = EmitterBuilder::new();
assert_eq!(builder.build().listeners().count(), 0);
```

**Returns** An empty builder; no callback runs.

### 3.2. EmitterBuilder::on

`on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register an exact literal listener at priority 0. Duplicate contributions remain separate.

```rust
let id = builder.on("greet", greet)?;
assert!(builder.remove(&id)?);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Nonempty literal event, at most 8,192 bytes. |
| `action` | `impl Handler + 'static` | Compatible named handler or helper closure. |

**Returns** A stable `ListenerId`. Empty/oversized stored pattern source, sequence/registry
exhaustion, or invalid registration metadata returns an error before insertion.

### 3.3. EmitterBuilder::on_priority

`on_priority(&mut self, event: &str, priority: i64, action: impl Handler + 'static) -> Result<ListenerId>`

Register an exact listener with explicit scheduling priority.

```rust
builder.on_priority("greet", 10, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Nonempty literal event, at most 8,192 bytes. |
| `priority` | `i64` | Higher first; ties preserve registration order. |
| `action` | `impl Handler + 'static` | Compatible callback. |

**Returns** A stable `ListenerId`. Empty/oversized stored pattern source, sequence/registry
exhaustion, or invalid registration metadata returns an error before insertion.

### 3.4. EmitterBuilder::on_pattern

`on_pattern(&mut self, pattern: EventPattern, action: impl Handler + 'static) -> Result<ListenerId>`

Move a compiled matcher into a priority-zero contribution without reparsing its display text.

```rust
builder.on_pattern(EventPattern::express("greet/:name", '/')?, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pattern` | `EventPattern` | Owned typed matcher. |
| `action` | `impl Handler + 'static` | Compatible callback. |

**Returns** A stable `ListenerId`. Empty/oversized stored pattern source, sequence/registry
exhaustion, or invalid registration metadata returns an error before insertion.

### 3.5. EmitterBuilder::route

`route(&mut self, method: MethodPattern, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a priority-zero method/path event contribution; this creates no native endpoint.

```rust
builder.route(MethodPattern::GET, "/greet/:name", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `MethodPattern` | Exact normalized verb or Any. |
| `path` | `&str` | Absolute `/` token pattern. |
| `action` | `impl Handler + 'static` | Compatible callback. |

**Returns** A listener ID, or a [route compilation](pattern.md) or registration error. To add
priority/label/plugin, use `listen(ListenerSpec::pattern(EventPattern::route(...)?), action)`.

### 3.6. EmitterBuilder::listen

`listen(&mut self, spec: ListenerSpec, action: impl Handler + 'static) -> Result<ListenerId>`

Register a full contribution specification, including priority, label, and plugin.

```rust
builder.listen(
    ListenerSpec::exact("greet")
        .priority(10)
        .label("hello")
        .plugin("profiles"),
    greet,
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `ListenerSpec` | Owned matcher and metadata; label/plugin each at most 8,192 bytes. |
| `action` | `impl Handler + 'static` | Compatible shared callback. |

**Returns** A stable `ListenerId`. Empty/oversized stored pattern source, sequence/registry
exhaustion, or invalid registration metadata returns an error before insertion.

### 3.7. EmitterBuilder::before

`before(&mut self, action: impl Handler + 'static) -> &mut Self`

Select or replace the one before hook; it runs before each matched action. Abort skips action/after.

```rust
builder.before(greet);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `impl Handler + 'static` | Same signature as listener actions; current listener metadata identifies the contribution. |

**Returns** The mutable builder for chaining. Hook registration is infallible and records its own
call site; a later call replaces the existing hook.

### 3.8. EmitterBuilder::after

`after(&mut self, action: impl Handler + 'static) -> &mut Self`

Select or replace the one after hook; it runs only after a continuing action. It does not run after
failure or abort.

```rust
builder.after(greet);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `impl Handler + 'static` | Same signature as listener actions; current listener metadata identifies the contribution. |

**Returns** The mutable builder for chaining. Hook registration is infallible and records its own
call site; a later call replaces the existing hook.

### 3.9. EmitterBuilder::depth_limit

`depth_limit(&mut self, limit: usize) -> Result<&mut Self>`

Set maximum nested dispatch depth, including the root dispatch. Siblings each inherit their parent
depth independently.

```rust
builder.depth_limit(16)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limit` | `usize` | Inclusive valid range 1–256; default 64. |

**Returns** The mutable builder, or an input error for an out-of-range limit; the previous limit is
retained on error.

### 3.10. EmitterBuilder::remove

`remove(&mut self, id: &ListenerId) -> Result<bool>`

Remove one owned/imported contribution without changing survivors' relative order.

```rust
let id = builder.on("greet", greet)?;
assert!(builder.remove(&id)?);
assert!(!builder.remove(&id)?);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&ListenerId` | Handle from this registry or one moved into it with extend. |

**Returns** True when removed, false for a stale owned handle, or a `Registration` error for a
foreign handle. A later registration obtains a new identity/order.

### 3.11. EmitterBuilder::replace

`replace(&mut self, id: &ListenerId, action: impl Handler + 'static) -> Result<()>`

Change a contribution's action while retaining ID, matcher, priority, position, and original site.
Records the replacement site.

```rust
let id = builder.on("greet", greet)?;
builder.replace(&id, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&ListenerId` | Existing stable handle. |
| `action` | `impl Handler + 'static` | Replacement shared callback. |

**Returns** Unit after replacement; stale/foreign handles return a `Registration` error without
changing the table.

### 3.12. EmitterBuilder::clear

`clear(&mut self, pattern: &EventPattern) -> usize`

Remove every contribution with the same typed matcher identity.

```rust
builder.on("greet", greet)?;
assert_eq!(builder.clear(&EventPattern::exact("greet")), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pattern` | `&EventPattern` | Kind/source/options identity, not an event to test for matches. |

**Returns** The number removed, including zero. Hooks remain selected; no registration error is
returned.

### 3.13. EmitterBuilder::extend

`extend(&mut self, other: Self) -> Result<()>`

Move another builder's listeners after existing equal-priority contributions while retaining
imported IDs, sites, compiled matchers, and internal tie order.

```rust
let mut plugin = EmitterBuilder::new();
let imported = plugin.on("greet", greet)?;
builder.extend(plugin)?;
assert!(builder.remove(&imported)?);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `other` | `EmitterBuilder` | Consumed contribution builder; its hooks/depth policy are not imported. |

**Returns** Unit, or sequence-exhaustion error before contributions are moved into the receiver.
Receiver hooks and depth policy apply to all resulting listeners.

### 3.14. EmitterBuilder::build

`build(self) -> EventEmitter`

Consume registration state and sort once by descending priority, then ascending contribution order.

```rust
builder.on("greet", greet)?;
let emitter = builder.build();
assert_eq!(emitter.listeners().count(), 1);
```

**Returns** An immutable emitter owning its callbacks and hooks. No callbacks run, and no runtime
registration API is exposed.

## 4. EventEmitter operations

The immutable table can be shared; requests/responses remain exclusively borrowed per call. Event
input must be nonempty and at most 8,192 bytes. No operation automatically loads requests, delivers
responses, wraps general lifecycle events, spawns work, or converts missing listeners into HTTP 404.

### 4.1. EventEmitter::listeners

`listeners(&self) -> impl Iterator<Item = &ListenerDescriptor>`

Inspect read-only descriptors in execution order.

```rust
builder.on("greet", greet)?;
let emitter = builder.build();
assert_eq!(emitter.listeners().next().unwrap().priority, 0);
```

**Returns** A borrowing iterator of descriptors, including duplicate registrations. Callback storage
is not exposed.

### 4.2. EventEmitter::matches

`matches(&self, event: &str) -> Result<Vec<EventMatch>>`

Collect all matching registrations in execution order without executing them.

```rust
builder.on("greet", greet)?;
let emitter = builder.build();
assert_eq!(emitter.matches("greet")?.len(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Nonempty event, at most 8,192 bytes. |

**Returns** Owned per-registration match metadata, or invalid-input error for empty/oversized
events. Empty matches are an empty vector, not an error; exactness gives no extra priority.

### 4.3. EventEmitter::emit

`emit<'a>(&'a self, event: &'a str, req: &'a mut Request, res: &'a mut Response) -> HandlerFuture<'a, Result<DispatchOutcome>>`

Dispatch all matches against the caller's exact mutable pair, preserving partial changes after
abort/failure/cancellation.

```rust
builder.on("greet", greet)?;
let emitter = builder.build();
let mut req = Request::new();
let mut res = Response::new();
assert_eq!(
    emitter.emit("greet", &mut req, &mut res).await?,
    DispatchOutcome::Completed
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Event under the table's input contract. |
| `req` | `&'a mut Request` | Caller-owned mutable input. |
| `res` | `&'a mut Response` | Caller-owned mutable output. |

**Returns** A borrowing Send future yielding `Completed`, `Aborted`, or `NoListener`. Input/depth
errors and callback failures return `Err`; failed phases gain invocation frames. Awaiting does not
set an implicit response status.

### 4.4. EventEmitter::resolve

`resolve<'a, I: IntoRequest + 'a>(&'a self, event: &'a str, input: I) -> HandlerFuture<'a, Result<Resolution>>`

Move input into a capability call, create a fresh response, and return its public envelope
separately from execution outcome.

```rust
builder.on("greet", greet)?;
let emitter = builder.build();
let call = emitter.resolve("greet", serde_json::json!({})).await?;
assert_eq!(call.response.code, Some(200));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Event under the table's input contract. |
| `input` | `I: IntoRequest + 'a` | Owned Request, Nest, or object-root serde_json::Value. |

**Returns** A borrowing Send future yielding `Resolution`. Conversion occurs when polled. Input
conversion or dispatch failures return `Err`, not error envelopes; NoListener keeps code unset
unless a response was explicitly set during work. Native values remain rich in-process values.

## 5. Invocation and input adapter

`Invocation<'a>` is a non-Clone immutable borrowed context valid only during a callback. Its
metadata is local to that listener/phase and remains stable across child awaits. The following
snippets are handler-body fragments; surrounding registration and boilerplate are omitted.

### 5.1. Invocation::param

`param(&self, name: &str) -> Option<&str>`

Read a present named capture without request-data injection.

```rust
let id = inv.param("id").unwrap_or("unknown");
res.results(&id)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact capture name. |

**Returns** Borrowed capture text, or None when missing.

### 5.2. Invocation::listener

`listener(&self) -> &InvocationFrame`

Inspect current contribution and phase provenance.

```rust
let label = inv.listener().label.clone();
res.results(&label)?;
```

**Returns** A borrowed current frame; see
[diagnostic fields](error.md#1-public-error-and-diagnostic-values).

### 5.3. Invocation::event

`event(&self) -> &str`

Read the actual emitted event for this invocation.

```rust
let event = inv.event().to_owned();
res.results(&event)?;
```

**Returns** Borrowed emitted text, unchanged by nested calls.

### 5.4. Invocation::captures

`captures(&self) -> &MatchData`

Read both positional and named match captures.

```rust
let args = inv.captures().args.clone();
res.results(&args)?;
```

**Returns** A borrowed immutable match record local to this listener.

### 5.5. Invocation::path

`path(&self) -> Vec<InvocationFrame>`

Materialize the root-first ancestry for explicit inspection.

```rust
let depth = inv.path().len();
res.results(&depth)?;
```

**Returns** An owned root-first vector including the current frame, allocated when requested.

### 5.6. Invocation::emit

`emit<'a>(&'a self, event: &'a str, req: &'a mut Request, res: &'a mut Response) -> HandlerFuture<'a, Result<DispatchOutcome>>`

Reborrow a pair for nested dispatch using this invocation's ancestry.

```rust
let child = inv.emit("audit", req, res).await?;
if child == DispatchOutcome::Aborted {
    // This parent explicitly propagates the cooperative child stop.
    return Ok(Flow::Abort);
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Child event. |
| `req` | `&'a mut Request` | Exclusive child input; often the parent's pair. |
| `res` | `&'a mut Response` | Exclusive child output. |

**Returns** The child dispatch outcome after awaiting, or a propagated input/depth/phase error.
Child Abort does not automatically abort its parent. Distinct concurrent children require distinct
mutable pairs.

### 5.7. Invocation::resolve

`resolve<'a, I: IntoRequest + 'a>(&'a self, event: &'a str, input: I) -> HandlerFuture<'a, Result<Resolution>>`

Resolve a child with fresh values while inheriting the current call path.

```rust
let child = inv
    .resolve("profile-read", serde_json::json!({"id": "42"}))
    .await?;
res.apply_envelope(child.response)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&'a str` | Child event. |
| `input` | `I: IntoRequest + 'a` | Owned input, with the same sealed choices as root resolve. |

**Returns** A child Resolution, or input/dispatch error. Separate child resolves own separate pairs
and can run concurrently; callers still choose how to handle NoListener/Aborted.

### 5.8. IntoRequest::into_request

`into_request(self) -> Result<Request>`

Use the sealed input adapter directly when constructing capability input. Implementations exist only
for owned Request, Nest, and serde_json::Value.

```rust
let req = serde_json::json!({"name": "Ada"}).into_request()?;
assert_eq!(req.data.path_as::<String>("name")?, Some("Ada".into()));
```

**Returns** The original Request moved unchanged, including load state; or a new default request
initialized from object-root data. Invalid JSON roots or construction limits return errors. Rich
`Value` is not directly an IntoRequest implementation; put it in an object-root Nest first.

## Related

[Events and routing guide](../guides/events-and-routing.md) · [Concepts](../concepts.md) ·
[Pattern](pattern.md) · [Queue](queue.md) · [Response](response.md) · [Error](error.md) ·
[API index](README.md)
