//! Real CLI, buffered HTTP and fixed native read parity through identical selected listeners.

use stackpress_example_profile_ui_contracts::{SearchOutcome, decode_search_reply};
use stackpress_example_profile_ui_native::{NativeCallContext, NativeError, NativeHost};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt,
    http::{HttpAdapter, HttpOutput},
};
use std::sync::{Arc, Mutex};

/// Test receipts contain stage identity only; public output never includes the probe.
#[derive(Default)]
struct Probe {
    stages: Mutex<Vec<&'static str>>,
    case: &'static str,
}

/// Record a stage and assert native metadata stays outside caller-controlled data.
fn stage<'a>(inv: &'a Invocation<'_>, label: &'static str) -> Result<&'a Probe> {
    let probe = inv.service::<Probe>()?;
    probe.stages.lock().unwrap().push(label);
    if let Some(context) = inv.adapter_context::<NativeCallContext>() {
        assert_eq!(context.window, "main");
        assert_eq!(context.call_id, "parity");
    }
    Ok(probe)
}

/// Deliberate partial headers/data must disappear when a later stage fails.
fn partial(res: &mut Response) -> Result<()> {
    res.headers_mut()?.set("x-private", "PARTIAL_SECRET")?;
    res.results(&serde_json::json!([{"id":"partial","name":"PARTIAL_SECRET"}]))?;
    res.set_total(1)?;
    Ok(())
}

/// Shared guard fixture, used unchanged by all adapters.
async fn request(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = stage(&inv, "request")?;
    if probe.case == "request-error" {
        partial(res)?;
        return Err(stackpress_lib::Error::new("PRIVATE_REQUEST_CAUSE")
            .with_code(403)
            .with_public_message("Public guard rejection"));
    }
    Ok(Flow::Continue)
}

/// Raw nested dispatch must not re-enter request/response hooks.
async fn operation(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = stage(&inv, "operation")?;
    match probe.case {
        "abort-empty" => Ok(Flow::Abort),
        "stop" => {
            res.stop()?;
            Ok(Flow::Continue)
        }
        "foreign" => {
            res.dispatch().await?;
            Ok(Flow::Continue)
        }
        "operation-error" | "error-failure" | "recover" => {
            partial(res)?;
            Err(stackpress_lib::Error::new("PRIVATE_OPERATION_CAUSE")
                .with_code(422)
                .with_public_message("Public rejection"))
        }
        _ => {
            let flow = inv.delegate("nested-read", req, res).await?;
            if probe.case == "abort-data" {
                Ok(Flow::Abort)
            } else {
                Ok(flow)
            }
        }
    }
}

/// One fixed public fixture isolates adapter lifecycle behavior without a search implementation.
async fn leaf(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    stage(&inv, "nested")?;
    let _ = req;
    res.results(&serde_json::json!([{ "id":"abc123", "name":"Parity result" }]))?;
    res.set_total(1)?;
    Ok(Flow::Continue)
}

/// Error replacement and terminal fallback use the existing host state machine.
async fn error(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = stage(&inv, "error")?;
    assert!(inv.failure()?.is_some());
    assert!(res.headers().get("x-private").is_none());
    assert!(!res.try_to_json()?.to_string().contains("PARTIAL_SECRET"));
    if probe.case == "error-failure" {
        return Err(stackpress_lib::Error::new("PRIVATE_SECONDARY_CAUSE"));
    }
    if probe.case == "recover" {
        *res = Response::new();
        res.results(&serde_json::json!([{"id":"recovered","name":"Recovered"}]))?;
        res.set_total(1)?;
    }
    Ok(Flow::Continue)
}

/// A response-stage failure is offered once and never re-enters response.
async fn response(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = stage(&inv, "response")?;
    if probe.case == "response-error" {
        partial(res)?;
        return Err(stackpress_lib::Error::new("PRIVATE_RESPONSE_CAUSE")
            .with_code(409)
            .with_public_message("Public response rejection"));
    }
    Ok(Flow::Continue)
}

