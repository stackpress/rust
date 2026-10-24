//! Generated code hosted by real CLI, buffered HTTP and direct capability entrypoints.

use serde_json::{Value, json};
use stackpress_cli::{CliSinks, Terminal};
use stackpress_example_generated_profile::build_application;
use stackpress_lib::ParseLimits;
use stackpress_server::ConfigLayers;
use stackpress_server::http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput};

/// All adapters return the complete same public envelope from the generated operation.
#[tokio::test]
async fn generated_cli_http_and_direct_match_slice_two() {
    // Compare the actual Slice 2 application instead of a handwritten expected surrogate.
    let reference = stackpress_example_capability_host::build_application(ConfigLayers::new())
        .await
        .unwrap();
    let app = build_application(ConfigLayers::new()).await.unwrap();
    for id in ["abc123", "abc", "unknown"] {
        let direct = app.call("profile-search", json!({"id":id})).await.unwrap();
        let expected = direct.response.try_to_json().unwrap();
        let original = reference
            .call("profile-search", json!({"id":id}))
            .await
            .unwrap();
        assert_eq!(original.response.try_to_json().unwrap(), expected);
        let args = ["emit", "profile-search", "--id", id].map(String::from);
        let terminal = Terminal::parse(&args, ParseLimits::default()).unwrap();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        stackpress_cli::run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
            .await
            .unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&stdout).unwrap(), expected);
        assert!(stderr.is_empty());
        let mut input = BufferedRequest::new(Vec::new());
        *input.uri_mut() = format!("/profiles/{id}").parse().unwrap();
        let http = HttpAdapter::new(HttpOptions::default())
            .unwrap()
            .handle(&app, input)
            .await
            .unwrap();
        let HttpOutput::Response(response) = http.output else {
            panic!("missing HTTP output");
        };
        assert_eq!(
            serde_json::from_slice::<Value>(response.body()).unwrap(),
            expected
        );
        if id == "abc123" {
            assert_eq!(
                expected["results"],
                json!([{"id":"abc123","name":"Ada Lovelace"}])
            );
        }
    }

    // Typed/guard failures must agree with the original operation as well as successful rows.
    for data in [json!({}), json!({"id":""}), json!({"id":3})] {
        let generated = app.call("profile-search", data.clone()).await.unwrap();
        let original = reference.call("profile-search", data).await.unwrap();
        assert_eq!(
            generated.response.try_to_json().unwrap(),
            original.response.try_to_json().unwrap()
        );
    }
}

/// Generated input/config validation is real executed source, not an emitter snapshot.
#[tokio::test]
async fn generated_validation_and_trusted_limit() {
    let mut layers = ConfigLayers::new();
    layers
        .value("test", json!({"profiles":{"limit":1}}))
        .unwrap();
    let app = build_application(layers).await.unwrap();
    let output = app
        .call("profile-search", json!({"id":"abc","limit":99}))
        .await
        .unwrap();
    assert_eq!(output.response.try_to_json().unwrap()["total"], 1);
    // Preserve the existing Slice 2 boundary: typed deserialization errors map to 500;
    // the explicit domain guard on missing/empty strings maps to invalid-input 400.
    for (input, code) in [
        (json!({}), 400),
        (json!({"id":""}), 400),
        (json!({"id":3}), 500),
        (json!({"id":null}), 500),
    ] {
        assert_eq!(
            app.call("profile-search", input)
                .await
                .unwrap()
                .response
                .try_to_json()
                .unwrap()["code"],
            code
        );
    }
    let mut bad = ConfigLayers::new();
    bad.value("bad", json!({"profiles":{"limit":0}})).unwrap();
    assert!(build_application(bad).await.is_err());
}
