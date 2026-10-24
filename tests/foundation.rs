//! Real facade consumers and an explicit bootstrap-to-immutable-runtime mapping.

use stackpress::lib::queue::{QueueOutcome, TaskQueue};
use stackpress::{
    DispatchOutcome, EmitterBuilder, EventEmitter, Flow, Invocation, MethodPattern, Request,
    Response, Result,
};

/// A typed result exercises facade Serialize-based helpers without a generic envelope cast.
#[derive(serde::Serialize)]
struct FeatureResult {
    id: String,
    value: u64,
}

/// Primary named operation works from any host's independently constructed request.
async fn feature_run(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A named facade consumer reads typed data and serializes a real result struct without
    // framework internals.

    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    res.results(&FeatureResult { id, value: 2 })?;
    Ok(Flow::Continue)
}
/// A route explicitly chooses capture injection and nested capability conversion.
async fn feature_route(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Explicitly choose one named capture and pass plain child data; raw emit never injects it.

    let id = inv.param("id").unwrap_or_default();
    let result = inv
        .resolve("feature-run", serde_json::json!({"id":id}))
        .await?;
    res.apply_envelope(result.response)?;
    Ok(Flow::Continue)
}
/// Minimal function plugin; no required class/trait/layout ceremony.
fn feature_plugin(builder: &mut EmitterBuilder) -> Result<()> {
    // A plugin is an ordinary registration function and needs no required trait or runtime loader.

    builder.on_priority("feature-run", 10, feature_run)?;
    Ok(())
}
/// Mutable host-owned startup data is deliberately separate from runtime Invocation.
struct Startup {
    builder: EmitterBuilder,
    phases: Vec<&'static str>,
}
/// Configuration phase has access to mutable host construction state.
async fn config(startup: &mut Startup) -> Result<Flow> {
    // Configuration runs against host-owned mutable startup state before the table is frozen.

    startup.phases.push("config");
    Ok(Flow::Continue)
}
/// Upstream on-listen registration maps to an ordered startup task calling a plugin.
async fn listen(startup: &mut Startup) -> Result<Flow> {
    // The startup listen phase may register capability handlers through the same explicit builder.

    startup.phases.push("listen");
    feature_plugin(&mut startup.builder)?;
    Ok(Flow::Continue)
}
/// Request-facing registration follows capability contributions.
async fn route(startup: &mut Startup) -> Result<Flow> {
    // Only after capability contributions does this host example add request-facing event patterns.

    startup.phases.push("route");
    startup
        .builder
        .route(MethodPattern::GET, "/feature/:id", feature_route)?;
    Ok(Flow::Continue)
}
/// No server crate/listener is needed to prove this construction-time composition.
async fn bootstrap() -> Result<EventEmitter> {
    // Run named startup tasks in order, then consume the builder once into its immutable runtime
    // table.

    let mut startup = Startup {
        builder: EmitterBuilder::new(),
        phases: Vec::new(),
    };
    let mut tasks = TaskQueue::new();
    tasks.add(config, 30)?.add(listen, 20)?.add(route, 10)?;
    assert_eq!(tasks.run(&mut startup).await?, QueueOutcome::Completed);
    assert_eq!(startup.phases, ["config", "listen", "route"]);
    Ok(startup.builder.build())
}
/// The facade and direct foundation expose the same operation/value contract.
#[tokio::test]
async fn bootstrap_named_plugin_and_plain_data_resolve_through_facade() -> Result<()> {
    // Exercise both capability and route consumers through facade exports, with no server adapter
    // or listener.

    let emitter = bootstrap().await?;
    let capability = emitter
        .resolve("feature-run", serde_json::json!({"id":"abc123"}))
        .await?;
    assert_eq!(capability.outcome, DispatchOutcome::Completed);
    assert_eq!(
        capability.response.try_to_json()?["results"],
        serde_json::json!({"id":"abc123","value":2})
    );
    // A separate raw dispatch proves route matching and child resolve without automatic
    // request-data capture injection.
    let mut req = Request::new();
    let mut res = Response::new();
    assert_eq!(
        emitter
            .emit("GET /feature/route-id", &mut req, &mut res)
            .await?,
        DispatchOutcome::Completed
    );
    assert_eq!(
        res.to_envelope().try_to_json()?["results"]["id"],
        "route-id"
    );
    // The original request must remain free of a capture that only the child capability explicitly
    // received.
    assert!(
        req.data.path("id").is_none(),
        "raw dispatch must not inject route captures"
    );
    Ok(())
}
