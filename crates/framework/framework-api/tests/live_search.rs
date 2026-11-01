//! Opt-in G46 searches through the real Application/Session stores and their source DDL.
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
    let profile = ProfileActions::new(engine.clone(), "seed");
    let app = ApplicationActions::new(engine.clone(), "seed");
    let session = SessionActions::new(engine, "seed");
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
    Ok(())
}
fn main() {
    support::main(scenario);
}
