//! Execute this application's native generated contracts and stores, including unexposed capabilities.
use generated_admin_web::generated::{contracts, stores};
use serde_json::{Value, json};
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::operations::{Operation, OperationPolicy, Policy, Registration};
use stackpress_framework_sql::{DataSession, InsertOutcome, stores::Visibility};
use stackpress_lib::{Flow, Request, Result, queue::task};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::Render;
use std::{sync::Arc, time::Duration};

#[path = "../../../generated-workspace/support/contracts.rs"]
mod contract_cases;
#[path = "../../../generated-workspace/support/watchdog.rs"]
mod watchdog;

/// The native inclusion is independently compiled; exercise its actual preparation and projection.
#[test]
fn native_contract_boundaries() {
    contract_cases::prove_contracts();
    contract_cases::prove_public_contracts();
    let context = ValueContext::default();
    for text in [Value::Null, json!(""), json!("supplied")] {
        let input: contracts::role_proof::CreateRoleProof = serde_json::from_value(json!({
            "id":"proof", "text":text, "flag":false,"zero":0,"sensitive":"secret","hostOnly":"private"
        })).unwrap();
        let row = input.prepare().unwrap();
        row.validate_with(&context, false).unwrap();
        let public = serde_json::to_value(row.project().unwrap()).unwrap();
        assert_eq!(public["text"], text);
        assert_eq!(public["flag"], false);
        assert_eq!(public["zero"], 0);
        assert!(public.get("sensitive").is_none());
        assert!(public.get("hostOnly").is_none());
        let input: contracts::public::PublicCreateRoleProof =
            serde_json::from_value(json!({"id":"proof","text":text,"sensitive":null})).unwrap();
        assert!(input.validate_client(&context).unwrap().is_empty());
        assert!(input.validate_create(&context).unwrap().is_empty());
        assert_eq!(input.prepare_create(&context).unwrap()["text"], text);
    }
    let patch: contracts::public::PublicPatchRoleProof =
        serde_json::from_value(json!({"text":null,"flag":false,"zero":0})).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    assert!(patch.validate_client(&context).unwrap().is_empty());
    assert!(
        serde_json::from_value::<contracts::public::PublicCreateRoleProof>(
            json!({"id":"x","hostOnly":"denied"})
        )
        .is_err()
    );
}

/// Supervise all native database work and close, even if a native worker or executor stalls.
#[test]
fn native_generated_capabilities() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "native_generated_capabilities",
        "admin-native",
        Duration::from_secs(45),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Shared assertions retain each generated type, transaction and authoritative returned row.
macro_rules! prove_store {
    ($engine:expr, $module:ident, $plural:ident, $store:ident, $key:ident, $patch_type:ident, $base:expr, $patch:expr) => {{
        use stores::$module::{self as family, $plural, $store};
        use contracts::public::{$key as Key, $patch_type as Patch};
        let context = ValueContext::default();
        let base = $base;
        let id = base["id"].as_str().unwrap();
        let key = Key { id: id.into() };
        let input = || serde_json::from_value(base.clone()).unwrap();
        let patch = || serde_json::from_value::<Patch>($patch).unwrap();
        let mut db = DataSession::pooled(&$engine);
        assert!(matches!($plural::create_if_absent(&mut db, input(), &context).await.unwrap(), InsertOutcome::Inserted(_)));
        assert!(matches!($plural::create_if_absent(&mut db, input(), &context).await.unwrap(), InsertOutcome::AlreadyExists));
        let row = $plural::get(&mut db, id).await.unwrap().unwrap();
        assert_eq!($plural::get_key(&mut db, &key).await.unwrap(), Some(row.clone()));
        assert_eq!($plural::get_maintenance(&mut db, &key, Visibility::All).await.unwrap(), Some(row.clone()));
        assert!(!$store::encode(&row).unwrap().is_empty());
        assert!($store::search_plan(&family::Query::default(), $engine.dialect()).unwrap().render($engine.dialect()).unwrap().sql().starts_with("SELECT"));
        // Public projections are checked at every generated finite include depth.
        let view = json!({"row":row});
        let public = serde_json::to_value(serde_json::from_value::<family::View0>(view.clone()).unwrap().project().unwrap()).unwrap();
        assert!(public["row"].get("hostOnly").is_none());
        assert!(public["row"].get("sensitive").is_none());
        serde_json::from_value::<family::View1>(view.clone()).unwrap().project().unwrap();
        serde_json::from_value::<family::View2>(view).unwrap().project().unwrap();
        assert!($plural::find(&mut db, family::Query::default()).await.unwrap().is_some());
        assert!(!$plural::find_all(&mut db, family::Query::default()).await.unwrap().is_empty());
        assert!($plural::count(&mut db, family::Query::default()).await.unwrap() > 0);
        assert!(!$plural::search_atomic(&$engine, family::Query::default()).await.unwrap().rows.is_empty());
        assert!($plural::detail(&mut db, &key).await.unwrap().is_some());
        assert!($plural::update(&mut db, id, patch(), &context).await.unwrap().is_some());
        $plural::upsert(&mut db, id, input(), patch(), &context).await.unwrap();
        $plural::upsert_on(&mut db, family::ConflictKey::Primary(key.clone()), input(), patch(), &context).await.unwrap();
        // Atomic batches mix all native plan modes and verify ordered results and rollback.
        let mut created = base.clone();
        created["id"] = json!(format!("{id}-batch"));
        let results = $plural::batch_atomic(&$engine, vec![
            family::BatchInput::Create { input: serde_json::from_value(created.clone()).unwrap() },
            family::BatchInput::Update { key: key.clone(), patch: patch() },
            family::BatchInput::Upsert { key: key.clone(), input: input(), patch: patch() },
            family::BatchInput::UpsertOn { target: family::ConflictKey::Primary(key.clone()), input: input(), patch: patch() },
        ], &context).await.unwrap();
        assert_eq!(results.len(),4);
        assert!(matches!(results[0], family::BatchOutcome::Created(_)));
        assert!(matches!(results[1], family::BatchOutcome::Updated(Some(_))));
        assert!(matches!(results[2], family::BatchOutcome::Upserted(_)));
        let mut tx = $engine.begin().await.unwrap();
        $plural::update_key(&mut DataSession::transaction(&mut tx), &key, patch(), &context).await.unwrap();
        tx.rollback().await.unwrap();
        assert!($plural::delete(&mut db, &Key { id: created["id"].as_str().unwrap().into() }).await.unwrap().is_some());
        assert!($plural::delete(&mut db, &Key { id:"missing".into() }).await.unwrap().is_none());
        assert!($plural::update(&mut db, "missing", patch(), &context).await.unwrap().is_none());
    }};
}

