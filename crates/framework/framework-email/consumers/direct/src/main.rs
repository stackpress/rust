//! Ordinary libraries require no server selection; only local JSON/stream adapters execute.
use serde_json::json;
use stackpress_framework_csrf::Service as Csrf;
use stackpress_framework_email::{EmailConfig, MessageOutput, NativeFactory, Service};
use stackpress_lib::{Request, Response};
use std::sync::Arc;

/// Exercise independent CSRF and per-call mail libraries from a standalone workspace.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let csrf = Csrf::default();
    let mut response = Response::new();
    let token = csrf.generate(&mut response)?;
    let mut request = Request::builder().data(json!({"csrf":token})).build()?;
    request.session.set("csrf", token, None)?;
    assert!(csrf.valid(&request, &mut response)?);
    csrf.clear(&mut request, &mut response)?;
    assert!(!response.data().has("csrf"));
    for config in [
        json!({"jsonTransport":true,"skipEncoding":true}),
        json!({"streamTransport":true,"buffer":true}),
    ] {
        let config: EmailConfig = serde_json::from_value(config)?;
        let mail = Service::new(config, Arc::new(NativeFactory::default()));
        for _ in 0..2 {
            let result=mail.send(serde_json::from_value(json!({"from":"from@example.test","to":"to@example.test","text":"consumer","messageId":"fixed@example.test"}))?).await?;
            assert!(result.fields.get("messageId").is_some());
            if let Some(MessageOutput::Buffer(bytes)) = result.message {
                assert!(String::from_utf8(bytes)?.contains("consumer"));
            }
        }
    }
    println!("slice9b direct consumer passed");
    Ok(())
}
