//! Real per-call adapter paths used by the finite form and by direct event consumers.

use super::{host, request, vectors};
use serde_json::{Value as Json, json};
use stackpress_framework_email::events::{EMAIL_SEND, NativeReader};
use stackpress_lib::{Request, Response};
use std::sync::atomic::Ordering;
use tokio::io::AsyncReadExt;

/// Each source transport variant crosses CSRF validation and the same frozen email event.
#[tokio::test]
async fn every_transport_option_fixture_crosses_the_finite_form_and_direct_event() {
    let case = vectors()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "otp-success")
        .unwrap()
        .clone();
    let variants = [
        (
            "smtp",
            json!({"host":"relay.example.test","port":25,"ignoreTLS":true,"auth":{"user":"fixture","pass":"SMTP_PRIVATE_SENTINEL"},"tls":{"rejectUnauthorized":false}}),
        ),
        (
            "pool",
            json!({"pool":true,"maxConnections":2,"maxMessages":3,"socketTimeout":17,"extension":{"retained":true}}),
        ),
        (
            "json-object",
            json!({"jsonTransport":true,"skipEncoding":true}),
        ),
        (
            "json-string",
            json!({"jsonTransport":true,"skipEncoding":false}),
        ),
        (
            "stream-buffer",
            json!({"streamTransport":true,"buffer":true,"newline":"windows"}),
        ),
        (
            "stream-reader",
            json!({"streamTransport":true,"buffer":false}),
        ),
        (
            "sendmail",
            json!({"sendmail":true,"path":"/fixture/sendmail","args":["fixture-argument"]}),
        ),
        ("ses", json!({"SES":{"extension":"native-binding"}})),
        (
            "string",
            json!("smtp://fixture:SMTP_PRIVATE_SENTINEL@relay.example.test:25/?ignoreTLS=true"),
        ),
    ];
    for (kind, config) in variants {
        let (server, state) = host(case.clone(), Some(config.clone())).await;
        let mut req = request(&case);
        let mut res = Response::new();
        server
            .handle("finite-page", &mut req, &mut res)
            .await
            .unwrap();
        assert_eq!(res.status().unwrap().code, 302, "{kind}");
        assert!(!res.data().has("csrf"));
        assert_eq!(state.factory.created.load(Ordering::SeqCst), 1, "{kind}");
        assert_eq!(
            serde_json::to_value(&state.factory.configs.lock().unwrap()[0]).unwrap(),
            config
        );
        assert_eq!(json!(state.factory.sent.lock().unwrap()[0]), case["mail"]);
        // A separate native event consumer can inspect provider info and consume a stream.
        let info = server
            .resolve(EMAIL_SEND, case["mail"].clone())
            .await
            .unwrap();
        assert_eq!(info.response.code, Some(200));
        assert_eq!(state.factory.created.load(Ordering::SeqCst), 2);
        let result = info.response.results.unwrap();
        let fields = result.as_object().unwrap();
        if kind == "stream-reader" {
            let mut reader = fields["message"]
                .as_shared()
                .unwrap()
                .downcast_ref::<NativeReader>()
                .unwrap()
                .take()
                .unwrap();
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await.unwrap();
            let mime = String::from_utf8(bytes).unwrap();
            assert!(mime.contains("fixture sign-in"));
            assert!(mime.contains("recipient@example.test"));
        } else if kind == "pool" {
            assert_eq!(fields["sourceConfig"].to_json().unwrap(), config);
        } else if kind.starts_with("json") {
            let value = fields["message"].to_json().unwrap();
            if kind == "json-object" {
                assert_eq!(value["text"], case["mail"]["text"]);
            } else {
                let decoded: Json = serde_json::from_str(value.as_str().unwrap()).unwrap();
                assert_eq!(decoded["html"], case["mail"]["html"]);
            }
        }
        let calls = state.factory.seams.calls.lock().unwrap();
        match kind {
            "smtp" | "string" => {
                assert_eq!(calls[0]["kind"], "smtp");
                assert_eq!(calls[0]["options"]["auth"]["pass"], "SMTP_PRIVATE_SENTINEL");
                assert!(
                    calls[0]["message"]
                        .as_str()
                        .unwrap()
                        .contains("fixture sign-in")
                );
            }
            "pool" => {
                assert_eq!(calls[0]["kind"], "provider");
                assert_eq!(calls[0]["config"], config);
            }
            "sendmail" => {
                assert_eq!(calls[0]["path"], "/fixture/sendmail");
                assert_eq!(calls[0]["args"][0], "-i");
                assert_eq!(calls[0]["args"][1], "fixture-argument");
            }
            "ses" => {
                assert_eq!(calls[0]["kind"], "ses");
                assert_eq!(
                    calls[0]["input"]["Destination"]["ToAddresses"],
                    json!(["recipient@example.test"])
                );
            }
            _ => assert!(calls.is_empty()),
        }
    }
}

/// Source error status skips the production factory; later independent roots still work.
#[tokio::test]
async fn invalid_csrf_and_child_error_skip_transports_and_preserve_prior_response() {
    for name in ["invalid-csrf", "otp-child-status"] {
        let case = vectors()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap()
            .clone();
        let (server, state) = host(
            case.clone(),
            Some(json!({"host":"relay.example.test","ignoreTLS":true})),
        )
        .await;
        let mut req = request(&case);
        let mut res = Response::new();
        server
            .handle("finite-page", &mut req, &mut res)
            .await
            .unwrap();
        assert_eq!(res.status().unwrap().code, 419);
        assert_eq!(state.factory.created.load(Ordering::SeqCst), 0);
        assert!(state.factory.seams.calls.lock().unwrap().is_empty());
        // The direct event with an already failed response is a normal Continue skip as well.
        let mut direct = Request::new();
        let before = res.to_envelope().try_to_json().unwrap();
        server
            .emit(EMAIL_SEND, &mut direct, &mut res)
            .await
            .unwrap();
        assert_eq!(res.to_envelope().try_to_json().unwrap(), before);
        assert_eq!(state.factory.created.load(Ordering::SeqCst), 0);
    }
}
