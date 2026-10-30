//! Incoming/outgoing isolation and the upstream variable-string comparison contract.

use serde_json::json;
use stackpress_framework_csrf::{CsrfConfig, DEFAULT_ERROR, Service};
use stackpress_lib::{CookieOptions, Request, Response, SessionValue, Value};

/// Build the source's two independently populated incoming values.
fn incoming(cookie: &str, input: &str) -> Request {
    let mut req = Request::new();
    req.session.set("csrf", cookie, None).unwrap();
    req.data.set("csrf", input).unwrap();
    req
}

/// Source defaults activate without requiring a CSRF configuration section.
#[test]
fn defaults_and_overrides_preserve_exact_error_and_field() {
    let config: CsrfConfig = serde_json::from_value(json!({})).unwrap();
    let default = Service::new(config);
    let mut res = Response::new();
    assert_eq!(default.name(), "csrf");
    assert!(!default.valid(&Request::new(), &mut res).unwrap());
    assert_eq!(res.error(), Some(DEFAULT_ERROR));
    assert_eq!(res.status().unwrap().code, 419);
    assert_eq!(res.status().unwrap().message, "Page Expired");

    // An explicit empty explanation is an override, not a fallback request.
    let custom = Service::new(CsrfConfig {
        name: Some("form".into()),
        error: Some("".into()),
    });
    let token = custom.generate(&mut Response::new()).unwrap();
    let mut req = Request::new();
    req.session.set("form", token.clone(), None).unwrap();
    req.data.set("form", token).unwrap();
    assert!(custom.valid(&req, &mut Response::new()).unwrap());
    assert!(!custom.valid(&Request::new(), &mut res).unwrap());
    assert_eq!(res.error(), Some(""));
}

