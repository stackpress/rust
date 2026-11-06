//! Real nine-stage native application shared by direct, aggregate, facade and disabled-API builds.
use crate::{framework, generated, pages_host};
use serde_json::{Value, json};
use stackpress_framework_schema::Model;
use stackpress_framework_session::columns::SourceSchema;
use stackpress_framework_session::{
    Service,
    auth::{self, Hooks},
    storage::AuthActions,
};
use stackpress_framework_sql::operations::{Operation, OperationPolicy, Policy};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[path = "source_family.rs"]
pub mod source_family;

/// This application's generated Note capabilities explicitly allow its finite local calls.
struct LocalPolicy;
impl OperationPolicy for LocalPolicy {
    /// This application allows only its generated Note operation family.
    fn authorize(&self, model: &'static str, _: Operation, _: &Request) -> Result<()> {
        assert_eq!(model, "Note");
        Ok(())
    }
}

/// Counters observe IO and callbacks without supplying a framework-wide security policy.
#[derive(Default)]
struct Calls {
    /// Explicit command execution count; registration must leave it zero.
    commands: AtomicUsize,
    /// Real startup stages observed on both sides of composition.
    phases: Mutex<Vec<String>>,
}
/// Count selection of the one real generated admin callback, independently of startup phases.
static ADMIN_CALLS: AtomicUsize = AtomicUsize::new(0);

/// The app's generated registrar is real codegen output and remains last in the source catalog.
fn admin(app: &mut ServerBuilder) -> Result<()> {
    ADMIN_CALLS.fetch_add(1, Ordering::SeqCst);
    pages_host::register(app)
}

/// Compiled source-client metadata includes real account model definitions.
fn describe() -> Vec<Model> {
    vec![
        stackpress_framework_session::profile::ProfileSchema::describe(),
        auth::AuthSchema::describe(),
    ]
}

/// Application-owned immutable model catalog; missing callbacks remain absent.
static CLIENT: framework::CompiledClient = framework::CompiledClient {
    bindings: framework::FrameworkBindings {
        session: None,
        api: None,
    },
    build: "slice11-private-client-v1",
    describe,
    models: &[
        framework::CompiledModel {
            name: "Note",
            listen: Some(stackpress_framework_sql::plugin::register),
            admin: Some(admin),
        },
        framework::CompiledModel {
            name: "Account",
            listen: Some(stackpress_framework_session::account::events::register_listen),
            admin: None,
        },
        #[cfg(feature = "api")]
        framework::CompiledModel {
            name: "Api",
            listen: Some(stackpress_framework_api::sql::register),
            admin: None,
        },
    ],
};

/// The ordinary plugin function closes over a static, statically compiled catalog.
fn aggregate(app: &mut ServerBuilder) -> Result<()> {
    framework::register_with(app, &CLIENT)
}

/// Observe the real host phase at both sides of source selection, without claiming route ownership.
fn phase(app: &mut ServerBuilder, calls: &Arc<Calls>, after: bool) -> Result<()> {
    for stage in [
        stackpress_server::StartupStage::Config,
        stackpress_server::StartupStage::Listen,
        stackpress_server::StartupStage::Route,
    ] {
        let calls = calls.clone();
        app.startup(
            stackpress_server::StartupSpec::new(stage),
            task(move |start: &mut StartupContext| {
                let calls = calls.clone();
                Box::pin(async move {
                    calls
                        .phases
                        .lock()
                        .unwrap()
                        .push(format!("{:?}:{after}", start.stage()));
                    Ok(Flow::Continue)
                })
            }),
        )?;
    }
    Ok(())
}

/// Independently compose the exact same ordinary functions, without calling the aggregate registrar.
fn direct(app: &mut ServerBuilder) -> Result<()> {
    framework::config::register(app)?;
    framework::commands::register_server(app)?;
    framework::client::register(app, Some(&CLIENT))?;
    framework::language::register(app)?;
    stackpress_framework_csrf::plugin::register(app)?;
    framework::commands::register_sql(app)?;
    for model in CLIENT.models {
        if let Some(register) = model.listen {
            register(app)?;
        }
    }
    framework::view::register(app)?;
    stackpress_framework_session::composite::register(app)?;
    #[cfg(feature = "api")]
    stackpress_framework_api::register(app)?;
    for model in CLIENT.models {
        if let Some(register) = model.admin {
            register(app)?;
        }
    }
    Ok(())
}

/// Explicit finite command provider: invoked only by the matching command, never at startup.
async fn command(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Calls>()?
        .commands
        .fetch_add(1, Ordering::SeqCst);
    res.results(&req.data.to_json()?)?;
    Ok(Flow::Continue)
}

