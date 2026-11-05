//! Real SQLite source parity, source SQL rendering and finite host composition.
use serde_json::{Value as Json, json};
use stackpress_framework_api::{models::*, sql};
use stackpress_framework_session::{profile::ProfileSchema, storage::ProfileActions};
use stackpress_lib::{Body, Flow, Nest, Request, Response, Result};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::{DialectId, Engine, Field, Render, SqlType};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn record(value: Json) -> Record {
    serde_json::from_value(value).unwrap()
}
/// Each test owns a unique file and explicitly closes its engine before removal.
struct Fixture {
    path: PathBuf,
    engine: Arc<Engine>,
    app: ApplicationActions,
    session: SessionActions,
    profile: ProfileActions,
}
impl Fixture {
    async fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "slice10-m1-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let engine = Arc::new(Engine::new(
            SqliteDatabase::open(SqliteOptions::file(&path).create_if_missing(true))
                .await
                .unwrap(),
        ));
        let app = ApplicationActions::new(engine.clone(), "seed");
        let session = SessionActions::new(engine.clone(), "seed");
        let profile = ProfileActions::new(engine.clone(), "seed");
        profile.install().await.unwrap();
        app.install().await.unwrap();
        session.install().await.unwrap();
        profile
            .create(&record(json!({"id":"p", "name":"Ada", "roles":["USER"]})))
            .await
            .unwrap();
        Self {
            path,
            engine,
            app,
            session,
            profile,
        }
    }
    async fn seeded() -> Self {
        let this = Self::new().await;
        this.app.create(&application("a")).await.unwrap();
        this.session.create(&session("s")).await.unwrap();
        this
    }
    async fn close(self) {
        self.engine.close().await.unwrap();
        std::fs::remove_file(self.path).unwrap();
    }
    fn builder(&self) -> ServerBuilder {
        let mut app = ServerBuilder::new();
        let engine = self.engine.clone();
        app.on_config(stackpress_lib::queue::task(
            move |start: &mut StartupContext| {
                let engine = engine.clone();
                Box::pin(async move {
                    start.register(engine)?;
                    Ok(Flow::Continue)
                })
            },
        ))
        .unwrap();
        sql::register(&mut app).unwrap();
        app
    }
    async fn server(&self) -> Server {
        self.builder().build().await.unwrap()
    }
}
fn application(id: &str) -> Record {
    let mut input = record(
        json!({"id":id,"profileId":"p","name":"Example","secret":"plaintext","scopes":["read","write"],"created":0,"updated":0}),
    );
    for key in ["expires", "created", "updated"] {
        input.insert(key.into(), date(0));
    }
    input
}
fn session(id: &str) -> Record {
    let mut input = application(id);
    input.insert("applicationId".into(), "a".into());
    input
}
async fn emit(server: &Server, event: &str, data: Json) -> (Request, Response) {
    let mut req = Request::builder()
        .data(Nest::from_object(data).unwrap())
        .build()
        .unwrap();
    let mut res = Response::new();
    server.emit(event, &mut req, &mut res).await.unwrap();
    (req, res)
}
fn result(res: &Response) -> Json {
    res.result().unwrap().to_json().unwrap()
}

/// G46 preserves PostgreSQL ILIKE and bound data; both other dialects use supported LIKE.
#[test]
fn g46_search_operators_follow_dialect_for_application_and_session() {
    let application = ApplicationStore::new("");
    let session = SessionStore::new("");
    let search = StoreQuery {
        q: Some("A LIKE B".into()),
        ..Default::default()
    };
    let like = StoreQuery {
        like: record(json!({"id":"A LIKE B"})),
        ..Default::default()
    };
    for (dialect, operator, excluded) in [
        (DialectId::Sqlite, " LIKE ", " ILIKE "),
        (DialectId::Postgres, " ILIKE ", " LIKE "),
        (DialectId::Mysql, " LIKE ", " ILIKE "),
    ] {
        for statement in [
            application.select(&search, dialect).unwrap(),
            application.select(&like, dialect).unwrap(),
            session.select(&like, dialect).unwrap(),
        ] {
            let query = statement.render(dialect).unwrap();
            assert!(
                query.sql().contains(operator),
                "{dialect:?}: {}",
                query.sql()
            );
            assert!(
                !query.sql().contains(excluded),
                "{dialect:?}: {}",
                query.sql()
            );
            assert!(
                query
                    .binds()
                    .contains(&stackpress_sql::SqlValue::Text("%A LIKE B%".into()))
            );
        }
    }
}

