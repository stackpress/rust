//! Compiled only inside the disposable app after model or selected-plugin removal.

use serde_json::{Value, json};
use stackpress_cli::{CliSinks, Terminal};
use stackpress_example_generated_profile::build_application;
use stackpress_server::ConfigLayers;
use stackpress_server::http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput};

/// No stale generated registration remains reachable through any application adapter.
#[tokio::test]
async fn removed_operation_is_absent_on_every_adapter() {
    let app = build_application(ConfigLayers::new()).await.unwrap();
    // The direct call enters the actual no-op generated registry.
    let direct = app
        .call("profile-search", json!({"id":"abc123"}))
        .await
        .unwrap();
    assert_eq!(direct.response.try_to_json().unwrap()["code"], 404);

    // Route registration disappears together with the capability registration.
    let mut input = BufferedRequest::new(Vec::new());
    *input.uri_mut() = "/profiles/abc123".parse().unwrap();
    let reply = HttpAdapter::new(HttpOptions::default())
        .unwrap()
        .handle(&app, input)
        .await
        .unwrap();
    let HttpOutput::Response(response) = reply.output else {
        panic!("missing HTTP output")
    };
    assert_eq!(response.status().as_u16(), 404);

    // The CLI still delivers one failing envelope and keeps its application exit policy.
    let args = ["emit", "profile-search", "--id", "abc123"].map(String::from);
    let terminal = Terminal::parse(&args, Default::default()).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let result = stackpress_cli::run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    assert_eq!(result.exit_code, 1);
    assert_eq!(
        serde_json::from_slice::<Value>(&stdout).unwrap()["code"],
        404
    );
}