/// This private regression host deliberately selects all generated capabilities; public routes do not.
struct Allow;
impl OperationPolicy for Allow {
    fn authorize(&self, _: &'static str, _: Operation, _: &Request) -> Result<()> {
        Ok(())
    }
}
async fn app(engine: Arc<stackpress_sql::Engine>) -> Server {
    let mut builder = ServerBuilder::new();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(Policy(Arc::new(Allow))))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                let mut selected = Registration::default();
                let ordinary = [
                    Operation::Get,
                    Operation::Detail,
                    Operation::Search,
                    Operation::Create,
                    Operation::Update,
                    Operation::Upsert,
                    Operation::Batch,
                    Operation::Delete,
                ];
                let mut active = ordinary.to_vec();
                active.extend([Operation::Remove, Operation::Restore]);
                stores::operations::profile::register(start, &mut selected, "p", &active)?;
                stores::operations::project::register(start, &mut selected, "j", &ordinary)?;
                stores::operations::task::register(start, &mut selected, "t", &ordinary)?;
                stores::operations::role_proof::register(start, &mut selected, "r", &ordinary)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder.build().await.unwrap()
}
async fn invoke(app: &Server, event: &str, input: Value) -> Value {
    let value = app
        .call(event, input)
        .await
        .unwrap()
        .response
        .try_to_json()
        .unwrap();
    assert!(!value.to_string().contains("hostOnly"));
    assert!(!value.to_string().contains("sensitive"));
    value
}
/// An independently selected policy can allow mutation while denying the form's required read.
struct NoGet;
impl OperationPolicy for NoGet {
    fn authorize(&self, _: &'static str, operation: Operation, _: &Request) -> Result<()> {
        if operation == Operation::Get {
            Err(stackpress_lib::Error::new("private denied read"))
        } else {
            Ok(())
        }
    }
}

