//! G57: source auth processes through actual Hyper HTTP/1 IO, without a listener.

use serde_json::{Value as Json, json};
use stackpress_framework_session::{
    Service,
    auth::{self, Hooks},
    storage::{AuthActions, AuthStore, ProfileActions, ProfileStore, StoreQuery},
};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, Value};
use stackpress_server::{
    ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext, http::HttpOptions,
};
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
async fn exchange(
    host: &HttpServer,
    method: &str,
    path: &str,
    form: &str,
    cookie: &str,
) -> (String, Json) {
    let (server, mut client) = tokio::io::duplex(4096);
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nCookie: {cookie}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{form}",
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
    assert!(
        exchange.host.as_ref().unwrap().failures.is_empty(),
        "{:?}",
        exchange.host
    );
    let split = bytes.windows(4).position(|v| v == b"\r\n\r\n").unwrap();
    let headers = String::from_utf8(bytes[..split].to_vec()).unwrap();
    let body = &bytes[split + 4..];
    assert!(headers.contains("content-type: application/json"));
    assert!(headers.contains(&format!("content-length: {}", body.len())));
    (headers, serde_json::from_slice(body).unwrap())
}

/// A redirect assertion shared by the existing native auth regression and cookie proof.
async fn post(host: &HttpServer, path: &str, form: &str) -> (String, Json) {
    let reply = exchange(host, "POST", path, form, "csrf=proof").await;
    assert!(reply.0.starts_with("HTTP/1.1 302"), "{}", reply.0);
    reply
}

/// This test-owned route checks real verified ADMIN claims; it adds no framework policy.
async fn admin_check(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let identity = inv.service::<Service>()?.load_request(req);
    let admin = identity
        .data()
        .await
        .is_some_and(|data| data["roles"] == json!(["ADMIN"]));
    res.results(&admin)?;
    res.status_code(if admin { 200 } else { 401 }, None)?;
    Ok(Flow::Continue)
}

/// Own the SQL file, registered application and native adapter for each independent test.
struct Fixture {
    _dir: tempfile::TempDir,
    engine: Arc<Engine>,
    auth: AuthActions,
    app: Arc<Server>,
    host: HttpServer,
}

/// Install the real generated stores and source registrars; no listener is opened.
async fn fixture(key: &str, roles: &[&str]) -> Fixture {
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
            json!({"auth":{"roles":roles},"session":{"key":key}}),
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
    app.get("/admin/check", admin_check).unwrap();
    let app = Arc::new(app.build().await.unwrap());
    let host = HttpServer::new(
        app.clone(),
        HttpOptions {
            base_url: "http://localhost/".into(),
            ..Default::default()
        },
    )
    .unwrap();
    Fixture {
        _dir: dir,
        engine,
        auth,
        app,
        host,
    }
}

/// The reported username-signin failure and signup both retain their original redirect bodies.
#[tokio::test]
async fn signup_and_username_signin_keep_dates_and_redirect_bodies_on_native_http() {
    let Fixture {
        _dir,
        engine,
        auth,
        app,
        host,
    } = fixture("identity", &[]).await;
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

/// Replay actual Set-Cookie values through an independent cookie jar, without network IO.
fn jar(responses: Json, targets: &[&str]) -> Vec<String> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new("python3")
        .args(["-c", include_str!("cookie_jar.py")])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("installed Python cookie jar");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&json!({"responses": responses, "targets": targets})).unwrap(),
        )
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "cookie jar failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

/// Preserve separate native fields; a comma-joined Set-Cookie string is not a cookie jar input.
fn cookies(headers: &str) -> Vec<String> {
    headers
        .lines()
        .filter_map(|line| line.strip_prefix("set-cookie: ").map(str::to_owned))
        .collect()
}

