//! Core non-atomic forwarding and cancellation contracts; native effects need live evidence.

use super::*;

/// Whole-script validation must happen even when a later query is invalid.
#[test]
fn g48_non_atomic_preflight_empty_and_ordered_results() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone()));
    assert!(complete(engine.execute_non_atomic(&[])).unwrap().is_empty());
    let plan = three_statement_plan(DialectId::Sqlite);
    let wrong = three_statement_plan(DialectId::Postgres);
    let mut queries = plan.queries().to_vec();
    queries.push(wrong.queries()[0].clone());
    let error = complete(engine.execute_non_atomic(&queries)).unwrap_err();
    assert_eq!(
        error.plan_progress(),
        Some(&PlanProgress {
            acknowledged: 0,
            failed_index: 3,
            total: 4
        })
    );
    assert!(calls.lock().unwrap().events.is_empty());
    let results = complete(engine.execute_non_atomic(plan.queries())).unwrap();
    assert_eq!(
        results
            .iter()
            .map(|result| result.affected)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let state = calls.lock().unwrap();
    assert_eq!(state.events, ["database-run"; 3]);
    assert_eq!(
        state
            .queries
            .iter()
            .map(|(query, _)| query.sql())
            .collect::<Vec<_>>(),
        plan.queries().iter().map(Query::sql).collect::<Vec<_>>()
    );
}

/// Driver failure at every position retains its kind/display/source and stops without retry.
#[test]
fn g48_non_atomic_error_progress_and_limits() {
    for failed_index in 0..3 {
        let calls = Arc::new(Mutex::new(Calls {
            fail_at: Some(failed_index + 1),
            ..Calls::default()
        }));
        let engine = Engine::new(Driver(calls.clone()));
        let error =
            complete(engine.execute_non_atomic(three_statement_plan(DialectId::Sqlite).queries()))
                .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Driver);
        assert_eq!(error.to_string(), "test statement failed");
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "primary native source"
        );
        assert_eq!(
            error.plan_progress(),
            Some(&PlanProgress {
                acknowledged: failed_index,
                failed_index,
                total: 3
            })
        );
        assert!(error.cleanup_error().is_none());
        assert_eq!(
            calls.lock().unwrap().events,
            vec!["database-run"; failed_index + 1]
        );
    }
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone())).with_limits(QueryLimits {
        binds: 0,
        ..QueryLimits::default()
    });
    let valid = Select::new(["id"])
        .from("a")
        .render(DialectId::Sqlite)
        .unwrap();
    let invalid = Select::new(["id"])
        .from("a")
        .where_eq("id", 1)
        .render(DialectId::Sqlite)
        .unwrap();
    let error = complete(engine.execute_non_atomic(&[valid, invalid])).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Limit);
    assert_eq!(
        error.plan_progress(),
        Some(&PlanProgress {
            acknowledged: 0,
            failed_index: 1,
            total: 2
        })
    );
    assert!(calls.lock().unwrap().events.is_empty());
    assert!(
        SqlError::new(ErrorKind::Closed, "closed")
            .plan_progress()
            .is_none()
    );
}

/// Dropping an unpolled or suspended future cannot submit later statements or invent rollback.
#[test]
fn g48_non_atomic_cancellation_stops_the_chain() {
    for pending_at in [1, 2] {
        let calls = Arc::new(Mutex::new(Calls {
            pending_at: Some(pending_at),
            ..Calls::default()
        }));
        let engine = Engine::new(Driver(calls.clone()));
        let plan = three_statement_plan(DialectId::Sqlite);
        drop(engine.execute_non_atomic(plan.queries()));
        assert!(calls.lock().unwrap().events.is_empty());
        let mut future = Box::pin(engine.execute_non_atomic(plan.queries()));
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        drop(future);
        let state = calls.lock().unwrap();
        assert_eq!(state.events, vec!["database-run"; pending_at]);
        assert_eq!(state.queries.len(), pending_at);
        assert_eq!(state.active_sessions, 0);
    }
}
