//! Construction contracts using real plugin functions, typed sections and startup tasks.
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Flow, Invocation, MethodPattern, Nest, Request, Response, Result};
use stackpress_server::{
    ConfigLayers, PluginSpec, RouteSpec, ServerBuilder, StartupContext, StartupSpec, StartupStage,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Preflight must precede this externally observable registration effect.
static REGISTRATIONS: AtomicUsize = AtomicUsize::new(0);
/// Count callback entry without publishing any runtime state.
fn counted(_: &mut ServerBuilder) -> Result<()> {
    // An observable counter detects any callback entry before graph preflight completed.
    REGISTRATIONS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}
/// Missing/order/cycle failures run no callbacks; valid selection preserves supplied order.
#[tokio::test]
async fn dependency_preflight() -> Result<()> {
    // Every invalid complete graph must fail before this test's callback counter can change.
    let cases = [
        vec![PluginSpec::new("a").requires(&["missing"])],
        vec![PluginSpec::new("a").requires(&["b"]), PluginSpec::new("b")],
        vec![
            PluginSpec::new("a").requires(&["b"]),
            PluginSpec::new("b").requires(&["a"]),
        ],
        vec![
            PluginSpec::new("a").after_if_present(&["b"]),
            PluginSpec::new("b"),
        ],
    ];
    for specs in cases {
        // Build each invalid graph in isolation so selection state cannot leak between cases.
        let mut app = ServerBuilder::new();
        for spec in specs {
            app.plugin_with(spec, counted)?;
        }
        let error = app.build().await.err().expect("invalid graph");
        assert_eq!(error.stage, StartupStage::Registration);
        assert!(error.plugin.is_some());
        assert!(error.registered.file.ends_with("construction.rs"));
        assert_eq!(REGISTRATIONS.load(Ordering::SeqCst), 0);
    }
    let mut app = ServerBuilder::new();
    assert!(
        app.plugin_with(PluginSpec::new("self").requires(&["self"]), counted)
            .is_err()
    );
    assert!(app.plugin("bad\nname", counted).is_err());
    app.plugin("a", counted)?;
    assert!(app.plugin("a", counted).is_err());
    app.plugin_with(
        PluginSpec::new("b")
            .requires(&["a"])
            .after_if_present(&["absent"]),
        counted,
    )?;
    let server = app.build().await.map_err(|e| e.error)?;
    assert_eq!(
        server.plugins().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(REGISTRATIONS.load(Ordering::SeqCst), 2);
    Ok(())
}

/// Field-level unknown-key policy remains owned by the section's serde type.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    nested: serde_json::Value,
    list: Vec<u8>,
    nullable: Option<String>,
    limit: u64,
}
/// Semantic validation occurs before startup, after the final typed decoding.
fn validate(value: &Settings) -> Result<()> {
    // A decoded but semantically invalid limit must be rejected before service startup.
    if value.limit == 0 {
        return Err(Error::invalid("limit must be positive"));
    }
    Ok(())
}
/// Defaults can be registered by a selected plugin after layers were supplied.
fn settings(app: &mut ServerBuilder) -> Result<()> {
    // Register defaults through the selected plugin so pre-supplied layers cannot bypass ownership.
    app.config_section(
        "settings",
        Settings {
            nested: serde_json::json!({"a":1,"b":2}),
            list: vec![1, 2],
            nullable: Some("default".into()),
            limit: 4,
        },
        validate,
    )
}
/// Objects merge recursively; null/arrays replace; unknown/invalid sections fail build.
#[tokio::test]
async fn typed_configuration() -> Result<()> {
    // Combine partial file/code layers against typed defaults, preserving null and array
    // replacement.
    let mut layers = ConfigLayers::new();
    layers.json(
        "file",
        r#"{"settings":{"nested":{"a":9},"list":[3],"nullable":null}}"#,
    )?;
    layers.value("code", serde_json::json!({"settings":{"limit":7}}))?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?.plugin("settings", settings)?;
    // Observe the final typed values only after registration, merge, decoding and validation all
    // succeed.
    let server = app.build().await.map_err(|e| e.error)?;
    let value = server.host().config::<Settings>()?;
    assert_eq!(value.nested, serde_json::json!({"a":9,"b":2}));
    assert_eq!(value.list, [3]);
    assert_eq!(value.nullable, None);
    assert_eq!(value.limit, 7);
    // Unknown roots, unknown typed fields, failed validation and null type mismatch all reject
    // build.
    for value in [
        serde_json::json!({"unknown":{}}),
        serde_json::json!({"settings":{"extra":1}}),
        serde_json::json!({"settings":{"limit":0}}),
        serde_json::json!({"settings":null}),
    ] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers.value("invalid", value)?;
        app.config_layers(layers)?.plugin("settings", settings)?;
        let error = app.build().await.err().expect("invalid configuration");
        assert_eq!(error.stage, StartupStage::Config);
        assert!(error.registered.file.ends_with("construction.rs"));
        assert!(error.label.contains("invalid"));
    }
    let mut app = ServerBuilder::new();
    settings(&mut app)?;
    assert!(settings(&mut app).is_err());
    Ok(())
}
/// Every invalid/budget-exceeding append retains the previous accepted layer sequence.
#[test]
fn bounded_configuration_is_atomic() -> Result<()> {
    // Accept one baseline layer and use the same owner for every rejected append.
    let mut layers = ConfigLayers::new();
    layers.json("ok", r#"{"s":{"key":1}}"#)?;
    assert!(layers.json("bad", "[]").is_err());
    assert!(layers.json("bad", "{").is_err());
    assert!(layers.value("bad\nlabel", serde_json::json!({})).is_err());
    let deep = format!("{{\"s\":{}0{}}}", "[".repeat(65), "]".repeat(65));
    assert!(layers.json("deep", &deep).is_err());
    assert!(
        layers
            .value("wide", serde_json::json!({"s":vec![0;10_001]}))
            .is_err()
    );
    assert!(layers.json("huge", &" ".repeat(1_048_577)).is_err());
    assert_eq!(layers.entries().count(), 1);
    let mut layers = ConfigLayers::new();
    layers.value("one", serde_json::json!({"s":"a".repeat(600_000)}))?;
    assert!(
        layers
            .value("two", serde_json::json!({"t":"b".repeat(600_000)}))
            .is_err()
    );
    assert_eq!(layers.entries().count(), 1);
    Ok(())
}
/// Register and replace typed services with explicit sharing and checked names.
async fn services(start: &mut StartupContext) -> Result<Flow> {
    // Explicit Arc counts distinguish retained ownership from silent duplicate replacement.
    assert!(start.replace_service(Arc::new(1_u64)).is_err());
    let first = Arc::new(2_u64);
    start.register(Arc::clone(&first))?;
    assert!(start.register(Arc::new(3_u64)).is_err());
    assert_eq!(Arc::strong_count(&first), 2);
    start.register_named("other", Arc::new(5_u64))?;
    assert!(
        start
            .register_named("other", Arc::new("wrong".to_owned()))
            .is_err()
    );
    assert!(start.service_named::<String>("other").is_err());
    assert_eq!(*start.service_named::<u64>("other")?, 5);
    start.replace_service(Arc::new(9_u64))?;
    assert_eq!(Arc::strong_count(&first), 1);
    assert!(start.on("too early", echo).is_err());
    assert!(start.get("/too-early", echo).is_err());
    Ok(Flow::Continue)
}
/// Services freeze in Listen; routes cannot be added before Route.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    // Config has completed, so service mutation and early route exposure must both be denied.
    assert!(start.register(Arc::new(1_u64)).is_err());
    assert!(start.replace_service(Arc::new(1_u64)).is_err());
    assert!(start.register_named("late", Arc::new(1_u64)).is_err());
    assert!(start.get("/early", echo).is_err());
    start.on("echo", echo)?;
    Ok(Flow::Continue)
}
/// Route cannot reopen operation or service registration.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    // Route may expose existing capabilities but cannot reopen operation/service registration.
    assert!(start.on("late", echo).is_err());
    assert!(start.register(Arc::new(1_u64)).is_err());
    start.get("/ok/:id", echo)?;
    Ok(Flow::Continue)
}
/// Plugins cannot expand their already-preflighted selection list.
fn phases(app: &mut ServerBuilder) -> Result<()> {
    // A plugin cannot expand the already-preflighted selected set while scheduling its own tasks.
    assert!(app.plugin("late", counted).is_err());
    app.on_config(services)?;
    app.on_listen(listen)?;
    app.on_route(routes)
}
/// Phase gates, immutable service access and replacement sites survive build.
#[tokio::test]
async fn services_and_phase_gates() -> Result<()> {
    // Build a real selected plugin through all phases before inspecting the frozen registry.
    let mut app = ServerBuilder::new();
    app.plugin("phases", phases)?;
    assert!(
        app.startup(StartupSpec::new(StartupStage::Registration), services)
            .is_err()
    );
    assert!(
        app.startup(
            StartupSpec::new(StartupStage::Config).label("bad\n"),
            services
        )
        .is_err()
    );
    // The immutable registry must retain the chosen type/name values and original/replacement
    // sites.
    let server = app.build().await.map_err(|e| e.error)?;
    assert_eq!(*server.host().service::<u64>()?, 9);
    assert_eq!(*server.host().service_named::<u64>("other")?, 5);
    assert!(server.host().service_named::<String>("other").is_err());
    assert!(server.host().service::<String>().is_err());
    let shared = server.host().shared_service::<u64>()?;
    assert!(std::ptr::eq(
        shared.as_ref(),
        server.host().service::<u64>()?
    ));
    let (site, replaced) = server.host().service_sites::<u64>()?;
    assert!(site.file.ends_with("construction.rs"));
    assert!(replaced.is_some());
    let result = server.resolve("GET /ok/7", Nest::new()).await?;
    assert_eq!(result.response.results.unwrap().to_json()?["id"], "7");
    Ok(())
}
/// Report aggregate capture data without adding another lifecycle.
async fn echo(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Snapshot aggregate data through a public response so capture precedence is observable.
    res.results(&req.data)?;
    Ok(Flow::Continue)
}
/// Replacing a host route retains capture policy and the original listener handle/site.
#[tokio::test]
async fn route_handles_and_capture_policy() -> Result<()> {
    // Register a real route handle, then replace/remove contributions through explicit authority.
    let mut app = ServerBuilder::new();
    let id = app.route_with(
        RouteSpec::new(MethodPattern::GET, "/items/:id").label("route"),
        echo,
    )?;
    app.replace(&id, echo)?;
    let removed = app.on("removed", echo)?;
    assert!(app.remove(&removed)?);
    assert!(!app.remove(&removed)?);
    let foreign = ServerBuilder::new().on("foreign", echo)?;
    assert!(app.replace(&foreign, echo).is_err());
    assert!(app.remove(&foreign).is_err());
    app.post("/post", echo)?;
    app.put("/put", echo)?;
    app.patch("/patch", echo)?;
    app.delete("/delete", echo)?;
    app.head("/head", echo)?;
    app.options("/options", echo)?;
    app.connect("/connect", echo)?;
    app.trace("/trace", echo)?;
    app.all("/any/:id", echo)?;
    assert!(
        app.listen(
            stackpress_lib::ListenerSpec::exact("x").label("bad\n"),
            echo
        )
        .is_err()
    );
    // Dispatch against the published table to verify provenance and capture-wrapper retention.
    let server = app.build().await.map_err(|e| e.error)?;
    let input = Request::builder()
        .query(serde_json::json!({"id":"query"}))
        .post(serde_json::json!({"id":"post"}))
        .data(serde_json::json!({"id":"explicit"}))
        .build()?;
    let reply = server.resolve("GET /items/route", input).await?;
    assert_eq!(reply.response.results.unwrap().to_json()?["id"], "route");
    let descriptor = server.listeners().find(|d| d.id == id).unwrap();
    assert!(descriptor.registered.file.ends_with("construction.rs"));
    assert!(descriptor.replaced.is_some());
    Ok(())
}

