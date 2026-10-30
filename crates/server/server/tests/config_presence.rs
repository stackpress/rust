//! Source key presence stays independent of value decoding and layer replacement.

use serde_json::{Value, json};
use stackpress_server::{ConfigLayers, ConfigPresence, ServerBuilder};

/// Null/false/objects remain present; nested and later layers never delete a section key.
#[tokio::test]
async fn explicit_presence_survives_value_replacement() {
    for (layers, present, expected) in [
        (vec![json!({})], false, json!({"default":true})),
        (vec![json!({"view":null})], true, Value::Null),
        (vec![json!({"view":false})], true, json!(false)),
        (vec![json!({"view":{}})], true, json!({"default":true})),
        (
            vec![
                json!({"view":{"engine":{"a":1}}}),
                json!({"view":{"engine":{"b":2}}}),
            ],
            true,
            json!({"default":true,"engine":{"a":1,"b":2}}),
        ),
        (
            vec![json!({"view":false}), json!({}), json!({"view":null})],
            true,
            Value::Null,
        ),
    ] {
        // The same typed value accepts every source value; defaults remain unchanged.
        let mut app = ServerBuilder::new();
        app.config_section("view", json!({"default":true}), |_| Ok(()))
            .unwrap();
        let mut input = ConfigLayers::new();
        for value in layers {
            input.value("explicit", value).unwrap();
        }
        app.config_layers(input).unwrap();
        let server = app.build().await.unwrap();
        let presence = server.host().config::<ConfigPresence>().unwrap();
        assert_eq!(presence.contains("view"), present);
        assert!(!presence.contains("view.engine"));
        assert!(!presence.contains("missing"));
        assert_eq!(server.host().config::<Value>().unwrap(), &expected);
    }
}
