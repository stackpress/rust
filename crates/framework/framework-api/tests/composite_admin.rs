//! Real source-admin routes over generated compound-key SQLite stores.
#[allow(dead_code, unexpected_cfgs)]
#[path = "fixtures/admin-composite/models/mod.rs"]
mod models;
#[allow(dead_code)]
#[path = "fixtures/admin-composite/presentation/mod.rs"]
mod presentation;

use models::stores::{catalog::CatalogModel, slugged::SluggedModel};
use serde_json::{Value, json};
use stackpress_framework_api::views::admin;
use stackpress_framework_sql::source::SourceModel;
use stackpress_lib::{Body, Flow, Nest, Request, Response};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::{path::PathBuf, sync::Arc};

/// An exclusively created file belongs to this fixture, even when an assertion unwinds.
struct OwnedFile(PathBuf);
impl Drop for OwnedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
/// Cleanup is armed before any asynchronous setup can fail.
struct Db {
    _file: OwnedFile,
    engine: Arc<Engine>,
    server: Server,
}
impl Db {
    /// Install actual generated DDL; registration itself performs no setup or IO.
    async fn new(queued: bool, row_actions: bool) -> Self {
        let path = std::env::temp_dir().join(format!(
            "stackpress-admin-compound-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let file = OwnedFile(path);
        let engine = Arc::new(Engine::new(
            stackpress_sql_sqlite::SqliteDatabase::open(
                stackpress_sql_sqlite::SqliteOptions::file(&file.0),
            )
            .await
            .unwrap(),
        ));
        CatalogModel::actions(engine.clone(), String::new(), 0)
            .install()
            .await
            .unwrap();
        SluggedModel::actions(engine.clone(), String::new(), 0)
            .install()
            .await
            .unwrap();
        // The generic page collaborator obtains selectable Profile rows from its family.
        stackpress_framework_session::storage::ProfileModel::actions(
            engine.clone(),
            String::new(),
            0,
        )
        .install()
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
        stackpress_framework_sql::source::register::<CatalogModel>(&mut builder).unwrap();
        stackpress_framework_sql::source::register::<SluggedModel>(&mut builder).unwrap();
        let config = admin::Config {
            presentation: json!({"models":presentation::ui::host::presentation()}),
            ..Default::default()
        };
        // Context is selected by the actual page root, without changing the handler catalog.
        let ui = if row_actions {
            trailing_ui()
        } else {
            presentation::ui::pages::ui()
        };
        if queued {
            admin::register_queued_with::<stackpress_framework_session::storage::DefaultModels>(
                &mut builder,
                ui,
                config,
                &["catalog", "slugged"],
            )
            .unwrap();
        } else {
            admin::register_with::<stackpress_framework_session::storage::DefaultModels>(
                &mut builder,
                ui,
                config,
                &["catalog", "slugged"],
            )
            .unwrap();
        }
        Self {
            _file: file,
            engine,
            server: builder.build().await.unwrap(),
        }
    }

    /// Route params, CSRF and event dispatch use the same real request pipeline as an app.
    async fn request(&self, method: &str, path: &str, data: Value) -> Response {
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

    /// Read through the real registered detail route, including both key params.
    async fn detail(&self, path: &str) -> Value {
        let response = self.request("GET", path, json!({"json":true})).await;
        assert_eq!(
            response.status().unwrap().code,
            200,
            "{path}: {:?}",
            response.error()
        );
        match response.body() {
            Body::Text(text) => serde_json::from_str::<Value>(text).unwrap()["results"].clone(),
            _ => panic!("noview JSON expected"),
        }
    }
}

/// The optional generated trailing action uses the same declared identity and slash order.
fn trailing_table(
    module: &str,
    rows: &Value,
    instance: &str,
) -> stackpress_framework_schema::SchemaResult<leptos::prelude::AnyView> {
    use leptos::prelude::*;
    provide_context(
        stackpress_framework_admin::row_actions::RowActions::new(
            "/admin",
            Callback::new(|_: (String, String)| true),
        )
        .with_href(Callback::new(
            |row: stackpress_framework_admin::row_actions::RowDetail| {
                format!("/admin/{}/detail/{}", row.model, row.path)
            },
        )),
    );
    presentation::ui::pages::table(module, rows, instance)
}
/// Only the table callback changes; fields, detail and metadata remain officially generated.
fn trailing_ui() -> stackpress_framework_admin::source::pages::Ui {
    stackpress_framework_admin::source::pages::Ui {
        table: trailing_table,
        ..presentation::ui::pages::ui()
    }
}
/// Read the actual rendered response rather than reconstructing a management DTO.
fn html(response: &Response) -> &str {
    match response.body() {
        Body::Text(text) => text,
        _ => panic!("HTML expected"),
    }
}

/// A committed compound identity redirects to search, then all item paths consume both keys.
#[tokio::test]
async fn source_catalog_create_redirects_after_committing_its_compound_identity() {
    for queued in [false, true] {
        let db = Db::new(queued, false).await;
        for (category, article) in [
            ("category", "article"),
            ("category", "other"),
            ("other", "article"),
        ] {
            let response = db.request("POST", "/admin/catalog/create", json!({
                "csrf":"incoming-token", "categoryId":category, "articleId":article, "title":"Original"
            })).await;
            assert_eq!(
                response.headers().get("location"),
                Some("/admin/catalog/search")
            );
        }
        let row = db.detail("/admin/catalog/detail/category/article").await;
        assert_eq!(row["categoryId"], "category");
        assert_eq!(row["articleId"], "article");
        let search = db.request("GET", "/admin/catalog/search", json!({})).await;
        assert!(html(&search).contains("href=\"/admin/catalog/detail/category/article\""));
        let detail = db
            .request("GET", "/admin/catalog/detail/category/article", json!({}))
            .await;
        for action in ["update", "remove", "create"] {
            assert!(html(&detail).contains(&format!(
                "href=\"/admin/catalog/{action}/category/article\""
            )));
        }
        let updated = db
            .request(
                "PUT",
                "/admin/catalog/update/category/article",
                json!({
                    "csrf":"incoming-token", "title":"Updated"
                }),
            )
            .await;
        assert_eq!(
            updated.headers().get("location"),
            Some("/admin/catalog/detail/category/article")
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/article").await["title"],
            "Updated"
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/other").await["title"],
            "Original"
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/other/article").await["title"],
            "Original"
        );
        // The source copy route is create/<keys>; only @field members are forwarded.
        let copied = db
            .request(
                "POST",
                "/admin/catalog/create/category/article",
                json!({
                    "csrf":"incoming-token", "title":"Copied"
                }),
            )
            .await;
        assert_eq!(
            copied.headers().get("location"),
            Some("/admin/catalog/search")
        );
        let rows = CatalogModel::actions(db.engine.clone(), String::new(), 0)
            .find_all(&serde_json::from_value(json!({"eq":{"title":"Copied"}})).unwrap())
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_ne!(rows[0]["categoryId"].as_str(), Some("category"));
        assert_ne!(rows[0]["articleId"].as_str(), Some("article"));
        let removed = db
            .request(
                "POST",
                "/admin/catalog/remove/category/article",
                json!({
                    "csrf":"incoming-token", "confirmed":true
                }),
            )
            .await;
        assert_eq!(
            removed.headers().get("location"),
            Some("/admin/catalog/search")
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/article").await["active"],
            false
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/other").await["active"],
            true
        );
        let restored = db
            .request(
                "POST",
                "/admin/catalog/restore/category/article",
                json!({
                    "csrf":"incoming-token", "confirmed":true
                }),
            )
            .await;
        assert_eq!(
            restored.headers().get("location"),
            Some("/admin/catalog/detail/category/article")
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/article").await["active"],
            true
        );
        // Import forwards source batch mode. Default upsert updates the full key;
        // creating supplied, previously absent keys requires explicit create mode.
        let imported = db
            .request(
                "POST",
                "/admin/catalog/import",
                json!({
                    "csv":"categoryId,articleId,title\ncategory,article,Imported update\n"
                }),
            )
            .await;
        assert_eq!(imported.status().unwrap().code, 200);
        assert_eq!(
            db.detail("/admin/catalog/detail/category/article").await["title"],
            "Imported update"
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/category/other").await["title"],
            "Original"
        );
        let imported = db
            .request(
                "POST",
                "/admin/catalog/import",
                json!({
                    "mode":"create",
                    "csv":"categoryId,articleId,title\nimported,one,First\nimported,two,Second\n"
                }),
            )
            .await;
        assert_eq!(
            imported.status().unwrap().code,
            200,
            "{:?}",
            imported.error()
        );
        assert!(imported.headers().get("location").is_none());
        assert_eq!(
            db.detail("/admin/catalog/detail/imported/one").await["title"],
            "First"
        );
        assert_eq!(
            db.detail("/admin/catalog/detail/imported/two").await["title"],
            "Second"
        );
        db.engine.close().await.unwrap();
    }
}

/// A one-member key with a different name still redirects to its source detail URL.
#[tokio::test]
async fn source_admin_uses_non_id_single_key_and_generated_search_links() {
    let db = Db::new(false, true).await;
    let compound = db.request("POST", "/admin/catalog/create", json!({
        "csrf":"incoming-token", "categoryId":"left", "articleId":"right", "title":"Compound"
    })).await;
    assert_eq!(
        compound.headers().get("location"),
        Some("/admin/catalog/search")
    );
    let search = db.request("GET", "/admin/catalog/search", json!({})).await;
    assert!(html(&search).contains("href=\"/admin/catalog/detail/left/right\""));
    let created = db
        .request(
            "POST",
            "/admin/slugged/create",
            json!({
                "csrf":"incoming-token", "slug":"named-key", "title":"Single"
            }),
        )
        .await;
    assert_eq!(
        created.headers().get("location"),
        Some("/admin/slugged/detail/named-key")
    );
    let search = db.request("GET", "/admin/slugged/search", json!({})).await;
    assert!(html(&search).contains("href=\"/admin/slugged/detail/named-key\""));
    let updated = db
        .request(
            "POST",
            "/admin/slugged/update/named-key",
            json!({
                "csrf":"incoming-token", "title":"Changed"
            }),
        )
        .await;
    assert_eq!(
        updated.headers().get("location"),
        Some("/admin/slugged/detail/named-key")
    );
    assert_eq!(
        db.detail("/admin/slugged/detail/named-key").await["title"],
        "Changed"
    );
    // Exercise the retained Rust copy alias too; it still selects the same source action.
    let copied = db
        .request(
            "POST",
            "/admin/slugged/copy/named-key",
            json!({
                "csrf":"incoming-token", "title":"Copy"
            }),
        )
        .await;
    let location = copied.headers().get("location").unwrap();
    assert!(location.starts_with("/admin/slugged/detail/"));
    assert_ne!(location, "/admin/slugged/detail/named-key");
    assert_eq!(db.detail(location).await["title"], "Copy");
    db.engine.close().await.unwrap();
}

/// Source create/copy validates the result object before consulting noview, not its id member.
#[tokio::test]
async fn source_admin_creation_checks_object_shape_even_with_noview() {
    for (module, suffix) in [("catalog", "/category/article"), ("slugged", "/slug")] {
        for value in [json!([]), json!(null), json!({})] {
            let mut app = ServerBuilder::new();
            let output = value.clone();
            app.on(
                &format!("{module}-create"),
                stackpress_lib::handler(move |_, res, _| {
                    let output = output.clone();
                    Box::pin(async move {
                        res.results(&output)?;
                        Ok(Flow::Abort)
                    })
                }),
            )
            .unwrap();
            admin::register_with::<stackpress_framework_session::storage::DefaultModels>(
                &mut app,
                presentation::ui::pages::ui(),
                Default::default(),
                &["catalog", "slugged"],
            )
            .unwrap();
            let server = app.build().await.unwrap();
            for copy in [false, true] {
                for noview in [false, true] {
                    let path = format!("/admin/{module}/create{}", if copy { suffix } else { "" });
                    let mut data = json!({"csrf":"incoming-token", "title":"Draft"});
                    if noview {
                        data["json"] = json!(true);
                    }
                    let mut req = Request::builder()
                        .method("POST")
                        .url(path)
                        .data(Nest::from_object(data).unwrap())
                        .build()
                        .unwrap();
                    req.session.set("csrf", "incoming-token", None).unwrap();
                    let mut res = Response::new();
                    let outcome = server.handle_route(&mut req, &mut res).await;
                    outcome.unwrap();
                    if !value.is_object() {
                        assert_eq!(res.status().unwrap().code, 400);
                        assert_eq!(res.error(), Some("Unknown creation response results"));
                    } else {
                        let expected = if noview {
                            None
                        } else if module == "catalog" {
                            Some("/admin/catalog/search")
                        } else {
                            Some("/admin/slugged/detail/undefined")
                        };
                        assert_eq!(res.headers().get("location"), expected);
                    }
                }
            }
        }
    }
}
