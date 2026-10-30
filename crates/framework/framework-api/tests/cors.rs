//! Exact URL-origin, header, preflight, priority and duplicate CORS behavior.

use std::sync::Arc;

use serde_json::json;
use stackpress_framework_api::{ApiType, Cors, cors};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt};

use super::endpoints::{State, direct, endpoint, request, status};

#[tokio::test]
async fn source_origin_and_header_table_uses_url_not_incoming_origin() -> Result<()> {
    for (origins, expected) in [
        (None, None),
        (Some(Cors::Enabled(false)), None),
        (Some(Cors::Enabled(true)), Some("*")),
        (Some(Cors::Origin("*".into())), Some("*")),
        (
            Some(Cors::Origin("https://api.invalid".into())),
            Some("https://api.invalid"),
        ),
        (
            Some(Cors::Origins(vec!["https://caller.invalid".into()])),
            None,
        ),
        (
            Some(Cors::Origins(vec!["https://api.invalid/".into()])),
            None,
        ),
        (Some(Cors::Origins(vec![])), None),
        (Some(Cors::Origin("".into())), None),
        (
            Some(Cors::Origins(vec!["unmatched".into(), "*".into()])),
            Some("*"),
        ),
    ] {
        let mut config = endpoint(ApiType::Public, "PATCH", "/api");
        config.cors = origins;
        let state = Arc::new(State::default());
        let server = direct(&[config], state.clone()).await?;
        let mut req = request("PATCH", "https://api.invalid:443/api", None, json!({}))?;
        req.headers.set("origin", "https://caller.invalid")?;
        let mut res = Response::new();
        server.emit("PATCH /api", &mut req, &mut res).await?;
        assert_eq!(res.headers().get("Access-Control-Allow-Origin"), expected);
        for (name, value) in [
            ("Access-Control-Request-Method", "PATCH"),
            ("Access-Control-Allow-Methods", "OPTIONS, GET"),
            ("Access-Control-Allow-Headers", "*"),
        ] {
            assert_eq!(res.headers().get(name), expected.map(|_| value));
        }
        assert!(
            res.headers()
                .get("Access-Control-Allow-Credentials")
                .is_none()
        );
        assert!(res.headers().get("Vary").is_none());
        assert!(res.headers().get("Access-Control-Allow-Method").is_none());
        assert!(
            state.events().contains(&"target".into()),
            "unmatched CORS is not admission"
        );
    }
    Ok(())
}

#[tokio::test]
async fn unmatched_preflight_is_200_continue_and_keeps_existing_error() -> Result<()> {
    let mut config = endpoint(ApiType::App, "POST", "/api");
    config.cors = Some(Cors::Origins(vec![]));
    let state = Arc::new(State::default());
    let server = direct(&[config], state.clone()).await?;
    let mut res = Response::new();
    stackpress_framework_api::helpers::unauthorized(&mut res)?;
    server
        .emit(
            "OPTIONS /api",
            &mut request("OPTIONS", "/api", None, json!({}))?,
            &mut res,
        )
        .await?;
    assert_eq!(status(&res), Some(200));
    assert_eq!(
        res.error(),
        Some("Unauthorized"),
        "preflight changes code only"
    );
    assert!(res.headers().get("Access-Control-Allow-Origin").is_none());
    assert_eq!(state.events(), ["route-tail"]);
    assert_eq!(state.calls()[0].status, Some(200));
    Ok(())
}

#[tokio::test]
async fn all_options_reaches_ordinary_non_get_admission_after_preflight() -> Result<()> {
    for (method, advertised) in [("ALL", "*"), ("all", "all")] {
        for (authorization, code, target) in [
            (None, 401, false),
            (Some("X id"), 401, false),
            (Some("X id:stored"), 200, true),
        ] {
            let mut config = endpoint(ApiType::App, method, "/api");
            config.cors = Some(Cors::Enabled(true));
            let state = Arc::new(State::default());
            let server = direct(&[config], state.clone()).await?;
            let mut res = Response::new();
            server
                .emit(
                    "OPTIONS /api",
                    &mut request("OPTIONS", "/api", authorization, json!({}))?,
                    &mut res,
                )
                .await?;
            assert_eq!(status(&res), Some(code));
            assert_eq!(
                res.headers().get("Access-Control-Request-Method"),
                Some(advertised)
            );
            assert_eq!(state.events().contains(&"target".into()), target);
            if target {
                assert_eq!(
                    state
                        .calls()
                        .into_iter()
                        .find(|call| call.event == "target")
                        .unwrap()
                        .status,
                    Some(200)
                );
            }
        }
    }
    Ok(())
}

/// CORS's priority must have run before the endpoint's lower-priority target event.
async fn see_headers(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(
        res.headers().get("Access-Control-Allow-Origin"),
        Some("https://api.invalid")
    );
    assert_eq!(
        res.headers().get("Access-Control-Request-Method"),
        Some("GET")
    );
    assert_eq!(inv.service::<State>()?.events(), ["target", "target-tail"]);
    Ok(Flow::Continue)
}

#[tokio::test]
async fn duplicate_policies_run_fifo_without_union_and_unmatched_keeps_prior_headers() -> Result<()>
{
    let mut first = endpoint(ApiType::Public, "GET", "/api");
    first.cors = Some(Cors::Enabled(true));
    let mut second = first.clone();
    second.cors = Some(Cors::Origin("https://api.invalid".into()));
    let mut third = first.clone();
    third.cors = Some(Cors::Origins(vec![]));
    let state = Arc::new(State::default());
    let mut app = super::endpoints::fixture(state.clone())?;
    cors::register(&mut app, first)?;
    cors::register(&mut app, second.clone())?;
    cors::register(&mut app, third)?;
    stackpress_framework_api::routes::open(&mut app, second)?;
    app.on("target", see_headers)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let mut res = Response::new();
    server
        .emit(
            "GET /api",
            &mut request("GET", "https://api.invalid/api", None, json!({}))?,
            &mut res,
        )
        .await?;
    assert_eq!(
        res.headers().get("Access-Control-Allow-Origin"),
        Some("https://api.invalid")
    );
    assert_eq!(
        server
            .listeners()
            .filter(|listener| listener.priority == 1000)
            .count(),
        6
    );
    Ok(())
}

#[tokio::test]
async fn disabled_direct_cors_does_not_validate_unused_route_fields() -> Result<()> {
    let mut config = endpoint(ApiType::Public, "invalid method", "/api");
    let mut app = ServerBuilder::new();
    cors::register(&mut app, config.clone())?;
    config.cors = Some(Cors::Enabled(false));
    stackpress_framework_api::routes::cors(&mut app, config.clone())?;
    config.cors = Some(Cors::Enabled(true));
    assert!(cors::register(&mut app, config).is_err());
    Ok(())
}