/// High priority Config tasks run before the equal-priority insertion-order pair.
#[tokio::test]
async fn stable_phase_priorities_and_failure_stops() -> Result<()> {
    // A user-owned trace observes priority order without any production registry lock.
    use stackpress_lib::queue::task;
    let trace = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut app = ServerBuilder::new();
    for (priority, name) in [(0, "first"), (10, "high"), (0, "last")] {
        let trace = Arc::clone(&trace);
        app.startup(
            StartupSpec::new(StartupStage::Config)
                .priority(priority)
                .label(name),
            task(move |_: &mut StartupContext| {
                let trace = Arc::clone(&trace);
                Box::pin(async move {
                    trace.lock().unwrap().push(name);
                    Ok(Flow::Continue)
                })
            }),
        )?;
    }
    app.build().await.map_err(|e| e.error)?;
    // The observed order must be priority-first with stable insertion ties.
    assert_eq!(*trace.lock().unwrap(), ["high", "first", "last"]);
    // Error and Abort at every phase must prevent publication with the correct task identity.
    for stage in [
        StartupStage::Config,
        StartupStage::Listen,
        StartupStage::Route,
    ] {
        for abort in [false, true] {
            let mut app = ServerBuilder::new();
            app.startup(
                StartupSpec::new(stage).label("failed step"),
                task(move |_: &mut StartupContext| {
                    Box::pin(async move {
                        if abort {
                            Ok(Flow::Abort)
                        } else {
                            Err(Error::new("original"))
                        }
                    })
                }),
            )?;
            let error = app
                .build()
                .await
                .err()
                .expect("partial host must not escape");
            assert_eq!(error.stage, stage);
            assert_eq!(error.label, "failed step");
        }
    }
    Ok(())
}

