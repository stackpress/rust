//! The source aggregate preserves constituent activation, ordering and cooperative dispatch.
use serde_json::{Value as Json, json};
use stackpress_framework_session::{Service, auth, composite, plugin};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, Server, ServerBuilder};

/// Select either grouping without changing the constituent registration order.
async fn host(config: Json, grouped: bool) -> Result<Server> {
    let mut layers = ConfigLayers::new();
    layers.value("source composite", config)?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    if grouped {
        app.plugin("session", composite::register)?;
    } else {
        auth::register(&mut app)?;
        stackpress_framework_email::plugin::register(&mut app)?;
        plugin::register(&mut app)?;
    }
    app.build().await.map_err(|failure| failure.error)
}
/// Filter only the original composite Listen inventory, keeping actual registration order.
fn events(server: &Server) -> Vec<&str> {
    server
        .listeners()
        .filter_map(|listener| {
            matches!(
                listener.pattern.as_str(),
                "auth-signup"
                    | "auth-signin"
                    | "auth-signout"
                    | "email-send"
                    | "me"
                    | "authorize"
                    | "request"
            )
            .then_some(listener.pattern.as_str())
        })
        .collect()
}
/// Config is local to each constituent; selecting the group adds no capability prerequisites.
#[tokio::test]
async fn grouped_and_standalone_keep_source_missing_config_modes() -> Result<()> {
    for grouped in [false, true] {
        for (config, identity, guard, auth_routes, email) in [
            (json!({}), false, false, false, false),
            (json!({"session":{}}), true, false, false, false),
            (json!({"session":{"access":{}}}), true, true, false, false),
            (json!({"auth":{}}), false, false, true, false),
            (
                json!({"email":{"jsonTransport":true}}),
                false,
                false,
                false,
                true,
            ),
        ] {
            let server = host(config, grouped).await?;
            let inventory = events(&server);
            assert!(inventory.starts_with(&["auth-signup", "auth-signin", "auth-signout"]));
            assert_eq!(server.host().service::<Service>().is_ok(), identity);
            assert_eq!(inventory.contains(&"me"), identity);
            assert_eq!(inventory.contains(&"authorize"), identity);
            assert_eq!(inventory.contains(&"request"), guard);
            assert_eq!(inventory.contains(&"email-send"), email);
            assert_eq!(
                server
                    .listeners()
                    .any(|listener| listener.pattern == "GET /auth/signup"),
                auth_routes
            );
            // Main selects CSRF earlier; the session source aggregate never selects it itself.
            assert!(
                server
                    .host()
                    .service::<stackpress_framework_csrf::Service>()
                    .is_err()
            );
            assert!(server.host().service::<stackpress_sql::Engine>().is_err());
        }
    }
    Ok(())
}
/// Original auth → email → identity order is the same for direct functions and one plugin group.
#[tokio::test]
async fn source_composite_listen_order_and_real_json_email() -> Result<()> {
    let config = json!({"session":{},"auth":{},"email":{"jsonTransport":true}});
    for grouped in [false, true] {
        let server = host(config.clone(), grouped).await?;
        assert_eq!(
            events(&server),
            [
                "auth-signup",
                "auth-signin",
                "auth-signout",
                "email-send",
                "me",
                "authorize"
            ]
        );
        let sent = server
            .resolve("email-send", json!({"text":"finite composite"}))
            .await?;
        assert_eq!(sent.response.code, Some(200));
    }
    Ok(())
}
/// Higher priority sees no identity result; lower priority sees the original me response.
async fn before(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    assert!(res.result().is_none());
    res.data_mut()?.set("before", true)?;
    Ok(Flow::Continue)
}
/// Additional listeners run after me without an ownership or uniqueness rule.
async fn after(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    assert_eq!(
        res.data().get("before").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(res.result().unwrap().to_json()?["id"], 0);
    res.data_mut()?.set("after", true)?;
    Ok(Flow::Continue)
}
/// An application may stop the same ordinary chain before identity contributes results.
async fn stop(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("stopped", true)?;
    Ok(Flow::Abort)
}
/// G42 remains priority then registration order with an ordinary cooperative Abort.
#[tokio::test]
async fn composite_events_still_allow_multiple_handlers_and_abort() -> Result<()> {
    for abort in [false, true] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers.value("identity", json!({"session":{}}))?;
        app.config_layers(layers)?;
        app.plugin("session", composite::register)?;
        if abort {
            app.on_priority("me", 10, stop)?;
        } else {
            app.on_priority("me", 10, before)?;
        }
        app.on_priority("me", -10, after)?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        let mut req = Request::new();
        let mut res = Response::new();
        server.handle("me", &mut req, &mut res).await?;
        assert_eq!(res.result().is_none(), abort);
        assert_eq!(res.data().has("after"), !abort);
        assert_eq!(res.data().has("stopped"), abort);
    }
    Ok(())
}
