//! Source-derived account invocation traces plus real SQLite/finite rendering proofs.
use serde_json::{Value as Json, json};
use stackpress_framework_csrf::{CsrfConfig, Service as Csrf};
use stackpress_framework_session::{
    Service, account,
    auth::{AuthConfig, Environment, events},
};
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::{Server, ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::{Arc, Mutex};

/// Native source helpers use a frozen epoch second for TOTP/CSV.
struct FixedClock;
impl stackpress_framework_session::Clock for FixedClock {
    fn now(&self) -> f64 {
        1_700_000_000.0
    }
}
/// Child results are explicit original fixture values, not a second Rust policy implementation.
struct Oracle {
    scenario: Json,
    fixture: Json,
    trace: Mutex<Vec<Json>>,
}
/// Record every nested invocation before applying the same finite child response as the source.
async fn child(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let oracle = inv.service::<Oracle>()?;
    let data = Json::Object(
        req.data
            .entries()
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.to_json()?)))
            .collect::<Result<_>>()?,
    );
    oracle
        .trace
        .lock()
        .unwrap()
        .push(json!(["resolve", inv.event(), data]));
    match inv.event() {
        "profile-detail" => {
            if oracle.scenario["missingProfile"] == true {
                res.set_error(stackpress_lib::PublicError {
                    code: 404,
                    status: "Not Found".into(),
                    error: "missing".into(),
                    errors: json!({}),
                })?;
            } else {
                res.results(
                    oracle
                        .scenario
                        .get("profile")
                        .unwrap_or(&oracle.fixture["profile"]),
                )?;
            }
        }
        "auth-search" => {
            let empty = json!([]);
            let rows = oracle
                .scenario
                .get("auths")
                .unwrap_or(if data["eq"]["type"] == "2fa" {
                    &empty
                } else {
                    &oracle.fixture["auths"]
                });
            res.results(rows)?;
        }
        _ => {
            let mut row = data.clone();
            if row.get("id").is_none() {
                row["id"] = json!("created");
            }
            res.results(&row)?;
        }
    }
    if oracle.scenario["failEvent"] == inv.event() {
        res.set_error(stackpress_lib::PublicError {
            code: 503,
            status: "Service Unavailable".into(),
            error: "child failure".into(),
            errors: json!({}),
        })?;
    }
    Ok(Flow::Continue)
}
/// Construct only the required typed services and ordinary nested-event registrations.
async fn oracle_host(fixture: &Json, case: &Json) -> Server {
    let mut app = ServerBuilder::new();
    let config:AuthConfig=serde_json::from_value(json!({"base":"/auth","password":case["scenario"].get("password").cloned().unwrap_or(json!({})),"2fa":{"issuer":"Brand & Co"}})).unwrap();
    app.config_section("auth", Some(config), |_| Ok(()))
        .unwrap();
    let oracle = Arc::new(Oracle {
        scenario: case["scenario"].clone(),
        fixture: fixture.clone(),
        trace: Mutex::new(Vec::new()),
    });
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let oracle = oracle.clone();
            Box::pin(async move {
                start.register(oracle)?;
                start.register(Arc::new(Service::default()))?;
                start.register(Arc::new(Csrf::new(CsrfConfig {
                    error: Some("csrf".into()),
                    ..Default::default()
                })))?;
                start.register(Arc::new(Environment {
                    clock: Arc::new(FixedClock),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    for name in [
        "profile-detail",
        "profile-update",
        "profile-remove",
        "auth-search",
        "auth-update",
        "auth-create",
        "auth-remove",
    ] {
        app.on(name, child).unwrap();
    }
    match case["scenario"]["target"].as_str().unwrap() {
        "account" => {
            app.on("subject", account::account).unwrap();
        }
        "update" => {
            app.on("subject", account::update).unwrap();
        }
        "password" => {
            app.on("subject", account::password).unwrap();
        }
        "remove" => {
            app.on("subject", account::remove).unwrap();
        }
        "export" => {
            app.on("subject", account::export).unwrap();
        }
        "2fa/detail" => {
            app.on("subject", account::enroll).unwrap();
        }
        "2fa/remove" => {
            app.on("subject", account::remove_factor).unwrap();
        }
        _ => unreachable!(),
    }
    app.build().await.unwrap()
}
/// Execute every source branch and compare child arguments, results, errors, redirects and flash.
#[tokio::test]
async fn source_account_invocation_traces() {
    let fixture: Json = serde_json::from_str(include_str!("../account-flows.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let name = case["scenario"]["name"].as_str().unwrap();
        let expected = &case["expected"];
        let host = oracle_host(&fixture, case).await;
        let mut req = Request::builder()
            .method(case["scenario"]["method"].as_str().unwrap_or("GET"))
            .data(case["scenario"].get("data").cloned().unwrap_or(json!({})))
            .build()
            .unwrap();
        req.session.set("csrf", "same", None).unwrap();
        req.data
            .set(
                "csrf",
                if case["scenario"]["csrf"] == false {
                    "wrong"
                } else {
                    "same"
                },
            )
            .unwrap();
        if case["scenario"]["guest"] != true {
            let token = host
                .host()
                .service::<Service>()
                .unwrap()
                .create(fixture["me"].as_object().unwrap().clone())
                .await
                .unwrap();
            req.session.set("session", token, None).unwrap();
        }
        let mut res = Response::new();
        if let Some(body) = case["scenario"]["body"].as_str() {
            res.html(body).unwrap();
        }
        if case["scenario"]["earlyError"] == true {
            res.set_error(stackpress_lib::PublicError {
                code: 409,
                status: "Conflict".into(),
                error: "early".into(),
                errors: json!({}),
            })
            .unwrap();
        }
        let result = host.emit("subject", &mut req, &mut res).await;
        if let Some(message) = expected["thrown"].as_str() {
            assert_eq!(result.unwrap_err().to_string(), message, "{name}: thrown");
            continue;
        }
        result.unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            res.status().map_or(0, |s| s.code),
            expected["code"].as_u64().unwrap() as u16,
            "{name}: code"
        );
        assert_eq!(
            res.error().map(Json::from).unwrap_or(Json::Null),
            expected["error"],
            "{name}: error"
        );
        assert_eq!(
            res.errors().to_json().unwrap(),
            expected["errors"],
            "{name}: errors"
        );
        assert_eq!(
            res.headers()
                .get("Location")
                .map(Json::from)
                .unwrap_or(Json::Null),
            expected["location"],
            "{name}: redirect"
        );
        let mut result = match res.body() {
            Body::Text(text) => json!(text),
            _ => events::json_result(&res).unwrap(),
        };
        if let Some(url) = result.get("url").and_then(Json::as_str) {
            use base64::{Engine, engine::general_purpose::STANDARD};
            let bytes = STANDARD
                .decode(url.strip_prefix("data:image/png;base64,").unwrap())
                .unwrap();
            assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
            assert_eq!(
                u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
                expected["qr"]["width"].as_u64().unwrap() as u32,
                "{name}: QR width"
            );
            result["url"] = json!("PNG");
        }
        assert_eq!(result, expected["result"], "{name}: exact result/CSV bytes");
        if case["scenario"]["target"] == "export" && case["scenario"]["data"]["download"] == true {
            assert_eq!(
                res.headers()
                    .get("Content-Disposition")
                    .map(Json::from)
                    .unwrap_or(Json::Null),
                expected["attachment"]
            );
        }
        let flash = res
            .session()
            .get("flash")
            .and_then(|value| value.as_str())
            .map(|raw| serde_json::from_str::<Json>(raw).unwrap())
            .unwrap_or(Json::Null);
        assert_eq!(flash, expected["flash"], "{name}: flash");
        let expected_calls = expected["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v[0] == "resolve")
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            *host
                .host()
                .service::<Oracle>()
                .unwrap()
                .trace
                .lock()
                .unwrap(),
            expected_calls,
            "{name}: child event order/arguments"
        );
    }
}

#[path = "account_flows/sql.rs"]
mod sql;
