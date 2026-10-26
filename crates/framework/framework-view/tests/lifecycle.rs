//! Real native page/lifecycle decision table with owned render/drop probes.
#![cfg(feature = "server")]

use stackpress_framework_view::*;
use stackpress_lib::{DeliveryState, Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{LifecycleOutcome, LifecycleStage, ServerBuilder};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Renderer invocation count proves exactly when buffered rendering is skipped.
struct Renderer {
    contract: PageContract,
    calls: Arc<AtomicUsize>,
    fail: bool,
}
impl PageRenderer<String> for Renderer {
    fn contract(&self) -> &PageContract {
        &self.contract
    }
    fn render(&self, page: PageSnapshot<String>) -> RenderFuture<'_> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let fail = self.fail;
        Box::pin(async move {
            if fail {
                return Err(ViewError::from_source(std::io::Error::other(
                    "private render sentinel",
                )));
            }
            RenderedPage::new(format!(
                "{}:{}:{}",
                page.status.code(),
                page.location,
                page.data
            ))
        })
    }
}

/// Exercise cooperative abort, Result failure, status and explicit finalization.
async fn load(
    req: &mut Request,
    res: &mut Response,
    _: Invocation<'_>,
) -> Result<PageLoad<String>> {
    let mode = req.data.path_as::<String>("mode")?.unwrap_or_default();
    match mode.as_str() {
        "abort" => {
            res.html("abort body")?;
            return Ok(PageLoad::Abort);
        }
        "fail" | "error-abort" | "error-fail" => return Err(Error::new("private loader sentinel")),
        "stop" => res.stop()?,
        "sent" => {
            res.dispatch().await?;
        }
        "status" => {
            res.status_code(404, None)?;
        }
        _ => {}
    }
    Ok(PageLoad::Ready("public".into()))
}

/// Response hooks see the already buffered page and may replace it without rerendering.
async fn response(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    match req.data.path_as::<String>("mode")?.as_deref() {
        Some("response-abort") => return Ok(Flow::Abort),
        Some("response-fail") => return Err(Error::new("private response sentinel")),
        Some("data") => {
            res.data_mut()?.set("new", "after-render")?;
        }
        Some("results") => {
            res.results(&"changed result after rendering")?;
        }
        Some("replace") => {
            *res = Response::new();
            res.html("replacement")?;
        }
        _ => {}
    }
    Ok(Flow::Continue)
}

/// Request decisions must never enter the page loader/render stage.
async fn request(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    match req.data.path_as::<String>("mode")?.as_deref() {
        Some("request-abort") => Ok(Flow::Abort),
        Some("request-fail") => Err(Error::new("private request sentinel")),
        _ => Ok(Flow::Continue),
    }
}

/// Error decisions retain the original failure without recursively offering another error.
async fn error(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    match req.data.path_as::<String>("mode")?.as_deref() {
        Some("error-abort") => Ok(Flow::Abort),
        Some("error-fail") => Err(Error::new("private error listener sentinel")),
        _ => Ok(Flow::Continue),
    }
}

/// The page adapter inherits the actual host table instead of inventing finally/retry.
#[tokio::test]
async fn stage_table_and_drop() {
    for (mode, renders, outcome, state) in [
        ("ok", 1, None, DeliveryState::Open),
        (
            "abort",
            0,
            Some((false, LifecycleStage::Operation)),
            DeliveryState::Open,
        ),
        (
            "fail",
            0,
            Some((true, LifecycleStage::Operation)),
            DeliveryState::Open,
        ),
        ("stop", 0, None, DeliveryState::Stopped),
        ("sent", 0, None, DeliveryState::Sent),
        ("status", 1, None, DeliveryState::Open),
        (
            "response-abort",
            1,
            Some((false, LifecycleStage::Response)),
            DeliveryState::Open,
        ),
        (
            "response-fail",
            1,
            Some((true, LifecycleStage::Response)),
            DeliveryState::Open,
        ),
        ("replace", 1, None, DeliveryState::Open),
        (
            "request-abort",
            0,
            Some((false, LifecycleStage::Request)),
            DeliveryState::Open,
        ),
        (
            "request-fail",
            0,
            Some((true, LifecycleStage::Request)),
            DeliveryState::Open,
        ),
        (
            "render-fail",
            1,
            Some((true, LifecycleStage::Operation)),
            DeliveryState::Open,
        ),
        (
            "error-abort",
            0,
            Some((true, LifecycleStage::Operation)),
            DeliveryState::Open,
        ),
        (
            "error-fail",
            0,
            Some((true, LifecycleStage::Operation)),
            DeliveryState::Open,
        ),
        ("results", 1, None, DeliveryState::Open),
        ("data", 1, None, DeliveryState::Open),
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let weak = Arc::downgrade(&calls);
        let mut app = ServerBuilder::new();
        app.on("request", request).unwrap();
        app.on("response", response).unwrap();
        app.on("error", error).unwrap();
        app.page(
            PageSpec::get("/page", "page", load).unwrap(),
            Renderer {
                contract: PageContract::new(1, "test").unwrap(),
                calls: calls.clone(),
                fail: mode == "render-fail",
            },
        )
        .unwrap();
        let server = app.build().await.unwrap();
        let mut req = Request::builder()
            .url(format!("/page?mode={mode}&secret=NEVER_PROJECT"))
            .build()
            .unwrap();
        let mut res = Response::new();
        let report = server.handle_route(&mut req, &mut res).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), renders, "{mode}");
        assert_eq!(res.delivery_state(), state, "{mode}");
        if let Some((failed, stage)) = outcome {
            let expected = if failed {
                LifecycleOutcome::Failed { stage }
            } else {
                LifecycleOutcome::Aborted { stage }
            };
            assert_eq!(report.outcome, expected, "{mode}");
        }
        let json = serde_json::to_string(&res.try_to_json().unwrap()).unwrap();
        assert!(
            !json.contains("private ") && !json.contains("NEVER_PROJECT"),
            "{mode}: {json}"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .filter(|row| row.stage == LifecycleStage::Input)
                .count(),
            1
        );
        if mode == "error-fail" {
            assert_eq!(report.failures.len(), 2);
            assert_eq!(report.error_offers.len(), 1);
            assert!(report.terminal_failure);
        }
        if mode == "data" {
            assert!(
                matches!(res.body(), stackpress_lib::Body::Text(body) if body == "200:/page:public")
            );
        }
        if mode == "status" {
            assert!(json.contains("404:/page:public"));
        }
        if mode == "replace" {
            assert!(json.contains("replacement"));
        }
        if mode == "render-fail" {
            assert!(
                report
                    .developer()
                    .to_string()
                    .contains("private render sentinel")
            );
        }
        drop(server);
        drop(calls);
        assert!(
            weak.upgrade().is_none(),
            "render registration retained host state"
        );
    }
}

/// Duplicate identity claims fail build even across different route definitions.
#[tokio::test]
async fn duplicate_and_invalid_registration() {
    assert!(PageSpec::get("/page?x=1", "page", load).is_err());
    assert!(PageSpec::get("/page", "../page", load).is_err());
    let mut app = ServerBuilder::new();
    for path in ["/a", "/b"] {
        app.page(
            PageSpec::get(path, "same", load).unwrap(),
            Renderer {
                contract: PageContract::new(1, "test").unwrap(),
                calls: Arc::new(AtomicUsize::new(0)),
                fail: false,
            },
        )
        .unwrap();
    }
    assert!(app.build().await.is_err());
}
