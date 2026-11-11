//! Direct-call proofs use fake transports and in-memory SMTP; no real delivery.

#[path = "cases/factory.rs"]
mod factory;
#[path = "cases/native.rs"]
mod native;
#[path = "cases/source_options.rs"]
mod source_options;
#[path = "cases/supervision.rs"]
mod supervision;

#[cfg(feature = "server")]
#[path = "cases/event_native.rs"]
mod event_native;
#[cfg(feature = "server")]
#[path = "cases/event_supervision.rs"]
mod event_supervision;
#[cfg(feature = "server")]
#[path = "cases/events.rs"]
mod events;

use serde_json::{Value, json};
use stackpress_framework_email::EmailConfig;

/// Keep source activation defaults distinct from provider-specific defaults.
#[test]
fn source_config_absence_and_truthiness() {
    assert_eq!(EmailConfig::from_section(None).unwrap(), None);
    for value in [Value::Null, json!(false), json!(0), json!(-0.0), json!("")] {
        assert_eq!(EmailConfig::from_section(Some(&value)).unwrap(), None);
    }
    assert_eq!(
        EmailConfig::from_section(Some(&json!({}))).unwrap(),
        Some(EmailConfig::Options(Default::default()))
    );
    assert!(EmailConfig::from_section(Some(&json!(true))).is_err());
    assert!(EmailConfig::from_section(Some(&json!([]))).is_err());
}

/// The carrier changes neither defaults nor extension options for any source variant.
#[test]
fn every_source_option_variant_round_trips_without_new_defaults() {
    for value in [
        json!({}),
        json!({"host":"localhost", "secure":false, "port":25, "tls":{"rejectUnauthorized":false}}),
        json!({"pool":true, "maxConnections":2, "maxMessages":4}),
        json!({"jsonTransport":true, "skipEncoding":true}),
        json!({"streamTransport":true, "buffer":false, "newline":"windows"}),
        json!({"sendmail":true, "path":"/custom/sendmail", "args":["-i"]}),
        json!({"SES":{"region":"fixture-region", "extension":{}}}),
        json!({"customProvider":{"option":false}, "auth":{"type":"custom"}}),
        json!("smtp://user:pass@localhost:25/?pool=true"),
    ] {
        let config = EmailConfig::from_section(Some(&value)).unwrap().unwrap();
        assert_eq!(serde_json::to_value(config).unwrap(), value);
    }
}