/// Actual SQLite q/like reads and registered search events now match ASCII case-insensitively.
#[tokio::test]
async fn g46_sqlite_ascii_searches_execute_for_application_and_session() {
    let f = Fixture::seeded().await;
    let query = StoreQuery {
        q: Some("eXaMpLe".into()),
        ..Default::default()
    };
    assert_eq!(f.app.find_all(&query).await.unwrap()[0]["id"], "a");
    assert_eq!(f.app.count(&query).await.unwrap(), 1);
    let like = StoreQuery {
        like: record(json!({"name":"eXaMpLe"})),
        ..Default::default()
    };
    assert_eq!(f.app.find_all(&like).await.unwrap()[0]["id"], "a");
    let like = StoreQuery {
        like: record(json!({"id":"S"})),
        ..Default::default()
    };
    assert_eq!(f.session.find_all(&like).await.unwrap()[0]["id"], "s");
    assert_eq!(f.session.count(&like).await.unwrap(), 1);
    let server = f.server().await;
    let (_, response) = emit(&server, "application-search", json!({"q":"eXaMpLe"})).await;
    assert_eq!(response.status().unwrap().code, 200);
    assert_eq!(result(&response)[0]["id"], "a");
    f.close().await;
}

#[test]
fn source_model_columns_defaults_provider_and_plaintext_codecs() {
    for (schema, names) in [
        (
            ApplicationSchema::new().columns().to_vec(),
            vec![
                "id",
                "profileId",
                "name",
                "logo",
                "website",
                "secret",
                "scopes",
                "active",
                "expires",
                "created",
                "updated",
            ],
        ),
        (
            SessionSchema::new().columns().to_vec(),
            vec![
                "id",
                "applicationId",
                "profileId",
                "secret",
                "scopes",
                "active",
                "expires",
                "created",
                "updated",
            ],
        ),
    ] {
        assert_eq!(
            schema
                .iter()
                .map(|column| column.name())
                .collect::<Vec<_>>(),
            names
        );
        assert_eq!(
            schema
                .iter()
                .find(|column| column.name() == "secret")
                .unwrap()
                .kind(),
            Kind::String
        );
        assert!(!schema.iter().any(|column| column.name() == "profile"));
    }
    let schema = ApplicationSchema::new();
    let first = schema.defaults_at(1234).unwrap();
    let second = schema.defaults_at(1234).unwrap();
    assert_eq!(first["scopes"], json!([]));
    assert_eq!(first["active"], true);
    assert!(!first.contains_key("expires"));
    assert_ne!(first["id"], first["secret"]);
    assert_ne!(first["id"], second["id"]);
    for key in ["id", "secret"] {
        assert_eq!(first[key].as_str().unwrap().len(), 24);
    }
    let populated = schema
        .populate(&record(json!({"secret":"chosen", "extra":true})))
        .unwrap();
    assert_eq!(populated["secret"], "chosen");
    assert_eq!(populated["extra"], true);
    let filtered = schema.filter(&populated, false).unwrap();
    assert!(!filtered.contains_key("extra"));
    let errors = schema.assert(&Record::new(), true).unwrap();
    assert!(errors.contains_key("expires"));
    let serialized = schema
        .serialize(&record(json!({"secret":"clear"})), true)
        .unwrap();
    assert_eq!(serialized["secret"], "clear");
    let store = ApplicationStore::new("any seed");
    let encoded = store
        .scalarize(&record(json!({"secret":"clear","logo":null})))
        .unwrap();
    assert!(
        encoded.iter().any(|(key, value)| key == "secret"
            && value == &stackpress_sql::SqlValue::Text("clear".into()))
    );
    assert_eq!(
        store
            .unscalarize(&record(json!({"profile_id":"p", "secret":"clear"})))
            .unwrap()["profileId"],
        "p"
    );
    assert_eq!(ProfileSchema::default().model().name, "Profile");
}
#[test]
fn provider_boundary_is_shared_and_failures_propagate() {
    struct Provider;
    impl stackpress_framework_schema::ValueProviders for Provider {
        fn generate(
            &self,
            name: &str,
            args: &[Json],
        ) -> stackpress_framework_schema::SchemaResult<Json> {
            assert_eq!(name, "cuid");
            assert!(args.is_empty());
            Ok("fixed".into())
        }
        fn validate(
            &self,
            _: &str,
            _: &Json,
            _: &[Json],
        ) -> stackpress_framework_schema::SchemaResult<bool> {
            unreachable!()
        }
    }
    assert_eq!(
        SessionSchema::new().defaults_with(0, &Provider).unwrap()["secret"],
        "fixed"
    );
    assert_eq!(
        ApplicationSchema::new()
            .defaults_with(0, &Provider)
            .unwrap()["id"],
        "fixed"
    );
    assert!(
        ApplicationSchema::new()
            .with_offset_minutes(i32::MAX)
            .serialize(&Record::from_iter([("expires".into(), date(0))]), false)
            .is_err()
    );
}
#[tokio::test]
async fn real_sqlite_full_create_projection_plaintext_and_no_expiry_gate() {
    let f = Fixture::seeded().await;
    let row = f
        .app
        .find(&StoreQuery::eq("id", "a"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row["profileId"], "p");
    assert_eq!(row["logo"], Value::Null);
    assert_eq!(row["website"], Value::Null);
    assert_eq!(row["secret"], "plaintext");
    assert_eq!(row["expires"], date(0));
    let mut query = StoreQuery::eq("id", "s");
    query.columns = vec![
        "*".into(),
        "profile.*".into(),
        "application.*".into(),
        "application.profile.name".into(),
    ];
    let row = f.session.find(&query).await.unwrap().unwrap();
    assert_eq!(row["applicationId"], "a");
    assert_eq!(row["profile"].get("name").unwrap(), "Ada");
    assert_eq!(row["application"].get("secret").unwrap(), "plaintext");
    assert_eq!(
        row["application"]
            .get("profile")
            .unwrap()
            .get("name")
            .unwrap(),
        "Ada"
    );
    f.profile
        .remove(&stackpress_framework_session::storage::StoreQuery::eq(
            "id", "p",
        ))
        .await
        .unwrap();
    assert!(f.session.find(&query).await.unwrap().is_some());
    query.columns = vec!["secret".into()];
    let row = f.session.find(&query).await.unwrap().unwrap();
    assert_eq!(row.len(), 1);
    assert_eq!(row["secret"], "plaintext");
    f.close().await;
}
#[tokio::test]
async fn create_defaults_required_errors_native_return_and_foreign_keys() {
    let f = Fixture::new().await;
    let error = f.app.create(&Record::new()).await.unwrap_err();
    assert_eq!(error.code, 400);
    assert_eq!(error.error, "Invalid parameters");
    assert!(error.errors.contains_key("name"));
    let mut input = application("a");
    input.shift_remove("id");
    input.shift_remove("secret");
    input.insert("unknown".into(), true.into());
    let created = f.app.create(&input).await.unwrap();
    assert!(created["secret"].as_str().unwrap().len() == 24);
    assert!(!created.contains_key("profileId"));
    assert!(!created.contains_key("unknown"));
    let rows = f.app.find_all(&StoreQuery::default()).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["profileId"], "p");
    let mut invalid = application("bad");
    invalid.insert("profileId".into(), "missing".into());
    assert_eq!(f.app.create(&invalid).await.unwrap_err().code, 500);
    assert!(f.session.create(&session("missing-app")).await.is_err());
    f.close().await;
}
#[tokio::test]
async fn source_partial_update_return_differs_from_write_and_remove_restore() {
    let f = Fixture::seeded().await;
    let query = StoreQuery::eq("id", "a");
    let mut patch = record(json!({"name":"", "website":null}));
    let rows = f.app.update(&query, &mut patch).await.unwrap();
    assert_eq!(rows[0]["name"], "Example");
    assert_eq!(rows[0]["scopes"], json!([]));
    assert_eq!(rows[0]["updated"], date(0));
    assert!(patch.contains_key("updated"));
    let row = f.app.find(&query).await.unwrap().unwrap();
    assert_eq!(row["name"], "");
    assert_eq!(row["scopes"], json!(["read", "write"]));
    assert_eq!(row["website"], Value::Null);
    assert_ne!(row["updated"], date(0));
    f.app.remove(&query).await.unwrap();
    assert!(f.app.find(&query).await.unwrap().is_none());
    assert!(f.app.restore(&query).await.unwrap().is_empty());
    f.app.restore(&query.clone().all()).await.unwrap();
    assert!(f.app.find(&query).await.unwrap().is_some());
    f.app
        .update(&query, &mut record(json!({"active":null})))
        .await
        .unwrap();
    assert!(f.app.find(&query).await.unwrap().is_none());
    assert!(
        f.app
            .update(
                &StoreQuery::eq("id", "none"),
                &mut record(json!({"name":"x"}))
            )
            .await
            .unwrap()
            .is_empty()
    );
    f.close().await;
}
#[tokio::test]
async fn source_filters_sort_paging_membership_and_sqlite_like_search() {
    let f = Fixture::seeded().await;
    f.app.create(&application("b")).await.unwrap();
    let mut query = StoreQuery {
        eq: record(json!({"id":["a","b"],"unknown":"ignored","website":null})),
        ne: record(json!({"id":"c"})),
        ge: record(json!({"id":"a"})),
        le: record(json!({"id":"b"})),
        sort: record(json!({"id":"DESC","name":"invalid"})),
        take: 1,
        ..Default::default()
    };
    // Source select.ts:567–570 flattens arrays to indexed paths. Scalar id.0
    // and id.1 resolve to the same stored id, producing AND, never an OR group.
    assert!(f.app.find(&query).await.unwrap().is_none());
    assert_eq!(f.app.count(&query).await.unwrap(), 0);
    query.eq.shift_remove("id");
    assert_eq!(f.app.find(&query).await.unwrap().unwrap()["id"], "b");
    assert_eq!(f.app.count(&query).await.unwrap(), 2);
    query.skip = 1;
    assert_eq!(f.app.find(&query).await.unwrap().unwrap()["id"], "a");
    query = StoreQuery {
        has: record(json!({"scopes":"read"})),
        ..Default::default()
    };
    assert_eq!(f.app.find_all(&query).await.unwrap().len(), 2);
    query.hasnt = std::mem::take(&mut query.has);
    assert_eq!(f.app.find_all(&query).await.unwrap().len(), 2);
    query = StoreQuery {
        q: Some("Example".into()),
        ..Default::default()
    };
    assert_eq!(f.app.find_all(&query).await.unwrap().len(), 2);
    query = StoreQuery {
        like: record(json!({"name":"%"})),
        ..Default::default()
    };
    assert_eq!(f.app.find_all(&query).await.unwrap().len(), 2);
    query = StoreQuery {
        eq: record(json!({"profile":{"name":"Ada"}})),
        ..Default::default()
    };
    assert_eq!(f.app.count(&query).await.unwrap(), 2);
    assert!(
        f.session
            .find_all(&StoreQuery {
                eq: record(json!({"application.unknown":"x"})),
                ..Default::default()
            })
            .await
            .is_err()
    );
    f.close().await;
}
#[tokio::test]
async fn batch_atomicity_attempt_status_and_upsert_missing_id() {
    let f = Fixture::seeded().await;
    assert!(
        f.app
            .upsert(&mut application("absent"))
            .await
            .unwrap()
            .is_none()
    );
    let mut without = application("discard");
    without.shift_remove("id");
    assert!(f.app.upsert(&mut without).await.unwrap().is_some());
    let statuses = f
        .app
        .batch(
            &mut [application("rolled-back"), Record::new()],
            BatchMode::Create,
        )
        .await;
    assert_eq!(statuses[0].code, Some(200));
    assert_eq!(statuses[1].code, Some(400));
    assert!(
        f.app
            .find(&StoreQuery::eq("id", "rolled-back"))
            .await
            .unwrap()
            .is_none()
    );
    let statuses = f
        .app
        .batch(
            &mut [record(json!({"id":"a","name":"kept"})), Record::new()],
            BatchMode::Update,
        )
        .await;
    assert_eq!(statuses[0].code, Some(200));
    assert!(statuses[1].code.is_none());
    assert_eq!(
        f.app
            .find(&StoreQuery::eq("id", "a"))
            .await
            .unwrap()
            .unwrap()["name"],
        "kept"
    );
    let statuses = f
        .app
        .batch(&mut [application("absent")], BatchMode::Upsert)
        .await;
    assert_eq!(statuses[0].results, Some(Value::Null));
    assert_eq!(statuses[0].total, Some(1));
    f.close().await;
}
#[tokio::test]
async fn delete_cascade_maintenance_and_explicit_transaction_affinity() {
    let f = Fixture::seeded().await;
    let mut transaction = f.engine.begin().await.unwrap();
    {
        let mut db = stackpress_framework_sql::DataSession::transaction(&mut transaction);
        f.app
            .create_in(&mut db, &application("rollback"))
            .await
            .unwrap();
        assert!(
            f.app
                .find_in(&mut db, &StoreQuery::eq("id", "rollback"))
                .await
                .unwrap()
                .is_some()
        );
    }
    transaction.rollback().await.unwrap();
    assert!(
        f.app
            .find(&StoreQuery::eq("id", "rollback"))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.app
            .delete(&StoreQuery::eq("id", "a"))
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.session.count(&StoreQuery::default().all()).await.unwrap(),
        0
    );
    let to = f
        .app
        .store
        .create()
        .add_field(Field::new("extra", SqlType::Text));
    f.app.upgrade(&to).await.unwrap();
    f.app.purge(false).await.unwrap();
    f.session.uninstall().await.unwrap();
    f.app.uninstall().await.unwrap();
    assert!(f.app.find(&StoreQuery::default()).await.is_err());
    f.close().await;
}
/// Retained relation paths are source selectors even when the child segment is unknown.
#[test]
fn source_forward_relation_fallbacks_preserve_the_last_edge() {
    let session = SessionStore::new("");
    for (path, model, table, column, joins) in [
        ("application", "Session", "session", "application", 1),
        (
            "application.unknown",
            "Session",
            "session",
            "application",
            1,
        ),
        (
            "application.session.*",
            "Session",
            "session",
            "application",
            1,
        ),
        ("profile.unknown", "Session", "session", "profile", 1),
        (
            "application.profile.unknown",
            "Application",
            "session__application",
            "profile",
            2,
        ),
        (
            "application.profile.name",
            "Profile",
            "session__application__profile",
            "name",
            2,
        ),
    ] {
        let selector = session.paths(path).into_iter().next().unwrap();
        assert_eq!(
            (
                selector.model,
                selector.table.as_str(),
                selector.column.as_str()
            ),
            (model, table, column),
            "{path}"
        );
        let query = StoreQuery {
            columns: vec![path.into()],
            ..Default::default()
        };
        assert_eq!(session.joins(&query).len(), joins, "{path}");
    }
    let app = ApplicationStore::new("");
    assert_eq!(app.paths("profile.unknown")[0].column, "profile");
    assert!(
        app.paths("session.*").is_empty(),
        "reverse arrays do not become joins"
    );
    assert!(
        session.paths("unknown.profile").is_empty(),
        "no valid leading edge"
    );
}

