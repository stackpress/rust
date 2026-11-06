//! G57: source auth processes through actual Hyper HTTP/1 IO, without a listener.

use serde_json::{Value as Json, json};
use stackpress_framework_session::{
    Service,
    auth::{self, Hooks},
    storage::{AuthActions, AuthStore, ProfileActions, ProfileStore, StoreQuery},
};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, Value};
use stackpress_server::{ConfigLayers, ServerBuilder, StartupContext, http::HttpOptions};
use stackpress_server_http::{HttpDelivery, HttpServer};
use stackpress_sql::Engine;
use std::{future::pending, sync::Arc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Observe the real lifecycle before HTTP converts a separate envelope copy.
async fn retained_dates(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.url.path() == "/auth/signin/username" && res.status().is_some_and(|s| s.code == 302) {
        let result = res.result().unwrap().as_object().unwrap();
        assert!(matches!(result.get("consumed"), Some(Value::DateTime(_))));
        assert!(result.get("secret").is_none());
        assert!(
            res.try_to_json().is_err(),
            "core serialization remains strict"
        );
    }
    Ok(Flow::Continue)
}

/// Drive the same native HTTP/1 path as a socket with bounded, owned in-memory IO.
async fn post(host: &HttpServer, path: &str, form: &str) -> (String, Json) {
    let (server, mut client) = tokio::io::duplex(4096);
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nCookie: csrf=proof\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{form}",
        form.len()
    );
    let exchange = async {
        client.write_all(request.as_bytes()).await.unwrap();
        let mut bytes = Vec::new();
        client.read_to_end(&mut bytes).await.unwrap();
        bytes
    };
    // Both futures are owned by this timeout: cancellation drops IO and the active
    // connection together, with no detached server task left behind on failure.
    let (report, bytes) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(
            host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), pending()),
            exchange
        )
    })
    .await
    .expect("native auth HTTP exchange exceeded its finite deadline");
    assert!(report.failure.is_none());
    assert_eq!(report.exchanges.len(), 1);
    let exchange = &report.exchanges[0];
    assert_eq!(exchange.delivery, HttpDelivery::Completed);
    assert!(exchange.failure.is_none());
    assert!(exchange.host.as_ref().unwrap().failures.is_empty());
    let split = bytes.windows(4).position(|v| v == b"\r\n\r\n").unwrap();
    let headers = String::from_utf8(bytes[..split].to_vec()).unwrap();
    let body = &bytes[split + 4..];
    assert!(headers.starts_with("HTTP/1.1 302"), "{headers}");
    assert!(headers.contains("content-type: application/json"));
    assert!(headers.contains(&format!("content-length: {}", body.len())));
    (headers, serde_json::from_slice(body).unwrap())
}

/// The reported username-signin failure and signup both retain their original redirect bodies.
#[tokio::test]
async fn signup_and_username_signin_keep_dates_and_redirect_bodies_on_native_http() {
    let dir = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("http.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(database));
    ProfileActions::new(engine.clone(), ProfileStore::new("seed".into(), 0))
        .install()
        .await
        .unwrap();
    let auth = AuthActions::new(engine.clone(), AuthStore::new("seed".into(), 0));
    auth.install().await.unwrap();
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value(
            "application",
            json!({"auth":{},"session":{"key":"identity"}}),
        )
        .unwrap();
    app.config_layers(layers).unwrap();
    let owned = engine.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = owned.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(auth::Environment {
                    database_seed: Some("seed".into()),
                    ..Default::default()
                }))?;
                start.register(Arc::new(stackpress_framework_csrf::Service::default()))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    auth::register(&mut app).unwrap();
    stackpress_framework_session::account::events::register(&mut app).unwrap();
    stackpress_framework_session::plugin::register(&mut app).unwrap();
    app.on("response", retained_dates).unwrap();
    let app = Arc::new(app.build().await.unwrap());
    let host = HttpServer::new(
        app.clone(),
        HttpOptions {
            base_url: "http://localhost/".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let (headers, signup) = post(
        &host,
        "/auth/signup",
        "csrf=proof&name=Ada&username=ada-user&secret=correct",
    )
    .await;
    assert!(headers.contains("location: /auth/signin"));
    let profile = &signup["results"];
    assert!(profile["created"].as_str().unwrap().ends_with(".000Z"));
    assert!(profile["auth"]["username"].get("secret").is_none());
    let (headers, signin) = post(
        &host,
        "/auth/signin/username",
        "csrf=proof&username=ada-user&secret=correct&redirect_uri=%2Faccount",
    )
    .await;
    assert!(headers.contains("location: /account"));
    assert!(headers.contains("set-cookie: identity="));
    assert_eq!(signin["code"], 302);
    assert_eq!(signin["results"]["profile"]["id"], profile["id"]);
    let consumed = signin["results"]["consumed"].as_str().unwrap();
    assert_eq!(consumed.len(), 24);
    assert!(consumed.ends_with('Z'));
    // Compare the actual persisted/reloaded source value, independently of HTTP conversion.
    let query: StoreQuery =
        serde_json::from_value(json!({"eq":{"profileId":profile["id"],"type":"username"}}))
            .unwrap();
    let stored = auth.find(&query).await.unwrap().unwrap();
    assert_eq!(serde_json::to_value(&stored["consumed"]).unwrap(), consumed);
    // Signing/cookie behavior is unchanged; JSON conversion never consumes the Session value.
    let token = headers
        .lines()
        .find_map(|line| line.strip_prefix("set-cookie: identity="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let session = app.host().service::<Service>().unwrap().load(token);
    assert_eq!(session.data().await.unwrap()["id"], profile["id"]);
    app.host().service::<Hooks>().unwrap().shutdown().await;
    engine.close().await.unwrap();
}