async fn prove() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = generated_admin_web::database::open(&root.path().join("native.sqlite"), false)
        .await
        .unwrap();
    generated_admin_web::database::initialize(&engine)
        .await
        .unwrap();
    prove_store!(
        engine,
        profile,
        Profiles,
        ProfileStore,
        ProfileKey,
        PatchProfile,
        json!({"id":"p1","name":"Owner","address":{"city":"Manila","country":"PH"}}),
        json!({"name":"Owner updated"})
    );
    prove_store!(
        engine,
        project,
        Projects,
        ProjectStore,
        ProjectKey,
        PatchProject,
        json!({"id":"j1","ownerId":"p1","title":"Parent","tags":["one"]}),
        json!({"title":"Parent updated"})
    );
    prove_store!(
        engine,
        task,
        Tasks,
        TaskStore,
        TaskKey,
        PatchTask,
        json!({"id":"t1","projectId":"j1","title":"Child","note":null,"priority":0}),
        json!({"done":true,"note":"supplied","priority":0})
    );
    prove_store!(
        engine,
        role_proof,
        RoleProofs,
        RoleProofStore,
        RoleProofKey,
        PatchRoleProof,
        json!({"id":"r1","sensitive":null,"flag":false,"zero":0,"rows":["x"],"metadata":{"key":"value"},"score":0,"slug":"slug"}),
        json!({"text":null,"flag":false,"zero":0})
    );
    let app = app(engine.clone()).await;
    for (prefix, base, patch) in [
        (
            "p",
            json!({"id":"op","name":"Original"}),
            json!({"name":"Updated"}),
        ),
        (
            "j",
            json!({"id":"op","ownerId":"p1","title":"Original"}),
            json!({"title":"Updated"}),
        ),
        (
            "t",
            json!({"id":"op","projectId":"j1","title":"Original"}),
            json!({"note":null,"priority":0,"done":false}),
        ),
        (
            "r",
            json!({"id":"op","sensitive":null}),
            json!({"text":null,"zero":0,"flag":false}),
        ),
    ] {
        assert_eq!(
            invoke(&app, &format!("{prefix}.create"), json!({"input":base})).await["code"],
            200
        );
        for action in ["get", "detail"] {
            assert_eq!(
                invoke(
                    &app,
                    &format!("{prefix}.{action}"),
                    json!({"key":{"id":"op"}})
                )
                .await["code"],
                200
            );
        }
        assert_eq!(
            invoke(&app, &format!("{prefix}.search"), json!({"total":true})).await["code"],
            200
        );
        assert_eq!(
            invoke(
                &app,
                &format!("{prefix}.update"),
                json!({"key":{"id":"op"},"patch":patch})
            )
            .await["code"],
            200
        );
        assert_eq!(
            invoke(
                &app,
                &format!("{prefix}.upsert"),
                json!({"key":{"id":"op"},"input":base,"patch":patch})
            )
            .await["code"],
            200
        );
        let mut new = base.clone();
        new["id"] = json!("batch-op");
        let batch = invoke(
            &app,
            &format!("{prefix}.batch"),
            json!({"items":[
                {"mode":"create","input":new},
                {"mode":"update","key":{"id":"op"},"patch":patch},
                {"mode":"upsert","key":{"id":"op"},"input":base,"patch":patch}
            ]}),
        )
        .await;
        assert_eq!(batch["code"], 200, "{batch}");
        assert_eq!(batch["results"].as_array().unwrap().len(), 3);
        if prefix == "p" {
            for action in ["remove", "restore"] {
                assert_eq!(
                    invoke(
                        &app,
                        &format!("{prefix}.{action}"),
                        json!({"key":{"id":"op"}})
                    )
                    .await["code"],
                    200
                );
            }
        }
        assert_eq!(
            invoke(
                &app,
                &format!("{prefix}.delete"),
                json!({"key":{"id":"op"}})
            )
            .await["code"],
            200
        );
        assert_eq!(
            invoke(&app, &format!("{prefix}.get"), json!({"key":{"id":"op"}})).await["code"],
            404
        );
    }
    drop(app);
    let host = generated_admin_web::host::build(
        engine.clone(),
        generated_admin_web::host::Config {
            transfer_limits: Default::default(),
            mount: "/".into(),
            csrf: "fixture".into(),
            csr: false,
        },
        Arc::new(NoGet),
    )
    .await
    .unwrap();
    let adapter = stackpress_server::http::HttpAdapter::new(Default::default()).unwrap();
    for (path, status) in [
        ("/profile/search", 200),
        ("/profile/copy?key=%7B%22id%22%3A%22p1%22%7D", 403),
        ("/profile/update?key=%7B%22id%22%3A%22p1%22%7D", 403),
    ] {
        let response = adapter
            .handle(
                &host,
                http::Request::builder().uri(path).body(Vec::new()).unwrap(),
            )
            .await
            .unwrap();
        let stackpress_server::http::HttpOutput::Response(response) = response.output else {
            panic!("buffered output required")
        };
        assert_eq!(response.status().as_u16(), status);
        if status == 200 {
            let html = String::from_utf8(response.into_body()).unwrap();
            let seed = html
                .split("<script id=\"admin-seed\" type=\"application/json\">")
                .nth(1)
                .unwrap()
                .split("</script>")
                .next()
                .unwrap();
            let seed: Value = serde_json::from_str(seed).unwrap();
            let allowed = seed["allowed"].as_array().unwrap();
            assert!(!allowed.contains(&json!("copy")));
            assert!(!allowed.contains(&json!("update")));
        }
    }
    drop(host);
    engine.close().await.unwrap();
}
