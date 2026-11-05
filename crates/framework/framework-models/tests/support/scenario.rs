//! Same generated four-model scenario on SQLite and opt-in owned network databases.
use serde_json::json;
use stackpress_framework_models::generated::stores::{
    application::ApplicationModel, auth::AuthModel, profile::ProfileModel, session::SessionModel,
};
use stackpress_framework_schema::source::{Record, date};
use stackpress_framework_sql::source::{BatchMode, SourceModel, StoreQuery};
use stackpress_sql::{DialectId, Engine, Field, Render, Select, SqlType};
use std::sync::Arc;
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn record(value: serde_json::Value) -> Record {
    serde_json::from_value(value).unwrap()
}
/// Real generated stores supply every DDL, codec and action; fixture input is only data.
pub async fn scenario(engine: Arc<Engine>) -> TestResult<()> {
    let profile = ProfileModel::actions(engine.clone(), "database-seed".into(), 0);
    let auth = AuthModel::actions(engine.clone(), "database-seed".into(), 0);
    let app = ApplicationModel::actions(engine.clone(), "database-seed".into(), 0);
    let session = SessionModel::actions(engine.clone(), "database-seed".into(), 0);
    println!("stage: generated DDL setup partial-state control");
    let plan = profile.store.create().render(engine.dialect())?;
    let mut queries = plan.queries().to_vec();
    let prefix = queries.len();
    queries.push(queries[0].clone());
    queries.push(app.store.create().render(engine.dialect())?.queries()[0].clone());
    let failure = stackpress_framework_sql::setup::execute(&engine, &queries)
        .await
        .unwrap_err();
    println!("generated setup first failure: {failure}");
    if let Some(cause) = std::error::Error::source(&failure) {
        println!("generated setup native cause (verbatim): {cause}");
    }
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("application"))
            .await
            .is_err()
    );
    if engine.dialect() == DialectId::Mysql {
        let progress = failure.plan_progress().unwrap();
        assert_eq!(progress.acknowledged, prefix);
        assert_eq!(progress.failed_index, prefix);
        assert_eq!(progress.total, prefix + 2);
        println!(
            "generated setup progress: acknowledged={} failed_index={} total={}",
            progress.acknowledged, progress.failed_index, progress.total
        );
        assert!(
            engine
                .fetch(&Select::new(["id"]).from("profile"))
                .await
                .is_ok()
        );
        profile.uninstall().await?;
        let atomic = engine.execute_plan(&plan).await.unwrap_err();
        assert_eq!(atomic.kind(), stackpress_sql::ErrorKind::Unsupported);
        println!("unchanged atomic MySQL refusal: {atomic}");
    } else {
        assert!(failure.plan_progress().is_none());
        assert!(
            engine
                .fetch(&Select::new(["id"]).from("profile"))
                .await
                .is_err()
        );
    }
    println!("stage: generated four-model install");
    profile.install().await?;
    // The actual action boundary keeps the typed SQL failure/progress as its cause.
    let duplicate = profile.install().await.unwrap_err();
    println!("generated install action failure: {duplicate}");
    let cause = std::error::Error::source(&duplicate)
        .unwrap()
        .downcast_ref::<stackpress_sql::SqlError>()
        .unwrap();
    if let Some(native) = std::error::Error::source(cause) {
        println!("generated install native cause (verbatim): {native}");
    }
    if engine.dialect() == DialectId::Mysql {
        let progress = cause.plan_progress().unwrap();
        assert_eq!(progress.acknowledged, 0);
        assert_eq!(progress.failed_index, 0);
    } else {
        assert!(cause.plan_progress().is_none());
    }
    auth.install().await?;
    app.install().await?;
    session.install().await?;
    for (id, name) in [("p1", "Ada"), ("p2", "Bert"), ("p3", "ADAM")] {
        let tags = if id == "p3" {
            json!(["red", "blue"])
        } else {
            json!(["red"])
        };
        profile
            .create(&record(json!({"id":id,"name":name,"tags":tags})))
            .await?;
    }
    println!("stage: G53 generated multi-value has requires all values");
    let all = StoreQuery {
        has: record(json!({"tags":["red","blue"]})),
        ..Default::default()
    };
    assert_eq!(profile.find_all(&all).await?[0]["id"], "p3");
    assert_eq!(profile.count(&all).await?, 1);
    let missing = StoreQuery {
        has: record(json!({"tags":["red","missing"]})),
        ..Default::default()
    };
    assert_eq!(profile.count(&missing).await?, 0);
    let mut query = StoreQuery {
        q: Some("ad".into()),
        sort: record(json!({"id":"ASC"})),
        ..Default::default()
    };
    let rows = profile.find_all(&query).await?;
    assert_eq!(
        rows.iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["p1", "p3"]
    );
    assert_eq!(profile.count(&query).await?, 2);
    query.q = None;
    query.like = record(json!({"name":"aD"}));
    assert_eq!(profile.find_all(&query).await?, rows);
    for (id, parent) in [("Alpha", "p1"), ("Bravo", "p2")] {
        auth.create(&record(
            json!({"id":id,"profileId":parent,"token":id,"secret":"hello"}),
        ))
        .await?;
    }
    query.like = record(json!({"profile.name":"ADA"}));
    query.columns = vec!["*".into(), "profile.*".into()];
    let rows = auth.find_all(&query).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "Alpha");
    assert_eq!(rows[0]["token"], "Alpha");
    assert_eq!(rows[0]["profile"]["name"], "Ada");
    assert_eq!(
        auth.find(&StoreQuery::eq("token", "Alpha")).await?.unwrap()["id"],
        "Alpha"
    );
    let error = auth
        .create(&record(
            json!({"profileId":"p1","token":"Alpha","secret":"hello"}),
        ))
        .await
        .unwrap_err();
    assert_eq!(error.code, 400);
    assert_eq!(error.errors["token"], "Already exists");
    let mut input = record(
        json!({"id":"a","profileId":"p1","name":"Example","secret":"plaintext","scopes":["read","write"]}),
    );
    for key in ["expires", "created", "updated"] {
        input.insert(key.into(), date(0));
    }
    app.create(&input).await?;
    input.insert("id".into(), "s".into());
    input.insert("applicationId".into(), "a".into());
    session.create(&input).await?;
    let query = StoreQuery {
        q: Some("eXaMpLe".into()),
        ..Default::default()
    };
    assert_eq!(app.find_all(&query).await?[0]["id"], "a");
    assert_eq!(app.count(&query).await?, 1);
    let query = StoreQuery {
        like: record(json!({"name":"eXaMpLe"})),
        ..Default::default()
    };
    assert_eq!(app.find_all(&query).await?[0]["id"], "a");
    let query = StoreQuery {
        like: record(json!({"id":"S"})),
        ..Default::default()
    };
    assert_eq!(session.find_all(&query).await?[0]["id"], "s");
    let query = StoreQuery {
        q: Some("ignored".into()),
        columns: vec!["application.profile.name".into(), "application.name".into()],
        ..Default::default()
    };
    let rows = session.find_all(&query).await?;
    assert_eq!(rows[0]["application"]["profile"]["name"], "Ada");
    assert_eq!(rows[0]["application"]["name"], "Example");
    // Source timestamp mutates input; the returned row is assembled before the storage update.
    let before = profile.find(&StoreQuery::eq("id", "p2")).await?.unwrap();
    let mut patch = record(json!({"name":"Bob"}));
    let changed = profile
        .update(&StoreQuery::eq("id", "p2"), &mut patch)
        .await?;
    assert_eq!(changed[0]["updated"], before["updated"]);
    assert!(patch.contains_key("updated"));
    let mut absent = record(json!({"id":"missing","name":"Never inserted"}));
    assert!(profile.upsert(&mut absent).await?.is_none());
    let statuses = profile
        .batch(
            &mut [
                record(json!({"id":"batch","name":"Rollback"})),
                record(json!({"id":"bad","name":""})),
                record(json!({"id":"after","name":"After"})),
            ],
            BatchMode::Create,
        )
        .await;
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].code, Some(200));
    assert_eq!(statuses[1].code, Some(400));
    assert!(
        profile
            .find(&StoreQuery::eq("id", "batch"))
            .await?
            .is_none()
    );
    let removed = profile.remove(&StoreQuery::eq("id", "p2")).await?;
    assert_eq!(removed[0]["active"], false);
    assert!(profile.find(&StoreQuery::eq("id", "p2")).await?.is_none());
    assert_eq!(
        profile
            .restore(&StoreQuery::eq("id", "p2").all())
            .await?
            .len(),
        1
    );
    // Adjacent generated definition uses G48 setup; DDL failure reporting remains native.
    let mut target = session.store.create().definition()?;
    target.fields.push(Field::new("m2_note", SqlType::Integer));
    session.upgrade(&target.create()).await?;
    engine
        .fetch(&Select::new(["m2_note"]).from("session"))
        .await?;
    // Source search spelling is dialect-aware for every generated model.
    for dialect in [DialectId::Postgres, DialectId::Mysql, DialectId::Sqlite] {
        let query = StoreQuery {
            like: record(json!({"id":"P"})),
            ..Default::default()
        };
        let sql = profile.store.select(&query, dialect)?.render(dialect)?;
        assert_eq!(
            sql.sql().contains(" ILIKE "),
            dialect == DialectId::Postgres
        );
    }
    println!("stage: generated FK purge");
    profile.purge(true).await?;
    assert_eq!(profile.count(&StoreQuery::default()).await?, 0);
    assert_eq!(auth.count(&StoreQuery::default()).await?, 0);
    assert_eq!(app.count(&StoreQuery::default()).await?, 0);
    assert_eq!(session.count(&StoreQuery::default()).await?, 0);
    session.uninstall().await?;
    app.uninstall().await?;
    auth.uninstall().await?;
    profile.uninstall().await?;
    println!("stage: generated four-model scenario passed");
    Ok(())
}
