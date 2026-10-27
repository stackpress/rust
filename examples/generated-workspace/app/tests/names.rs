//! Reserved aggregate names and Rust keywords compile and execute without namespace collisions.
#![cfg(feature = "names")]
use generated_workspace_app::generated::stores::operations::{
    _operations as operations, _schema as schema,
};
use serde_json::json;
use stackpress_framework_sql::operations::{Operation, OperationPolicy, Policy, Registration};
use stackpress_lib::{Flow, Request, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use std::{sync::Arc, time::Duration};
#[path = "support/native_calls.rs"]
mod native_calls;
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// Explicit application authorization for this isolated native fixture.
struct Allow;
impl OperationPolicy for Allow {
    fn authorize(&self, _: &'static str, _: Operation, _: &Request) -> Result<()> {
        Ok(())
    }
    fn values(
        &self,
        model: &'static str,
        _: Operation,
        _: &Request,
    ) -> stackpress_framework_sql::DataResult<stackpress_framework_sql::operations::HostInputs>
    {
        Ok(stackpress_framework_sql::operations::HostInputs {
            create: if model == "Identity" {
                json!({"id":99})
            } else {
                json!({})
            },
            ..Default::default()
        })
    }
}
/// Bound startup, generated IO and completion under a kill/reap parent.
#[test]
fn reserved_names() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run("reserved_names", "names", Duration::from_secs(20));
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Import both escaped stores, publish named handlers, and execute every selected operation.
async fn prove() {
    let fixture = support::fixture().await;
    let engine = Arc::new(fixture.engine);
    let db = engine.clone();
    let mut builder = ServerBuilder::new();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let db = db.clone();
            Box::pin(async move {
                start.register(db)?;
                start.register(Arc::new(Policy(Arc::new(Allow))))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                let mut registry = Registration::default();
                let operations = [
                    Operation::Create,
                    Operation::Get,
                    Operation::Detail,
                    Operation::Search,
                    Operation::Update,
                    Operation::Upsert,
                    Operation::Batch,
                    Operation::Delete,
                ];
                schema::register(start, &mut registry, "names-schema", &operations)?;
                operations::register(start, &mut registry, "names-operations", &operations)?;
                generated_workspace_app::generated::stores::operations::identity::register(
                    start,
                    &mut registry,
                    "identity",
                    &[
                        Operation::Create,
                        Operation::Get,
                        Operation::Search,
                        Operation::Detail,
                        Operation::Batch,
                        Operation::Delete,
                    ],
                )?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    let app = builder.build().await.unwrap();
    // Reserved aggregate names compile and execute through their escaped store modules.
    for prefix in ["names-schema", "names-operations"] {
        let create = json!({"id":"named","value":"Original"});
        for (operation, input) in [
            ("create", json!({"input":create})),
            ("get", json!({"key":{"id":"named"}})),
            ("detail", json!({"key":{"id":"named"}})),
            ("search", json!({"total":true})),
            (
                "update",
                json!({"key":{"id":"named"},"patch":{"value":"Updated"}}),
            ),
            (
                "upsert",
                json!({"key":{"id":"named"},"input":create,"patch":{"value":"Upserted"}}),
            ),
            (
                "batch",
                json!({"items":[{"mode":"update","key":{"id":"named"},"patch":{"value":"Batch"}}]}),
            ),
            ("delete", json!({"key":{"id":"named"}})),
        ] {
            let reply = app
                .call(&format!("{prefix}.{operation}"), input)
                .await
                .unwrap();
            let value = reply.response.try_to_json().unwrap();
            assert_eq!(value["code"], 200, "{value}");
        }
    }
    // A business rule on the active member still runs before a soft-state write.
    use generated_workspace_app::generated::{
        contracts::{
            identity::CreateIdentity,
            public::{IdentityKey, PatchIdentity, SchemaKey},
            schema::CreateSchema,
        },
        stores::{_schema::Schemas, identity::Identitys},
    };
    use stackpress_framework_schema::ValueContext;
    use stackpress_framework_sql::{DataErrorKind, DataSession};
    let mut session = DataSession::pooled(&engine);
    let input: CreateSchema = serde_json::from_value(json!({"id":"active-rule"})).unwrap();
    Schemas::create(&mut session, input).await.unwrap();
    fixture.calls.lock().unwrap().clear();
    let error = Schemas::remove(
        &mut session,
        &SchemaKey {
            id: "active-rule".into(),
        },
        &ValueContext::default(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::Validation);
    assert!(fixture.calls.lock().unwrap().is_empty());
    assert!(
        Schemas::get(&mut session, "active-rule")
            .await
            .unwrap()
            .unwrap()
            .active
    );
    // A key-only model can be created/read/deleted, but it has no invented mutable operation.
    let input: CreateIdentity = serde_json::from_value(json!({"id":7})).unwrap();
    Identitys::create(&mut session, input).await.unwrap();
    let patch: PatchIdentity = serde_json::from_value(json!({})).unwrap();
    fixture.calls.lock().unwrap().clear();
    assert_eq!(
        Identitys::update_key(
            &mut session,
            &IdentityKey { id: 7 },
            patch,
            &ValueContext::default()
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Unsupported
    );
    assert!(fixture.calls.lock().unwrap().is_empty());
    assert!(
        Identitys::delete(&mut session, &IdentityKey { id: 7 })
            .await
            .unwrap()
            .is_some()
    );
    native_calls::native_calls!(
        _schema,
        Schemas,
        SchemaStore,
        CreateSchema,
        SchemaKey,
        generated_workspace_app::generated::contracts::public::PatchSchema,
        json!({"id":"native-breadth"}),
        json!({"id":"native-breadth"}),
        json!({"value":"Changed"}),
        &mut session,
        &ValueContext::default()
    );
    native_calls::native_calls!(
        _operations,
        Operationss,
        OperationsStore,
        generated_workspace_app::generated::contracts::operations::CreateOperations,
        generated_workspace_app::generated::contracts::public::OperationsKey,
        generated_workspace_app::generated::contracts::public::PatchOperations,
        json!({"id":"native-breadth"}),
        json!({"id":"native-breadth"}),
        json!({"value":"Changed"}),
        &mut session,
        &ValueContext::default()
    );
    let ui = generated_workspace_app::generated::metadata::ui::schema().unwrap();
    assert!(ui.models.iter().all(|model| model.id != "Identity"));
    // A private model receives identity only from app policy and never exposes its stored member.
    for (event, input, code) in [
        ("identity.create", json!({"input":{}}), 200),
        ("identity.get", json!({"key":{"id":99}}), 200),
        ("identity.search", json!({"total":true}), 200),
        ("identity.detail", json!({"key":{"id":99}}), 400),
        ("identity.delete", json!({"key":{"id":99}}), 200),
        (
            "identity.batch",
            json!({"items":[{"mode":"create","input":{}}]}),
            200,
        ),
        ("identity.get", json!({"key":{"id":99}}), 200),
    ] {
        let reply = app.call(event, input).await.unwrap();
        let value = reply.response.try_to_json().unwrap();
        assert_eq!(value["code"], code, "{event}: {value}");
        assert!(!value["results"].to_string().contains("99"));
    }
    // All native key-only read helpers work; immutable updates/upserts remain explicit refusals.
    use generated_workspace_app::generated::stores::identity::{
        Filter as IdentityFilter, Query as IdentityQuery,
    };
    use stackpress_framework_schema::Predicate;
    let identity = IdentityKey { id: 99 };
    let query = IdentityQuery {
        filters: vec![IdentityFilter::Id(Predicate::Eq(99))],
        ..Default::default()
    };
    assert!(Identitys::get(&mut session, 99).await.unwrap().is_some());
    assert!(
        Identitys::get_maintenance(
            &mut session,
            &identity,
            stackpress_framework_sql::stores::Visibility::All
        )
        .await
        .unwrap()
        .is_some()
    );
    assert_eq!(
        Identitys::count(&mut session, query.clone()).await.unwrap(),
        1
    );
    assert!(
        Identitys::find(&mut session, query.clone())
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        Identitys::find_all(&mut session, query.clone())
            .await
            .unwrap()
            .len(),
        1
    );
    let page = Identitys::search_atomic(&engine, query).await.unwrap();
    assert_eq!(page.rows[0].row.id, 99);
    let input: CreateIdentity = serde_json::from_value(json!({"id":99})).unwrap();
    assert!(matches!(
        Identitys::create_if_absent(&mut session, input, &ValueContext::default())
            .await
            .unwrap(),
        stackpress_framework_sql::InsertOutcome::AlreadyExists
    ));
    let input: CreateIdentity = serde_json::from_value(json!({"id":99})).unwrap();
    let patch: PatchIdentity = serde_json::from_value(json!({})).unwrap();
    fixture.calls.lock().unwrap().clear();
    assert_eq!(
        Identitys::upsert_key(
            &mut session,
            &identity,
            input,
            patch,
            &ValueContext::default()
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Unsupported
    );
    assert!(fixture.calls.lock().unwrap().is_empty());
    // A relation literally named row cannot replace the base row or collide with its Rust member.
    let parent: CreateSchema =
        serde_json::from_value(json!({"id":"row-parent","value":"Parent"})).unwrap();
    Schemas::create(&mut session, parent).await.unwrap();
    use generated_workspace_app::generated::{
        contracts::operations::CreateOperations,
        stores::_operations::{Filter as OperationsFilter, Operationss, Query as OperationsQuery},
    };
    let child: CreateOperations =
        serde_json::from_value(json!({"id":"row-child","schemaId":"row-parent"})).unwrap();
    Operationss::create(&mut session, child).await.unwrap();
    let mut page = Operationss::search_atomic(
        &engine,
        OperationsQuery {
            includes: vec!["row".into()],
            filters: vec![OperationsFilter::Id(Predicate::Eq("row-child".into()))],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let view = page.rows.remove(0);
    assert_eq!(view.row.id, "row-child");
    assert!(
        matches!(&view._relation_row, stackpress_framework_sql::stores::Loaded::Loaded(Some(parent)) if parent.row.id=="row-parent")
    );
    let wire = serde_json::to_value(view.clone().project().unwrap()).unwrap();
    assert_eq!(wire["row"]["id"], "row-child");
    assert_eq!(wire["__row"]["value"]["row"]["value"], "Parent");
    // Selecting a whole containing relation restores logical names inside an actual generated view.
    let container = json!({"row":{"label":"Container"},"items":{"state":"loaded","value":[wire]}});
    let selected =
        stackpress_framework_sql::projection::select(&container, &["label", "items"], false)
            .unwrap();
    assert_eq!(selected["label"], "Container");
    assert_eq!(selected["items"][0]["id"], "row-child");
    assert_eq!(selected["items"][0]["row"]["id"], "row-parent");
    assert!(selected["items"][0].get("__row").is_none());
    assert_eq!(
        view.select_default().unwrap().value(),
        &json!({"id":"row-child","row":{"value":"Parent"}})
    );
    let reply = app
        .call("names-operations.detail", json!({"key":{"id":"row-child"}}))
        .await
        .unwrap();
    let wire = reply.response.try_to_json().unwrap();
    assert_eq!(wire["code"], 200);
    assert_eq!(
        wire["results"],
        json!({"id":"row-child","row":{"value":"Parent"}})
    );
    engine.close().await.unwrap();
}
