# Listener and route API

ServerBuilder and StartupContext expose the same listener/route registration signatures. Builder
methods forward to StartupContext and retain the original caller site. Sources:
[registration.rs](../../src/registration.rs), [router.rs](../../src/router.rs).

```rust
use stackpress_lib::{
    Flow, Invocation, ListenerSpec, MethodPattern, Request, Response, Result,
};
use stackpress_server::{CapturePolicy, RouteSpec, ServerBuilder};

async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Registration rules](#1-registration-rules) · [2. Operations](#2-operations) · [3. Host
routes](#3-host-routes) · [4. Customization](#4-customization)

## 1. Registration rules

Operations may change in Registration or Listen; host routes may change in Registration or Route.
Config permits neither. Public methods below apply to both receivers; examples use ServerBuilder.
Lib owns Handler, ListenerSpec, ListenerId and matching/priority/tie semantics. Handler storage is
static; each call's mutable request/response and Send future remain borrowed.

Plugin identity overrides a supplied ListenerSpec plugin during selected-plugin contributions.
Duplicate matchers are independent listeners. Use IDs for customization; labels do not confer
replacement authority. See [Lib events](../../../../lib/docs/api/event.md) and
[patterns](../../../../lib/docs/api/pattern.md).

## 2. Operations

These methods add ordinary Lib listeners. A route-shaped pattern supplied to listen does not install
the host's named-capture injection wrapper.

### 2.1. on

`on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register an exact event at priority zero.

```rust
let id = app.on("greet", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Exact event name validated by Lib. |
| `action` | `impl Handler + 'static` | Named handler or Lib handler helper. |

**Returns** A stable ListenerId, or phase/pattern/label/sequence validation error from the
registrar/Lib.

### 2.2. on_priority

`on_priority(&mut self, event: &str, priority: i64, action: impl Handler + 'static) -> Result<ListenerId>`

Register an exact event with explicit ordering.

```rust
let id = app.on_priority("greet", 10, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | Exact event name. |
| `priority` | `i64` | Higher runs first; stable registration ties. |
| `action` | `impl Handler + 'static` | Operation handler. |

**Returns** A stable ListenerId or checked registration/phase error.

### 2.3. listen

`listen(&mut self, spec: ListenerSpec, action: impl Handler + 'static) -> Result<ListenerId>`

Register a full Lib listener specification.

```rust
let id = app.listen(ListenerSpec::exact("greet").label("greeting"), greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `ListenerSpec` | Exact/pattern matcher, priority, hooks, label, and provenance. |
| `action` | `impl Handler + 'static` | Main action. |

**Returns** A stable ListenerId or phase/Lib registration error; raw capture semantics stay
unchanged.

## 3. Host routes

NamedIntoData, the default CapturePolicy, stages literal-key capture overlay immediately before each
action. InvocationOnly leaves captures in Invocation. The enum is Clone/Copy/Debug/Default/
Eq/PartialEq. RouteSpec is Debug and has no public fields.

### 3.1. RouteSpec::new

`RouteSpec::new(method: MethodPattern, path: &str) -> Self`

Describe method/path matching with default injection.

```rust
let spec = RouteSpec::new(MethodPattern::GET, "/hello/:id");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `MethodPattern` | Checked method pattern including Any/extension verbs. |
| `path` | `&str` | Compiled later at route registration. |

**Returns** A zero-priority, unlabeled spec with NamedIntoData; construction is infallible.

### 3.2. RouteSpec::priority

`priority(self, priority: i64) -> Self`

Choose listener execution order.

```rust
let spec = RouteSpec::new(MethodPattern::GET, "/hello").priority(10);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `priority` | `i64` | Higher first, ties preserve insertion. |

**Returns** The owned spec; specificity does not change order.

### 3.3. RouteSpec::label

`label(self, label: impl Into<String>) -> Self`

Attach a diagnostic identity.

