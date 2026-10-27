//! Same explicitly selected generated handlers through direct, CLI and buffered HTTP entry points.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::{
    contracts::{profile::CreateProfile, project::CreateProject},
    stores::{
        operations::{profile as po, project as jo, task as to},
        profile::Profiles,
        project::Projects,
    },
};
use serde_json::{Value, json};
use stackpress_framework_sql::{
    DataSession,
    operations::{Operation, OperationPolicy, Policy, Registration},
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    Server, ServerBuilder, ServerInvocationExt, StartupContext,
    http::{HttpAdapter, HttpOptions, HttpOutput},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Explicit app route mapping: fixed selected capabilities, never a generic event endpoint.
const ROUTES: &[(&str, &str)] = &[
    ("/p/get", "p.get"),
    ("/p/detail", "p.detail"),
    ("/p/search", "p.search"),
    ("/p/create", "p.create"),
    ("/p/update", "p.update"),
    ("/p/upsert", "p.upsert"),
    ("/p/batch", "p.batch"),
    ("/p/remove", "p.remove"),
    ("/p/restore", "p.restore"),
    ("/p/delete", "p.delete"),
    ("/j/get", "j.get"),
    ("/j/detail", "j.detail"),
    ("/j/search", "j.search"),
    ("/j/create", "j.create"),
    ("/j/update", "j.update"),
    ("/j/upsert", "j.upsert"),
    ("/j/batch", "j.batch"),
    ("/j/delete", "j.delete"),
    ("/t/get", "t.get"),
    ("/t/detail", "t.detail"),
    ("/t/search", "t.search"),
    ("/t/create", "t.create"),
    ("/t/update", "t.update"),
    ("/t/upsert", "t.upsert"),
    ("/t/batch", "t.batch"),
    ("/t/delete", "t.delete"),
];
/// Demonstrate app authorization even through nested transport delegation.
struct Allow;
impl OperationPolicy for Allow {
    fn authorize(&self, _: &'static str, _: Operation, request: &Request) -> Result<()> {
        if request.url.path() == "/denied" {
            return Err(Error::new("PRIVATE_POLICY_PASSWORD_SENTINEL"));
        }
        Ok(())
    }
}
/// Delegate the selected fixed route using the existing raw primitive, preserving Flow::Abort.
async fn route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let event = ROUTES
        .iter()
        .find(|(path, _)| *path == req.url.path())
        .map(|(_, event)| *event)
        .ok_or_else(|| Error::invalid("route unavailable"))?;
    inv.delegate(event, req, res).await
}
/// The application deliberately completes before the generated handler; it must be respected.
async fn prior(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    match req.url.path() {
        "/stop" => res.stop()?,
        "/status" => {
            res.status_code(202, None)?;
        }
        _ => {
            res.results(&"preselected")?;
        }
    }
    Ok(Flow::Continue)
}
/// Abort propagates out of nested dispatch instead of running the following generated listener.
async fn abort(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Ok(if req.url.path() == "/abort" {
        Flow::Abort
    } else {
        Flow::Continue
    })
}
/// A second named application operation delegates without starting a new top-level lifecycle.
async fn nested(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("p.get", req, res).await
}
/// Observe whether an aborted nested dispatch incorrectly continued its caller's queue.
async fn after(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<AtomicUsize>()?.fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}
/// Build only the capabilities the app selects, after its required typed services exist.
async fn app(engine: Arc<stackpress_sql::Engine>) -> Server {
    let mut builder = ServerBuilder::new();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(Policy(Arc::new(Allow))))?;
                start.register(Arc::new(AtomicUsize::new(0)))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder.on("p.get", abort).unwrap();
    builder.on("guard.get", prior).unwrap();
    builder.on("nested", nested).unwrap();
    builder.on("nested", after).unwrap();
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
                po::register(start, &mut selected, "p", &active)?;
                jo::register(start, &mut selected, "j", &ordinary)?;
                to::register(start, &mut selected, "t", &ordinary)?;
                po::register(start, &mut selected, "guard", &[Operation::Get])?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    for (path, _) in ROUTES {
        builder.post(path, route).unwrap();
    }
    builder.build().await.unwrap()
}
/// Outer supervision covers native sessions, host startup and all adapter execution paths.
#[test]
fn generated_operation_parity() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_operation_parity",
        "operations",
        Duration::from_secs(25),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Exercise the same sequence against an independent fixture per transport.