#[test]
fn source_backend_rendering_and_selected_paths() {
    let store = SessionStore::new("");
    let query = StoreQuery {
        columns: vec![
            "id".into(),
            "application.profile.name".into(),
            "profile.roles:item".into(),
        ],
        eq: record(json!({"application":{"profile":{"name":["Ada","Bob"]}}})),
        ..Default::default()
    };
    assert!(store.paths("unknown").is_empty());
    assert_eq!(
        store.paths("application.session.*")[0].column,
        "application"
    );
    assert_eq!(store.joins(&query).len(), 3);
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        let sql = store
            .select(&query, dialect)
            .unwrap()
            .render(dialect)
            .unwrap();
        assert!(sql.sql().contains("INNER JOIN"));
        assert_eq!(sql.binds().len(), 3);
        assert!(store.create().render(dialect).is_ok());
    }
    let query = StoreQuery {
        q: Some("search".into()),
        ..Default::default()
    };
    let sql = ApplicationStore::new("")
        .select(&query, DialectId::Postgres)
        .unwrap()
        .render(DialectId::Postgres)
        .unwrap();
    assert_eq!(sql.sql().matches("ILIKE").count(), 2);
    assert!(
        store
            .insert_for(&session("s"), DialectId::Mysql)
            .unwrap()
            .render(DialectId::Mysql)
            .is_ok()
    );
    assert!(
        !store
            .insert_for(&session("s"), DialectId::Mysql)
            .unwrap()
            .render(DialectId::Mysql)
            .unwrap()
            .sql()
            .contains("RETURNING")
    );
    assert!(store.alter(None).is_ok());
    assert!(store.alter(Some(&store.create())).is_ok());
}
#[tokio::test]
async fn generated_events_detail_search_crud_restore_and_batch() {
    let f = Fixture::seeded().await;
    let server = f.server().await;
    let (_, res) = emit(&server, "application-detail", json!({"id":"a"})).await;
    assert_eq!(result(&res)["profile"]["name"], "Ada");
    assert_eq!(result(&res)["secret"], "plaintext");
    let (_, res) = emit(&server, "session-detail", json!({"id":"s"})).await;
    assert_eq!(result(&res)["application"]["secret"], "plaintext");
    let (req, res) = emit(
        &server,
        "application-search",
        json!({"columns":false,"take":1}),
    )
    .await;
    assert_eq!(result(&res).as_array().unwrap().len(), 1);
    assert!(req.data.get("columns").unwrap().as_array().is_some());
    let (_, res) = emit(
        &server,
        "application-get",
        json!({"key":"id","value":"a","columns":false}),
    )
    .await;
    assert!(result(&res).get("profile").is_none());
    let (req, res) = emit(
        &server,
        "application-update",
        json!({"id":"a","name":"changed","eq":{"id":"wrong"}}),
    )
    .await;
    assert_eq!(result(&res)["name"], "changed");
    assert!(req.data.get("updated").is_some());
    for event in ["application-remove", "application-restore"] {
        let (_, res) = emit(&server, event, json!({"id":"a"})).await;
        assert_eq!(res.status().unwrap().code, 200);
    }
    let (_, res) = emit(
        &server,
        "application-upsert",
        json!({"id":"not-present","name":"x"}),
    )
    .await;
    assert_eq!(result(&res), Json::Null);
    let (_, res) = emit(
        &server,
        "application-batch",
        json!({"mode":"create","rows":[{}]}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 400);
    let (_, res) = emit(&server, "session-purge", json!({})).await;
    assert!(matches!(res.body(),Body::Text(value) if value=="session"));
    f.close().await;
}
#[tokio::test]
async fn generated_event_errors_missing_database_and_g42_chain() -> Result<()> {
    let mut app = ServerBuilder::new();
    sql::register(&mut app)?;
    sql::register(&mut app)?;
    let server = app.build().await.map_err(|e| e.error)?;
    assert_eq!(server.listeners().count(), 40);
    let (_, res) = emit(&server, "application-create", json!({})).await;
    assert!(matches!(res.body(), Body::Empty));
    let f = Fixture::seeded().await;
    let server = f.server().await;
    for (event, data, code, message) in [
        ("application-detail", json!({}), 400, "Invalid Parameters"),
        (
            "application-detail",
            json!({"id":"absent"}),
            404,
            "Not Found",
        ),
        ("application-get", json!({}), 400, "Invalid Parameters"),
        (
            "application-batch",
            json!({"rows":null}),
            400,
            "Invalid Parameters",
        ),
        ("application-create", json!({}), 400, "Invalid parameters"),
    ] {
        let (_, res) = emit(&server, event, data).await;
        assert_eq!(res.status().unwrap().code, code);
        assert_eq!(res.error(), Some(message));
    }
    let mut response = Response::new();
    response.results(&false)?;
    server
        .emit("application-create", &mut Request::new(), &mut response)
        .await?;
    assert_eq!(response.error(), Some("Invalid parameters"));
    let mut response = Response::new();
    response.results(&json!({"keep":true}))?;
    server
        .emit("application-create", &mut Request::new(), &mut response)
        .await?;
    assert_eq!(result(&response), json!({"keep":true}));
    f.close().await;
    Ok(())
}
#[test]
#[cfg(feature = "codegen")]
fn full_idea_normalization_retains_management_fields_and_composed_profile() {
    let definitions = stackpress_framework_api::codegen::definitions().unwrap();
    let schema = stackpress_framework_api::codegen::normalize(&definitions).unwrap();
    assert_eq!(schema.models().len(), 4);
    for name in ["Application", "Session"] {
        let model = schema
            .models()
            .iter()
            .find(|model| model.name == name)
            .unwrap();
        let secret = model
            .fields
            .iter()
            .find(|field| field.name == "secret")
            .unwrap();
        assert!(!secret.access.sensitive);
        // Source catalogs do not use Typed DTO read grants. The original declares
        // @list.string/@view.string on both secrets (schema.idea:61,167).
        let ui = schema.project_source_ui(&definitions).unwrap();
        let public = ui.models.iter().find(|model| model.id == name).unwrap();
        let public_secret = public
            .fields
            .iter()
            .find(|field| field.wire_name == "secret")
            .unwrap();
        assert!(
            public_secret
                .roles
                .iter()
                .any(|binding| binding.role == stackpress_framework_schema::Role::View)
        );
        assert_eq!(secret.value_rule.as_ref().unwrap().provider, "cuid");
        // Preserve source @query("*" "profile.*" ...), rather than the old
        // Typed adapter's expanded synthetic Profile field list.
        assert!(model.query.contains(&"profile.*".into()));
    }
}

#[test]
#[cfg(unix)]
fn real_sqlite_concurrent_source_lookups_have_process_deadline() {
    assert!(
        super::proof::watchdog(false, "sql::sql_child"),
        "M1 SQL deadline"
    );
}
#[test]
#[ignore = "run only under the parent process watchdog"]
fn sql_child() {
    std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready").unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let f = Fixture::seeded().await;
            let server = f.server().await;
            let barrier = tokio::sync::Barrier::new(2);
            let read = |name| {
                let server = &server;
                let barrier = &barrier;
                async move {
                    barrier.wait().await;
                    emit(
                        server,
                        name,
                        json!({"id": if name=="application-detail" { "a" } else { "s" }}),
                    )
                    .await
                    .1
                }
            };
            let (app, session) = tokio::join!(read("application-detail"), read("session-detail"));
            assert_eq!(result(&app)["id"], "a");
            assert_eq!(result(&session)["id"], "s");
            // Cancellation after a native transaction write drops its exclusive connection;
            // the subsequent read proves rollback and pool reuse without a sleep assertion.
            let mut transaction = f.engine.begin().await.unwrap();
            f.app
                .create_in(
                    &mut stackpress_framework_sql::DataSession::transaction(&mut transaction),
                    &application("cancelled"),
                )
                .await
                .unwrap();
            drop(transaction);
            assert!(
                f.app
                    .find(&StoreQuery::eq("id", "cancelled"))
                    .await
                    .unwrap()
                    .is_none()
            );
            f.close().await;
        });
}