/// Composition moves handles and wrappers without cloning runtime ownership.
#[tokio::test]
async fn composed_builder_preserves_handles() -> Result<()> {
    // Move the source registrar into a target; its original handle must still replace the wrapper.
    let mut source = ServerBuilder::new();
    let route = source.get("/composed/:id", echo)?;
    let mut target = ServerBuilder::new();
    target.extend(source)?;
    target.replace(&route, echo)?;
    let server = target.build().await.map_err(|e| e.error)?;
    let result = server.resolve("GET /composed/yes", Nest::new()).await?;
    assert_eq!(result.response.results.unwrap().to_json()?["id"], "yes");
    assert_eq!(server.listeners().next().unwrap().id, route);
    Ok(())
}

/// Exact raw byte/layer limits accept the boundary and reject the next unit atomically.
#[test]
fn configuration_exact_boundaries() -> Result<()> {
    // Use exact byte/layer limits and reject only the next unit without changing accepted inputs.
    let mut layers = ConfigLayers::new();
    let mut text = "{}".to_owned();
    text.push_str(&" ".repeat(1_048_576 - 2));
    layers.json("exact raw bytes", &text)?;
    text.push(' ');
    assert!(layers.json("one byte too many", &text).is_err());
    let mut layers = ConfigLayers::new();
    for _ in 0..1024 {
        layers.json("small", "{}")?;
    }
    assert!(layers.json("one layer too many", "{}").is_err());
    assert_eq!(layers.entries().count(), 1024);
    Ok(())
}