/// Cooperative duplicate handlers retain source FIFO and any-handler Abort behavior.
async fn first(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"first")?;
    Ok(Flow::Continue)
}
/// Stop the cooperative chain after preserving the second handler's response mutation.
async fn second(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"second")?;
    Ok(Flow::Abort)
}

/// Resolve a raw event and retain only the deterministic projected response for comparison.
async fn resolve(server: &Server, name: &str, input: Value) -> Result<Value> {
    let mut req = Request::builder().data(input).build()?;
    let mut res = Response::new();
    assert_ne!(
        server.emit(name, &mut req, &mut res).await?,
        stackpress_lib::DispatchOutcome::NoListener,
        "missing fixture event {name}"
    );
    res.try_to_json()
}

/// Open one finite file explicitly, prove the real application, then shut down all native work.
pub async fn prove(
    use_direct: bool,
) -> std::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    source_family::prove(use_direct).await?;
    let directory = tempfile::tempdir()?;
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(directory.path().join("application.sqlite"))
                .create_if_missing(true),
        )
        .await?,
    ));
    for table in generated::stores::schema::definition()? {
        engine
            .execute_plan(&table.create().render(engine.dialect())?)
            .await?;
    }
    let accounts = AuthActions::new(
        engine.clone(),
        stackpress_framework_session::storage::AuthStore::new(
            "DATABASE-SEED-MUST-NOT-REACH-BROWSER".into(),
            0,
        ),
    );
    let profiles = stackpress_framework_session::storage::ProfileActions::new(
        engine.clone(),
        stackpress_framework_session::storage::ProfileStore::new(
            "DATABASE-SEED-MUST-NOT-REACH-BROWSER".into(),
            0,
        ),
    );
    profiles.install().await?;
    accounts.install().await?;
    profiles
        .create(&serde_json::from_value(
            json!({"id":"person","name":"Finite Person","roles":["USER"]}),
        )?)
        .await?;
    #[cfg(feature = "api")]
    {
        stackpress_framework_api::models::ApplicationActions::new(
            engine.clone(),
            stackpress_framework_api::models::ApplicationStore::new("".into(), 0),
        )
        .install()
        .await?;
        stackpress_framework_api::models::SessionActions::new(
            engine.clone(),
            stackpress_framework_api::models::SessionStore::new("".into(), 0),
        )
        .install()
        .await?;
    }
    let calls = Arc::new(Calls::default());
    let resources = engine.clone();
    let counters = calls.clone();
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    let mut config = json!({"session":{"key":"identity","seed":"SIGNING-SEED-MUST-NOT-REACH-BROWSER"},"auth":{},"email":{"jsonTransport":true,"auth":{"user":"finite","pass":"SMTP-PASSWORD-MUST-NOT-REACH-BROWSER"}},"language":{"locale":"en_US","languages":{"en_US":{"Greeting":"Hello"}}},"view":{},"client":{"tsconfig":"native-source.json"}});
    #[cfg(feature = "api")]
    {
        config["api"] = json!({"endpoints":[{"method":"GET","route":"/api/notes","type":"public","event":"note.search"}]});
    }
    // Keep the same source config shape even when the optional API stage is compiled out.
    let _ = &mut config;
    layers.value("application", config)?;
    app.config_layers(layers)?;
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = resources.clone();
        let calls = counters.clone();
        Box::pin(async move {
            assert!(
                start
                    .service::<framework::client::Client>()?
                    .get(false)?
                    .is_some()
            );
            start.register(engine)?;
            start.register(calls)?;
            start.register(Arc::new(Policy(Arc::new(LocalPolicy))))?;
            start.register(Arc::new(pages_host::Mount::new("/admin/")?))?;
            start.register(Arc::new(auth::Environment {
                database_seed: Some("DATABASE-SEED-MUST-NOT-REACH-BROWSER".into()),
                email_configured: true,
                ..Default::default()
            }))?;
            let mut bindings = framework::commands::Bindings::default();
            for name in ["serve", "develop", "generate", "output"] {
                bindings.bind(name, command);
            }
            start.register(Arc::new(bindings))?;
            Ok(Flow::Continue)
        })
    }))?;
    ADMIN_CALLS.store(0, Ordering::SeqCst);
    phase(&mut app, &calls, false)?;
    app.plugin("framework", if use_direct { direct } else { aggregate })?;
    phase(&mut app, &calls, true)?;
    app.on("cooperate", first)?;
    app.on("cooperate", second)?;
    let server = Arc::new(app.build().await.map_err(|failure| failure.error)?);
    assert_eq!(ADMIN_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(calls.commands.load(Ordering::SeqCst), 0);
    // The sole request listener is the configured source language adapter; access is absent.
    assert_eq!(
        server
            .listeners()
            .filter(|listener| listener.pattern == "request")
            .count(),
        1
    );
    let hooks = server.host().service::<Hooks>()?;
    hooks.bind(&server, tokio::runtime::Handle::current());
    let created = resolve(
        &server,
        "note.create",
        json!({"input":{"id":"note-1","title":"Finite native composition"}}),
    )
    .await?;
    let searched = resolve(&server, "note.search", json!({})).await?;
    let mut request = Request::builder().url("/admin/note/search").build()?;
    let mut response = Response::new();
    server.handle_route(&mut request, &mut response).await?;
    assert!(
        response.result().is_some(),
        "admin response: {:?}; request {:?} {}; routes {:?}",
        response.try_to_json()?,
        request.method,
        request.url,
        server
            .listeners()
            .filter(|listener| listener.pattern.contains("admin"))
            .map(|listener| listener.pattern.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(response.result().unwrap().to_json()?, searched["results"]);
    assert!(searched.to_string().contains("Finite native composition"));
    #[cfg(feature = "api")]
    {
        // The source public branch performs no identity lookup; the generated operation owns policy.
        let mut request = Request::builder().url("/api/notes").build()?;
        let mut response = Response::new();
        server.handle_route(&mut request, &mut response).await?;
        assert_eq!(response.result().unwrap().to_json()?, searched["results"]);
    }
    let serve = resolve(&server, "serve", json!({})).await?;
    assert_eq!(calls.commands.load(Ordering::SeqCst), 1);
    let cooperation = resolve(&server, "cooperate", json!({})).await?;
    assert_eq!(cooperation["results"], "second");
    let token = server
        .host()
        .service::<Service>()?
        .create(
            json!({"id":"person","roles":["USER"]})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await?;
    let mut request = Request::new();
    request.session.set("identity", token.clone(), None)?;
    let mut response = Response::new();
    server.emit("me", &mut request, &mut response).await?;
    assert_eq!(response.result().unwrap().to_json()?["id"], "person");
    // Use the real JSON transport event; no SMTP connection or email spy is substituted.
    let email = server.resolve("email-send", json!({"from":"sender@example.test","to":"recipient@example.test","subject":"Finite","text":"No network"})).await?;
    assert_eq!(email.response.code, Some(200));
    assert!(email.response.results.is_some());
    let csrf = server
        .host()
        .service::<stackpress_framework_csrf::Service>()?;
    let mut generated = Response::new();
    let value = csrf.generate(&mut generated)?;
    let mut request = Request::builder()
        .data(json!({csrf.name():value}))
        .build()?;
    request.session.set(csrf.name(), value, None)?;
    assert!(csrf.valid(&request, &mut Response::new())?);
    if let Ok(output) = std::env::var("AGGREGATE_PAGE_OUT") {
        // Only the separately selected SSR consumer supplies this explicit artifact destination.
        std::fs::create_dir_all(&output)?;
        for (name, path) in [
            ("signin", "/auth/signin"),
            ("email", "/auth/signin/email"),
            ("account", "/auth/account"),
        ] {
            let mut request = Request::builder()
                .url(format!("https://aggregate.invalid{path}"))
                .build()?;
            if name == "account" {
                request.session.set("identity", token.clone(), None)?;
            }
            let mut response = Response::new();
            server.handle_route(&mut request, &mut response).await?;
            let stackpress_lib::Body::Text(html) = response.body() else {
                panic!("SSR {name} missing");
            };
            for secret in [
                "SIGNING-SEED-MUST-NOT-REACH-BROWSER",
                "DATABASE-SEED-MUST-NOT-REACH-BROWSER",
                "SMTP-PASSWORD-MUST-NOT-REACH-BROWSER",
            ] {
                assert!(!html.contains(secret));
            }
            std::fs::write(
                std::path::Path::new(&output).join(format!("{name}.html")),
                html,
            )?;
        }
    }
    let inventory = json!({"phases":calls.phases.lock().unwrap().clone(),"events":server.listeners().map(|listener|json!({"pattern":listener.pattern,"priority":listener.priority})).collect::<Vec<_>>(),"responses":{"create":created,"search":searched,"serve":serve,"cooperate":cooperation}});
    hooks.shutdown().await;
    engine.close().await?;
    Ok(inventory)
}