```rust
let spec = RouteSpec::new(MethodPattern::GET, "/hello").label("hello");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `label` | `impl Into<String>` | Lib validates the label when installed. |

**Returns** The owned spec; it does not replace another labeled listener.

### 3.4. RouteSpec::captures

`captures(self, policy: CapturePolicy) -> Self`

Choose whether named captures enter aggregate data.

```rust
let spec =
    RouteSpec::new(MethodPattern::GET, "/hello/:id").captures(CapturePolicy::InvocationOnly);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `policy` | `CapturePolicy` | NamedIntoData or InvocationOnly. |

**Returns** The owned spec with selected wrapper policy, retained across handle replacement.

### 3.5. route

`route(&mut self, method: MethodPattern, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a default host route.

```rust
let id = app.route(MethodPattern::GET, "/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `MethodPattern` | Method matcher. |
| `path` | `&str` | Lib path grammar. |
| `action` | `impl Handler + 'static` | Invoked after successful capture overlay. |

**Returns** A stable ID or phase/pattern/registration error; registering opens no endpoint.

### 3.6. route_with

`route_with(&mut self, spec: RouteSpec, action: impl Handler + 'static) -> Result<ListenerId>`

Install a route with explicit capture/ordering metadata.

```rust
let id = app.route_with(
    RouteSpec::new(MethodPattern::GET, "/hello").priority(10),
    greet,
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `RouteSpec` | Consumed host route policy. |
| `action` | `impl Handler + 'static` | Wrapped route action. |

**Returns** A stable ID or phase/pattern/label/sequence error. Named injection is atomic under
req.data limits; route values win while query/post remain unchanged.

### 3.7. get

`get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a GET host route at priority zero.

```rust
let id = app.get("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to route(MethodPattern::GET,
path, action).

### 3.8. post

`post(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a POST host route at priority zero.

```rust
let id = app.post("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::POST, path, action).

### 3.9. put

`put(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a PUT host route at priority zero.

```rust
let id = app.put("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to route(MethodPattern::PUT,
path, action).

### 3.10. patch

`patch(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a PATCH host route at priority zero.

```rust
let id = app.patch("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::PATCH, path, action).

### 3.11. delete

`delete(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a DELETE host route at priority zero.

```rust
let id = app.delete("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::DELETE, path, action).

### 3.12. head

`head(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a HEAD host route at priority zero.

```rust
let id = app.head("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::HEAD, path, action).

### 3.13. options

`options(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a OPTIONS host route at priority zero.

```rust
let id = app.options("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::OPTIONS, path, action).

### 3.14. connect

`connect(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a CONNECT host route at priority zero.

```rust
let id = app.connect("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::CONNECT, path, action).

### 3.15. trace

`trace(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a TRACE host route at priority zero.

```rust
let id = app.trace("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to
route(MethodPattern::TRACE, path, action).

### 3.16. all

`all(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId>`

Register a Any host route at priority zero.

```rust
let id = app.all("/hello/:id", greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `&str` | Lib route path pattern. |
| `action` | `impl Handler + 'static` | Route action using default NamedIntoData policy. |

**Returns** A stable ID or the same checked errors as route. Equivalent to route(MethodPattern::Any,
path, action).

## 4. Customization

The current listener owner is checked by Lib. A composed builder retains imported handles; foreign
IDs error and a removed owned ID remains distinguishable from a foreign one.

### 4.1. replace

`replace(&mut self, id: &ListenerId, action: impl Handler + 'static) -> Result<()>`

Replace an owned action without changing matching, priority, ID, or capture policy.

```rust
let id = app.get("/hello", greet)?;
app.replace(&id, greet)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&ListenerId` | Owned still-registered listener. |
| `action` | `impl Handler + 'static` | New main action; existing hooks/provenance remain. |

**Returns** Unit or phase/foreign/removed-handle error. Host routes re-wrap the replacement with
their existing capture policy and retain replacement-site metadata.

### 4.2. remove

`remove(&mut self, id: &ListenerId) -> Result<bool>`

Remove an owned contribution during its permitted phase.

```rust
let id = app.on("greet", greet)?;
assert!(app.remove(&id)?);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&ListenerId` | Owned listener ID. |

**Returns** True if removed, false if already removed, or foreign-handle/phase error. A removed
route no longer has stored host capture policy; later removal uses the ordinary-listener phase gate.

Related: [route guide](../guides/operation-routes.md), [startup](startup.md).