/// Aggregate defaults and many small layers cannot bypass merged node/byte budgets.
#[tokio::test]
async fn merged_configuration_budgets_cover_defaults_and_layers() -> Result<()> {
    // Separate newtypes allow distinct section claims whose combined defaults exceed the budget.
    /// Newtypes permit two deliberately distinct package-owned string sections.
    #[derive(Serialize, Deserialize)]
    struct First(String);
    /// Concrete type identity remains separate from section naming.
    #[derive(Serialize, Deserialize)]
    struct Second(String);
    /// These sections have no semantic constraint beyond their host-enforced budget.
    fn valid<T>(_: &T) -> Result<()> {
        // These fixtures leave semantic validation unconstrained so only host budgets decide
        // failure.
        Ok(())
    }
    let mut app = ServerBuilder::new();
    app.config_section("first", First("a".repeat(600_000)), valid)?;
    app.config_section("second", Second("b".repeat(600_000)), valid)?;
    let error = app
        .build()
        .await
        .err()
        .expect("aggregate defaults exceed budget");
    assert_eq!(error.stage, StartupStage::Config);
    assert!(error.label.contains("second"));
    assert!(error.label.contains("defaults"));
    let mut deep = serde_json::Value::Null;
    for _ in 0..65 {
        deep = serde_json::Value::Array(vec![deep]);
    }
    assert!(
        ConfigLayers::new()
            .value("owned deep JSON", serde_json::json!({"root":deep}))
            .is_err()
    );
    let mut layers = ConfigLayers::new();
    layers.value("first", serde_json::json!({"first":vec![0;6000]}))?;
    assert!(
        layers
            .value("second", serde_json::json!({"second":vec![0;6000]}))
            .is_err()
    );
    assert_eq!(layers.entries().count(), 1);
    Ok(())
}

/// Failed aggregate append preserves the builder's previously accepted typed configuration.
#[tokio::test]
async fn rejected_composed_config_keeps_accepted_layers() -> Result<()> {
    // Each individual layer fits, but their cumulative encoded input exceeds the host budget.
    let mut accepted = ConfigLayers::new();
    accepted.value(
        "accepted",
        serde_json::json!({"settings":{"nested":"a".repeat(600_000)}}),
    )?;
    let mut excessive = ConfigLayers::new();
    excessive.value(
        "excessive",
        serde_json::json!({"settings":{"nested":"b".repeat(600_000)}}),
    )?;
    let mut app = ServerBuilder::new();
    app.plugin("settings", settings)?;
    app.config_layers(accepted)?;
    // Rejection must not replace the first value or spend its accounting a second time.
    assert!(app.config_layers(excessive).is_err());
    let host = app.build().await.map_err(|error| error.error)?;
    let typed = host.host().config::<Settings>()?;
    assert_eq!(typed.nested.as_str(), Some("a".repeat(600_000).as_str()));
    assert_eq!(typed.limit, 4);
    Ok(())
}
