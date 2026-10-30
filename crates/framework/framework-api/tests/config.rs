//! Every source configuration field and its inherited typed representation boundary.

use serde_json::json;
use stackpress_framework_api::{ApiConfig, ApiType, Cors, Service, plugin};
use stackpress_framework_schema::Input;
use stackpress_lib::{Error, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};

#[tokio::test]
async fn all_config_keys_survive_selected_registration() -> Result<()> {
    let wire = json!({
        "expires": -0.5,
        "scopes": {"read": {"icon": "key", "name": "Read", "description": "Read records"}},
        "webhooks": [{"event": "written", "uri": "http://configured/private", "method": "POST",
            "validity": {"ok": true}, "data": {"secret": "source-fixed"}}],
        "endpoints": [{"name": "Endpoint", "description": "Source description", "example": "Example",
            "method": "PATCH", "route": "/api/:id", "type": "session",
            "scopes": ["unregistered", "write"], "cors": ["https://api.invalid"], "event": "custom",
            "priority": -9, "data": {"active": false, "nested": {"keep": true}}}]
    });
    let expected: ApiConfig = serde_json::from_value(wire.clone()).map_err(Error::from_source)?;
    let roundtrip: ApiConfig =
        serde_json::from_value(serde_json::to_value(&expected).map_err(Error::from_source)?)
            .map_err(Error::from_source)?;
    assert_eq!(roundtrip, expected);
    let mut layers = ConfigLayers::new();
    layers.value("test", json!({"api": wire}))?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("configured-api", plugin::register)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let actual = &server.host().service::<Service>()?.config;
    assert_eq!(actual, &expected);
    assert_eq!(actual.expires, Input::Value(-0.5));
    assert_eq!(actual.endpoints[0].api_type, ApiType::Session);
    assert_eq!(actual.endpoints[0].priority, -9);
    assert_eq!(
        server.listeners().count(),
        6 + usize::from(cfg!(feature = "ui-ssr")),
        "M3 process pair, CORS pair, endpoint and M4 webhook"
    );
    Ok(())
}

#[test]
fn source_defaults_and_all_cors_shapes_are_represented() -> Result<()> {
    let config: ApiConfig = serde_json::from_value(json!({"extra": "ignored", "endpoints": [
        {"method": "GET", "route": "/", "type": "public", "event": "e"}
    ]}))
    .map_err(Error::from_source)?;
    assert!(config.expires.is_missing());
    assert!(config.webhooks.is_empty());
    assert!(config.scopes.is_empty());
    let endpoint = &config.endpoints[0];
    assert!(
        endpoint.name.is_none() && endpoint.description.is_none() && endpoint.example.is_none()
    );
    assert!(endpoint.scopes.is_empty() && endpoint.data.is_empty() && endpoint.cors.is_none());
    assert_eq!(endpoint.priority, 0);
    for (wire, expected) in [
        (json!(false), Cors::Enabled(false)),
        (json!(true), Cors::Enabled(true)),
        (json!(""), Cors::Origin("".into())),
        (json!("*"), Cors::Origin("*".into())),
        (json!([]), Cors::Origins(vec![])),
        (
            json!(["a", "b"]),
            Cors::Origins(vec!["a".into(), "b".into()]),
        ),
    ] {
        assert_eq!(
            serde_json::from_value::<Cors>(wire).map_err(Error::from_source)?,
            expected
        );
    }
    Ok(())
}

#[tokio::test]
async fn typed_config_rejects_unrepresentable_values_without_new_policy_checks() -> Result<()> {
    for api in [
        json!(false),
        json!(7),
        json!({"expires": "one day"}),
        json!({"scopes": []}),
        json!({"webhooks": {}}),
        json!({"endpoints": [{"method": "GET", "route": "/", "type": "unknown", "event": "e"}]}),
        json!({"endpoints": [{"method": "GET", "route": "/", "type": "public", "event": "e", "priority": 1.5}]}),
        json!({"endpoints": [{"method": "GET", "route": "/", "type": "app", "event": "e", "scopes": "read"}]}),
    ] {
        let mut app = ServerBuilder::new();
        plugin::register(&mut app)?;
        let mut layers = ConfigLayers::new();
        layers.value("invalid typed fixture", json!({"api": api}))?;
        app.config_layers(layers)?;
        assert!(
            app.build().await.is_err(),
            "typed fixture unexpectedly decoded: {api}"
        );
    }
    Ok(())
}
