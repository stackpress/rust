//! Opt-in G46 searches through the real Profile/Auth stores and their source DDL.
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
    let profile = ProfileActions::new(engine.clone(), "database-seed");
    let auth = AuthActions::new(engine, "database-seed");
    println!("stage: Profile install");
    profile.install().await?;
    println!("stage: Auth install");
    auth.install().await?;
    println!("stage: Profile create");
    for (id, name) in [("p1", "Ada"), ("p2", "Bert"), ("p3", "ADAM")] {
        profile
            .create(&record(json!({"id":id,"name":name})))
            .await?;
    }
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
    Ok(())
}
fn main() {
    support::main(scenario);
}
