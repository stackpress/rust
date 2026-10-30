//! Replay fixtures obtained from the original event, not a handwritten expected mapping.

use serde_json::{Value, json};
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, SendMailOptions, Service, Transport, TransportFactory,
};
use std::sync::{Arc, Mutex};

/// Collect the native factory/send boundary trace for comparison with the source callback.
struct RecordingFactory {
    calls: Arc<Mutex<Vec<Value>>>,
    result: Value,
}

/// An inert per-call transport; callback success preserves the provider result verbatim.
struct RecordingTransport {
    calls: Arc<Mutex<Vec<Value>>>,
    result: Value,
}

impl TransportFactory for RecordingFactory {
    /// Log the exact source configuration presented on this invocation.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        self.calls.lock().unwrap().push(json!(["create", config]));
        Ok(Box::new(RecordingTransport {
            calls: self.calls.clone(),
            result: self.result.clone(),
        }))
    }
}

impl Transport for RecordingTransport {
    /// Log all message fields and return the original fake provider info.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(json!(["send", options]));
            Ok(self.result.clone().into())
        })
    }
}

/// Falsy activation, object/string options, source lack of defaults and info survive the port.
#[tokio::test]
async fn source_event_option_and_default_vectors() {
    let fixtures: Value =
        serde_json::from_str(include_str!("../fixtures/source-options.json")).unwrap();
    for case in fixtures["cases"].as_array().unwrap() {
        let config = EmailConfig::from_section(Some(&case["config"])).unwrap();
        let calls = Arc::new(Mutex::new(vec![]));
        let mut result = stackpress_framework_email::SentMessageInfo::from(Value::Null);
        if let Some(config) = config {
            let factory = Arc::new(RecordingFactory {
                calls: calls.clone(),
                result: case["result"].clone(),
            });
            let service = Service::new(config, factory);
            result = service
                .send(case["options"].as_object().unwrap().clone())
                .await
                .unwrap();
        }
        assert_eq!(json!(*calls.lock().unwrap()), case["calls"]);
        assert_eq!(result, case["result"]);
    }
}
