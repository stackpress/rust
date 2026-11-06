//! Real SQLite proves account writes, source unchecked branches and insertion-ordered exports.
use super::*;
use stackpress_framework_session::{
    auth,
    columns::{Record, Value},
    storage::{AuthActions, StoreQuery},
};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

/// One finite file-backed database and source table installation per test.
async fn database() -> (tempfile::TempDir, Arc<Engine>, AuthActions) {
    let dir = tempfile::tempdir().unwrap();
    let db = SqliteDatabase::open(
        SqliteOptions::file(dir.path().join("account.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(db));
    let actions = AuthActions::new(
        engine.clone(),
        stackpress_framework_session::storage::AuthStore::new("database-seed".into(), 0),
    );
    account(&actions).profile().install().await.unwrap();
    actions.install().await.unwrap();
    (dir, engine, actions)
}
/// Register native dependencies and direct page/events without automatically rendering a route.
async fn host(engine: Option<Arc<Engine>>) -> Server {
    let mut app = ServerBuilder::new();
    app.config_section(
        "auth",
        Some(AuthConfig {
            base: Some("/auth".into()),
            ..Default::default()
        }),
        |_| Ok(()),
    )
    .unwrap();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                if let Some(engine) = engine {
                    start.register(engine)?;
                }
                start.register(Arc::new(Service::default()))?;
                start.register(Arc::new(Csrf::default()))?;
                start.register(Arc::new(Environment {
                    database_seed: Some("database-seed".into()),
                    clock: Arc::new(FixedClock),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    account::events::register(&mut app).unwrap();
    app.on("page-account", account::account).unwrap();
    app.on("page-update", account::update).unwrap();
    app.on("page-password", account::password).unwrap();
    app.on("page-remove", account::remove).unwrap();
    app.on("page-export", account::export).unwrap();
    app.on("page-enroll", account::enroll).unwrap();
    app.on("page-remove-factor", account::remove_factor)
        .unwrap();
    app.build().await.unwrap()
}
/// Native values retain nested insertion order when decoded, stored, read and exported.
fn record(value: Json) -> Record {
    serde_json::from_value(value).unwrap()
}
/// Caller-owned cookie and optional CSRF pair; no global request gate is installed.
async fn invoke(
    host: &Server,
    event: &str,
    method: &str,
    input: Json,
    token: Option<&str>,
    csrf: bool,
) -> Response {
    let mut req = Request::builder()
        .method(method)
        .data(input)
        .build()
        .unwrap();
    if let Some(token) = token {
        req.session.set("session", token, None).unwrap();
    }
    if csrf {
        req.session.set("csrf", "same", None).unwrap();
        req.data.set("csrf", "same").unwrap();
    }
    let mut res = Response::new();
    host.emit(event, &mut req, &mut res).await.unwrap();
    res
}
/// Issue the source claims without adding a persisted session record or live-account check.
async fn token(host: &Server, id: &Value) -> String {
    host.host()
        .service::<Service>()
        .unwrap()
        .create(json!({"id":id,"name":"Ada"}).as_object().unwrap().clone())
        .await
        .unwrap()
}

#[tokio::test]
async fn sql_account_update_password_and_ordered_nested_export() {
    let (_dir, engine, actions) = database().await;
    let host = host(Some(engine.clone())).await;
    let profile = account(&actions)
        .signup(&record(
            json!({"name":"Ada","email":"ada@example.test","secret":"correct"}),
        ))
        .await
        .unwrap();
    let id = &profile["id"];
    let token = token(&host, id).await;
    let response = invoke(&host, "page-account", "GET", json!({}), Some(&token), false).await;
    let projected = events::json_result(&response).unwrap();
    assert_eq!(projected["auth"]["email"]["token"], "ada@example.test");
    assert!(projected["auth"]["email"].get("secret").is_none());
    let missing = invoke(&host, "page-account", "GET", json!({}), None, false).await;
    assert_eq!(missing.status().unwrap().code, 401);
    assert_eq!(missing.error(), Some("Unauthorized"));
    let update = invoke(
        &host,
        "page-update",
        "POST",
        json!({"name":"Changed","image":"","email":"new@example.test","phone":""}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(update.headers().get("Location"), Some("/auth/account"));
    assert_eq!(
        actions
            .find(&StoreQuery::eq("token", "new@example.test"))
            .await
            .unwrap()
            .unwrap()["secret"],
        stackpress_framework_session::codecs::hash("correct").as_str()
    );
    let denied = invoke(
        &host,
        "page-update",
        "POST",
        json!({"name":"Blocked","username":"ada-user","phone":""}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(denied.status().unwrap().code, 400);
    let added = invoke(
        &host,
        "page-update",
        "POST",
        json!({"name":"Changed","username":"ada-user","phone":"","current":"correct"}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(added.status().unwrap().code, 302);
    let password = invoke(
        &host,
        "page-password",
        "POST",
        json!({"secret":"replacement","current":"correct"}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(password.status().unwrap().code, 302);
    for auth in actions
        .find_all(&StoreQuery::eq("profileId", id.clone()))
        .await
        .unwrap()
    {
        assert_eq!(
            auth["secret"],
            stackpress_framework_session::codecs::hash("replacement").as_str()
        );
    }
    assert_eq!(
        host.host()
            .service::<Service>()
            .unwrap()
            .load(&token)
            .data()
            .await
            .unwrap()["name"],
        "Ada"
    );
    // Deliberately reverse lexical order at two nested levels. The source preserves
    // this insertion order through JSON storage and the final flattened CSV cell.
    let patch:Record=serde_json::from_str(r#"{"references":{"z-last-alphabetically":{"b":"first","a":"second"},"a-first-alphabetically":"last"}}"#).unwrap();
    account(&actions)
        .profile()
        .update(&StoreQuery::eq("id", id.clone()), &mut patch.clone())
        .await
        .unwrap();
    let exported = invoke(
        &host,
        "page-export",
        "GET",
        json!({"download":true}),
        Some(&token),
        false,
    )
    .await;
    let Body::Text(csv) = exported.body() else {
        panic!("CSV text")
    };
    assert!(csv.contains("references,\"z-last-alphabetically: b: first, a: second, a-first-alphabetically: last\""),"{csv}");
    let columns = csv
        .lines()
        .map(|line| line.split(',').next().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        columns,
        vec![
            "id",
            "image",
            "name",
            "type",
            "roles",
            "tags",
            "references",
            "active",
            "created",
            "updated"
        ]
    );
    assert_eq!(exported.mimetype(), Some("text/csv"));
    engine.close().await.unwrap();
}

#[tokio::test]
async fn sql_totp_enroll_regenerate_remove_get_and_account_soft_removal() {
    let (_dir, engine, actions) = database().await;
    let host = host(Some(engine.clone())).await;
    let profile = account(&actions)
        .signup(&record(
            json!({"name":"Ada","username":"ada-user","secret":"correct"}),
        ))
        .await
        .unwrap();
    let id = &profile["id"];
    let token = token(&host, id).await;
    let secret = "JBSWY3DPEHPK3PXP";
    let enroll = invoke(
        &host,
        "page-enroll",
        "GET",
        json!({"secret":secret}),
        Some(&token),
        false,
    )
    .await;
    let data = events::json_result(&enroll).unwrap();
    assert_eq!(data["secret"], secret);
    assert!(data["profile"].is_object());
    assert!(
        data["url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    let code = auth::totp::generate_at(secret, 1_700_000_000, 0, 30).unwrap();
    let invalid = invoke(
        &host,
        "page-enroll",
        "POST",
        json!({"secret":secret,"code":code}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(invalid.status().unwrap().code, 419);
    let saved = invoke(
        &host,
        "page-enroll",
        "POST",
        json!({"secret":secret,"code":code}),
        Some(&token),
        true,
    )
    .await;
    assert_eq!(saved.status().unwrap().code, 302);
    let existing = invoke(&host, "page-enroll", "GET", json!({}), Some(&token), false).await;
    let existing = events::json_result(&existing).unwrap();
    assert_eq!(existing["secret"], secret);
    assert!(existing["authId"].is_string());
    let regenerated = invoke(
        &host,
        "page-enroll",
        "GET",
        json!({"regenerate":""}),
        Some(&token),
        false,
    )
    .await;
    let generated = events::json_result(&regenerated).unwrap();
    let generated = generated["secret"].as_str().unwrap();
    assert_eq!(generated.len(), 16);
    assert!(
        generated
            .bytes()
            .all(|b| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".contains(&b))
    );
    let removed = invoke(
        &host,
        "page-remove-factor",
        "GET",
        json!({"confirmed":"1","authId":existing["authId"]}),
        Some(&token),
        false,
    )
    .await;
    assert_eq!(removed.status().unwrap().code, 302);
    assert_eq!(
        actions.count(&StoreQuery::eq("type", "2fa")).await.unwrap(),
        0
    );
    let removed = invoke(
        &host,
        "page-remove",
        "POST",
        json!({"confirmed":true,"secret":"correct"}),
        Some(&token),
        false,
    )
    .await;
    let source: Json =
        serde_json::from_str(include_str!("../../account-removal-join.json")).unwrap();
    assert_eq!(
        removed.headers().get("Location"),
        source["location"].as_str()
    );
    assert_eq!(
        removed.status().unwrap().code,
        source["code"].as_u64().unwrap() as u16
    );
    assert_eq!(source["profileStillActive"], true);
    // The source generated profile-remove sees the parent's existing profile body
    // and passes through. Credential removals ran with fresh child responses.
    assert!(
        account(&actions)
            .profile()
            .find(&StoreQuery::eq("id", id.clone()))
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        actions
            .count(&StoreQuery::eq("profileId", id.clone()))
            .await
            .unwrap(),
        0
    );
    assert!(
        host.host()
            .service::<Service>()
            .unwrap()
            .load(&token)
            .data()
            .await
            .is_some()
    ); // Source JWT has no revocation.
    engine.close().await.unwrap();
}

#[tokio::test]
async fn generated_sql_missing_engine_and_id_modes() {
    let no_engine = host(None).await;
    for event in [
        "auth-search",
        "auth-update",
        "auth-create",
        "auth-remove",
        "profile-detail",
        "profile-update",
        "profile-remove",
    ] {
        let result = invoke(&no_engine, event, "GET", json!({}), None, false).await;
        assert!(result.status().is_none(), "{event}");
    }
    let (_dir, engine, _actions) = database().await;
    let host = host(Some(engine.clone())).await;
    for event in [
        "auth-update",
        "auth-remove",
        "profile-detail",
        "profile-update",
        "profile-remove",
    ] {
        for input in [json!({}), json!({"id":null}), json!({"id":""})] {
            let result = invoke(&host, event, "GET", input, None, false).await;
            assert_eq!(result.status().unwrap().code, 400, "{event}");
            assert_eq!(
                result.errors().to_json().unwrap(),
                json!({"id":"Missing or invalid value"})
            );
        }
    }
    let missing = invoke(
        &host,
        "profile-detail",
        "GET",
        json!({"id":"missing"}),
        None,
        false,
    )
    .await;
    assert_eq!(missing.status().unwrap().code, 404);
    assert_eq!(missing.error(), Some("Not Found"));
    assert_eq!(auth::totp::generate_secret(0).unwrap(), "");
    engine.close().await.unwrap();
}

use stackpress_framework_schema::source::SourceSchema;

/// Bind account orchestration to the same generated store environment.
fn account(
    actions: &AuthActions,
) -> stackpress_framework_session::storage::AccountActions<stackpress_framework_models::Models> {
    stackpress_framework_session::storage::AccountActions::new(
        actions.engine.clone(),
        actions.store.schema.seed().into(),
        actions.store.schema.offset_minutes(),
        Default::default(),
    )
}
