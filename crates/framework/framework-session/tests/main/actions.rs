//! Actual SQLite source Store/Actions behavior, including rollback and sequential partial effects.
use serde_json::{Value as JsonValue, json};
use stackpress_framework_session::columns::Value;
use stackpress_framework_session::{
    columns::{self, Record},
    storage::{
        ActionOptions, AuthActions, AuthStore, BatchMode, PasswordConfig, ProfileActions,
        ProfileStore, StoreQuery,
    },
};
use stackpress_sql::{DialectId, Engine, Field, Render, Sql, SqlType};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::Arc;

/// Each test owns one real file database and closes its engine before removing the directory.
async fn database() -> (tempfile::TempDir, Arc<Engine>, ProfileActions, AuthActions) {
    let dir = tempfile::tempdir().unwrap();
    let db = SqliteDatabase::open(
        SqliteOptions::file(dir.path().join("accounts.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(db));
    let profile = ProfileActions::new(engine.clone(), "database-seed");
    let auth = AuthActions::new(engine.clone(), "database-seed");
    profile.install().await.unwrap();
    auth.install().await.unwrap();
    (dir, engine, profile, auth)
}
/// Compact source object construction, preserving omitted keys and explicit null.
fn record(value: JsonValue) -> Record {
    serde_json::from_value(value).unwrap()
}

/// Only equality's literal -1 bypasses active filtering; other operators keep their codec.
#[tokio::test]
async fn active_override_keeps_non_equality_predicates() {
    let (_dir, engine, profile, _) = database().await;
    for (id, active) in [("enabled", true), ("disabled", false)] {
        profile
            .create(&record(json!({"id":id,"name":id,"active":active})))
            .await
            .unwrap();
    }
    // Missing eq.active supplies active=true; equality's -1 removes that default.
    assert_eq!(profile.count(&StoreQuery::default()).await.unwrap(), 1);
    let mut query = StoreQuery::default().all();
    assert_eq!(profile.count(&query).await.unwrap(), 2);
    // ne.active=-1 still means active != true after source Boolean serialization.
    query.ne.insert("active".into(), (-1).into());
    let rows = profile.find_all(&query).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "disabled");
    query.eq.clear();
    assert_eq!(profile.count(&query).await.unwrap(), 0);
    engine.close().await.unwrap();
}

#[tokio::test]
async fn profile_crud_projection_active_and_original_update_return() {
    let (_dir, engine, profile, _) = database().await;
    let mut input = record(
        json!({"id":"p-1","name":"Ada","roles":["USER"],"tags":["tag"],"references":{"fbid":"abc"},"created":0,"updated":0,"unknown":true}),
    );
    input.insert("created".into(), columns::date(0));
    input.insert("updated".into(), columns::date(0));
    let created = profile.create(&input).await.unwrap();
    assert_eq!(created["name"], "Ada");
    assert_eq!(created["image"], Value::Null);
    let mut query = StoreQuery::eq("id", "p-1");
    let before = profile.find(&query).await.unwrap().unwrap();
    let mut patch = record(json!({"name":"","references":null}));
    let returned = profile.update(&query, &mut patch).await.unwrap();
    assert_eq!(returned[0]["name"], "Ada"); // Empty text was removed from the merged return map.
    assert_eq!(returned[0]["roles"], json!([])); // Omitted lists still decode to [].
    assert_eq!(returned[0]["updated"], before["updated"]);
    assert!(columns::as_date(&patch["updated"]).unwrap() > 0); // Store mutates the original input.
    let persisted = profile.find(&query).await.unwrap().unwrap();
    assert_eq!(persisted["name"], ""); // Original input, not sanitized return map, is written.
    assert_eq!(persisted["roles"], json!(["USER"]));
    assert_eq!(persisted["references"], Value::Null);
    assert_ne!(persisted["updated"], before["updated"]);
    profile.remove(&query).await.unwrap();
    assert!(profile.find(&query).await.unwrap().is_none());
    assert!(profile.restore(&query).await.unwrap().is_empty());
    query = query.all();
    profile.restore(&query).await.unwrap();
    let projection = StoreQuery {
        columns: vec!["name".into(), "references:fbid".into()],
        ..query.clone()
    };
    let selected = profile.find(&projection).await.unwrap().unwrap();
    assert!(!selected.contains_key("id"));
    assert_eq!(selected["references"]["fbid"], Value::Null); // Whole nullable object column is nested.
    assert_eq!(profile.delete(&query).await.unwrap().len(), 1);
    assert_eq!(
        profile.count(&StoreQuery::default().all()).await.unwrap(),
        0
    );
    engine.close().await.unwrap();
}
#[tokio::test]
async fn source_scalar_goldens_and_auth_returning_branch() {
    let (_dir, engine, profile, auth) = database().await;
    profile
        .create(&record(json!({"id":"profile-1","name":"Ada"})))
        .await
        .unwrap();
    let fixture: JsonValue = serde_json::from_str(include_str!("../records.json")).unwrap();
    let mut input = record(
        json!({"id":"record-1","name":"Ada","profileId":"profile-1","token":"user@example.test","secret":"hello","type":"email","roles":["USER"],"tags":["tag"],"references":{"fbid":"abc"},"verified":true,"active":true}),
    );
    for name in ["created", "updated", "consumed"] {
        input.insert(name.into(), columns::date(1700000000123));
    }
    let sqlite = |values: Vec<(String, stackpress_sql::SqlValue)>| {
        values
            .into_iter()
            .map(|(name, value)| {
                (
                    name,
                    match value {
                        stackpress_sql::SqlValue::Text(value) => Value::String(value),
                        stackpress_sql::SqlValue::Bool(value) => Value::Bool(value),
                        _ => panic!("unexpected scalar"),
                    },
                )
            })
            .collect::<Record>()
    };
    assert_eq!(
        Value::Object(sqlite(
            ProfileStore::new("database-seed")
                .scalarize(&input)
                .unwrap()
        )),
        fixture["scalar"]["Profile"]
    );
    assert_eq!(
        Value::Object(sqlite(
            AuthStore::new("database-seed").scalarize(&input).unwrap()
        )),
        fixture["scalar"]["Auth"]
    );
    let returned = auth.create(&input).await.unwrap();
    assert_eq!(returned["token"], fixture["authReturned"]["token"]);
    assert_eq!(returned["secret"], fixture["authReturned"]["secret"]);
    assert!(!returned.contains_key("profileId"));
    let read = auth
        .find(&StoreQuery::eq("id", "record-1"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read["token"], "user@example.test");
    assert_eq!(read["profileId"], "profile-1");
    assert_eq!(read["secret"], fixture["scalar"]["Auth"]["secret"]);
    let raw = engine
        .fetch(&Sql::literal(
            DialectId::Sqlite,
            "SELECT token,secret FROM auth",
        ))
        .await
        .unwrap();
    assert_eq!(
        raw[0].try_get::<String>("token").unwrap(),
        fixture["scalar"]["Auth"]["token"]
    );
    engine.close().await.unwrap();
}
#[tokio::test]
async fn token_global_uniqueness_preflight_sql_constraint_and_inactive() {
    let (_dir, engine, profile, auth) = database().await;
    profile
        .create(&record(json!({"id":"p","name":"Ada"})))
        .await
        .unwrap();
    let input = record(
        json!({"id":"a","profileId":"p","type":"email","token":"same-token","secret":"hello"}),
    );
    auth.create(&input).await.unwrap();
    let mut duplicate = input.clone();
    duplicate.insert("id".into(), "b".into());
    duplicate.insert("type".into(), "phone".into());
    let error = auth.create(&duplicate).await.unwrap_err();
    assert_eq!(error.code, 400);
    assert_eq!(error.errors["token"], "Already exists");
    // Bypassing the action preflight still reaches the real global unique constraint.
    let mut populated = auth.store.populate(&duplicate).unwrap();
    assert!(
        engine
            .execute(&auth.store.insert(&populated).unwrap())
            .await
            .is_err()
    );
    auth.remove(&StoreQuery::eq("id", "a")).await.unwrap();
    let error = auth.create(&duplicate).await.unwrap_err();
    assert_eq!(error.code, 500);
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(auth.count(&StoreQuery::default().all()).await.unwrap(), 1);
    // Inactive auth cannot sign in; no further joined-profile active guard is introduced.
    assert_eq!(
        auth.signin(
            "email",
            &record(json!({"email":"same-token","secret":"hello"})),
            true
        )
        .await
        .unwrap_err()
        .code,
        404
    );
    populated.insert("token".into(), "different-token".into());
    assert!(auth.create(&populated).await.is_ok());
    engine.close().await.unwrap();
}
#[tokio::test]
async fn signin_flags_consumed_reload_and_inactive_profile_join() {
    let (_dir, engine, profile, auth) = database().await;
    profile
        .create(&record(json!({"id":"p","name":"Ada","active":false})))
        .await
        .unwrap();
    let input = record(
        json!({"id":"a","profileId":"p","type":"email","token":"ada@example.test","secret":"hello","consumed":0}),
    );
    auth.create(&input).await.unwrap();
    let error = auth
        .signin(
            "email",
            &record(json!({"email":"ada@example.test","secret":"bad"})),
            true,
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, 401);
    assert_eq!(error.error, "Invalid Parameters");
    assert_eq!(error.errors["secret"], "Invalid Password");
    let signed = auth
        .signin("email", &record(json!({"email":"ada@example.test"})), false)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(signed["profile"]["active"], false);
    assert!(columns::as_date(&signed["consumed"]).unwrap() > 0);
    assert_eq!(
        auth.signin("username", &Record::new(), false)
            .await
            .unwrap_err()
            .errors["username"],
        "User Not Found"
    );
    assert!(
        auth.signin(
            "email",
            &record(json!({"email":"ada@example.test","secret":"hello"})),
            true
        )
        .await
        .unwrap()
        .is_some()
    );
    engine.close().await.unwrap();
}
#[tokio::test]
async fn source_batch_rolls_back_item_errors_but_retains_status_rows() {
    let (_dir, engine, profile, auth) = database().await;
    let mut inputs = vec![
        record(json!({"id":"p1","name":"first"})),
        record(json!({"id":"bad"})),
        record(json!({"id":"p2","name":"second"})),
    ];
    let results = profile.batch(&mut inputs, BatchMode::Create).await;
    assert_eq!(
        results.iter().map(|result| result.code).collect::<Vec<_>>(),
        vec![Some(200), Some(400), Some(200)]
    );
    assert_eq!(results[0].results.as_ref().unwrap()["id"], "p1");
    assert_eq!(
        profile.count(&StoreQuery::default().all()).await.unwrap(),
        0
    );
    let mut inputs = vec![record(json!({"id":"p","name":"Ada"}))];
    profile.batch(&mut inputs, BatchMode::Create).await;
    let mut updates = vec![
        record(json!({"id":"p","name":"changed"})),
        record(json!({"name":"no selector"})),
    ];
    let results = profile.batch(&mut updates, BatchMode::Update).await;
    assert_eq!(results[1].code, None);
    assert!(results[1].error.as_ref().unwrap().contains("ID or unique"));
    assert_eq!(
        profile
            .find(&StoreQuery::eq("id", "p"))
            .await
            .unwrap()
            .unwrap()["name"],
        "changed"
    );
    let mut auths = vec![record(
        json!({"profileId":"p","token":"identifier","secret":"hello"}),
    )];
    assert_eq!(
        auth.batch(&mut auths, BatchMode::Create).await[0].code,
        Some(200)
    );
    let mut auths = vec![record(json!({"token":"identifier","verified":true}))];
    assert_eq!(
        auth.batch(&mut auths, BatchMode::Update).await[0].code,
        Some(200)
    );
    let mut auths = vec![record(json!({"token":"identifier","verified":false}))];
    assert_eq!(
        auth.batch(&mut auths, BatchMode::Upsert).await[0].code,
        Some(200)
    );
    engine.close().await.unwrap();
}
#[tokio::test]
async fn signup_optional_phone_and_sequential_partial_side_effects() {
    let (_dir, engine, profile, auth) = database().await;
    let result = auth
        .signup(&record(
            json!({"name":"Ada","email":"ada@example.test","secret":"hello"}),
        ))
        .await
        .unwrap();
    assert_eq!(result["roles"], json!([]));
    assert!(result["auth"].get("phone").is_none());
    let error=auth.signup(&record(json!({"name":"Other","email":"new@example.test","username":"ada@example.test","secret":"hello"}))).await.unwrap_err();
    assert_eq!(error.code, 400);
    assert_eq!(error.errors["username"], "Already exists");
    assert_eq!(error.errors["token"], "Already exists");
    assert_eq!(profile.count(&StoreQuery::default()).await.unwrap(), 2);
    assert_eq!(auth.count(&StoreQuery::default()).await.unwrap(), 2); // New email remains after username failure.
    let result=auth.signup(&record(json!({"name":"Phone","phone":"0012345","username":"phone-user","roles":["ADMIN"],"secret":"hello"}))).await.unwrap();
    assert_eq!(result["roles"], json!(["ADMIN"]));
    assert!(result["auth"]["phone"].is_object());
    engine.close().await.unwrap();
}
#[tokio::test]
async fn predicates_sort_projection_join_and_maintenance() {
    let (_dir, engine, profile, auth) = database().await;
    for (id, name, tags) in [
        ("p1", "Ada", json!(["red"])),
        ("p2", "Bert", json!(["blue"])),
        ("p3", "ADAM", json!(["red", "green"])),
    ] {
        profile
            .create(&record(json!({"id":id,"name":name,"tags":tags})))
            .await
            .unwrap();
    }
    let mut query = StoreQuery {
        q: Some("ad".into()),
        sort: record(json!({"name":"DESC","id":"invalid"})),
        take: 1,
        ..Default::default()
    };
    assert!(profile.find(&query).await.is_err());
    assert!(profile.count(&query).await.is_err()); // Literal source ILIKE is rejected by SQLite.
    query = StoreQuery::default();
    query.has = record(json!({"tags":"red"}));
    let found = profile.find_all(&query).await.unwrap();
    assert_eq!(found.len(), 2);
    query.has.clear();
    query.hasnt = record(json!({"tags":"red"}));
    assert_eq!(profile.find_all(&query).await.unwrap(), found);
    query = StoreQuery::default();
    query.eq = record(json!({"id":["p1","p2"]}));
    query.ne = record(json!({"name":"Bert"}));
    query.ge = record(json!({"name":"A"}));
    query.le = record(json!({"name":"Z"}));
    query.like = record(json!({"name":"d"}));
    assert!(profile.find_all(&query).await.is_err());
    query.like.clear();
    assert_eq!(profile.find_all(&query).await.unwrap().len(), 1);
    auth.create(&record(
        json!({"id":"a1","profileId":"p1","token":"identifier","secret":"hello"}),
    ))
    .await
    .unwrap();
    query = StoreQuery::default();
    query.eq = record(json!({"profile":{"name":"Ada"}}));
    query.columns = vec!["id".into(), "profile.name".into()];
    assert_eq!(
        auth.find(&query).await.unwrap().unwrap(),
        record(json!({"id":"a1","profile":{"name":"Ada"}}))
    );
    assert_eq!(auth.count(&query).await.unwrap(), 1);
    assert!(ProfileStore::new("").paths("auth.*").is_empty());
    assert_eq!(
        auth.store
            .selectors(&["*".into(), "id".into(), "profile.*".into()])
            .len(),
        20
    );
    let target = profile
        .store
        .create()
        .add_field(Field::new("note", SqlType::Text));
    assert!(
        !profile
            .store
            .alter(Some(&target))
            .unwrap()
            .build()
            .unwrap()
            .operations
            .is_empty()
    );
    profile.upgrade(&target).await.unwrap();
    engine
        .execute(
            &Sql::literal(
                DialectId::Sqlite,
                "UPDATE profile SET note = ? WHERE id = ?",
            )
            .bind("upgraded")
            .bind("p1"),
        )
        .await
        .unwrap();
    let mut membership = StoreQuery {
        has: record(json!({"tags":["blue","green"]})),
        ..Default::default()
    };
    assert_eq!(profile.find_all(&membership).await.unwrap().len(), 2);
    membership.has = record(json!({"unknown":"red"}));
    assert!(profile.find_all(&membership).await.is_err());
    let extra = record(json!({"id":"nulls","name":"Nulls","references":{"flags":[null,true,0]}}));
    profile.create(&extra).await.unwrap();
    membership.has = record(json!({"references:flags":null}));
    assert!(profile.find_all(&membership).await.unwrap().is_empty());
    membership.has = record(json!({"references:flags":true}));
    assert_eq!(profile.find_all(&membership).await.unwrap().len(), 1);
    // FK cascade is a schema contract, not an action-only callback.
    profile.delete(&StoreQuery::eq("id", "p1")).await.unwrap();
    assert_eq!(auth.count(&StoreQuery::default().all()).await.unwrap(), 0);
    profile.purge(false).await.unwrap();
    assert_eq!(
        profile.count(&StoreQuery::default().all()).await.unwrap(),
        0
    );
    auth.purge(true).await.unwrap();
    auth.uninstall().await.unwrap();
    profile.uninstall().await.unwrap();
    assert!(profile.find(&StoreQuery::default()).await.is_err());
    engine.close().await.unwrap();
}
#[tokio::test]
async fn password_precedence_upsert_and_validation() {
    let (_dir, engine, profile, _) = database().await;
    let auth = AuthActions::with_options(
        engine.clone(),
        ActionOptions {
            seed: "database-seed".into(),
            password: PasswordConfig {
                min: Some(4),
                max: Some(8),
                upper: true,
                lower: true,
                number: true,
                special: true,
            },
        },
    );
    let configured = AuthActions::make(engine.clone(), None, None);
    let direct = AuthActions::with_options(engine.clone(), ActionOptions::default());
    let token = record(json!({"token":"identifier"}));
    assert_ne!(
        configured.store.scalarize(&token).unwrap(),
        direct.store.scalarize(&token).unwrap()
    );
    let secret_error = |secret: &str| {
        auth.assert(&record(
            json!({"name":"A","username":"user-name","secret":secret}),
        ))
        .map(|errors| errors.get("secret").cloned())
    };
    for (secret, message) in [
        ("a", "at least"),
        ("123456789", "less than"),
        ("abcd", "uppercase"),
        ("ABCD", "lowercase"),
        ("Abcd", "number"),
        ("Abc1", "special"),
    ] {
        assert!(
            secret_error(secret)
                .unwrap()
                .unwrap()
                .as_str()
                .unwrap()
                .contains(message)
        );
    }
    assert!(secret_error("Ab1 ").is_none());
    assert!(
        auth.assert(&Record::new())
            .unwrap()
            .contains_key("username")
    );
    assert_eq!(
        auth.assert(&record(json!({"name":"A","email":"bad","secret":"Ab1 "})))
            .unwrap()["email"],
        "Invalid email"
    );
    assert!(
        profile
            .create(&Record::new())
            .await
            .unwrap_err()
            .errors
            .contains_key("name")
    );
    let mut supplied = record(json!({"id":"missing","name":"no insert"}));
    assert!(profile.upsert(&mut supplied).await.unwrap().is_none());
    let mut supplied = record(json!({"name":"insert"}));
    assert!(profile.upsert(&mut supplied).await.unwrap().is_some());
    assert!(auth.store.create().render(DialectId::Postgres).is_ok());
    assert!(auth.store.create().render(DialectId::Mysql).is_ok());
    let input = record(json!({"id":"a","profileId":"p","token":"identifier","secret":"hello"}));
    assert!(
        !auth
            .store
            .insert_for(&input, DialectId::Mysql)
            .unwrap()
            .render(DialectId::Mysql)
            .unwrap()
            .sql()
            .contains("RETURNING")
    );
    engine.close().await.unwrap();
}
/// Called only by the external deadlock watchdog: using the pool inside a transaction would
/// stall this batch on the transaction's SQLite write lock instead of finishing its rollback.
pub(super) async fn transaction_child() {
    let (_dir, engine, profile, _) = database().await;
    let mut inputs = vec![
        record(json!({"id":"p1","name":"first"})),
        record(json!({"id":"bad"})),
        record(json!({"id":"p2","name":"last"})),
    ];
    let statuses = profile.batch(&mut inputs, BatchMode::Create).await;
    assert_eq!(statuses.len(), 3);
    assert_eq!(profile.count(&StoreQuery::default()).await.unwrap(), 0);
    // Futures must remain Send while retaining the exact transaction borrow across await.
    fn send<T: Send>(_: T) {}
    let mut transaction = engine.begin().await.unwrap();
    let mut session = stackpress_framework_sql::DataSession::transaction(&mut transaction);
    let query = StoreQuery::default();
    let input = Record::new();
    send(profile.create_in(&mut session, &input));
    send(profile.find_in(&mut session, &query));
    transaction.rollback().await.unwrap();
    engine.close().await.unwrap();
}
