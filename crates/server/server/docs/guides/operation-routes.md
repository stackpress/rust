# Exposing an operation as a route

Keep a capability's input rules in one handler, then delegate to it from an explicit method/path.
The host places named route captures in aggregate data before the route action runs.

## Delegate with the same request

This example exposes `profile-search` at one GET path and proves that the captured ID wins a query
collision. It constructs values in process and opens no port:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt};

async fn search(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&req.data.path_as::<String>("id")?.unwrap_or_default())?;
    Ok(Flow::Continue)
}

async fn expose(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("profile-search", req, res).await
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.on("profile-search", search)?;
    app.get("/profiles/:id", expose)?;
    let server = app.build().await?;
    let mut req = Request::builder()
        .url("/profiles/abc123?id=other")
        .build()?;
    let mut res = Response::new();
    let report = server.handle_route(&mut req, &mut res).await?;
    assert_eq!(report.event, "GET /profiles/abc123");
    assert_eq!(
        res.result().and_then(|value| value.as_str()),
        Some("abc123")
    );
    assert_eq!(req.query.path_as::<String>("id")?, Some("other".into()));
    Ok(())
}
```

`delegate` preserves child errors and Abort without repeating Request/Response lifecycle hooks.
Route injection changes literal keys in `req.data`; query/post stores stay unchanged, and positional
captures stay in Invocation. A later matching route action overlays its own captures before it runs.

## Choose capture and ordering policy

Use `RouteSpec` for priority, label, or `CapturePolicy::InvocationOnly`. Duplicate routes remain
independent all-matching listeners: priority and registration order decide execution, not
specificity. Replacing a host route by ListenerId retains its capture policy. Registration via
ordinary `listen` uses Lib semantics and does not add the host route wrapper.

Authorization belongs to your exposed surface and operation policy. Query/body/header data does not
select an arbitrary operation in the HTTP adapter. The example explicitly exposes one capability.

You have reused a capability with route data and one lifecycle. Continue with [buffered
HTTP](using-http.md), or consult [registration](../api/registration.md) and [Lib
patterns](../../../../lib/docs/api/pattern.md) for exact matching grammar.
