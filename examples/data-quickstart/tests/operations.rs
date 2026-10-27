//! Prove the newly emitted operations for each revision with app-owned authorization, no listener.
#![cfg(feature = "native")]

use data_quickstart::generated::{
    contracts::{profile::CreateProfile, project::CreateProject},
    stores::{operations, profile::Profiles, project::Projects},
};
use serde_json::{Value, json};
use stackpress_framework_sql::{
    DataSession,
    operations::{Operation, OperationPolicy, Policy, Registration},
};
use stackpress_lib::{Error, Flow, Request, Result, queue::task};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::Arc;

mod support;
#[path = "../../generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Authorization precedes decoding; even a malformed body cannot bypass app policy.
struct AllowDemo;
impl OperationPolicy for AllowDemo {
    fn authorize(&self, _: &'static str, _: Operation, request: &Request) -> Result<()> {
        if request.data.path_as::<bool>("denied")?.unwrap_or(false) {
            return Err(Error::new("demo operation denied")
                .with_kind(stackpress_lib::ErrorKind::Domain)
                .with_code(403));
        }
        Ok(())
    }
}

/// Native startup/operations/cleanup run under a parent that kills and reaps on timeout.
#[test]
fn generated_operation_lifecycle() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_operation_lifecycle",
        "quickstart-operations",
        std::time::Duration::from_secs(25),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Call the normal host lifecycle and inspect only its public envelope.
async fn invoke(app: &Server, event: &str, input: Value, code: u16) -> Value {
    let value = app
        .call(event, input)
        .await
        .unwrap()
        .response
        .try_to_json()
        .unwrap();
    assert_eq!(value["code"], code, "{value}");
    assert!(!value.to_string().contains("SELECT "));
    value
}

/// Exercise successful create/read/search/update/upsert/batch/delete and refusal for every model.
async fn prove() {
    let root = tempfile::tempdir().unwrap();
    let engine = Arc::new(support::installed(&root.path().join("operations.sqlite")).await);
    let mut db = DataSession::pooled(&engine);
    Profiles::create(
        &mut db,
        serde_json::from_value::<CreateProfile>(json!({"id":"parent","name":"Parent"})).unwrap(),
    )
    .await
    .unwrap();
    let title = if cfg!(feature = "evolved") {
        "heading"
    } else {
        "title"
    };
    let project = json!({"id":"parent-project","ownerId":"parent",(title):"Parent"});
    Projects::create(
        &mut db,
        serde_json::from_value::<CreateProject>(project).unwrap(),
    )
    .await
    .unwrap();
    let app = application(engine.clone()).await;
    let mut cases = vec![
        (
            "p",
            json!({"id":"item","name":"Original"}),
            json!({"name":"Updated"}),
        ),
        (
            "j",
            json!({"id":"item","ownerId":"parent",(title):"Original"}),
            json!({(title):"Updated"}),
        ),
        (
            "t",
            json!({"id":"item","projectId":"parent-project","title":"Original"}),
            json!({"done":true}),
        ),
    ];
    if cfg!(feature = "expanded") {
        cases.push((
            "l",
            json!({"id":"item","ownerId":"parent"}),
            json!({"ownerId":"parent"}),
        ));
    }
    for (prefix, create, patch) in cases {
        model_lifecycle(&app, prefix, create, patch).await;
    }
    drop(app);
    engine.close().await.unwrap();
}

/// Register only selected generated capabilities after the engine and authorization service exist.
async fn application(engine: Arc<Engine>) -> Server {
    let mut builder = ServerBuilder::new();
    let registered = engine.clone();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let engine = registered.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(Policy(Arc::new(AllowDemo))))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                let mut selected = Registration::default();
                let actions = [
                    Operation::Get,
                    Operation::Detail,
                    Operation::Search,
                    Operation::Create,
                    Operation::Update,
                    Operation::Upsert,
                    Operation::Batch,
                    Operation::Delete,
                ];
                operations::profile::register(start, &mut selected, "p", &actions)?;
                operations::project::register(start, &mut selected, "j", &actions)?;
                operations::task::register(start, &mut selected, "t", &actions)?;
                #[cfg(feature = "expanded")]
                operations::link::register(start, &mut selected, "l", &actions)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder.build().await.unwrap()
}

/// One model's observable state transitions and public refusals share the same explicit host.
async fn model_lifecycle(app: &Server, prefix: &str, create: Value, patch: Value) {
    invoke(
        app,
        &format!("{prefix}.create"),
        json!({"input":create}),
        200,
    )
    .await;
    for action in ["get", "detail"] {
        let value = invoke(
            app,
            &format!("{prefix}.{action}"),
            json!({"key":{"id":"item"}}),
            200,
        )
        .await;
        assert_eq!(value["results"]["id"], "item");
        invoke(
            app,
            &format!("{prefix}.{action}"),
            json!({"denied":true}),
            403,
        )
        .await;
    }
    invoke(
        app,
        &format!("{prefix}.update"),
        json!({"key":{"id":"item"},"patch":patch}),
        200,
    )
    .await;
    invoke(
        app,
        &format!("{prefix}.upsert"),
        json!({"key":{"id":"item"},"input":create,"patch":patch}),
        200,
    )
    .await;
    let mut second = create.clone();
    second["id"] = "batch".into();
    let batch=invoke(app,&format!("{prefix}.batch"),json!({"items":[{"mode":"create","input":second},{"mode":"update","key":{"id":"batch"},"patch":patch},{"mode":"upsert","key":{"id":"batch"},"input":second,"patch":patch}]}),200).await;
    assert_eq!(batch["results"].as_array().unwrap().len(), 3);
    let page = invoke(
        app,
        &format!("{prefix}.search"),
        json!({"total":true,"take":2}),
        200,
    )
    .await;
    assert!(page["results"]["total"].as_u64().unwrap() >= 2);
    invoke(
        app,
        &format!("{prefix}.update"),
        json!({"key":{"id":"item"},"patch":{}}),
        400,
    )
    .await;
    invoke(
        app,
        &format!("{prefix}.create"),
        json!({"input":{"unknown":true}}),
        400,
    )
    .await;
    for id in ["item", "batch"] {
        invoke(
            app,
            &format!("{prefix}.delete"),
            json!({"key":{"id":id}}),
            200,
        )
        .await;
        invoke(app, &format!("{prefix}.get"), json!({"key":{"id":id}}), 404).await;
    }
}
