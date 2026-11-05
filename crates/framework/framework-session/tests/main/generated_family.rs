//! Real generated extension metadata reaches handwritten auth and account handlers.
//!
//! Import the maintained, officially regenerated schemas. The local family changes
//! only its Profile action collaborator so the stock signup boundary can be spied.
#[allow(dead_code)]
#[path = "../../../framework-models/tests/extended/generated/source/mod.rs"]
mod generated;

use generated::{auth::AuthSchema, profile::ProfileSchema};
use serde_json::{Value as Json, json};
use stackpress_framework_schema::source::Record;
use stackpress_framework_session::{account, auth, storage::PasswordConfig};
use stackpress_framework_sql::{
    DataSession,
    source::{self, SessionModels, SourceActions, SourceModel, StoreQuery},
};
use stackpress_lib::{Body, Flow, Request, Response};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::{Create, Engine};
use std::sync::Arc;

/// The app's complete generated Profile, with an action override at the real create boundary.
struct Profile;
impl SourceModel for Profile {
    type Role = source::ProfileRole;
    type Schema = ProfileSchema;
    type Store = source::NativeStore<ProfileSchema>;
    type Actions = SignupSpy;
}
/// The same generated Auth descriptor and ordinary native actions.
struct Auth;
impl SourceModel for Auth {
    type Role = source::AuthRole;
    type Schema = AuthSchema;
    type Store = source::NativeStore<AuthSchema>;
    type Actions = source::NativeActions<AuthSchema>;
}
struct Family;
impl SessionModels for Family {
    type Profile = Profile;
    type Auth = Auth;
}

/// No ambient/global spy state: every signup asserts its exact input before real SQLite IO.
struct SignupSpy(source::NativeActions<ProfileSchema>);
impl SourceActions for SignupSpy {
    type Store = source::NativeStore<ProfileSchema>;
    type BatchStatus = source::BatchStatus;
    fn new(engine: Arc<Engine>, store: Self::Store) -> Self {
        Self(source::NativeActions::new(engine, store))
    }
    fn engine(&self) -> &Engine {
        &self.0.engine
    }
    fn store(&self) -> &Self::Store {
        &self.0.store
    }
    async fn create_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a Record,
    ) -> source::SourceResult<Record> {
        // Q50-SIGNUP: authored app columns never become extra stock signup inputs.
        assert_eq!(
            input.keys().map(String::as_str).collect::<Vec<_>>(),
            ["name", "type", "roles"]
        );
        self.0.create_in(db, input).await
    }
    async fn find_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<Option<Record>> {
        self.0.find_in(db, query).await
    }
    async fn find_all_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<Vec<Record>> {
        self.0.find_all_in(db, query).await
    }
    async fn count_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<u64> {
        self.0.count_in(db, query).await
    }
    async fn update_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
        input: &'a mut Record,
    ) -> source::SourceResult<Vec<Record>> {
        self.0.update_in(db, query, input).await
    }
    async fn delete_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<Vec<Record>> {
        self.0.delete_in(db, query).await
    }
    async fn upsert_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a mut Record,
    ) -> source::SourceResult<Option<Record>> {
        self.0.upsert_in(db, input).await
    }
    async fn remove_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<Vec<Record>> {
        self.0.remove_in(db, query).await
    }
    async fn restore_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a StoreQuery,
    ) -> source::SourceResult<Vec<Record>> {
        self.0.restore_in(db, query).await
    }
    async fn batch(&self, rows: &mut [Record], mode: source::BatchMode) -> Vec<Self::BatchStatus> {
        self.0.batch(rows, mode).await
    }
    async fn install(&self) -> source::SourceResult<()> {
        self.0.install().await
    }
    async fn upgrade(&self, to: Option<&Create>) -> source::SourceResult<()> {
        SourceActions::upgrade(&self.0, to).await
    }
    async fn purge(&self, cascade: bool) -> source::SourceResult<()> {
        self.0.purge(cascade).await
    }
    async fn uninstall(&self) -> source::SourceResult<()> {
        self.0.uninstall().await
    }
}