/// Real entropy produces independently staged 32-byte lowercase-hex values.
#[test]
fn random_generation_stages_cookie_and_ssr_without_replacing_incoming() {
    let service = Service::default();
    let req = incoming("previous", "previous");
    let mut res = Response::new();
    // Existing cookie policy belongs to the host and survives token replacement.
    res.session_mut()
        .unwrap()
        .set(
            "csrf",
            "old",
            Some(CookieOptions {
                path: Some("/forms".into()),
                http_only: true,
                ..Default::default()
            }),
        )
        .unwrap();
    let first = service.generate(&mut res).unwrap();
    let second = service.generate(&mut Response::new()).unwrap();
    assert_ne!(first, second);
    assert_eq!(first.len(), 64);
    assert!(
        first
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_eq!(
        res.session().get("csrf").unwrap().as_str(),
        Some(first.as_str())
    );
    assert_eq!(
        res.data().get("csrf").unwrap().to_json().unwrap(),
        json!({"name":"csrf", "token":first})
    );
    assert_eq!(
        res.session().options("csrf").unwrap().path.as_deref(),
        Some("/forms")
    );
    assert!(service.valid(&req, &mut res).unwrap());
    assert_eq!(req.session.get("csrf").unwrap().as_str(), Some("previous"));
}

/// No source length/format policy beyond nonempty equal strings is introduced.
#[test]
fn arbitrary_strings_and_utf8_byte_lengths_follow_source() {
    let service = Service::default();
    for token in ["x", " ", "not hexadecimal!", "é", "😀", &"a".repeat(1024)] {
        let mut res = Response::new();
        assert!(service.valid(&incoming(token, token), &mut res).unwrap());
        assert!(res.status().is_none());
    }
    // Equal character counts can have different byte lengths; equal byte lengths can mismatch.
    for (cookie, input) in [
        ("", ""),
        ("a", ""),
        ("", "a"),
        ("é", "e"),
        ("é", "ab"),
        ("aa", "ab"),
    ] {
        let mut res = Response::new();
        assert!(!service.valid(&incoming(cookie, input), &mut res).unwrap());
        assert_eq!(res.status().unwrap().code, 419);
    }
}

/// Missing values, non-string input and list-valued cookies never coerce to strings.
#[test]
fn shape_rejections_and_query_aggregate_input() {
    let service = Service::default();
    for input in [
        json!(null),
        json!(false),
        json!(0),
        json!([]),
        json!({}),
        json!(["x"]),
    ] {
        let mut req = incoming("x", "x");
        req.data
            .set("csrf", Value::try_from(input).unwrap())
            .unwrap();
        assert!(!service.valid(&req, &mut Response::new()).unwrap());
    }
    let mut req = incoming("x", "x");
    req.session
        .set("csrf", SessionValue::List(vec!["x".into()]), None)
        .unwrap();
    assert!(!service.valid(&req, &mut Response::new()).unwrap());
    req = incoming("x", "x");
    req.data.remove("csrf").unwrap();
    assert!(!service.valid(&req, &mut Response::new()).unwrap());
    req = Request::builder().url("/?csrf=from-query").build().unwrap();
    req.session.set("csrf", "from-query", None).unwrap();
    assert!(service.valid(&req, &mut Response::new()).unwrap());
}

/// Clearing follows source deletion scope and low-level outgoing deletion semantics.
#[test]
fn clear_retains_incoming_cookie_and_unrelated_fields() {
    let service = Service::default();
    let mut req = incoming("old", "old");
    req.data.set("keep", true).unwrap();
    let mut res = Response::new();
    service.generate(&mut res).unwrap();
    res.data_mut().unwrap().set("keep", true).unwrap();
    res.session_mut()
        .unwrap()
        .set("keep", "cookie", None)
        .unwrap();
    service.clear(&mut req, &mut res).unwrap();
    assert!(!req.data.has("csrf"));
    assert!(!res.data().has("csrf"));
    assert!(res.session().get("csrf").is_none());
    assert_eq!(req.session.get("csrf").unwrap().as_str(), Some("old"));
    assert!(req.data.has("keep") && res.data().has("keep"));
    assert_eq!(res.session().get("keep").unwrap().as_str(), Some("cookie"));
    assert!(
        res.session()
            .cookie_headers()
            .unwrap()
            .iter()
            .any(|line| line.starts_with("csrf=") && line.contains("Max-Age=0"))
    );
    service.clear(&mut req, &mut res).unwrap();
}

/// Rust low-level cookie validation is propagated rather than silently renaming the field.
#[test]
fn invalid_cookie_name_propagates_existing_mechanics_error() {
    let service = Service::new(CsrfConfig {
        name: Some("bad name".into()),
        error: None,
    });
    let mut res = Response::new();
    assert!(service.generate(&mut res).is_err());
    assert!(!res.data().has("bad name"));
}

/// Source selection always publishes defaults but never installs a global request listener.
#[cfg(feature = "server")]
#[tokio::test]
async fn explicit_registrar_defaults_overrides_and_duplicate_mechanics() {
    use stackpress_framework_csrf::plugin;
    use stackpress_server::{ConfigLayers, ServerBuilder};
    for section in [
        None,
        Some(json!(null)),
        Some(json!(false)),
        Some(json!(0)),
        Some(json!("")),
        Some(json!({})),
        Some(json!({"name":"form","error":"custom"})),
    ] {
        let mut app = ServerBuilder::new();
        if let Some(section) = section.clone() {
            let mut layers = ConfigLayers::new();
            layers.value("fixture", json!({"csrf":section})).unwrap();
            app.config_layers(layers).unwrap();
        }
        app.plugin("csrf", plugin::register).unwrap();
        let host = app.build().await.unwrap();
        let csrf = host.host().service::<Service>().unwrap();
        assert_eq!(
            csrf.name(),
            if section
                .as_ref()
                .is_some_and(|section| section.get("name").is_some())
            {
                "form"
            } else {
                "csrf"
            }
        );
        assert_eq!(host.listeners().count(), 0);
        let mut res = Response::new();
        assert!(!csrf.valid(&Request::new(), &mut res).unwrap());
        assert_eq!(
            res.error(),
            Some(if csrf.name() == "form" {
                "custom"
            } else {
                DEFAULT_ERROR
            })
        );
    }
    let mut app = ServerBuilder::new();
    plugin::register(&mut app).unwrap();
    assert!(plugin::register(&mut app).is_err());
}
