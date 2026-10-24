# Register events and route to operations

A route should be able to reuse an operation without hiding where its input came from. Register the
operation first, then make a route handler pass its capture into a nested capability call.

## Connect a route to a named operation

Add these declarations to an async consumer using the dependencies from
[Getting started](../getting-started.md):

```rust
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Flow, Invocation, MethodPattern, Request, Response, Result,
};

async fn profile(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    res.results(&serde_json::json!({"id": id}))?;
    Ok(Flow::Continue)
}

async fn profile_route(
    _: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Only the selected capture enters the child operation's plain input.
    let id = inv.param("id").unwrap_or_default();
    let child = inv
        .resolve("profile-read", serde_json::json!({"id": id}))
        .await?;
    if child.outcome == DispatchOutcome::Aborted {
        // Propagating a child abort is an explicit parent decision.
        return Ok(Flow::Abort);
    }
    res.apply_envelope(child.response)?;
    Ok(Flow::Continue)
}
```

The operation reads request data; the route reads invocation captures. `inv.resolve` creates a new
input/output pair and preserves the parent diagnostic path. The response envelope transfers the
child's public result without transferring delivery state.

Inside an async function returning `Result<()>`, register and verify them:

```rust
let mut builder = EmitterBuilder::new();
builder.on("profile-read", profile)?;
builder.route(MethodPattern::GET, "/profiles/:id", profile_route)?;
let emitter = builder.build();
let mut req = Request::new();
let mut res = Response::new();
assert_eq!(
    emitter.emit("GET /profiles/42", &mut req, &mut res).await?,
    DispatchOutcome::Completed,
);
assert_eq!(res.try_to_json()?["results"]["id"], "42");
assert!(!req.data.has("id"));
```

The emitted string matches the route, but no HTTP endpoint is created. The original request stays
free of injected captures because only the child received `id`.

## Choose matching and order explicitly

`on` always matches a literal string. For token patterns, call
`on_pattern(EventPattern::express("audit-**", '-')?, handler)`; for regex, use `EventPattern::regex`
with Rust regex syntax. Route methods are uppercase event tokens; `MethodPattern::Any` is the
wildcard choice.

Every matching listener runs, including exact matches and fallbacks. Add priorities when order
matters; ties retain registration order across matcher kinds. If a fallback should stop or avoid
replacing an earlier result, express that policy in the handler.

## Customize during registration

Keep the `ListenerId` returned by registration to replace or remove that contribution before
`build`. `replace` retains its identity, matcher, priority, and position. `extend` moves another
builder's listeners after existing ties, preserving imported handles; receiver hooks apply and
source hooks are not imported.

If configuration must be captured, use the secondary closure form:

```rust
use stackpress_lib::handler;

let label = String::from("audit");
let mut builder = EmitterBuilder::new();
builder.on(
    "audit",
    handler(move |_, res, _| {
        // Copy owned configuration per invocation; the stored callback remains reusable.
        let label = label.clone();
        Box::pin(async move {
            res.results(&label)?;
            Ok(Flow::Continue)
        })
    }),
)?;
```

`handler` supplies the borrowing closure's lifetime context. `Box::pin` produces its required
`HandlerFuture`, a pinned boxed `Send` future; it does not extend any borrow. Prefer named
`async fn` when no capture is needed.

You have connected route captures to an operation through an explicit nested call. Use
[event API](../api/event.md) for registration and dispatch, [pattern API](../api/pattern.md) for
grammar and limits, and [startup queues](startup-with-task-queues.md) when registration itself has
ordered stages.
