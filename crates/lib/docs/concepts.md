# Concepts

When several plugins contribute to one operation, their order and ownership need to be predictable.
Think of the emitter as an ordered list of matching contributions: each receives the same mutable
request and response, plus its own immutable description of the current call.

## Contents

 - [Events, listeners, and registration](#events-listeners-and-registration)
 - [Outcomes and phases](#outcomes-and-phases)
 - [Emit, resolve, and nesting](#emit-resolve-and-nesting)
 - [Invocation and captures](#invocation-and-captures)
 - [Requests and responses](#requests-and-responses)
 - [Nest, Value, and explicit sharing](#nest-value-and-explicit-sharing)
 - [Differences from upstream](#differences-from-upstream)

## Events, listeners, and registration

An event is a string such as `profile-search`. A listener contributes a handler and a matcher.
Exact, Express-style, regex, and route matchers all participate in one ordering rule: higher signed
priority runs first; ties run in registration order. Exact matches gain no special precedence, and
duplicate registrations run separately.

For example, inside a registration function with named handlers `check` and `search`:

```rust
builder.on_priority("profile-search", 10, check)?;
builder.on("profile-search", search)?;
let emitter = builder.build();
```

`check` runs before `search` because its priority is 10 rather than 0. Building consumes the mutable
accumulator. Removal, replacement, composition, and hook selection all happen on `EmitterBuilder`;
runtime dispatch uses an immutable `EventEmitter` without a registry lock.

## Outcomes and phases

Each selected listener runs the configured before hook, its action, and the configured after hook,
awaiting one phase before entering the next. The hooks run for each matched listener. Replacing a
hook during registration replaces the single hook for that phase; it does not append another one.

| Phase result | Effect |
| --- | --- |
| `Ok(Flow::Continue)` | Enter the next phase or listener |
| `Ok(Flow::Abort)` | Return `DispatchOutcome::Aborted`; skip all remaining work |
| `Err(error)` | Return that failure with invocation frames; skip all remaining work |

If there are no matches, dispatch returns `NoListener` without running hooks. If every phase
continues, it returns `Completed`. The after hook is a success-path phase, so it does not run after
an action abort or error.

None of these outcomes sets a response code or sends output. Abort also preserves completed
mutations and external effects; dropping a pending dispatch stops future phases without rollback or
async cleanup. Application panics propagate normally.

## Emit, resolve, and nesting

Use `emit` when the caller must retain and inspect its exact request and response. Use `resolve` for
a capability call with owned input and a fresh output pair:

```rust
// Inside an async function returning Result<()>, with an existing emitter.
let mut req = Request::new();
let mut res = Response::new();
let outcome = emitter.emit("profile-search", &mut req, &mut res).await?;

let call = emitter
    .resolve("profile-search", serde_json::json!({"name": "Ada"}))
    .await?;
```

The first call exposes handler mutations through `req` and `res`; the second returns a response
envelope alongside its outcome. Resolve accepts an owned `Request`, `Nest`, or object-root
`serde_json::Value`. It does not automatically load input or deliver output.

Within a handler, call `inv.emit` or `inv.resolve` to inherit the parent invocation path. A child
abort is an ordinary outcome that the parent chooses to propagate or handle; a child error
propagates through `?`. Recursive depth defaults to 64 and can be configured from 1 through 256
before building. Concurrent children need distinct mutable pairs, or separate `resolve` inputs.

Handler futures are `Send`, meaning they may move between threads. They still borrow their inputs:
`Send` does not make a future `'static`. If an executor requires owned spawned work, move ownership
of the emitter and values into the outer task yourself. The foundation never spawns or detaches
handlers.

## Invocation and captures

`Invocation` describes this listener's event, phase, metadata, captures, and root-first call path.
It remains stable across nested awaits. For a route `/profiles/:id`, a named handler can inspect its
capture directly:

```rust
let id = inv.param("id").unwrap_or("unknown");
res.results(&id)?;
```

The ID belongs to this listener's match. Raw dispatch leaves `req.data` alone, so two matching
patterns cannot overwrite each other's captured names there. Passing a capture to a child is an
explicit operation shown in [Events and routing](guides/events-and-routing.md).

## Requests and responses

`Request` holds separate query, post, aggregate data, headers, session, URL, method, and a buffered
body. Construction overlays query, then post, then explicit data at the top level. Later edits to
query or post do not automatically synchronize aggregate data.

Loading and delivery each track one attempt. Success permits repeated calls as a no-op; failure or
cancellation prevents retry through the same owner. Request/Response are not `Clone`. Their
data-only views are clonable snapshots, so adapters can inspect values without duplicating read/send
ownership.

A `ResponseEnvelope` carries public status/results/error fields. It omits headers, session, MIME,
metadata, private causes, delivery state, and retained limits. `Response::try_to_json` uses the
owning response's limits automatically. After extracting an envelope, use `try_to_json_with` to
select a wire policy; its plain `try_to_json` uses defaults. JSON conversion remains explicit and
rejects unsupported rich values. Learn the conversion choices and state transitions in
[Requests and responses](guides/requests-and-responses.md).

## Nest, Value, and explicit sharing

`Nest` is an owned object-root store with literal-key and path operations. `Value` distinguishes
null, booleans, finite JSON numbers, strings, arrays, objects, timestamps, bytes, files, and shared
native payloads. Missing lookups return `None`; present null is a value.

Cloning ordinary data copies its contents. Only `Value::Shared(SharedNative)` retains shared
allocation identity. Native access is an immutable typed downcast; any interior synchronization is
chosen by the application's payload type. Dates, bytes, files, and shared natives require a
deliberate wire representation before JSON output.

## Differences from upstream

The Rust surface preserves ordered contributions while making boundaries explicit. The main
migration choices are construction-time registration, named async functions with `Send` futures,
captures in `Invocation`, explicit shared native values, checked parsers with finite limits, scoped
cookie tombstones, and one-attempt I/O callbacks.

JavaScript callable stores, dynamic executable imports, reflection, and numeric status codes as
dispatch outcomes are not Rust APIs here. Top-level `request`, selected-operation, `response`, and
`error` orchestration belongs to a host; these names are ordinary events in this crate.

You should now recognize which state belongs to registration, one invocation, the caller's values,
and a boundary adapter. Continue with a [task guide](README.md#common-tasks), or use the
[API index](api/README.md) to look up an individual operation.
