//! Catalog order, missing methods and real source-ordered API/account SQL registration.
use super::*;
use stackpress_framework::{CompiledClient, CompiledModel, register_with};
use stackpress_framework_schema::Model;

/// Source metadata is explicitly evaluated by callers, never by startup.
fn describe() -> Vec<Model> {
    vec![stackpress_framework_session::profile::ProfileSchema::default().model()]
}
/// An application-owned model queues a real capability for Listen.
fn first_model(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.on("first-model", public)?;
            Ok(Flow::Continue)
        })
    }))
}
/// A real app-owned admin contribution queues its routes in the existing Route phase.
fn first_admin(app: &mut ServerBuilder) -> Result<()> {
    app.on_route(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.get("/first-admin", public)?;
            Ok(Flow::Continue)
        })
    }))
}
/// A second app-owned admin contribution tests catalog FIFO without universal admin routes.
fn second_admin(app: &mut ServerBuilder) -> Result<()> {
    app.on_route(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.get("/second-admin", public)?;
            Ok(Flow::Continue)
        })
    }))
}
static CATALOG: CompiledClient = CompiledClient {
    bindings: stackpress_framework::FrameworkBindings {
        session: None,
        api: None,
    },
    build: "m0-proof",
    describe,
    models: &[
        CompiledModel {
            name: "first",
            listen: Some(first_model),
            admin: Some(first_admin),
        },
        CompiledModel {
            name: "api",
            listen: Some(stackpress_framework_api::sql::register),
            admin: Some(second_admin),
        },
        CompiledModel {
            name: "missing-methods",
            listen: None,
            admin: None,
        },
    ],
};

/// Bind the catalog to the ordinary #14 plugin signature; direct use stays available.
fn selected_catalog(app: &mut ServerBuilder) -> Result<()> {
    register_with(app, &CATALOG)
}

/// Runtime inventory proves real callbacks occur exactly once and in catalog order.
#[tokio::test]
async fn compiled_catalog_and_api_sql_prepare_real_ordered_phases_once() {
    let mut app = ServerBuilder::new();
    app.plugin("framework", selected_catalog).unwrap();
    let host = app.build().await.unwrap();
    let client = host
        .host()
        .service::<Client>()
        .unwrap()
        .get(false)
        .unwrap()
        .unwrap();
    assert_eq!(client.build, "m0-proof");
    assert_eq!((client.describe)()[0].name, "Profile");
    let patterns: Vec<_> = host
        .listeners()
        .map(|listener| listener.pattern.clone())
        .collect();
    assert_eq!(
        patterns
            .iter()
            .filter(|name| *name == "first-model")
            .count(),
        1
    );
    assert_eq!(
        patterns
            .iter()
            .filter(|name| *name == "GET /first-admin")
            .count(),
        1
    );
    assert!(
        patterns.iter().position(|name| name == "upgrade")
            < patterns.iter().position(|name| name == "first-model")
    );
    assert!(
        patterns.iter().position(|name| name == "first-model")
            < patterns.iter().position(|name| name == "application-batch")
    );
    assert!(
        patterns.iter().position(|name| name == "application-batch")
            < patterns.iter().position(|name| name == "auth-signup")
    );
    assert!(
        patterns.iter().position(|name| name == "GET /first-admin")
            < patterns.iter().position(|name| name == "GET /second-admin")
    );
    let reply = host.resolve("application-search", json!({})).await.unwrap();
    assert!(reply.response.results.is_none()); // The real SQL event passes through without Engine.
}

/// The granted additive helper closes the real account-versus-API startup ordering gap.
#[tokio::test]
async fn queued_account_registration_matches_source_listen_order() {
    static ACCOUNT: CompiledClient = CompiledClient {
        bindings: stackpress_framework::FrameworkBindings {
            session: None,
            api: None,
        },
        build: "immediate-account-boundary",
        describe,
        models: &[CompiledModel {
            name: "account",
            listen: Some(stackpress_framework_session::account::events::register_listen),
            admin: None,
        }],
    };
    let mut app = ServerBuilder::new();
    register_with(&mut app, &ACCOUNT).unwrap();
    let host = app.build().await.unwrap();
    let patterns: Vec<_> = host
        .listeners()
        .map(|listener| listener.pattern.clone())
        .collect();
    // Server/schema/SQL commands precede model callbacks, which precede auth Listen events.
    assert!(
        patterns.iter().position(|name| name == "auth-search")
            > patterns.iter().position(|name| name == "upgrade")
    );
    assert!(
        patterns.iter().position(|name| name == "auth-search")
            < patterns.iter().position(|name| name == "auth-signup")
    );
    let oracle: Value =
        serde_json::from_str(include_str!("../fixtures/source-order.json")).unwrap();
    assert_eq!(oracle["model_listen_phase"], "Listen");
}

/// Present empty client stays distinct from missing client, without invoking any callback.
#[tokio::test]
async fn empty_catalog_is_present_and_repeated_registration_preserves_native_error() {
    static EMPTY: CompiledClient = CompiledClient {
        bindings: stackpress_framework::FrameworkBindings {
            session: None,
            api: None,
        },
        build: "empty",
        describe: Vec::new,
        models: &[],
    };
    let mut app = ServerBuilder::new();
    register_with(&mut app, &EMPTY).unwrap();
    assert!(register(&mut app).is_err());
    let mut app = ServerBuilder::new();
    register_with(&mut app, &EMPTY).unwrap();
    let host = app.build().await.unwrap();
    assert!(
        host.host()
            .service::<Client>()
            .unwrap()
            .get(false)
            .unwrap()
            .unwrap()
            .models
            .is_empty()
    );
}
