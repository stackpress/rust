//! Route-phase request middleware keeps source order without reopening operations.

use stackpress_lib::{Flow, Invocation, Request, Response, Result, handler, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};

/// Append a visible call and optionally abort the ordinary all-matching request chain.
fn append(name: &'static str, abort: bool) -> impl stackpress_lib::Handler {
    handler(
        move |req: &mut Request, res: &mut Response, _: Invocation<'_>| {
            Box::pin(async move {
                let mut order = req
                    .data
                    .get("order")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned();
                order.push_str(name);
                req.data.set("order", order.clone())?;
                res.results(&order)?;
                Ok(if abort { Flow::Abort } else { Flow::Continue })
            })
        },
    )
}

/// Listen priorities sort around Route defaults, and any handler may abort.
#[tokio::test]
async fn route_request_hook_preserves_phase_order_and_abort() -> Result<()> {
    for (abort, expected) in [
        (false, "high,listen,route,low,"),
        (true, "high,listen,route,"),
    ] {
        let mut app = ServerBuilder::new();
        app.on_config(task(|start: &mut StartupContext| {
            Box::pin(async move {
                assert!(start.on_request(append("config", false)).is_err());
                Ok(Flow::Continue)
            })
        }))?;
        app.on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.on_priority("request", -1, append("low,", false))?;
                start.on_request(append("listen,", false))?;
                start.on_priority("request", 1, append("high,", false))?;
                Ok(Flow::Continue)
            })
        }))?;
        app.on_route(task(move |start: &mut StartupContext| {
            Box::pin(async move {
                for event in ["request", "other"] {
                    assert_eq!(
                        start
                            .on(event, append("invalid", false))
                            .unwrap_err()
                            .message(),
                        "operation registration requires Registration or Listen"
                    );
                }
                start.on_request(append("route,", abort))?;
                Ok(Flow::Continue)
            })
        }))?;
        let host = app.build().await.map_err(|error| error.error)?;
        let outcome = host.resolve("request", serde_json::json!({})).await?;
        assert_eq!(outcome.response.results.unwrap().as_str(), Some(expected));
        let listeners: Vec<_> = host.listeners().collect();
        assert_eq!(listeners.len(), 4);
        assert!(
            listeners
                .iter()
                .any(|listener| listener.label == "request (Route)")
        );
        assert!(
            listeners
                .iter()
                .all(|listener| listener.registered.file.ends_with("on_request.rs"))
        );
    }
    Ok(())
}