async fn invoke(app: &Server, transport: u8, event: &str, input: Value) -> Value {
    let value = match transport {
        0 => app
            .call(event, input)
            .await
            .unwrap()
            .response
            .try_to_json()
            .unwrap(),
        1 => {
            let mut args = vec!["emit".into(), event.into()];
            for (key, value) in input.as_object().unwrap() {
                args.push(format!("--{key}"));
                args.push(if let Some(value) = value.as_str() {
                    value.into()
                } else {
                    value.to_string()
                });
            }
            let terminal = stackpress_cli::Terminal::parse(&args, Default::default()).unwrap();
            let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
            stackpress_cli::run(
                app,
                terminal,
                &mut stackpress_cli::CliSinks::new(&mut stdout, &mut stderr),
            )
            .await
            .unwrap();
            assert!(!String::from_utf8(stderr).unwrap().contains("PRIVATE_"));
            serde_json::from_slice(&stdout).unwrap()
        }
        _ => {
            let path = ROUTES
                .iter()
                .find(|(_, selected)| *selected == event)
                .unwrap()
                .0;
            let input = http::Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(serde_json::to_vec(&input).unwrap())
                .unwrap();
            let reply = HttpAdapter::new(HttpOptions::default())
                .unwrap()
                .handle(app, input)
                .await
                .unwrap();
            let HttpOutput::Response(response) = reply.output else {
                panic!("buffered result unavailable")
            };
            serde_json::from_slice(response.body()).unwrap()
        }
    };
    let text = value.to_string();
    for private in [
        "PRIVATE_",
        "INSERT INTO",
        "SELECT ",
        "sqlite",
        "password",
        "binds",
    ] {
        assert!(
            !text.contains(private),
            "public response disclosed {private}"
        );
    }
    value
}
/// Register/startup boundaries and one successful/error sequence are identical across transports.
async fn prove() {
    let mut expected = None;
    for transport in 0..3 {
        let fixture = support::fixture().await;
        let engine = Arc::new(fixture.engine);
        let profile: CreateProfile =
            serde_json::from_value(json!({"id":"parent","name":"Parent"})).unwrap();
        Profiles::create(&mut DataSession::pooled(&engine), profile)
            .await
            .unwrap();
        let project: CreateProject = serde_json::from_value(
            json!({"id":"parent-project","title":"Parent","ownerId":"parent"}),
        )
        .unwrap();
        Projects::create(&mut DataSession::pooled(&engine), project)
            .await
            .unwrap();
        let app = app(engine.clone()).await;
        let mut responses = Vec::new();
        for (prefix, create, patch) in [
            (
                "p",
                json!({"id":"item","name":"Original","address":{"city":"Manila","country":"PH"}}),
                json!({"name":"Updated"}),
            ),
            (
                "j",
                json!({"id":"item","title":"Original","ownerId":"parent"}),
                json!({"title":"Updated"}),
            ),
            (
                "t",
                json!({"id":"item","title":"Original","projectId":"parent-project"}),
                json!({"title":"Updated"}),
            ),
        ] {
            let created = invoke(
                &app,
                transport,
                &format!("{prefix}.create"),
                json!({"input":create}),
            )
            .await;
            assert_eq!(created["code"], 200);
            responses.push(created);
            for operation in ["get", "detail"] {
                let value = invoke(
                    &app,
                    transport,
                    &format!("{prefix}.{operation}"),
                    json!({"key":{"id":"item"}}),
                )
                .await;
                assert_eq!(value["code"], 200);
                if operation == "detail" && prefix == "p" {
                    assert_eq!(value["results"], json!({"id":"item","name":"Original"}));
                }
                responses.push(value);
            }
            let value = invoke(
                &app,
                transport,
                &format!("{prefix}.update"),
                json!({"key":{"id":"item"},"patch":patch}),
            )
            .await;
            assert_eq!(value["code"], 200);
            responses.push(value);
            let value = invoke(
                &app,
                transport,
                &format!("{prefix}.upsert"),
                json!({"key":{"id":"item"},"input":create,"patch":patch}),
            )
            .await;
            assert_eq!(value["code"], 200);
            responses.push(value);
            let mut second = create.clone();
            second["id"] = json!("batch");
            let value=invoke(&app,transport,&format!("{prefix}.batch"),json!({"items":[{"mode":"create","input":second},{"mode":"update","key":{"id":"batch"},"patch":patch},{"mode":"upsert","key":{"id":"batch"},"input":second,"patch":patch}]})).await;
            assert_eq!(value["code"], 200);
            assert_eq!(value["results"].as_array().unwrap().len(), 3);
            responses.push(value);
            let value = invoke(
                &app,
                transport,
                &format!("{prefix}.search"),
                json!({"total":true,"take":2}),
            )
            .await;
            assert_eq!(value["code"], 200);
            assert!(value["results"]["total"].as_u64().unwrap() >= 2);
            responses.push(value);
            for (operation, input, code) in [
                ("get", json!({"key":{"id":"absent"}}), 404),
                ("update", json!({"key":{"id":"item"},"patch":{}}), 400),
                (
                    "create",
                    json!({"input":{"id":"bad","private":"PRIVATE_REJECTED_INPUT"}}),
                    400,
                ),
                (
                    "upsert",
                    json!({"key":{"id":"other"},"input":create,"patch":patch}),
                    400,
                ),
            ] {
                let value = invoke(&app, transport, &format!("{prefix}.{operation}"), input).await;
                assert_eq!(value["code"], code);
                responses.push(value);
            }
            fixture.calls.lock().unwrap().clear();
            let conflict = invoke(
                &app,
                transport,
                &format!("{prefix}.create"),
                json!({"input":create}),
            )
            .await;
            assert_eq!(conflict["code"], 500);
            assert_eq!(conflict["errors"]["outcome"], "unknown");
            assert_eq!(
                fixture.calls.lock().unwrap().len(),
                1,
                "a submitted failed write must never be retried or refetched"
            );
            responses.push(conflict);
            if prefix == "p" {
                for operation in ["remove", "remove", "restore", "restore"] {
                    let value = invoke(
                        &app,
                        transport,
                        &format!("p.{operation}"),
                        json!({"key":{"id":"item"}}),
                    )
                    .await;
                    assert_eq!(value["code"], 200);
                    responses.push(value);
                }
            }
            let value = invoke(
                &app,
                transport,
                &format!("{prefix}.delete"),
                json!({"key":{"id":"item"}}),
            )
            .await;
            assert_eq!(value["code"], 200);
            responses.push(value);
        }
        if let Some(expected) = &expected {
            assert_eq!(
                &responses, expected,
                "transport {transport} changed selected operation semantics"
            );
        } else {
            expected = Some(responses);
        }
        fixture.calls.lock().unwrap().clear();
        let denied = Request::builder()
            .url("/denied")
            .data_serialized(&json!({"key":{"id":"parent"}}))
            .build()
            .unwrap();
        let denied = app.call("p.get", denied).await.unwrap();
        assert_eq!(denied.response.status().unwrap().code, 403);
        assert!(
            !denied
                .response
                .try_to_json()
                .unwrap()
                .to_string()
                .contains("PRIVATE_")
        );
        for path in ["/result", "/status", "/stop"] {
            let mut request = Request::builder().url(path).build().unwrap();
            let mut response = Response::new();
            app.emit("guard.get", &mut request, &mut response)
                .await
                .unwrap();
        }
        let mut request = Request::builder().url("/abort").build().unwrap();
        let mut response = Response::new();
        assert_eq!(
            app.emit("nested", &mut request, &mut response)
                .await
                .unwrap(),
            stackpress_lib::DispatchOutcome::Aborted
        );
        assert_eq!(
            app.host()
                .service::<AtomicUsize>()
                .unwrap()
                .load(Ordering::SeqCst),
            0
        );
        assert!(
            fixture.calls.lock().unwrap().is_empty(),
            "denial/completion/abort must precede DB effects"
        );
        engine.close().await.unwrap();
    }
    // Missing services, duplicate selection and soft-delete selection on a non-active model fail startup.
    let mut missing = ServerBuilder::new();
    missing
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                po::register(start, &mut Registration::default(), "p", &[Operation::Get])?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    assert!(missing.build().await.is_err());
    let fixture = support::fixture().await;
    let engine = Arc::new(fixture.engine);
    for duplicate in [false, true] {
        let mut builder = ServerBuilder::new();
        let db = engine.clone();
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
            .on_listen(task(move |start: &mut StartupContext| {
                Box::pin(async move {
                    let mut marker = Registration::default();
                    if duplicate {
                        po::register(start, &mut marker, "p", &[Operation::Get])?;
                        po::register(start, &mut marker, "p", &[Operation::Get])?;
                    } else {
                        jo::register(start, &mut marker, "j", &[Operation::Remove])?;
                    }
                    Ok(Flow::Continue)
                })
            }))
            .unwrap();
        assert!(builder.build().await.is_err());
    }
    engine.close().await.unwrap();
}
