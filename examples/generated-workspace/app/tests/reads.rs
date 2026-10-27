//! Real generated page/filter/include proofs with statement counts from the native seam.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::{
    contracts::{
        profile::CreateProfile, project::CreateProject, public::ProfileKey, task::CreateTask,
    },
    stores::{
        profile::{Filter as ProfileFilter, Profiles, Query as ProfileQuery, SortField},
        project::{Filter as ProjectFilter, Projects, Query as ProjectQuery},
        task::Tasks,
    },
};
use stackpress_framework_schema::{NullOrder, Predicate, Sort, SortDirection, ValueContext};
use stackpress_framework_sql::{
    DataErrorKind, DataSession,
    stores::{Loaded, RelationLimits, Visibility},
};
use stackpress_sql::{QueryLimits, Render, SqlValue};
use std::time::Duration;
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Parent supervision includes the native worker and transaction-close paths.
#[test]
fn generated_reads_and_relations() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_reads_and_relations",
        "reads",
        Duration::from_secs(20),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Populate independent parents, two projects per parent and two tasks per project.
async fn seed(engine: &stackpress_sql::Engine) {
    let mut transaction = engine.begin().await.unwrap();
    let mut db = DataSession::transaction(&mut transaction);
    for owner in 0..20 {
        let id = format!("u{owner:02}");
        let input: CreateProfile = serde_json::from_value(
            serde_json::json!({"id":id,"name":if owner<2{"Duplicate"}else{"Owner"}}),
        )
        .unwrap();
        Profiles::create(&mut db, input).await.unwrap();
        for project in 0..2 {
            let id = format!("p{owner:02}-{project}");
            let input:CreateProject=serde_json::from_value(serde_json::json!({"id":id,"ownerId":format!("u{owner:02}"),"title":if owner==0{"Literal%_"}else{"literal"},"tags":["a","b"]})).unwrap();
            Projects::create(&mut db, input).await.unwrap();
            for task in 0..2 {
                let input:CreateTask=serde_json::from_value(serde_json::json!({"id":format!("t{owner:02}-{project}-{task}"),"projectId":format!("p{owner:02}-{project}"),"title":"Task"})).unwrap();
                Tasks::create(&mut db, input).await.unwrap();
            }
        }
    }
    transaction.commit().await.unwrap();
}
/// Measure the same path at one and twenty parents; growth occurs only at documented bind chunks.
async fn prove() {
    let mut fixture = support::fixture().await;
    seed(&fixture.engine).await;
    for take in [1, 20] {
        fixture.calls.lock().unwrap().clear();
        let page = Profiles::search_atomic(
            &fixture.engine,
            ProfileQuery {
                take,
                total: true,
                includes: vec!["projects.tasks".into()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page.total, Some(20));
        assert_eq!(page.rows.len(), usize::from(take));
        assert_eq!(
            fixture.calls.lock().unwrap().len(),
            4,
            "base/count/projects/tasks must each batch all parents"
        );
        for (index, profile) in page.rows.iter().enumerate() {
            assert_eq!(profile.row.id, format!("u{index:02}"));
            let Loaded::Loaded(projects) = &profile.projects else {
                panic!("requested projects were unloaded")
            };
            assert_eq!(projects.len(), 2);
            for project in projects {
                let Loaded::Loaded(tasks) = &project.tasks else {
                    panic!("requested tasks were unloaded")
                };
                assert_eq!(tasks.len(), 2);
                assert!(matches!(project.owner, Loaded::Unloaded));
            }
        }
        let projected = page
            .rows
            .into_iter()
            .map(|row| row.project().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(projected.len(), usize::from(take));
    }
    // Literal q is case sensitive and treats percent/underscore as ordinary characters.
    let mut pooled = DataSession::pooled(&fixture.engine);
    let page = Projects::search(
        &mut pooled,
        ProjectQuery {
            q: Some("Literal%_".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.rows.len(), 2);
    assert!(
        Projects::search(
            &mut pooled,
            ProjectQuery {
                q: Some("literal%_".into()),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .rows
        .is_empty()
    );
    let page = Projects::search(
        &mut pooled,
        ProjectQuery {
            filters: vec![ProjectFilter::Tags(Predicate::Has("a".into()))],
            skip: 3,
            take: 4,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.rows.len(), 4);
    assert_eq!(page.rows[0].row.id, "p01-1");
    assert_eq!(
        Projects::count(
            &mut pooled,
            ProjectQuery {
                filters: vec![ProjectFilter::Tags(Predicate::Hasnt("missing".into()))],
                take: 1,
                skip: 100,
                ..Default::default()
            }
        )
        .await
        .unwrap(),
        40
    );
    assert!(
        Projects::search(
            &mut pooled,
            ProjectQuery {
                filters: vec![ProjectFilter::Tags(Predicate::Eq("a".into()))],
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    let page = Profiles::search(
        &mut pooled,
        ProfileQuery {
            sort: vec![Sort {
                field: SortField::Name,
                direction: SortDirection::Asc,
                nulls: NullOrder::Last,
            }],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.rows[0].row.id, "u00");
    assert_eq!(page.rows[1].row.id, "u01");
    // Pure plans retain predicates as binds, with a complete stable identity tie breaker.
    let plan = generated_workspace_app::generated::stores::profile::ProfileStore::search_plan(
        &ProfileQuery {
            filters: vec![ProfileFilter::Name(Predicate::Like("%plicate".into()))],
            ..Default::default()
        },
        fixture.engine.dialect(),
    )
    .unwrap()
    .render(fixture.engine.dialect())
    .unwrap();
    assert_eq!(
        plan.binds(),
        &[SqlValue::Bool(true), "%plicate".into(), "\\".into()]
    );
    assert!(plan.sql().contains("ORDER BY \"id\" ASC"));
    assert_eq!(
        Profiles::find_all(
            &mut pooled,
            ProfileQuery {
                filters: vec![ProfileFilter::Name(Predicate::Like("%plicate".into()))],
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .len(),
        2
    );
    assert!(
        Profiles::find(
            &mut pooled,
            ProfileQuery {
                filters: vec![ProfileFilter::Name(Predicate::Eq("absent".into()))],
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .is_none()
    );
    // Count/includes require a caller-owned snapshot, and bad paths fail before any native call.
    fixture.calls.lock().unwrap().clear();
    assert_eq!(
        Profiles::search(
            &mut pooled,
            ProfileQuery {
                total: true,
                ..Default::default()
            }
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Unsupported
    );
    assert!(
        Profiles::search(
            &mut pooled,
            ProfileQuery {
                includes: vec!["projects.nope".into()],
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert!(fixture.calls.lock().unwrap().is_empty());
    // Lower path/depth ceilings reject the entire include plan before acquisition/IO.
    for limits in [
        RelationLimits {
            paths: 0,
            ..Default::default()
        },
        RelationLimits {
            depth: 1,
            ..Default::default()
        },
    ] {
        assert!(
            Profiles::search_with_limits(
                &mut pooled,
                ProfileQuery {
                    includes: vec!["projects.tasks".into()],
                    ..Default::default()
                },
                Visibility::Active,
                limits
            )
            .await
            .is_err()
        );
        assert!(fixture.calls.lock().unwrap().is_empty());
    }
    // Ordinary reverse reads expose loaded empty independently of unloaded.
    let empty: CreateProfile =
        serde_json::from_value(serde_json::json!({"id":"empty","name":"Empty"})).unwrap();
    Profiles::create(&mut pooled, empty).await.unwrap();
    let page = Profiles::search_atomic(
        &fixture.engine,
        ProfileQuery {
            filters: vec![ProfileFilter::Id(Predicate::Eq("empty".into()))],
            includes: vec!["projects".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(&page.rows[0].projects,Loaded::Loaded(values) if values.is_empty()));
    // A required owner hidden by the active policy is an integrity error, never silently None.
    Profiles::remove(
        &mut pooled,
        &ProfileKey { id: "u00".into() },
        &ValueContext::default(),
    )
    .await
    .unwrap();
    assert!(Profiles::get(&mut pooled, "u00").await.unwrap().is_none());
    assert!(
        !Profiles::get_maintenance(
            &mut pooled,
            &ProfileKey { id: "u00".into() },
            Visibility::Inactive
        )
        .await
        .unwrap()
        .unwrap()
        .active
    );
    assert_eq!(
        Projects::search_atomic(
            &fixture.engine,
            ProjectQuery {
                filters: vec![ProjectFilter::Id(Predicate::Eq("p00-0".into()))],
                includes: vec!["owner".into()],
                ..Default::default()
            }
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Decode
    );
    Profiles::restore(
        &mut pooled,
        &ProfileKey { id: "u00".into() },
        &ValueContext::default(),
    )
    .await
    .unwrap();
    // A real WAL writer commits between two reads; this exact transaction retains one snapshot.
    let mut snapshot = fixture.engine.begin().await.unwrap();
    assert_eq!(
        Profiles::count(
            &mut DataSession::transaction(&mut snapshot),
            ProfileQuery::default()
        )
        .await
        .unwrap(),
        21
    );
    Profiles::create(
        &mut pooled,
        serde_json::from_value(serde_json::json!({"id":"late","name":"Late"})).unwrap(),
    )
    .await
    .unwrap();
    let page = Profiles::search(
        &mut DataSession::transaction(&mut snapshot),
        ProfileQuery {
            take: 100,
            total: true,
            includes: vec!["projects".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.total, Some(21));
    assert_eq!(page.rows.len(), 21);
    assert!(!page.rows.iter().any(|row| row.row.id == "late"));
    snapshot.rollback().await.unwrap();
    assert_eq!(
        Profiles::count(&mut pooled, ProfileQuery::default())
            .await
            .unwrap(),
        22
    );
    let page = Profiles::search_atomic(
        &fixture.engine,
        ProfileQuery {
            filters: vec![ProfileFilter::Id(Predicate::Eq("absent".into()))],
            total: true,
            includes: vec!["projects".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.total, Some(0));
    assert!(page.rows.is_empty());
    // Lowered aggregate budgets fail, retaining no partial collection.
    let mut tx = fixture.engine.begin().await.unwrap();
    for limits in [
        RelationLimits {
            rows: 0,
            ..Default::default()
        },
        RelationLimits {
            bytes: 1,
            ..Default::default()
        },
        RelationLimits {
            statements: 1,
            ..Default::default()
        },
        RelationLimits {
            keys: 0,
            ..Default::default()
        },
    ] {
        assert!(
            Profiles::search_with_limits(
                &mut DataSession::transaction(&mut tx),
                ProfileQuery {
                    includes: vec!["projects".into()],
                    ..Default::default()
                },
                Visibility::Active,
                limits
            )
            .await
            .is_err()
        );
    }
    tx.rollback().await.unwrap();
    // The exact executor's reduced bind ceiling controls chunks, reserving active=true's bind.
    fixture.engine = fixture.engine.with_limits(QueryLimits {
        binds: 7,
        ..Default::default()
    });
    fixture.calls.lock().unwrap().clear();
    let page = Projects::search_atomic(
        &fixture.engine,
        ProjectQuery {
            take: 20,
            includes: vec!["owner".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.rows.len(), 20);
    let calls = fixture.calls.lock().unwrap().clone();
    assert_eq!(
        calls.len(),
        3,
        "one base plus ten distinct owners in two six-key chunks"
    );
    assert!(calls.iter().all(|(_, binds)| *binds <= 7));
    for row in page.rows {
        let Loaded::Loaded(owner) = row.owner else {
            panic!("owner unloaded")
        };
        assert_eq!(owner.row.id, row.row.owner_id);
    }
    fixture.engine = fixture.engine.with_limits(QueryLimits {
        binds: 1,
        ..Default::default()
    });
    fixture.calls.lock().unwrap().clear();
    assert!(
        Projects::search_atomic(
            &fixture.engine,
            ProjectQuery {
                includes: vec!["owner".into()],
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        fixture.calls.lock().unwrap().len(),
        1,
        "zero key capacity after fixed active predicate must fail before relation IO"
    );
    fixture.engine.close().await.unwrap();
}
