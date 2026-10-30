//! Original-source invocation oracles and finite child-event adapters.
use super::*;

/// Source external calls remain observable through ordinary event handlers.
struct Oracle {
    scenario: Json,
    nested: Json,
    trace: Mutex<Vec<Json>>,
}
/// Return the captured original child outcome; the production parent is the unit under test.
async fn fake_signin(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let oracle = inv.service::<Oracle>()?;
    oracle
        .trace
        .lock()
        .unwrap()
        .push(json!(["emit", inv.event()]));
    let nested = &oracle.nested[0];
    oracle.trace.lock().unwrap().extend(
        nested["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry[0] == "resolve")
            .cloned(),
    );
    if !nested["result"].is_null() {
        res.results_value(native(&nested["result"]))?;
    }
    if let Some(location) = nested["location"].as_str() {
        res.redirect(location)?;
    }
    if let Some(error) = nested["error"].as_str() {
        res.set_error(stackpress_lib::PublicError {
            code: nested["code"].as_u64().unwrap() as u16,
            status: "Bad Request".into(),
            error: error.into(),
            errors: nested["errors"].clone(),
        })?;
    }
    Ok(Flow::Continue)
}
/// Source resolve(...) calls fresh data; update/email pass through the parent's response.
async fn fake_child(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let oracle = inv.service::<Oracle>()?;
    let input = Json::Object(
        req.data
            .entries()
            .iter()
            .filter(|(key, _)| key.as_str() != "csrf")
            .map(|(key, value)| (key.clone(), json_value(value.clone())))
            .collect(),
    );
    oracle
        .trace
        .lock()
        .unwrap()
        .push(json!(["resolve", inv.event(), input]));
    let fixture: Json = serde_json::from_str(include_str!("../../auth-flows.json"))?;
    let row = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["scenario"]["name"] == "otp-get-no-code-check")
        .unwrap()["scenario"]["search"][0]
        .clone();
    let searches = oracle
        .trace
        .lock()
        .unwrap()
        .iter()
        .filter(|entry| entry[1] == "auth-search")
        .count();
    match inv.event() {
        "auth-search" => {
            let absent = json!([]);
            let fallback = if oracle.scenario["issue"] == true {
                json!([row])
            } else {
                json!([])
            };
            let rows = if searches > 1 && oracle.scenario["disappear"] == true {
                &absent
            } else {
                oracle.scenario.get("search").unwrap_or(&fallback)
            };
            res.results_value(native(rows))?;
        }
        "auth-update" => {
            res.results_value(native(&row))?;
        }
        "email-send" => {
            res.results(&json!({"sent":true}))?;
        }
        "auth-signout" => {
            events::signout(req, res, inv).await?;
            return Ok(Flow::Continue);
        }
        _ => unreachable!(),
    }
    if oracle.scenario["failEvent"] == inv.event()
        || (oracle.scenario["failEvent"] == "reload"
            && inv.event() == "auth-search"
            && searches > 1)
    {
        res.set_error(stackpress_lib::PublicError {
            code: 503,
            status: "Service Unavailable".into(),
            error: "child failure".into(),
            errors: json!({}),
        })?;
    }
    Ok(Flow::Continue)
}
/// Build a finite process host with only explicit dependencies; no port is opened.
async fn process_host(case: &Json) -> Arc<Server> {
    let scenario = case["scenario"].clone();
    let mut config = json!({"base":"/auth","roles":["USER"],"email":{"name":"Sender","address":"sender@example.test"}});
    if scenario["noSender"] == true {
        config.as_object_mut().unwrap().remove("email");
    }
    if scenario["noBase"] == true {
        config.as_object_mut().unwrap().remove("base");
    }
    let mut app = ServerBuilder::new();
    app.config_section(
        "auth",
        Some(serde_json::from_value::<AuthConfig>(config).unwrap()),
        |_| Ok(()),
    )
    .unwrap();
    let oracle = Arc::new(Oracle {
        scenario: scenario.clone(),
        nested: case["nested"].clone(),
        trace: Mutex::new(Vec::new()),
    });
    let signed_in = scenario["signedIn"] == true;
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let oracle = oracle.clone();
            let scenario = scenario.clone();
            Box::pin(async move {
                start.register(oracle)?;
                start.register(Arc::new(Service::default()))?;
                start.register(Arc::new(Csrf::new(CsrfConfig {
                    error: Some("csrf".into()),
                    ..Default::default()
                })))?;
                start.register(Arc::new(Environment {
                    email_configured: scenario["noEmail"] != true,
                    host: Some("https://example.test".into()),
                    brand: "Brand".into(),
                    clock: Arc::new(FixedClock),
                    random: Arc::new(|| Ok(0.000007)),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("auth-signin", fake_signin).unwrap();
    app.on("auth-signup", fake_signin).unwrap();
    for event in ["auth-search", "auth-update", "email-send", "auth-signout"] {
        app.on(event, fake_child).unwrap();
    }
    // Direct handlers deliberately need no selected auth registrar or route ownership marker.
    match case["scenario"]["target"].as_str().unwrap() {
        "auth/pages/signup" => {
            app.on("subject", pages::signup).unwrap();
        }
        "auth/pages/signin/username" => {
            app.on("subject", pages::username).unwrap();
        }
        "auth/pages/signin/phone" => {
            app.on("subject", pages::phone).unwrap();
        }
        "auth/pages/signin/email" => {
            app.on("subject", pages::email).unwrap();
        }
        "auth/pages/signin/otp" => {
            app.on("subject", pages::otp).unwrap();
        }
        "auth/pages/signin/link" => {
            app.on("subject", pages::magic_link).unwrap();
        }
        "auth/pages/signin/2fa" => {
            app.on("subject", pages::two_factor).unwrap();
        }
        "auth/pages/signin" => {
            app.on("subject", pages::signin).unwrap();
        }
        "auth/pages/signout" => {
            app.on("subject", pages::signout).unwrap();
        }
        _ => unreachable!(),
    }
    let host = Arc::new(app.build().await.unwrap());
    // Keep the fixture branch visible; the caller installs the verified token in its request.
    assert_eq!(signed_in, case["scenario"]["signedIn"] == true);
    host
}
#[tokio::test]
async fn source_process_invocation_traces() {
    let fixture: Json = serde_json::from_str(include_str!("../../auth-flows.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap().iter().filter(|case| {
        case["scenario"]["target"]
            .as_str()
            .unwrap()
            .starts_with("auth/pages/")
    }) {
        let name = case["scenario"]["name"].as_str().unwrap();
        let host = process_host(case).await;
        let mut req = Request::builder()
            .method(case["scenario"]["method"].as_str().unwrap_or("GET"))
            .data(case["scenario"]["data"].clone())
            .build()
            .unwrap();
        req.session.set("csrf", "same", None).unwrap();
        req.data.set("csrf", "same").unwrap();
        if case["scenario"]["csrf"] == false {
            req.data.set("csrf", "wrong").unwrap();
        }
        if case["scenario"]["signedIn"] == true {
            let svc = host.host().service::<Service>().unwrap();
            let token = svc
                .create(json!({"id":"p1"}).as_object().unwrap().clone())
                .await
                .unwrap();
            req.session.set("session", token, None).unwrap();
        }
        let mut res = Response::new();
        if let Some(body) = case["scenario"]["body"].as_str() {
            res.html(body).unwrap();
        }
        host.emit("subject", &mut req, &mut res)
            .await
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let expected = &case["expected"];
        assert_eq!(
            res.status().map_or(0, |status| status.code),
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
        let body = match res.body() {
            stackpress_lib::Body::Text(text) => json!(text),
            _ => events::json_result(&res).unwrap(),
        };
        assert_eq!(body, expected["result"], "{name}: result");
        let mut actual = Json::Object(
            req.data
                .entries()
                .iter()
                .filter(|(key, _)| key.as_str() != "csrf")
                .map(|(key, value)| (key.clone(), json_value(value.clone())))
                .collect(),
        );
        // Source phone normalization stores undefined; JSON snapshot omits that key likewise.
        actual.as_object_mut().unwrap().remove("csrf");
        assert_eq!(actual, expected["input"], "{name}: mutations");
        assert_eq!(
            res.session()
                .get("flash")
                .and_then(|value| value.as_str())
                .map(Json::from)
                .unwrap_or(Json::Null),
            expected["flash"],
            "{name}: flash"
        );
        let expected_trace: Vec<_> = expected["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| ["resolve", "emit"].contains(&entry[0].as_str().unwrap_or("")))
            .cloned()
            .collect();
        assert_eq!(
            *host
                .host()
                .service::<Oracle>()
                .unwrap()
                .trace
                .lock()
                .unwrap(),
            expected_trace,
            "{name}: nested calls"
        );
    }
}
#[test]
fn source_templates_totp_and_phone_vectors() {
    let fixture: Json = serde_json::from_str(include_str!("../../auth-flows.json")).unwrap();
    for case in fixture["templates"].as_array().unwrap() {
        let input = &case["input"];
        let input = auth::templates::SigninEmailTemplateInput {
            brand: input["brand"].as_str(),
            email: input["email"].as_str().unwrap(),
            link: input["link"].as_str().unwrap(),
            name: input["name"].as_str(),
            pin: input["pin"].as_str(),
        };
        let template = if case["otp"] == true {
            auth::templates::make_otp_template(input)
        } else {
            auth::templates::make_magic_link_template(input)
        };
        assert_eq!(serde_json::to_value(template).unwrap(), case["expected"]);
    }
    for vector in fixture["totp"].as_array().unwrap() {
        assert_eq!(
            auth::totp::generate_at(
                "JBSWY3DPEHPK3PXP",
                vector["seconds"].as_u64().unwrap(),
                0,
                30
            )
            .unwrap(),
            vector["code"].as_str().unwrap()
        );
    }
    assert_eq!(auth::totp::base32_decode("mzxw6===").unwrap(), b"foo");
    assert!(auth::totp::base32_decode("!").is_err());
    assert_eq!(
        events::normalize_phone(Some(" +1 (234)-567 ")).as_deref(),
        Some("+1234567")
    );
    assert_eq!(events::normalize_phone(None), None);
    assert_eq!(events::normalize_phone(Some("+---")), None);
}

/// Direct exported helper intentionally bypasses page method/session/CSRF checks.
async fn direct_otp(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    pages::otp_signin(req, res, &inv).await?;
    Ok(Flow::Continue)
}
#[tokio::test]
async fn direct_helper_without_session_csrf_or_transport_still_rotates_before_skip() {
    let fixture: Json = serde_json::from_str(include_str!("../../auth-flows.json")).unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["scenario"]["name"] == "otp-missing-transport")
        .unwrap();
    let oracle = Arc::new(Oracle {
        scenario: case["scenario"].clone(),
        nested: json!([]),
        trace: Mutex::new(Vec::new()),
    });
    let mut app = ServerBuilder::new();
    app.config_section("auth", Some(AuthConfig::default()), |_| Ok(()))
        .unwrap();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let oracle = oracle.clone();
            Box::pin(async move {
                start.register(oracle)?;
                start.register(Arc::new(Environment {
                    clock: Arc::new(FixedClock),
                    random: Arc::new(|| Ok(0.000007)),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("auth-search", fake_child).unwrap();
    app.on("auth-update", fake_child).unwrap();
    app.on("direct", direct_otp).unwrap();
    let server = app.build().await.unwrap();
    let result = invoke(&server, "direct", json!({"email":"ada@example.test"})).await;
    assert_eq!(result.status().unwrap().code, 200);
    assert!(
        events::json_result(&result)
            .unwrap()
            .get("secret")
            .is_some()
    );
    let calls = server
        .host()
        .service::<Oracle>()
        .unwrap()
        .trace
        .lock()
        .unwrap();
    let names: Vec<_> = calls.iter().map(|call| call[1].as_str().unwrap()).collect();
    assert_eq!(names, vec!["auth-search", "auth-update", "auth-search"]);
    assert!(server.host().service::<Service>().is_err());
    assert!(server.host().service::<Csrf>().is_err());
}
#[test]
fn native_result_projection_preserves_dates_and_rejects_unrepresentable_io() {
    let mut response = Response::new();
    response
        .results_value(Lib::DateTime(Timestamp::new(-1, 999_000_000).unwrap()))
        .unwrap();
    assert_eq!(
        events::json_result(&response).unwrap(),
        json!("1969-12-31T23:59:59.999Z")
    );
    response.results_value(Lib::Bytes(vec![255])).unwrap();
    assert!(events::json_result(&response).is_err());
    assert!(auth::totp::verify_at("anything", "", 100, i64::MIN).is_err());
    assert!(auth::totp::generate_at("", 0, -1, 30).is_err());
    assert!(auth::totp::generate_at("", 0, 0, 0).is_err());
}