#[tokio::test]
async fn event_request_mutations_defaults_status_guards_and_empty_paths() -> Result<()> {
    let f = Fixture::seeded().await;
    let server = f.server().await;
    let (req, _) = emit(
        &server,
        "application-detail",
        json!({"id":"a","eq":{"id":"wrong"}}),
    )
    .await;
    assert_eq!(req.data.get("eq.id").unwrap().as_str(), Some("a"));
    let (req, res) = emit(
        &server,
        "application-upsert",
        json!({"id":"a","name":"new"}),
    )
    .await;
    assert_eq!(result(&res)["name"], "new");
    assert!(matches!(
        req.data.get("updated"),
        Some(stackpress_lib::Value::DateTime(_))
    ));
    let (req, res) = emit(
        &server,
        "application-batch",
        json!({"mode":"update","rows":[{"id":"a","name":"batch"}]}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 200);
    assert!(
        req.data.get("rows").unwrap().as_array().unwrap()[0]
            .as_object()
            .unwrap()
            .contains_key("updated")
    );
    let (_, res) = emit(
        &server,
        "application-batch",
        json!({"mode":"other","rows":[{}]}),
    )
    .await;
    assert_eq!(result(&res), json!([]));
    let (_, res) = emit(
        &server,
        "application-detail",
        json!({"id":"a","columns":[]}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 500);
    let (_, res) = emit(
        &server,
        "application-detail",
        json!({"id":"a","columns":["name"]}),
    )
    .await;
    assert_eq!(result(&res), json!({"name":"batch"}));
    let (_, res) = emit(
        &server,
        "application-get",
        json!({"key":"id","value":"absent"}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 404);
    let (_, res) = emit(&server, "application-search", json!({"q":"Example"})).await;
    assert_eq!(res.status().unwrap().code, 200);
    assert_eq!(result(&res), json!([]));
    for body in [Body::Text("existing".into()), Body::Bytes(Vec::new())] {
        let mut res = Response::new();
        res.set("text/plain", body.clone())?;
        server
            .emit("application-purge", &mut Request::new(), &mut res)
            .await?;
        assert_eq!(f.app.count(&StoreQuery::default()).await.unwrap(), 1);
    }
    let mut res = Response::new();
    res.status_code(201, None)?;
    server
        .emit("application-purge", &mut Request::new(), &mut res)
        .await?;
    assert_eq!(f.app.count(&StoreQuery::default()).await.unwrap(), 1);
    f.close().await;
    Ok(())
}

#[tokio::test]
async fn scripted_backend_fallback_and_failures_are_not_native_backend_claims() {
    use stackpress_sql::{
        Database, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
        TransactionSession,
    };
    /// This lane proves source return branches, not a live MySQL installation.
    struct Script {
        dialect: DialectId,
        insert_id: Option<i64>,
        fail: bool,
    }
    impl Database for Script {
        fn dialect(&self) -> DialectId {
            self.dialect
        }
        fn run(&self, query: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
            Box::pin(async move {
                if self.fail {
                    return Err(stackpress_sql::SqlError::new(
                        stackpress_sql::ErrorKind::Driver,
                        "scripted",
                    ));
                }
                Ok(QueryResult {
                    insert_id: if query.sql().starts_with("INSERT") {
                        self.insert_id
                    } else {
                        None
                    },
                    rows: Vec::new(),
                    affected: 1,
                })
            })
        }
        fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
            Box::pin(async {
                Err(stackpress_sql::SqlError::new(
                    stackpress_sql::ErrorKind::Driver,
                    "scripted begin",
                ))
            })
        }
        fn close(&self) -> SqlFuture<'_, ()> {
            Box::pin(async { Ok(()) })
        }
    }
    for insert_id in [None, Some(0), Some(7)] {
        let engine = Arc::new(Engine::new(Script {
            dialect: DialectId::Mysql,
            insert_id,
            fail: false,
        }));
        let actions = ApplicationActions::new(engine.clone(), "");
        let input = application("a");
        assert_eq!(actions.create(&input).await.unwrap(), input);
        assert!(
            actions
                .batch(&mut [input], BatchMode::Create)
                .await
                .is_empty()
        );
        engine.close().await.unwrap();
    }
    let engine = Arc::new(Engine::new(Script {
        dialect: DialectId::Postgres,
        insert_id: None,
        fail: true,
    }));
    let actions = ApplicationActions::new(engine.clone(), "");
    assert_eq!(
        actions.create(&application("a")).await.unwrap_err().code,
        500
    );
    assert!(actions.purge(true).await.is_err());
    engine.close().await.unwrap();
}

#[tokio::test]
async fn existing_routes_use_real_application_and_session_events() -> Result<()> {
    let f = Fixture::seeded().await;
    let mut app = f.builder();
    async fn target(
        req: &mut Request,
        res: &mut Response,
        _: stackpress_lib::Invocation<'_>,
    ) -> Result<Flow> {
        res.results(&req.data)?;
        Ok(Flow::Continue)
    }
    app.on("target", target)?;
    for (kind, path) in [("app", "/app"), ("session", "/session")] {
        let endpoint = serde_json::from_value(
            json!({"method":"ALL","route":path,"type":kind,"event":"target","scopes":["missing","read"]}),
        )?;
        if kind == "app" {
            stackpress_framework_api::routes::app(&mut app, endpoint)?;
        } else {
            stackpress_framework_api::routes::session(&mut app, endpoint)?;
        }
    }
    let server = app.build().await.map_err(|e| e.error)?;
    for (path, id, method, credential, profile) in [
        ("/app", "a", "GET", "a", false),
        ("/app", "a", "POST", "a:plaintext", false),
        ("/session", "s", "GET", "s", true),
        ("/session", "s", "POST", "s:plaintext", true),
    ] {
        let mut headers = stackpress_lib::Headers::new();
        headers.set("Authorization", &format!("Any {credential}"))?;
        let mut req = Request::builder()
            .url(path)
            .method(method)
            .headers(headers)
            .data(Nest::from_object(
                json!({"caller":id,"profileId":"forged"}),
            )?)
            .build()?;
        req.session.set("session", "browser-cookie", None)?;
        let mut res = Response::new();
        server.handle_route(&mut req, &mut res).await?;
        assert_eq!(result(&res)["caller"], id);
        assert_eq!(
            result(&res)["profileId"],
            if profile { "p" } else { "forged" }
        );
        assert_eq!(
            req.session.get("session").unwrap().as_str(),
            Some("browser-cookie")
        );
    }
    // SQL's base active filter, rather than endpoint hardening, now decides this lookup.
    f.session.remove(&StoreQuery::eq("id", "s")).await.unwrap();
    let mut headers = stackpress_lib::Headers::new();
    headers.set("Authorization", "Any s")?;
    let mut req = Request::builder()
        .url("/session")
        .headers(headers)
        .build()?;
    let mut res = Response::new();
    server.handle_route(&mut req, &mut res).await?;
    assert_eq!(res.status().unwrap().code, 401);
    let (_,res)=emit(&server,"application-create",json!({"id":"new", "profileId":"p", "name":"Event", "expires":"1970-01-01T00:00:00.000Z", "secret":"chosen", "active":false, "unrelated":true})).await;
    assert_eq!(result(&res)["secret"], "chosen");
    assert_eq!(result(&res)["active"], false);
    assert!(result(&res).get("unrelated").is_none());
    assert!(res.try_to_json()?["results"]["created"].is_string());
    let (_, res) = emit(
        &server,
        "application-batch",
        json!({"mode":"create","rows":[null,7]}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 400);
    assert_eq!(result(&res)[0]["code"], 500);
    assert_eq!(result(&res)[1]["code"], 400);
    f.close().await;
    Ok(())
}

#[tokio::test]
async fn javascript_query_numbers_truthiness_and_computed_keys_remain_source_values() {
    let f = Fixture::seeded().await;
    f.app.create(&application("b")).await.unwrap();
    f.app.remove(&StoreQuery::eq("id", "b")).await.unwrap();
    // JavaScript has one Number family: integer and floating JSON -1 remove active.
    for sentinel in [json!(-1), json!(-1.0)] {
        let query = sql::defaults::query(&record(
            json!({"eq":{"active":sentinel},"take":0.0,"skip":0.0}),
        ))
        .unwrap();
        assert_eq!(query.take, 0);
        assert_eq!(f.app.find_all(&query).await.unwrap().len(), 2);
    }
    for value in [Json::Null, json!(false), json!("")] {
        assert_eq!(
            sql::defaults::query(&record(json!({"take":value})))
                .unwrap()
                .take,
            0
        );
    }
    assert_eq!(
        sql::defaults::query(&record(json!({"take":1000.0})))
            .unwrap()
            .take,
        1000
    );
    assert!(sql::defaults::query(&record(json!({"take":-1}))).is_err());
    assert!(sql::defaults::query(&record(json!({"take":1.5}))).is_err());
    // Truthy values retain source coercion: [] searches LIKE %%, while the other strings
    // match no seeded application. Session ignores any q value, as before G46.
    for (q, count) in [
        (json!([]), 1),
        (json!({}), 0),
        (json!(true), 0),
        (json!(1.0), 0),
    ] {
        let query = sql::defaults::query(&record(json!({"q":q}))).unwrap();
        assert_eq!(f.app.find_all(&query).await.unwrap().len(), count);
        assert_eq!(f.session.find_all(&query).await.unwrap().len(), 1);
    }
    for q in [json!(0.0), json!(false), Json::Null, json!("")] {
        let query = sql::defaults::query(&record(json!({"q":q}))).unwrap();
        assert_eq!(f.app.count(&query).await.unwrap(), 1);
    }
    let server = f.server().await;
    let (_, res) = emit(
        &server,
        "application-get",
        json!({"key":["id"],"value":"a"}),
    )
    .await;
    assert_eq!(result(&res)["id"], "a");
    let (_, res) = emit(&server, "application-detail", json!({"id":"a","take":-1})).await;
    assert_eq!(res.status().unwrap().code, 200);
    let (_, res) = emit(
        &server,
        "application-get",
        json!({"key":["id"], "value":"missing"}),
    )
    .await;
    assert_eq!(res.status().unwrap().code, 404);
    f.close().await;
}