/// Build the same listener graph per adapter, preserving all-matching native routing.
async fn application(case: &'static str) -> (stackpress_server::Server, Arc<Probe>) {
    let probe = Arc::new(Probe {
        case,
        ..Default::default()
    });
    let mut app = ServerBuilder::new();
    let service = Arc::clone(&probe);
    app.on_config(task(
        move |start: &mut stackpress_server::StartupContext| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                start.register(service)?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("profile-search", operation).unwrap();
    app.get("/parity", operation).unwrap();
    app.on("nested-read", leaf).unwrap();
    app.on("request", request).unwrap();
    app.on("response", response).unwrap();
    app.on("error", error).unwrap();
    (app.build().await.unwrap(), probe)
}

/// One finite table asserts real adapter output, hook order, recovery and privacy together.
#[tokio::test]
async fn adapters_share_g12_table() {
    for (case, expected) in [
        (
            "success",
            vec!["request", "operation", "nested", "response"],
        ),
        ("abort-data", vec!["request", "operation", "nested"]),
        ("abort-empty", vec!["request", "operation"]),
        ("request-error", vec!["request", "error", "response"]),
        (
            "operation-error",
            vec!["request", "operation", "error", "response"],
        ),
        ("error-failure", vec!["request", "operation", "error"]),
        (
            "response-error",
            vec!["request", "operation", "nested", "response", "error"],
        ),
        ("recover", vec!["request", "operation", "error", "response"]),
        ("stop", vec!["request", "operation"]),
        ("foreign", vec!["request", "operation"]),
    ] {
        let (app, probe) = application(case).await;
        let host = NativeHost::from_server(app);
        let native = host
            .search("main", "parity", r#"{"revision":1,"id":"abc"}"#)
            .await;
        assert_eq!(*probe.stages.lock().unwrap(), expected, "native {case}");
        assert_eq!(host.in_flight().unwrap(), 0);
        let (app, probe) = application(case).await;
        let terminal = stackpress_cli::Terminal::parse(
            &["emit", "profile-search", "--id", "abc"].map(String::from),
            Default::default(),
        )
        .unwrap();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let cli = stackpress_cli::run(
            &app,
            terminal,
            &mut stackpress_cli::CliSinks::new(&mut stdout, &mut stderr),
        )
        .await
        .unwrap();
        assert_eq!(*probe.stages.lock().unwrap(), expected, "CLI {case}");
        let cli_body = (!stdout.is_empty())
            .then(|| serde_json::from_slice::<serde_json::Value>(&stdout).unwrap());
        let (app, probe) = application(case).await;
        let http = HttpAdapter::new(Default::default())
            .unwrap()
            .handle(
                &app,
                http::Request::builder()
                    .uri("/parity?id=abc")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(*probe.stages.lock().unwrap(), expected, "HTTP {case}");
        if case == "stop" {
            assert_eq!(native, Err(NativeError::Suppressed));
            assert!(stdout.is_empty());
            assert!(matches!(http.output, HttpOutput::Suppressed));
            continue;
        }
        if case == "foreign" {
            assert_eq!(native, Err(NativeError::Unavailable));
            assert!(stdout.is_empty());
            assert!(matches!(http.output, HttpOutput::Unavailable { .. }));
            continue;
        }
        let HttpOutput::Response(http) = http.output else {
            panic!("missing HTTP output {case}");
        };
        let cli_body = cli_body.unwrap();
        assert!(!String::from_utf8_lossy(http.body()).contains("PRIVATE_"));
        assert!(!cli_body.to_string().contains("PRIVATE_"), "{cli_body}");
        assert!(!cli_body.to_string().contains("PARTIAL_SECRET"));
        if case == "abort-empty" {
            assert_eq!(native, Err(NativeError::NoContent));
            assert_eq!(http.status(), 204);
            assert_eq!(cli_body["code"], 204);
            continue;
        }
        let native = native.unwrap();
        assert!(!native.contains("PRIVATE_") && !native.contains("PARTIAL_SECRET"));
        match decode_search_reply(&native, None).unwrap() {
            SearchOutcome::Success(reply) => {
                assert_eq!(cli.exit_code, 0);
                assert_eq!(http.status(), 200);
                assert_eq!(
                    serde_json::to_value(&reply.rows).unwrap(),
                    cli_body["results"]
                );
                assert_eq!(
                    cli_body["results"],
                    serde_json::from_slice::<serde_json::Value>(http.body()).unwrap()["results"]
                );
            }
            SearchOutcome::Rejected(error) => {
                assert_eq!(cli.exit_code, 1);
                assert_eq!(error.code, http.status().as_u16());
                assert_eq!(
                    serde_json::to_value(error).unwrap(),
                    serde_json::from_slice::<serde_json::Value>(http.body()).unwrap()
                );
            }
        }
        // There is exactly one request hook and one correlation per top-level invocation.
        assert_eq!(
            cli.report
                .stages
                .iter()
                .filter(|row| row.stage == stackpress_server::LifecycleStage::Request)
                .count(),
            1
        );
    }
}