/// Task 00043: a genuine ADMIN login reaches /admin, then source signout clears the same scope.
#[tokio::test]
async fn native_signin_cookie_jar_reaches_admin_and_signout_clears_root_scope() {
    let fixture = fixture("session", &["ADMIN"]).await;
    let host = &fixture.host;
    post(
        host,
        "/auth/signup",
        "csrf=proof&name=Ada&username=ada-user&secret=correct",
    )
    .await;
    let (signin, _) = post(
        host,
        "/auth/signin/username",
        "csrf=proof&username=ada-user&secret=correct",
    )
    .await;
    let issued = cookies(&signin);
    let token_cookie = issued
        .iter()
        .find(|cookie| cookie.starts_with("session="))
        .unwrap();
    assert!(token_cookie.ends_with("; Path=/"));
    // The source adds none of Secure/HttpOnly/SameSite/lifetime unless configured explicitly.
    assert_eq!(token_cookie.split(';').count(), 2);
    assert!(
        issued
            .iter()
            .any(|cookie| cookie.starts_with("csrf=;") && cookie.contains("; Path=/"))
    );
    let targets = [
        "http://localhost/admin/check",
        "http://localhost/auth/signout",
    ];
    let issued_response = json!({"url":"http://localhost/auth/signin/username", "headers":issued});
    let sent = jar(json!([issued_response]), &targets);
    assert!(sent.iter().all(|header| header.starts_with("session=")));
    let (admin, result) = exchange(host, "GET", "/admin/check", "", &sent[0]).await;
    assert!(admin.starts_with("HTTP/1.1 200"));
    assert_eq!(result["results"], true);
    // Negative control reproduces the old browser scope: /auth/signin cannot reach /admin.
    let unscoped = token_cookie.strip_suffix("; Path=/").unwrap();
    assert_eq!(
        jar(
            json!([{"url":"http://localhost/auth/signin/username", "headers":[unscoped]}]),
            &targets
        )[0],
        ""
    );
    let (signout, _) = exchange(host, "GET", "/auth/signout", "", &sent[1]).await;
    assert!(signout.starts_with("HTTP/1.1 302"));
    let cleared = cookies(&signout);
    assert!(cleared.iter().any(|cookie| cookie.starts_with("session=;")
        && cookie.contains("; Path=/")
        && cookie.contains("Max-Age=0")));
    let after = jar(
        json!([issued_response, {"url":targets[1], "headers":cleared}]),
        &targets,
    );
    assert!(after.iter().all(String::is_empty));
    let (admin, _) = exchange(host, "GET", "/admin/check", "", &after[0]).await;
    assert!(admin.starts_with("HTTP/1.1 401"));
    fixture
        .app
        .host()
        .service::<Hooks>()
        .unwrap()
        .shutdown()
        .await;
    fixture.engine.close().await.unwrap();
}

/// Explicit response options replace the entire host policy through actual native IO too.
#[tokio::test]
async fn native_cookie_options_override_the_host_without_merging() {
    use stackpress_lib::{CookieOptions, session::SameSite};
    async fn write(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        res.session_mut()?.set("host", "one", None)?;
        res.session_mut()?.set(
            "scoped",
            "two",
            Some(CookieOptions {
                path: Some("/own".into()),
                ..Default::default()
            }),
        )?;
        res.session_mut()?
            .set("empty", "three", Some(CookieOptions::default()))?;
        res.results(&true)?;
        Ok(Flow::Continue)
    }
    let mut builder = ServerBuilder::new();
    builder.get("/cookies", write).unwrap();
    let app = Arc::new(builder.build().await.unwrap());
    let host = HttpServer::new(
        app,
        HttpOptions {
            cookie: CookieOptions {
                path: Some("/host".into()),
                http_only: true,
                secure: true,
                same_site: Some(SameSite::Strict),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    let (headers, _) = exchange(&host, "GET", "/cookies", "", "").await;
    let values = cookies(&headers);
    assert!(values[0].contains("Path=/host"));
    assert!(values[0].contains("HttpOnly"));
    assert!(values[0].contains("Secure"));
    assert!(values[0].contains("SameSite=Strict"));
    assert_eq!(values[1], "scoped=two; Path=/own");
    assert_eq!(values[2], "empty=three");
}
