//! Generated UI against real API SQL, original CSRF and source admin action URLs.
use serde_json::{Value as Json, json};
use stackpress_framework_admin::pages::Action;
use stackpress_framework_api::{
    models::{ApplicationActions, SessionActions},
    views::{
        admin,
        management::{Management, import_csv},
        roles::Relations,
    },
};
use stackpress_lib::{Body, Flow, Nest, Request, Response};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Db {
    path: std::path::PathBuf,
    engine: Arc<Engine>,
    server: Server,
}
impl Db {
    async fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "slice10-m5-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let engine = Arc::new(Engine::new(
            stackpress_sql_sqlite::SqliteDatabase::open(
                stackpress_sql_sqlite::SqliteOptions::file(&path).create_if_missing(true),
            )
            .await
            .unwrap(),
        ));
        let profile =
            stackpress_framework_session::storage::ProfileActions::new(engine.clone(), "");
        profile.install().await.unwrap();
        ApplicationActions::new(engine.clone(), "")
            .install()
            .await
            .unwrap();
        SessionActions::new(engine.clone(), "")
            .install()
            .await
            .unwrap();
        profile
            .create(
                &serde_json::from_value(json!({"id":"p","name":"Ada Lovelace","roles":["USER"]}))
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut builder = ServerBuilder::new();
        let pool = engine.clone();
        builder
            .on_config(stackpress_lib::queue::task(
                move |start: &mut StartupContext| {
                    let pool = pool.clone();
                    Box::pin(async move {
                        start.register(pool)?;
                        Ok(Flow::Continue)
                    })
                },
            ))
            .unwrap();
        stackpress_framework_api::sql::register(&mut builder).unwrap();
        admin::register(
            &mut builder,
            super::shared::management(),
            Default::default(),
        )
        .unwrap();
        let server = builder.build().await.unwrap();
        Self {
            path,
            engine,
            server,
        }
    }
    async fn request(&self, method: &str, path: &str, data: Json) -> Response {
        let mut req = Request::builder()
            .method(method)
            .url(path)
            .data(Nest::from_object(data).unwrap())
            .build()
            .unwrap();
        req.session.set("csrf", "incoming-token", None).unwrap();
        let mut res = Response::new();
        self.server.handle_route(&mut req, &mut res).await.unwrap();
        res
    }
    async fn close(self) {
        drop(self.server);
        self.engine.close().await.unwrap();
        std::fs::remove_file(self.path).unwrap();
    }
}
fn payload() -> Json {
    json!({"csrf":"incoming-token","id":"a","profileId":"p","name":"Application <title>","secret":"visible-secret","expires":"2100-01-01T00:00:00.000Z","scopes":["read","write"]})
}
fn result(response: &Response) -> Json {
    match response.body() {
        Body::Text(text) => serde_json::from_str(text).unwrap(),
        _ => response.result().unwrap().to_json().unwrap(),
    }
}
fn html(response: &Response) -> &str {
    match response.body() {
        Body::Text(html) => html,
        _ => panic!("HTML expected"),
    }
}
#[tokio::test]
async fn source_admin_crud_plaintext_keys_joins_copy_remove_restore_and_population() {
    let db = Db::new().await;
    let created = db
        .request("POST", "/admin/application/create", payload())
        .await;
    assert_eq!(
        created.headers().get("location"),
        Some("/admin/application/detail/a")
    );
    let detail = db
        .request("GET", "/admin/application/detail/a", json!({}))
        .await;
    let page = html(&detail);
    assert!(page.contains("visible-secret"));
    assert!(page.contains("Ada Lovelace"));
    assert!(page.contains("Application &lt;title&gt;"));
    assert!(page.contains(", 2100 ") && page.contains(":iAM"));
    assert!(page.contains("Related sessions"));
    assert!(page.contains("api-tag-warning api-tag-pill"));
    assert!(!page.contains("src=\"—\""));
    assert!(
        !page.contains("alt=\"Image\""),
        "empty optional image has no reserved image box"
    );
    assert!(
        !page.contains("href=\"—\""),
        "empty optional website has no placeholder link"
    );
    assert!(page.contains("api-admin-shell"));
    let search = db
        .request("GET", "/admin/application/search", json!({}))
        .await;
    assert!(html(&search).contains("visible-secret"));
    assert!(!html(&search).contains("Active action"));
    assert!(html(&search).contains("api-filter-panel"));
    let form = db
        .request("GET", "/admin/session/create", json!({"applicationId":"a"}))
        .await;
    assert!(html(&form).contains("Application &lt;title&gt;"));
    assert!(html(&form).contains("Ada Lovelace"));
    assert!(html(&form).contains("combobox"));
    let session=db.request("POST","/admin/session/create",json!({"csrf":"incoming-token","id":"s","profileId":"p","applicationId":"a","secret":"session-key","scopes":["read"],"expires":"2100-01-01T00:00:00.000Z"})).await;
    assert_eq!(
        session.headers().get("location"),
        Some("/admin/session/detail/s")
    );
    let session = db
        .request("GET", "/admin/session/detail/s", json!({}))
        .await;
    assert!(html(&session).contains("session-key"));
    assert!(html(&session).contains("Ada Lovelace"));
    assert!(html(&session).contains("Application &lt;title&gt;"));
    let updated = db
        .request(
            "PUT",
            "/admin/application/update/a",
            json!({"csrf":"incoming-token","name":"Updated"}),
        )
        .await;
    assert_eq!(updated.status().unwrap().code, 302);
    let mut copy = payload();
    copy["name"] = json!("Copy");
    copy["json"] = json!(true);
    let copied = db.request("POST", "/admin/application/copy/a", copy).await;
    let copied = result(&copied);
    let row = &copied["results"];
    assert_ne!(row["id"], "a");
    assert_ne!(row["secret"], "visible-secret");
    assert_eq!(row["name"], "Copy");
    let unconfirmed = db
        .request("POST", "/admin/application/remove/a", json!({}))
        .await;
    assert!(html(&unconfirmed).contains("Confirm"));
    let removed = db
        .request(
            "GET",
            "/admin/application/remove/a",
            json!({"confirmed":true,"csrf":"incoming-token"}),
        )
        .await;
    assert_eq!(
        removed.headers().get("location"),
        Some("/admin/application/search")
    );
    let detail = db
        .request("GET", "/admin/application/detail/a", json!({}))
        .await;
    assert!(
        html(&detail).contains("visible-secret"),
        "{}",
        html(&detail)
    );
    let restored = db
        .request(
            "POST",
            "/admin/application/restore/a",
            json!({"confirmed":true,"csrf":"incoming-token"}),
        )
        .await;
    assert_eq!(
        restored.headers().get("location"),
        Some("/admin/application/detail/a")
    );
    db.close().await;
}
#[tokio::test]
async fn source_csrf_methods_invalid_draft_json_mode_import_export_and_event_guards() {
    let db = Db::new().await;
    let missing = db
        .request(
            "POST",
            "/admin/application/create",
            json!({"name":"missing csrf"}),
        )
        .await;
    assert_eq!(missing.status().unwrap().code, 419);
    let invalid = db
        .request(
            "POST",
            "/admin/application/remove/a",
            json!({"confirmed":true}),
        )
        .await;
    assert_eq!(invalid.status().unwrap().code, 302);
    assert_eq!(
        invalid.headers().get("location"),
        Some("/admin/application/remove/a")
    );
    assert!(invalid.session().get("flash").is_some());
    let form = db
        .request("GET", "/admin/application/create", json!({}))
        .await;
    assert_eq!(
        form.session().get("csrf").unwrap().as_str().unwrap().len(),
        64
    );
    assert!(html(&form).contains("name=\"csrf\""));
    let mut input = payload();
    input["json"] = json!(true);
    let saved = db.request("POST", "/admin/application/create", input).await;
    assert_eq!(saved.status().unwrap().code, 200);
    assert_eq!(result(&saved)["results"]["secret"], "visible-secret");
    let invalid=db.request("POST","/admin/application/create",json!({"csrf":"incoming-token","id":"invalid","profileId":"p","name":"","expires":"2100-01-01T00:00:00.000Z"})).await;
    assert_eq!(invalid.status().unwrap().code, 400);
    assert!(html(&invalid).contains("Invalid parameters"));
    let imported=db.request("POST","/admin/application/import?json",json!({"mode":"create","rows":[{"id":"i","profileId":"p","name":"Imported","secret":"import-key","expires":"2100-01-01T00:00:00.000Z"}]})).await;
    assert_eq!(imported.status().unwrap().code, 200);
    assert!(result(&imported).to_string().contains("Imported"));
    let exported = db
        .request("GET", "/admin/application/export", json!({"take":1}))
        .await;
    assert!(
        exported
            .headers()
            .get("content-disposition")
            .unwrap()
            .contains("application-")
    );
    assert!(html(&exported).contains("visible-secret"));
    assert!(html(&exported).contains("import-key"));
    assert!(html(&exported).contains("Ada Lovelace"));
    db.close().await;
}
#[test]
fn generated_forms_roles_data_and_csv_representation_limits() {
    let ui = super::shared::management();
    let spec = (ui.spec)("application").unwrap();
    assert_eq!(spec.model.label, "App");
    let page = Management {
        module: "application".into(),
        action: Action::Create,
        data: json!({}),
        query: json!({}),
        relations: Relations::default(),
        csrf: Some(("csrf".into(), "token".into())),
        config: json!({"admin":{"name":"Manage","base":"/admin","menu":[{"label":"Home","url":"/home"}]}}),
        theme: None,
        error: None,
    };
    let form = admin::document(page, ui).unwrap();
    assert!(form.contains("type=\"url\""));
    assert!(form.contains("href=\"/home\""));
    assert!(form.contains("Expires"));
    assert!(!form.contains("name=\"secret\""));
    assert!(!form.contains("name=\"id\""));
    let rows = import_csv("name,profileId\nExample,p\n\"Two, three\",q\n").unwrap();
    assert_eq!(rows[1]["name"], "Two, three");
    assert_eq!(rows[0]["profileId"], "p");
    assert!(import_csv("\"unclosed").is_err());
    let csv =
        admin::export(&json!([{"name":"=1+1","secret":"plain","profile":{"name":"Ada"}}])).unwrap();
    assert!(csv.contains("\"=1+1\""));
    assert!(!csv.contains("'=1+1"));
}

#[test]
#[cfg(unix)]
fn concurrent_sql_and_ssr_have_process_deadline() {
    assert!(super::proof::watchdog(
        false,
        "management::concurrent_child"
    ));
}
#[test]
#[ignore = "executed by the process watchdog parent"]
fn concurrent_child() {
    std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready").unwrap();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let db = Db::new().await;
            db.request("POST", "/admin/application/create", payload())
                .await;
            let (first, second) = tokio::join!(
                db.request("GET", "/admin/application/detail/a", json!({})),
                db.request("GET", "/admin/session/create", json!({"applicationId":"a"}))
            );
            assert!(html(&first).contains("visible-secret"));
            assert!(html(&second).contains("Ada Lovelace"));
            db.close().await;
        });
}