/// Native boundary only; no JSON conversion occurs inside the actual generated actions.
fn record(value: Json) -> Record {
    serde_json::from_value(value).unwrap()
}
async fn emit(host: &Server, event: &str, input: Json, token: Option<&str>) -> Response {
    let mut req = Request::builder().data(input).build().unwrap();
    if let Some(token) = token {
        req.session.set("session", token, None).unwrap();
    }
    let mut response = Response::new();
    host.emit(event, &mut req, &mut response).await.unwrap();
    response
}

/// The appended field reaches signup defaults, arbitrary CRUD, signin joins and ordered export.
#[tokio::test]
async fn generated_family_registered_auth_account_and_export_share_app_profile() {
    let dir = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("family.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(database));
    let profile = Profile::actions(engine.clone(), "seed".into(), 0);
    let auth = Auth::actions(engine.clone(), "seed".into(), 0);
    profile.install().await.unwrap();
    auth.install().await.unwrap();
    let mut app = ServerBuilder::new();
    let mut layers = stackpress_server::ConfigLayers::new();
    layers
        .value("application", json!({"auth": {"base":"/auth"}}))
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
                start.register(Arc::new(stackpress_framework_session::Service::default()))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    auth::register_with::<Family>(&mut app).unwrap();
    account::events::register_with::<Family>(&mut app).unwrap();
    app.on("account-export", account::export).unwrap();
    let host = app.build().await.unwrap();
    // The actual action override rejects forwarding the app's nickname to stock signup.
    let signup = emit(&host, "auth-signup", json!({"name":"Ada", "username":"ada-user", "secret":"correct", "nickname":"must not forward"}), None).await;
    let created = auth::events::json_result(&signup).unwrap();
    assert_eq!(
        created["nickname"],
        "application",
        "{:?}: {:?}",
        signup.error(),
        signup.errors()
    );
    let id = created["id"].as_str().unwrap();
    let update = emit(
        &host,
        "profile-update",
        json!({"id":id,"nickname":"App Ada","references":{"z":{"b":1,"a":2},"a":"last"}}),
        None,
    )
    .await;
    assert_eq!(update.status().unwrap().code, 200);
    let detail = emit(&host, "profile-detail", json!({"id":id}), None).await;
    let value = auth::events::json_result(&detail).unwrap();
    assert_eq!(value["nickname"], "App Ada");
    assert_eq!(
        serde_json::to_string(&value["references"]).unwrap(),
        r#"{"z":{"b":1,"a":2},"a":"last"}"#
    );
    // Direct generic collaborator is the same one used by auth-signin's registered handler.
    let actions = stackpress_framework_session::storage::AccountActions::<Family>::new(
        engine.clone(),
        "seed".into(),
        0,
        PasswordConfig::default(),
    );
    let signed = actions
        .signin(
            "username",
            &record(json!({"username":"ada-user","secret":"correct"})),
            true,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        signed["profile"].as_object().unwrap()["nickname"],
        "App Ada"
    );
    let signin = emit(
        &host,
        "auth-signin",
        json!({"username":"ada-user","secret":"correct","2fa":false}),
        None,
    )
    .await;
    assert_eq!(
        auth::events::json_result(&signin).unwrap()["profile"]["nickname"],
        "App Ada"
    );
    let token = host
        .host()
        .service::<stackpress_framework_session::Service>()
        .unwrap()
        .create(json!({"id":id,"name":"Ada"}).as_object().unwrap().clone())
        .await
        .unwrap();
    let export = emit(
        &host,
        "account-export",
        json!({"download":true}),
        Some(&token),
    )
    .await;
    let Body::Text(csv) = export.body() else {
        panic!("CSV body");
    };
    assert!(csv.contains("nickname,\"App Ada\""), "{csv}");
    assert!(
        csv.contains("references,\"z: b: 1, a: 2, a: last\""),
        "{csv}"
    );
    assert!(
        csv.find("updated,").unwrap() < csv.find("nickname,").unwrap(),
        "source extension order"
    );
    host.host()
        .service::<auth::Hooks>()
        .unwrap()
        .shutdown()
        .await;
    engine.close().await.unwrap();
}

/// Actual session-only emitted code retains reverse links without compiling hidden API stores.
#[allow(dead_code)]
#[path = "../../../framework-sql/tests/source-session-only/source/mod.rs"]
mod session_only;

// This host's `server` feature means identity, not the generated app's optional SQL-event
// feature. Bind the emitted schemas directly to the same ordinary native source runtime.
struct ExternalProfile;
impl SourceModel for ExternalProfile {
    type Role = source::ProfileRole;
    type Schema = session_only::profile::ProfileSchema;
    type Store = source::NativeStore<Self::Schema>;
    type Actions = source::NativeActions<Self::Schema>;
}
struct ExternalAuth;
impl SourceModel for ExternalAuth {
    type Role = source::AuthRole;
    type Schema = session_only::auth::AuthSchema;
    type Store = source::NativeStore<Self::Schema>;
    type Actions = source::NativeActions<Self::Schema>;
}
struct ExternalFamily;
impl SessionModels for ExternalFamily {
    type Profile = ExternalProfile;
    type Auth = ExternalAuth;
}

#[tokio::test]
async fn generated_session_only_external_family_registers_and_installs_only_two_tables() {
    use self::{
        ExternalAuth as AuthModel, ExternalFamily as Models, ExternalProfile as ProfileModel,
    };
    use stackpress_framework_schema::source::SourceSchema;
    let catalog = session_only::profile::ProfileSchema::catalog();
    assert_eq!(
        catalog
            .models
            .iter()
            .map(|model| model.name.as_str())
            .collect::<Vec<_>>(),
        ["Profile", "Auth"]
    );
    assert_eq!(catalog.external_models, ["Application", "Session"]);
    let directory = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(directory.path().join("session-only.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(db));
    ProfileModel::actions(engine.clone(), "seed".into(), 0)
        .install()
        .await
        .unwrap();
    AuthModel::actions(engine.clone(), "seed".into(), 0)
        .install()
        .await
        .unwrap();
    let tables = engine
        .execute(&stackpress_sql::Select::new(["name"]).from("sqlite_master"))
        .await
        .unwrap();
    let names = tables
        .rows
        .iter()
        .map(|row| row.try_get::<String>("name").unwrap())
        .collect::<Vec<_>>();
    assert!(names.iter().any(|name| name == "profile"));
    assert!(names.iter().any(|name| name == "auth"));
    assert!(
        !names
            .iter()
            .any(|name| name == "application" || name == "session")
    );
    let mut app = ServerBuilder::new();
    let mut layers = stackpress_server::ConfigLayers::new();
    layers
        .value("session only", json!({"session":{},"auth":{}}))
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
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    stackpress_framework_session::composite::register_with::<Models>(&mut app).unwrap();
    account::events::register_listen_with::<Models>(&mut app).unwrap();
    let host = app.build().await.unwrap();
    assert!(
        !host
            .listeners()
            .any(|listener| listener.pattern.starts_with("application-")
                || listener.pattern.starts_with("session-"))
    );
    let signup = emit(
        &host,
        "auth-signup",
        json!({"name":"Session Only","username":"session-only","secret":"correct"}),
        None,
    )
    .await;
    let profile = auth::events::json_result(&signup).unwrap();
    let id = profile["id"].as_str().unwrap();
    assert_eq!(
        emit(
            &host,
            "profile-update",
            json!({"id":id,"nickname":"External links"}),
            None
        )
        .await
        .status()
        .unwrap()
        .code,
        200
    );
    let signin = emit(
        &host,
        "auth-signin",
        json!({"username":"session-only","secret":"correct","2fa":false}),
        None,
    )
    .await;
    assert_eq!(
        auth::events::json_result(&signin).unwrap()["profile"]["nickname"],
        "External links"
    );
    host.host()
        .service::<auth::Hooks>()
        .unwrap()
        .shutdown()
        .await;
    engine.close().await.unwrap();
}
