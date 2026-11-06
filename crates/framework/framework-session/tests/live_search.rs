//! Opt-in G46 searches through the real Profile/Auth stores and their source DDL.

#[path = "../../framework-sql/tests/support/g54.rs"]
mod g54;
#[path = "live/support.rs"]
mod support;

use serde_json::json;
use stackpress_framework_session::{
    columns::Record,
    storage::{AuthActions, ProfileActions, StoreQuery},
};
use stackpress_sql::Engine;
use std::sync::Arc;

fn record(value: serde_json::Value) -> Record {
    serde_json::from_value(value).unwrap()
}

/// Mirror the SQLite regression on actual installed tables; no fixture DDL replaces a store.
async fn scenario(engine: Arc<Engine>) -> support::TestResult<()> {
    let result = scenario_steps(engine).await;
    if let Err(error) = &result {
        // This fixture contains synthetic values only. Expose the trusted native chain
        // at the failing boundary without printing connection strings or changing policy.
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

/// Keep search and setup observations distinct when a live backend rejects an operation.
async fn scenario_steps(engine: Arc<Engine>) -> support::TestResult<()> {
    let profile = ProfileActions::new(
        engine.clone(),
        stackpress_framework_session::storage::ProfileStore::new("database-seed".into(), 0),
    );
    let auth = AuthActions::new(
        engine,
        stackpress_framework_session::storage::AuthStore::new("database-seed".into(), 0),
    );
    println!("stage: Profile install");
    profile.install().await?;
    println!("stage: Auth install");
    auth.install().await?;
    println!("stage: Profile create");
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
    println!("stage: G53 Profile multi-value has requires all values");
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
    println!("stage: Profile q/like search");
    let rows = profile.find_all(&query).await?;
    assert_eq!(
        rows.iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["p1", "p3"]
    );
    assert_eq!(profile.count(&query).await?, 2);
    query.q = None;
    query.like = record(json!({"name":"aD"}));
    assert_eq!(profile.find_all(&query).await?, rows);
    println!("stage: Auth create");
    for (id, profile_id) in [("Alpha", "p1"), ("Bravo", "p2")] {
        auth.create(&record(
            json!({"id":id,"profileId":profile_id,"token":id,"secret":"hello"}),
        ))
        .await?;
    }
    println!("stage: Auth-to-Profile like search");
    query.like = record(json!({"profile.name":"ADA"}));
    let rows = auth.find_all(&query).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"], "Alpha");
    query.like = record(json!({"id":"ALPHA"}));
    assert_eq!(auth.count(&query).await?, 1);
    // G48 keeps real store alteration and explicit teardown on the same native setup policy.
    let mut target = auth.store.create().definition()?;
    target.fields.push(stackpress_sql::Field::new(
        "g48_note",
        stackpress_sql::SqlType::Integer,
    ));
    println!("stage: Auth upgrade");
    auth.upgrade(&target.create()).await?;
    assert!(
        auth.engine
            .fetch(&stackpress_sql::Select::new(["g48_note"]).from("auth"))
            .await
            .is_ok()
    );
    println!("stage: Auth purge");
    auth.purge(true).await?;
    println!("stage: Auth uninstall");
    auth.uninstall().await?;
    let mut target = profile.store.create().definition()?;
    target.fields.push(stackpress_sql::Field::new(
        "g48_note",
        stackpress_sql::SqlType::Integer,
    ));
    println!("stage: Profile upgrade");
    profile.upgrade(&target.create()).await?;
    println!("stage: Profile purge");
    profile.purge(true).await?;
    println!("stage: Profile uninstall");
    profile.uninstall().await?;
    if profile.engine.dialect() == stackpress_sql::DialectId::Mysql {
        // Append boundary controls after the unchanged fixture teardown: missing
        // targets must fail, and application-owned dependents must never be purged.
        mysql_purge_boundaries(&profile).await?;
    }
    g54::prove(profile).await?;
    Ok(())
}

/// Retain and print the exact backend cause for an expected native refusal.
fn assert_mysql_error(error: &(dyn std::error::Error + 'static), code: &str) {
    let mut cause = Some(error);
    let mut found = false;
    while let Some(error) = cause {
        let message = error.to_string();
        assert!(!message.contains("postgres://") && !message.contains("mysql://"));
        found |= message.starts_with(code);
        println!("MySQL purge boundary (verbatim): {message}");
        cause = error.source();
    }
    assert!(found, "expected native MySQL code {code}");
}

/// G48 discovery follows installed FKs, without adopting outside-catalog tables.
async fn mysql_purge_boundaries(profile: &ProfileActions) -> support::TestResult<()> {
    use stackpress_sql::{Create, DialectId, ForeignKey, Select};

    // Metadata discovery returning no edges must not turn a missing target into success.
    let error = profile.purge(true).await.unwrap_err();
    assert_mysql_error(&error, "1146");

    profile.install().await?;
    profile
        .create(&record(json!({"id":"blocked","name":"Outside FK"})))
        .await?;
    // Reuse the real store's id definition so FK type/length match the installed
    // generated parent. Only this app-owned boundary table needs fixture DDL.
    let mut field = profile
        .store
        .create()
        .definition()?
        .fields
        .into_iter()
        .find(|field| field.name == "id")
        .unwrap();
    field.name = "parent_id".into();
    let outside = Create::new("g48_outside")
        .add_field(field)
        .add_foreign_key(ForeignKey::new(
            "g48_outside_parent",
            ["parent_id"],
            "profile",
            ["id"],
        ));
    stackpress_framework_sql::setup::execute_plan(
        &profile.engine,
        &outside.render(DialectId::Mysql)?,
    )
    .await?;
    profile
        .engine
        .execute(
            &profile
                .engine
                .insert("g48_outside")
                .columns(["parent_id"])
                .values(["blocked"]),
        )
        .await?;
    let error = profile.purge(true).await.unwrap_err();
    assert_mysql_error(&error, "1451");
    assert_eq!(profile.count(&StoreQuery::default()).await?, 1);
    assert_eq!(
        profile
            .engine
            .fetch(&Select::new(["parent_id"]).from("g48_outside"))
            .await?
            .len(),
        1
    );
    // Only the fixture owner removes its own outside table; purge itself never does.
    profile
        .engine
        .execute(&profile.engine.drop_table("g48_outside"))
        .await?;
    profile.purge(true).await?;
    assert_eq!(profile.count(&StoreQuery::default()).await?, 0);
    profile.uninstall().await?;
    Ok(())
}
fn main() {
    support::main(scenario);
}
