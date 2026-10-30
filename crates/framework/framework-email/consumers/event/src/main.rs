//! Public registrars and frozen event names compose without a session crate or global guard.
use serde_json::json;
use stackpress_framework_csrf::{Service as Csrf, plugin as csrf};
use stackpress_framework_email::{events::EMAIL_SEND, plugin as email, tasks::TaskSupervisor};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};
use std::sync::Arc;

/// An application explicitly connects the distinct signup hook to the reusable send event.
async fn delegate(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.emit(EMAIL_SEND, req, res).await?;
    Ok(Flow::Continue)
}
/// Missing config disables only email integration; selected CSRF remains a default service.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for section in [
        None,
        Some(json!(null)),
        Some(json!(false)),
        Some(json!(0)),
        Some(json!("")),
        Some(json!({"jsonTransport":true})),
    ] {
        let mut app = ServerBuilder::new();
        if let Some(section) = section.clone() {
            let mut layers = ConfigLayers::new();
            layers.value("consumer", json!({"email":section}))?;
            app.config_layers(layers)?;
        }
        app.plugin("csrf", csrf::register)?;
        app.plugin("email", email::register)?;
        app.on("email-email-send", delegate)?;
        let host = Arc::new(app.build().await.map_err(|failure| failure.error)?);
        assert_eq!(host.host().service::<Csrf>()?.name(), "csrf");
        assert!(
            !host
                .listeners()
                .any(|listener| listener.pattern == "request")
        );
        let active = section.is_some_and(|section| section.is_object());
        assert_eq!(
            host.listeners()
                .any(|listener| listener.pattern == EMAIL_SEND),
            active
        );
        let sent = host
            .resolve(EMAIL_SEND, json!({"text":"fresh direct event"}))
            .await?;
        assert_eq!(sent.response.code, active.then_some(200));
        let mut jobs = TaskSupervisor::new();
        jobs.resolve_on(
            &tokio::runtime::Handle::current(),
            host,
            "email-email-send",
            json!({"text":"owned nonawaited child"}),
        );
        jobs.join_next().await.unwrap()??;
        jobs.shutdown().await;
    }
    println!("slice9b event consumer passed");
    Ok(())
}
