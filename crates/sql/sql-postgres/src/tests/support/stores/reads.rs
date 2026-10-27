//! Common generated reads and explicit refusals; only SQLite promises multi-statement snapshots.
use super::*;
use stackpress_framework_schema::{NullOrder, Sort, SortDirection};
use stackpress_framework_sql::stores::{Loaded, Visibility};
use stackpress_sql::QueryLimits;

/// Read all three actual row codecs, deterministic paging, scalar/list filters and counts.
pub(super) async fn common(engine: &Engine) -> TestResult<()> {
    let mut db = DataSession::pooled(engine);
    let owner = Profiles::get(&mut db, "u00").await?.expect("seeded owner");
    assert!(owner.active);
    assert_eq!(owner.address.expect("seeded address").city, "Manila");
    assert!(Profiles::get(&mut db, "hidden").await?.is_none());
    assert!(
        !Profiles::get_maintenance(
            &mut db,
            &ProfileKey {
                id: "hidden".into()
            },
            Visibility::Inactive
        )
        .await?
        .expect("hidden maintenance row")
        .active
    );
    let project = Projects::get(&mut db, "p00-0")
        .await?
        .expect("seeded project");
    assert_eq!(project.owner_id, "u00");
    assert_eq!(project.tags, ["rust", "sql"]);
    assert_eq!(serde_json::to_value(project.state).unwrap(), "open");
    assert!(!Tasks::get(&mut db, "t00-0-0").await?.expect("task").done);
    assert!(Tasks::get(&mut db, "t00-0-1").await?.expect("task").done);

    let query = profile::Query {
        filters: vec![profile::Filter::Name(Predicate::Eq("Owner".into()))],
        sort: vec![Sort {
            field: profile::SortField::Name,
            direction: SortDirection::Asc,
            nulls: NullOrder::Last,
        }],
        take: 3,
        skip: 2,
        ..Default::default()
    };
    let page = Profiles::search(&mut db, query).await?;
    assert_eq!(
        page.rows
            .iter()
            .map(|row| row.row.id.as_str())
            .collect::<Vec<_>>(),
        ["u02", "u03", "u04"]
    );
    assert!(
        page.rows
            .iter()
            .all(|row| matches!(row.projects, Loaded::Unloaded))
    );
    assert_eq!(
        Profiles::count(
            &mut db,
            profile::Query {
                take: 1,
                skip: 100,
                ..Default::default()
            }
        )
        .await?,
        21
    );
    assert_eq!(
        Projects::count(
            &mut db,
            project::Query {
                filters: vec![project::Filter::Tags(Predicate::Has("rust".into()))],
                ..Default::default()
            }
        )
        .await?,
        40
    );
    assert_eq!(
        Projects::count(
            &mut db,
            project::Query {
                filters: vec![project::Filter::Tags(Predicate::Hasnt("sql".into()))],
                ..Default::default()
            }
        )
        .await?,
        20
    );
    assert_eq!(Tasks::count(&mut db, task::Query::default()).await?, 80);
    assert_eq!(
        Projects::find_all(
            &mut db,
            project::Query {
                take: 2,
                ..Default::default()
            }
        )
        .await?
        .len(),
        2
    );
    assert_eq!(
        Tasks::find(&mut db, task::Query::default())
            .await?
            .expect("found task")
            .row
            .id,
        "t00-0-0"
    );

    // A base-only atomic read is supported on every backend; do not gate all network BEGINs.
    let page = Profiles::search_atomic(
        engine,
        profile::Query {
            take: 1,
            ..Default::default()
        },
    )
    .await?;
    assert_eq!(page.rows[0].row.id, "empty");
    let mut transaction = engine.begin().await?;
    let selected = Profiles::detail(
        &mut DataSession::transaction(&mut transaction),
        &ProfileKey { id: "u00".into() },
    )
    .await?
    .expect("selected profile");
    assert_eq!(selected.value()["name"], "Owner");
    transaction.commit().await?;
    Ok(())
}
/// SQLite's exact common sequence proves bounded BFS, inverse paths, empty and unloaded states.
async fn sqlite(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    for take in [1, 20] {
        let before = calls.snapshot();
        let page = Profiles::search_atomic(
            engine,
            profile::Query {
                filters: vec![profile::Filter::Name(Predicate::Eq("Owner".into()))],
                take,
                total: true,
                includes: vec!["projects.tasks".into()],
                ..Default::default()
            },
        )
        .await?;
        assert_eq!(page.total, Some(20));
        assert_eq!(page.rows.len(), usize::from(take));
        assert_eq!(
            calls.snapshot().since(before),
            observe::Snapshot {
                begins: 1,
                runs: 4,
                commits: 1,
                rollbacks: 0
            }
        );
        for row in page.rows {
            let Loaded::Loaded(projects) = row.projects else {
                panic!("projects unloaded")
            };
            assert_eq!(projects.len(), 2);
            for project in projects {
                let Loaded::Loaded(tasks) = project.tasks else {
                    panic!("tasks unloaded")
                };
                assert_eq!(tasks.len(), 2);
                assert!(matches!(project.owner, Loaded::Unloaded));
            }
        }
    }
    let page = Profiles::search_atomic(
        engine,
        profile::Query {
            filters: vec![profile::Filter::Id(Predicate::Eq("empty".into()))],
            includes: vec!["projects.tasks".into()],
            ..Default::default()
        },
    )
    .await?;
    assert!(matches!(&page.rows[0].projects, Loaded::Loaded(rows) if rows.is_empty()));
    let page = Projects::search_atomic(
        engine,
        project::Query {
            filters: vec![project::Filter::Id(Predicate::Eq("p00-0".into()))],
            includes: vec!["owner".into(), "tasks".into()],
            ..Default::default()
        },
    )
    .await?;
    assert!(matches!(&page.rows[0].owner,Loaded::Loaded(owner) if owner.row.id=="u00"));
    assert!(matches!(&page.rows[0].tasks,Loaded::Loaded(tasks) if tasks.len()==2));
    let mut db = DataSession::pooled(engine);
    assert_eq!(
        Projects::search(
            &mut db,
            project::Query {
                q: Some("Literal%_".into()),
                ..Default::default()
            }
        )
        .await?
        .rows
        .len(),
        2
    );
    assert!(
        Projects::search(
            &mut db,
            project::Query {
                q: Some("literal%_".into()),
                ..Default::default()
            }
        )
        .await?
        .rows
        .is_empty()
    );
    Ok(())
}
/// Real network owners must refuse before even BEGIN; borrowed transaction refusal submits no run.
async fn network(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    for query in [
        profile::Query {
            q: Some("Owner".into()),
            ..Default::default()
        },
        profile::Query {
            total: true,
            ..Default::default()
        },
        profile::Query {
            includes: vec!["projects.tasks".into()],
            ..Default::default()
        },
    ] {
        let before = calls.snapshot();
        preflight::refused(
            Profiles::search_atomic(engine, query).await,
            DataErrorKind::Unsupported,
        );
        assert_eq!(calls.snapshot(), before);
    }
    let mut transaction = engine.begin().await?;
    let before = calls.snapshot();
    preflight::refused(
        Profiles::search(
            &mut DataSession::transaction(&mut transaction),
            profile::Query {
                includes: vec!["projects.tasks".into()],
                total: true,
                ..Default::default()
            },
        )
        .await,
        DataErrorKind::Unsupported,
    );
    assert_eq!(calls.snapshot(), before);
    transaction.rollback().await?;
    Ok(())
}
/// Lowered limits come from the exact selected Engine/Transaction and alter chunking, not rows.
pub(super) async fn check(
    database: Arc<dyn Database>,
    engine: &Engine,
    calls: &observe::Calls,
) -> TestResult<()> {
    common(engine).await?;
    if engine.dialect() == DialectId::Sqlite {
        sqlite(engine, calls).await?;
        let (limited, limited_calls) = observe::engine(database);
        let limited = limited.with_limits(QueryLimits {
            binds: 2,
            ..QueryLimits::default()
        });
        assert_eq!(limited.limits().binds, 2);
        let page = Profiles::search_atomic(
            &limited,
            profile::Query {
                filters: vec![profile::Filter::Name(Predicate::Eq("Owner".into()))],
                includes: vec!["projects.tasks".into()],
                take: 20,
                total: true,
                ..Default::default()
            },
        )
        .await?;
        assert_eq!(page.rows.len(), 20);
        // Twenty parents use ten project chunks and forty projects use twenty task chunks.
        assert_eq!(
            limited_calls.snapshot(),
            observe::Snapshot {
                begins: 1,
                runs: 32,
                commits: 1,
                rollbacks: 0
            }
        );
        let mut transaction = limited.begin().await?;
        assert_eq!(transaction.limits().binds, 2);
        assert_eq!(DataSession::transaction(&mut transaction).limits().binds, 2);
        transaction.rollback().await?;
    } else {
        network(engine, calls).await?;
    }
    Ok(())
}
