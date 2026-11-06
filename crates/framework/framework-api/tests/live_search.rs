//! Opt-in G46 searches through the real Application/Session stores and their source DDL.

#[path = "../../framework-sql/tests/support/g54.rs"]
mod g54;
#[path = "../../framework-session/tests/live/support.rs"]
mod support;

use serde_json::json;
use stackpress_framework_api::models::{
    ApplicationActions, Record, SessionActions, StoreQuery, date,
};
use stackpress_framework_session::storage::ProfileActions;
use stackpress_sql::Engine;
use std::sync::Arc;

fn record(value: serde_json::Value) -> Record {
    serde_json::from_value(value).unwrap()
}

/// Use the same synthetic values as SQLite, including API's unchanged plaintext secret codec.
async fn scenario(engine: Arc<Engine>) -> support::TestResult<()> {
    let result = scenario_steps(engine).await;
    if let Err(error) = &result {
        // Synthetic fixture diagnostics retain the native server message, never its URL.
        let mut cause = Some(error.as_ref() as &(dyn std::error::Error + 'static));
        while let Some(error) = cause {
            let message = error.to_string();
            assert!(!message.contains("postgres://") && !message.contains("mysql://"));
            eprintln!("live search native cause (verbatim): {message}");
            cause = error.source();
        }
    }
    result
}

/// Observe each unchanged search/setup step before diagnosing a backend boundary.
async fn scenario_steps(engine: Arc<Engine>) -> support::TestResult<()> {
    let profile = ProfileActions::new(
        engine.clone(),
        stackpress_framework_session::storage::ProfileStore::new("seed".into(), 0),
    );
    let app = ApplicationActions::new(
        engine.clone(),
        stackpress_framework_api::models::ApplicationStore::new("seed".into(), 0),
    );
    let session = SessionActions::new(
        engine,
        stackpress_framework_api::models::SessionStore::new("seed".into(), 0),
    );
    println!("stage: Profile/Application/Session install");
    profile.install().await?;
    app.install().await?;
    session.install().await?;
    println!("stage: Profile/Application/Session create");
    profile
        .create(&record(json!({"id":"p","name":"Ada","roles":["USER"]})))
        .await?;
    let mut input = record(
        json!({"id":"a","profileId":"p","name":"Example","secret":"plaintext","scopes":["read","write"]}),
    );
    for key in ["expires", "created", "updated"] {
        input.insert(key.into(), date(0));
    }
    app.create(&input).await?;
    input.insert("id".into(), "s".into());
    input.insert("applicationId".into(), "a".into());
    session.create(&input).await?;
    println!("stage: G53 Application/Session multi-value has requires all scopes");
    let all = StoreQuery {
        has: record(json!({"scopes":["read","write"]})),
        ..Default::default()
    };
    assert_eq!(app.count(&all).await?, 1);
    assert_eq!(session.count(&all).await?, 1);
    let missing = StoreQuery {
        has: record(json!({"scopes":["read","missing"]})),
        ..Default::default()
    };
    assert_eq!(app.count(&missing).await?, 0);
    assert_eq!(session.count(&missing).await?, 0);
    println!("stage: Application q/like search");
    let query = StoreQuery {
        q: Some("eXaMpLe".into()),
        ..Default::default()
    };
    assert_eq!(app.find_all(&query).await?[0]["id"], "a");
    assert_eq!(app.count(&query).await?, 1);
    let like = StoreQuery {
        like: record(json!({"name":"eXaMpLe"})),
        ..Default::default()
    };
    assert_eq!(app.find_all(&like).await?[0]["id"], "a");
    println!("stage: Session like search");
    let like = StoreQuery {
        like: record(json!({"id":"S"})),
        ..Default::default()
    };
    assert_eq!(session.find_all(&like).await?[0]["id"], "s");
    assert_eq!(session.count(&like).await?, 1);
    // Session declares no q search fields; a nonmatching q must remain a source no-op.
    let ignored_q = StoreQuery {
        q: Some("does-not-match".into()),
        ..Default::default()
    };
    assert_eq!(session.find_all(&ignored_q).await?[0]["id"], "s");
    assert_eq!(session.count(&ignored_q).await?, 1);
    // G48 changes only setup selection; each real source action still owns its own teardown.
    let mut target = session.store.create().definition()?;
    target.fields.push(stackpress_sql::Field::new(
        "g48_note",
        stackpress_sql::SqlType::Integer,
    ));
    println!("stage: Session upgrade");
    session.upgrade(&target.create()).await?;
    println!("stage: Session purge");
    session.purge(true).await?;
    println!("stage: Session uninstall");
    session.uninstall().await?;
    let mut target = app.store.create().definition()?;
    target.fields.push(stackpress_sql::Field::new(
        "g48_note",
        stackpress_sql::SqlType::Integer,
    ));
    println!("stage: Application upgrade");
    app.upgrade(&target.create()).await?;
    println!("stage: Application purge");
    app.purge(true).await?;
    println!("stage: Application uninstall");
    app.uninstall().await?;
    println!("stage: Profile purge");
    profile.purge(true).await?;
    println!("stage: Profile uninstall");
    profile.uninstall().await?;
    g54::prove(profile).await?;
    Ok(())
}
fn main() {
    support::main(scenario);
}
