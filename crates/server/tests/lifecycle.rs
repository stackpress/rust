//! Decision-table tests with real lib responses and immutable script services.

use stackpress_lib::queue::task;
use stackpress_lib::{
    Body, DeliveryState, DispatchOutcome, Error, Flow, Invocation, Nest, ParseLimits, Request,
    Response, Result, SourceLocation,
};
use stackpress_server::{
    ErrorHandling, HandleOptions, LifecycleOutcome, LifecycleStage as Stage, Server, ServerBuilder,
    ServerInvocationExt, StartupContext,
};
use std::sync::{Arc, Mutex};

/// An explicit action for one stage in the observable lifecycle table.
#[derive(Clone, Copy, Debug)]
enum Action {
    Continue,
    Reply,
    Abort,
    Fail,
    Recover,
    CosmeticRecovery,
    DoubleFail,
    InvalidPublic,
    Clear,
    Finalize(DeliveryState),
    FinalizeFail(DeliveryState),
}

/// User-owned synchronization records only test observations and never surrounds dispatch.
struct Script {
    actions: [Action; 4],
    trace: Mutex<Vec<Stage>>,
    errors: Mutex<Vec<usize>>,
}

/// Execute one table action after recording its true host stage.
async fn scripted(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // The host-provided stage selects this script slot; caller input cannot choose the action.
    let script = inv.service::<Script>()?;
    let stage = inv.stage()?;
    script.trace.lock().unwrap().push(stage);
    let index = match stage {
        Stage::Request => 0,
        Stage::Operation => 1,
        Stage::Response => 2,
        Stage::Error => 3,
        _ => unreachable!(),
    };
    // Error must expose the original failed stage/allocation, never a fabricated Error failure.
    if stage == Stage::Error {
        let failure = inv.failure()?.expect("original failure in Error context");
        assert_ne!(failure.stage, Stage::Error);
        script
            .errors
            .lock()
            .unwrap()
            .push(Arc::as_ptr(&failure.error) as usize);
    }
    // Each action isolates one decision-table consequence while preserving actual response state.
    match script.actions[index] {
        // A no-op permits the host to decide absent-output behavior.
        Action::Continue => Ok(Flow::Continue),
        Action::Reply => {
            // A deliberate truthy body distinguishes operation success from no supplied output.
            res.results(&true)?;
            Ok(Flow::Continue)
        }
        // Cooperative Abort preserves the current response and suppresses later stage hooks.
        Action::Abort => Ok(Flow::Abort),
        Action::Recover => {
            // Recovery must replace error state rather than only changing its public status.
            *res = Response::with_limits(res.limits());
            res.results(&"recovered")?;
            Ok(Flow::Continue)
        }
        Action::CosmeticRecovery => {
            // Results alone must not erase the existing public error and diagnostic ownership.
            res.results(&"cosmetic")?;
            Ok(Flow::Continue)
        }
        Action::Clear => {
            // Clear all supplied output so final normalization must select an empty status.
            *res = Response::with_limits(res.limits());
            Ok(Flow::Continue)
        }
        Action::Finalize(state) => {
            // Consume delivery through the real value API before reporting ordinary completion.
            finalize(res, state).await?;
            Ok(Flow::Continue)
        }
        Action::FinalizeFail(state) => {
            // A trailing failure cannot reopen a delivery state already consumed by this callback.
            finalize(res, state).await?;
            Err(Error::new("private finalized cause"))
        }
        Action::Fail | Action::DoubleFail | Action::InvalidPublic => {
            // Seed every response surface to catch stale success or partial recovery leakage.
            if res.delivery_state() == DeliveryState::Open {
                res.html("secret body")?;
                res.headers_mut()?.set("x-secret", "secret header")?;
                res.session_mut()?.set("secret", "cookie", None)?;
                res.data_mut()?.set("secret", "data")?;
                res.set_stack(vec![SourceLocation::caller()])?;
            }
            // Keep developer detail/source distinct from the explicitly safe public error view.
            let error = Error::new("private stage detail")
                .with_cause(std::io::Error::other("secret source"))
                .with_code(403)
                .with_public_message("safe failure");
            // Invalid structured public errors force setup failure before any Error listener runs.
            if matches!(script.actions[index], Action::InvalidPublic) {
                return Err(error.with_errors(serde_json::json!([])));
            }
            Err(error)
        }
    }
}

/// Reach real one-attempt states, including cancellation and partial-failure history.
async fn finalize(res: &mut Response, state: DeliveryState) -> Result<()> {
    // Reach the real one-attempt state instead of faking sent/failed flags in a response clone.
    match state {
        DeliveryState::Open => {}
        DeliveryState::Sent => res.dispatch().await?,
        DeliveryState::Stopped => res.stop()?,
        DeliveryState::Failed => {
            // A writer failure consumes authority even when some native output might have escaped.
            let _ = res
                .dispatch_with(|_| async { Err(Error::new("partial")) })
                .await;
        }
        DeliveryState::Attempted => {
            // Cancel only after the actual writer suspends, leaving an incomplete attempt.
            let mut future = Box::pin(res.dispatch_with(|_| std::future::pending()));
            std::future::poll_fn(|cx| {
                assert!(future.as_mut().poll(cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            drop(future);
        }
    }
    Ok(())
}

/// Install a single explicit script service and optional lifecycle hooks.
async fn server(actions: [Action; 4], error_listener: bool) -> Result<Server> {
    // Publish the script as an explicit Config service so callbacks use real immutable host access.
    let script = Arc::new(Script {
        actions,
        trace: Mutex::new(Vec::new()),
        errors: Mutex::new(Vec::new()),
    });
    let mut app = ServerBuilder::new();
    app.on_config(task(move |start: &mut StartupContext| {
        let script = Arc::clone(&script);
        Box::pin(async move {
            start.register(script)?;
            Ok(Flow::Continue)
        })
    }))?;
    app.on("request", scripted)?;
    app.on("work", scripted)?;
    app.on("response", scripted)?;
    // Absence is a separate table input; a no-op Error callback would observe a different outcome.
    if error_listener {
        app.on("error", scripted)?;
    }
    app.build().await.map_err(|e| e.error)
}

/// Guard/operation/response failures follow the complete finite error-offer table.
#[tokio::test]
async fn failures_and_error_offers_follow_decision_table() -> Result<()> {
    // Cross each ordinary failing stage with all supported optional Error-listener outcomes.
    for (failed_index, failed_stage) in [
        (0, Stage::Request),
        (1, Stage::Operation),
        (2, Stage::Response),
    ] {
        for error_action in [
            None,
            Some(Action::Continue),
            Some(Action::Abort),
            Some(Action::DoubleFail),
            Some(Action::Recover),
            Some(Action::CosmeticRecovery),
        ] {
            let mut actions = [
                Action::Continue,
                Action::Reply,
                Action::Continue,
                Action::Continue,
            ];
            actions[failed_index] = Action::Fail;
            actions[3] = error_action.unwrap_or(Action::Continue);
            let app = server(actions, error_action.is_some()).await?;
            let reply = app.call("work", Nest::new()).await?;
            // Derive the exact hook trace: guard failure skips Operation; Error never retries it.
            let mut expected = vec![Stage::Request];
            if failed_index >= 1 {
                expected.push(Stage::Operation);
            }
            if failed_index >= 2 {
                expected.push(Stage::Response);
            }
            if error_action.is_some() {
                expected.push(Stage::Error);
            }
            // Only an earlier nonterminal failure can continue into the sole Response pass.
            if failed_index < 2 && !matches!(error_action, Some(Action::Abort | Action::DoubleFail))
            {
                expected.push(Stage::Response);
            }
            assert_eq!(
                *app.host().service::<Script>()?.trace.lock().unwrap(),
                expected,
                "{failed_stage:?} / {error_action:?}"
            );
            // Public recovery must retain original diagnostic precedence and remove stale output.
            assert_eq!(
                reply.report.outcome,
                LifecycleOutcome::Failed {
                    stage: failed_stage
                }
            );
            assert_eq!(reply.report.failures[0].stage, failed_stage);
            assert_eq!(
                reply.report.failures[0].error.message(),
                "private stage detail"
            );
            assert_eq!(reply.response.headers().get("x-secret"), None);
            assert!(reply.response.session().get("secret").is_none());
            assert!(!reply.response.data().has("secret"));
            assert!(reply.response.stack().is_none());
            // Offer status records actual dispatch, independently of final public success.
            let expected_handling = match error_action {
                None => ErrorHandling::NoListener,
                Some(Action::Abort) => ErrorHandling::Aborted,
                Some(Action::DoubleFail) => ErrorHandling::Failed,
                _ => ErrorHandling::Completed,
            };
            assert_eq!(reply.report.error_offers[0].outcome, expected_handling);
            if error_action.is_some() {
                assert_eq!(
                    app.host().service::<Script>()?.errors.lock().unwrap()[0],
                    Arc::as_ptr(&reply.report.failures[0].error) as usize
                );
            }
            // Secondary failure terminates with 500; only whole-state recovery establishes success.
            match error_action {
                Some(Action::DoubleFail) => {
                    assert_eq!(reply.report.failures.len(), 2);
                    assert_eq!(reply.report.failures[1].stage, Stage::Error);
                    assert_eq!(reply.response.status().unwrap().code, 500);
                    assert_eq!(reply.response.error(), Some("Internal Server Error"));
                    assert!(reply.report.terminal_failure);
                }
                Some(Action::Recover) => {
                    assert!(reply.report.is_success(&reply.response));
                }
                Some(Action::CosmeticRecovery) => {
                    assert_eq!(reply.response.status().unwrap().code, 200);
                    assert!(!reply.report.is_success(&reply.response));
                }
                _ => assert_eq!(reply.response.error(), Some("safe failure")),
            }
            if matches!(
                error_action,
                None | Some(Action::Continue | Action::Abort | Action::CosmeticRecovery)
            ) {
                // Fresh public output retains the original allocation through its source chain.
                let original = reply.report.failures[0].error.as_ref();
                let mut source: Option<&(dyn std::error::Error + 'static)> = reply
                    .response
                    .diagnostic()
                    .map(|e| e as &dyn std::error::Error);
                let mut found = false;
                for _ in 0..8 {
                    let Some(error) = source else {
                        break;
                    };
                    found |= error
                        .downcast_ref::<Error>()
                        .is_some_and(|e| std::ptr::eq(e, original));
                    source = error.source();
                }
                assert!(found, "fresh response lost original source identity");
            }
            // Default formatting is safe for public logs; developer formatting explicitly opts in.
            let display = reply.report.to_string();
            assert!(display.contains("work"));
            assert!(!display.contains("private stage detail"));
            assert!(!display.contains("secret source"));
            assert!(
                reply
                    .report
                    .developer()
                    .to_string()
                    .contains("secret source")
            );
        }
    }
    Ok(())
}

/// Abort at any stage stops the lifecycle; Open empty abort normalizes to 204.
#[tokio::test]
async fn aborts_preserve_response_and_stop_hooks() -> Result<()> {
    // Place Abort at each ordinary stage and observe that later hooks never enter the trace.
    for (index, stage) in [
        (0, Stage::Request),
        (1, Stage::Operation),
        (2, Stage::Response),
    ] {
        let mut actions = [
            Action::Continue,
            Action::Reply,
            Action::Continue,
            Action::Continue,
        ];
        actions[index] = Action::Abort;
        let app = server(actions, true).await?;
        let reply = app.call("work", Nest::new()).await?;
        assert_eq!(reply.report.outcome, LifecycleOutcome::Aborted { stage });
        assert_eq!(
            app.host().service::<Script>()?.trace.lock().unwrap().len(),
            index + 1
        );
        assert!(reply.report.failures.is_empty());
        assert_eq!(
            reply.response.status().unwrap().code,
            if index < 2 { 204 } else { 200 }
        );
        assert!(reply.report.output_eligible);
    }
    Ok(())
}

/// Body variants, not truthiness, distinguish deliberate output from missing/no-op operations.
#[tokio::test]
async fn bodies_status_only_missing_and_response_clear() -> Result<()> {
    // Register explicit body variants separately so truthiness cannot stand in for typed emptiness.
    let mut app = ServerBuilder::new();
    for (name, body) in [
        ("null", Body::Value(stackpress_lib::Value::Null)),
        ("false", Body::Value(false.into())),
        ("zero", Body::Value(0.into())),
        ("text", Body::Text(String::new())),
        ("bytes", Body::Bytes(Vec::new())),
    ] {
        app.on(
            name,
            stackpress_lib::handler(move |_, res, _| {
                let body = body.clone();
                Box::pin(async move {
                    res.set("application/json", body)?;
                    Ok(Flow::Continue)
                })
            }),
        )?;
    }
    app.on("noop", noop)?;
    app.on("status", status)?;
    let app = app.build().await.map_err(|e| e.error)?;
    // All deliberate bodies receive 200; the explicit redirect keeps its supplied status.
    for event in ["null", "false", "zero", "text", "bytes", "status"] {
        let reply = app.call(event, Nest::new()).await?;
        assert!(reply.report.failures.is_empty());
        assert_eq!(
            reply.response.status().unwrap().code,
            if event == "status" { 302 } else { 200 }
        );
    }
    // Matched no-op and unmatched events both fail absent-output policy with one optional offer.
    for event in ["noop", "missing"] {
        let reply = app.call(event, Nest::new()).await?;
        assert_eq!(reply.response.status().unwrap().code, 404);
        assert_eq!(
            reply.report.error_offers[0].outcome,
            ErrorHandling::NoListener
        );
    }
    // Response may intentionally clear earlier output; final normalization then selects 204.
    let app = server(
        [
            Action::Continue,
            Action::Reply,
            Action::Clear,
            Action::Continue,
        ],
        true,
    )
    .await?;
    let reply = app.call("work", Nest::new()).await?;
    assert_eq!(reply.response.status().unwrap().code, 204);
    assert!(reply.report.failures.is_empty());
    Ok(())
}

/// Matched no-op deliberately supplies neither body nor status.
async fn noop(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A matched no-op is deliberately empty, distinct from a deliberate status-only response.
    Ok(Flow::Continue)
}
/// A status-only redirect is deliberate output without a body.
async fn status(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // An explicit redirect/status must survive the host's absent-output/default-status checks.
    res.status_code(302, None)?;
    Ok(Flow::Continue)
}

/// Finalized entry and finalization inside every stage skip hooks and never retry output.
#[tokio::test]
async fn delivery_states_are_authoritative() -> Result<()> {
    // Each preconsumed state must skip hooks and writes, including failures returned after
    // finalization.
    for state in [
        DeliveryState::Sent,
        DeliveryState::Stopped,
        DeliveryState::Attempted,
        DeliveryState::Failed,
    ] {
        let app = server(
            [
                Action::Continue,
                Action::Reply,
                Action::Continue,
                Action::Continue,
            ],
            true,
        )
        .await?;
        let mut req = Request::new();
        let mut res = Response::new();
        finalize(&mut res, state).await?;
        // Entry handling must leave load state untouched and return an honest delivery diagnostic.
        let report = app.handle("work", &mut req, &mut res).await?;
        assert_eq!(res.delivery_state(), state);
        assert!(!req.loaded());
        assert!(
            app.host()
                .service::<Script>()?
                .trace
                .lock()
                .unwrap()
                .is_empty()
        );
        assert!(!report.output_eligible);
        if matches!(state, DeliveryState::Attempted | DeliveryState::Failed) {
            assert_eq!(
                report.outcome,
                LifecycleOutcome::Failed {
                    stage: Stage::Output
                }
            );
        } else {
            assert_eq!(
                report.outcome,
                LifecycleOutcome::Finalized {
                    stage: Stage::Input
                }
            );
        }
        // Then reach each state inside Request/Operation/Response, with and without a trailing
        // error.
        for index in 0..3 {
            // Check both continuing and error-returning callbacks after delivery state is consumed.
            for fails in [false, true] {
                let mut actions = [
                    Action::Continue,
                    Action::Reply,
                    Action::Continue,
                    Action::Continue,
                ];
                actions[index] = if fails {
                    Action::FinalizeFail(state)
                } else {
                    Action::Finalize(state)
                };
                let app = server(actions, true).await?;
                let reply = app.call("work", Nest::new()).await?;
                assert_eq!(reply.response.delivery_state(), state);
                assert_eq!(
                    app.host().service::<Script>()?.trace.lock().unwrap().len(),
                    index + 1
                );
                assert!(!reply.report.output_eligible);
                assert!(reply.report.error_offers.is_empty());
            }
        }
        // Finalization inside Error must also stop Response, even when Error itself then fails.
        for fails in [false, true] {
            let action = if fails {
                Action::FinalizeFail(state)
            } else {
                Action::Finalize(state)
            };
            let app = server(
                [Action::Continue, Action::Fail, Action::Continue, action],
                true,
            )
            .await?;
            let reply = app.call("work", Nest::new()).await?;
            assert_eq!(reply.response.delivery_state(), state);
            assert_eq!(
                app.host()
                    .service::<Script>()?
                    .trace
                    .lock()
                    .unwrap()
                    .as_slice(),
                [Stage::Request, Stage::Operation, Stage::Error]
            );
            assert_eq!(
                reply
                    .report
                    .failures
                    .iter()
                    .filter(|f| f.stage == Stage::Error)
                    .count(),
                usize::from(fails)
            );
        }
    }
    Ok(())
}

/// Public-error setup failure is terminal; original and setup diagnostics survive tiny budgets.
#[tokio::test]
async fn invalid_public_error_setup_is_nonrecursive() -> Result<()> {
    // Malformed public error data must fail setup without reaching the otherwise installed hook.
    let app = server(
        [
            Action::Continue,
            Action::InvalidPublic,
            Action::Continue,
            Action::Reply,
        ],
        true,
    )
    .await?;
    let reply = app.call("work", Nest::new()).await?;
    // Retain original/setup diagnostics while exposing only a fresh generic terminal response.
    assert_eq!(reply.report.failures.len(), 2);
    assert_eq!(reply.report.failures[1].stage, Stage::Error);
    assert!(reply.report.error_offers.is_empty());
    assert_eq!(reply.response.status().unwrap().code, 500);
    assert!(!reply.response.try_to_json()?.to_string().contains("secret"));
    // An empty zero-budget response still receives a status-only 500 without another hook pass.
    let mut app = ServerBuilder::new();
    app.on("work", fail_without_mutation)?;
    let app = app.build().await.map_err(|e| e.error)?;
    let mut res = Response::with_limits(ParseLimits {
        max_bytes: 0,
        max_entries: 0,
        ..Default::default()
    });
    let report = app.handle("work", &mut Request::new(), &mut res).await?;
    assert_eq!(res.status().unwrap().code, 500);
    assert_eq!(report.failures.len(), 2);
    assert!(matches!(res.body(), Body::Empty));
    Ok(())
}
/// Fail without touching tiny-budget output first.
async fn fail_without_mutation(
    _: &mut Request,
    _: &mut Response,
    _: Invocation<'_>,
) -> Result<Flow> {
    // Fail before touching tiny-budget response data so the error-setup fallback is the tested
    // path.
    Err(Error::new("secret original"))
}

/// No normalized input means no fabricated Request and no callback at all (G17).
#[tokio::test]
async fn conversion_and_existing_load_failure_are_distinct() -> Result<()> {
    // A malformed plain input has no normalized Request; an already failed load still has one.
    let app = server(
        [
            Action::Continue,
            Action::Reply,
            Action::Continue,
            Action::Continue,
        ],
        true,
    )
    .await?;
    let reply = app.call("work", serde_json::json!([])).await?;
    assert_eq!(reply.response.status().unwrap().code, 400);
    assert_eq!(reply.report.failures[0].stage, Stage::Input);
    assert!(reply.report.error_offers.is_empty());
    assert!(
        app.host()
            .service::<Script>()?
            .trace
            .lock()
            .unwrap()
            .is_empty()
    );
    // Now provide a real request with consumed failed load state so Error/Response hooks may run.
    let mut req = Request::new();
    assert!(
        req.load_with(|_| async { Err(Error::new("private load cause")) })
            .await
            .is_err()
    );
    let mut res = Response::new();
    let report = app.handle("work", &mut req, &mut res).await?;
    assert_eq!(report.failures[0].stage, Stage::Input);
    assert_eq!(
        *app.host().service::<Script>()?.trace.lock().unwrap(),
        [Stage::Error, Stage::Response]
    );
    Ok(())
}

/// Raw calls skip loading/status/error hooks; route selection excludes query.
#[tokio::test]
async fn raw_and_handled_entry_points() -> Result<()> {
    // Use the same real host to distinguish raw metadata/loading from normalized route selection.
    let mut app = ServerBuilder::new();
    app.on("raw", raw_context)?;
    app.get("/items/:id", route_context)?;
    let app = app.build().await.map_err(|e| e.error)?;
    let mut req = Request::new();
    let mut res = Response::new();
    assert_eq!(
        app.emit("raw", &mut req, &mut res).await?,
        DispatchOutcome::Completed
    );
    assert!(res.status().is_none());
    assert!(!req.loaded());
    // Raw route resolution still injects host captures without running general lifecycle hooks.
    let result = app
        .resolve_route(stackpress_lib::Method::GET, "/items/7", Nest::new())
        .await?;
    assert_eq!(result.response.results.unwrap().as_str(), Some("7"));
    // Handled route selection uses normalized pathname only; query secrets stay out of reports.
    let mut req = Request::builder()
        .url("/items/8?event=raw&secret=hidden")
        .build()?;
    let report = app.handle_route(&mut req, &mut Response::new()).await?;
    assert_eq!(report.event, "GET /items/8");
    assert!(!report.to_string().contains("hidden"));
    // Invalid event text must be bounded/sanitized before it enters report diagnostics.
    let reply = app.call("bad\nevent", Nest::new()).await?;
    assert_eq!(reply.report.event, "<invalid event>");
    assert_eq!(reply.report.failures[0].stage, Stage::Input);
    Ok(())
}
/// Raw host roots have Dispatch context and no invented correlation/adapter context.
async fn raw_context(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Raw host roots expose services but must not invent correlation, failure or adapter metadata.
    assert_eq!(inv.stage()?, Stage::Dispatch);
    assert_eq!(inv.correlation_id()?, None);
    assert!(inv.failure()?.is_none());
    assert!(inv.adapter_context::<u64>().is_none());
    Ok(Flow::Continue)
}
/// Use the actual injected capture after either raw or handled host-route dispatch.
async fn route_context(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Read the host-injected aggregate key rather than selecting an event from caller data.
    res.results(&req.data.path_as::<String>("id")?.unwrap())?;
    Ok(Flow::Continue)
}

/// Trusted adapter metadata survives nested dispatch without colliding with untrusted data.
#[tokio::test]
async fn adapter_context_is_inherited_separately() -> Result<()> {
    // Trusted adapter metadata is supplied outside Request.data and must survive raw child
    // delegation.
    /// Inspect the trusted root metadata inherited by a delegated child.
    async fn child(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        // Observe inherited metadata after the parent explicitly delegates into this raw child.
        assert_eq!(inv.adapter_context::<u64>(), Some(&99));
        assert_eq!(inv.stage()?, Stage::Operation);
        assert!(inv.correlation_id()?.is_some());
        res.results(&true)?;
        Ok(Flow::Continue)
    }
    /// Delegate the original pair without creating a second lifecycle boundary.
    async fn parent(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        // Pass the exact caller pair to the child; this must not allocate a second lifecycle ID.
        inv.delegate("child", req, res).await
    }
    let mut app = ServerBuilder::new();
    app.on("parent", parent)?;
    app.on("child", child)?;
    let app = app.build().await.map_err(|e| e.error)?;
    let mut req = Request::builder()
        .data(serde_json::json!({"adapter_context":0}))
        .build()?;
    let mut res = Response::new();
    let report = app
        .handle_with_options(
            "parent",
            &mut req,
            &mut res,
            HandleOptions {
                adapter_context: Some(Arc::new(99_u64)),
            },
        )
        .await?;
    assert!(report.is_success(&res));
    Ok(())
}

/// A later Response failure receives its own error offer, but Response never runs a second time.
#[tokio::test]
async fn two_stage_failures_have_exactly_two_offers() -> Result<()> {
    // Request and Response both fail, proving a later offer does not re-enter Response again.
    let app = server(
        [Action::Fail, Action::Reply, Action::Fail, Action::Continue],
        true,
    )
    .await?;
    let reply = app.call("work", Nest::new()).await?;
    // Exact counts/order prove finite error handling and that Operation stayed skipped after guard
    // failure.
    assert_eq!(reply.report.failures.len(), 2);
    assert_eq!(reply.report.failures[0].stage, Stage::Request);
    assert_eq!(reply.report.failures[1].stage, Stage::Response);
    assert_eq!(reply.report.error_offers.len(), 2);
    assert_eq!(
        *app.host().service::<Script>()?.trace.lock().unwrap(),
        [Stage::Request, Stage::Error, Stage::Response, Stage::Error]
    );
    Ok(())
}

/// Each route injects its own incoming captures in priority order; raw nested capability sees them.
#[tokio::test]
async fn capture_collisions_literal_names_and_budget_failure() -> Result<()> {
    // Two all-matching routes deliberately overwrite the same aggregate key in registration order.
    /// A capability can read captures injected by its exposing host route.
    async fn capability(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // Expose aggregate data after injection so nested raw capability input remains observable.
        res.results(&req.data)?;
        Ok(Flow::Continue)
    }
    /// Route delegates without recursively handling the general lifecycle.
    async fn delegate(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        // Use raw nesting to retain the exposing route's stage and capture mutation.
        inv.delegate("capability", req, res).await
    }
    let mut app = ServerBuilder::new();
    app.on("capability", capability)?;
    app.get("/collision/:id/:tail", delegate)?;
    app.get("/collision/:head/:id", delegate)?;
    app.get("/punct/:user-id/*", delegate)?;
    app.get("/bounded/:id", delegate)?;
    let app = app.build().await.map_err(|e| e.error)?;
    let mut req = Request::builder()
        .url("/collision/first/last?id=query")
        .post(serde_json::json!({"id":"post"}))
        .data(serde_json::json!({"id":"explicit"}))
        .build()?;
    let mut res = Response::new();
    app.handle_route(&mut req, &mut res).await?;
    assert_eq!(req.data.path_as::<String>("id")?, Some("last".into()));
    assert_eq!(req.query.path_as::<String>("id")?, Some("query".into()));
    assert_eq!(req.post.path_as::<String>("id")?, Some("post".into()));
    // Literal punctuation survives named capture injection; positional captures stay separate.
    let result = app
        .resolve("GET /punct/value/positional", Nest::new())
        .await?;
    let value = result.response.results.unwrap().to_json()?;
    assert_eq!(value["user-id"], "value");
    assert!(value.get("args").is_none());
    assert!(value.get("0").is_none());
    // A separate low-budget aggregate store must remain unchanged when capture injection fails.
    let mut req = Request::builder().url("/bounded/long-capture").build()?;
    req.data = Nest::with_limits(ParseLimits {
        max_bytes: 8,
        ..Default::default()
    });
    req.data.set("k", "v")?;
    let before = req.data.to_json()?;
    let mut res = Response::new();
    let report = app.handle_route(&mut req, &mut res).await?;
    assert_eq!(req.data.to_json()?, before);
    assert_eq!(report.failures[0].stage, Stage::Operation);
    assert_eq!(report.failures[0].error.frames().len(), 1);
    Ok(())
}

/// Prebuffered bodies without status and early-abort status-only output follow typed defaults.
#[tokio::test]
async fn prebuffered_output_and_abort_status_defaults() -> Result<()> {
    // Prepare bodies through an envelope so status is truly absent before host default selection.
    /// An explicit early stop leaves the exact caller response available to its adapter.
    async fn abort(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // An early cooperative stop preserves the caller's prebuffered body and optional custom
        // status.
        Ok(Flow::Abort)
    }
    let mut builder = ServerBuilder::new();
    builder.on("work", noop)?;
    let app = builder.build().await.map_err(|e| e.error)?;
    let mut response = Response::new();
    response.apply_envelope(stackpress_lib::ResponseEnvelope {
        results: Some(false.into()),
        ..Default::default()
    })?;
    assert!(response.status().is_none());
    let report = app
        .handle("work", &mut Request::new(), &mut response)
        .await?;
    assert!(report.failures.is_empty());
    assert_eq!(response.status().unwrap().code, 200);
    // Early Abort preserves explicit statuses, including unconventional accepted status values.
    for (event, stage) in [("request", Stage::Request), ("work", Stage::Operation)] {
        let mut builder = ServerBuilder::new();
        builder.on(event, abort)?;
        let app = builder.build().await.map_err(|e| e.error)?;
        for code in [None, Some(0), Some(700)] {
            for body in [false, true] {
                let mut response = Response::new();
                if body {
                    response.apply_envelope(stackpress_lib::ResponseEnvelope {
                        results: Some(false.into()),
                        ..Default::default()
                    })?;
                }
                if let Some(code) = code {
                    response.status_code(code, None)?;
                }
                let report = app
                    .handle("work", &mut Request::new(), &mut response)
                    .await?;
                assert_eq!(report.outcome, LifecycleOutcome::Aborted { stage });
                assert!(report.failures.is_empty());
                assert_eq!(
                    response.status().unwrap().code,
                    code.unwrap_or(if body { 200 } else { 204 })
                );
                assert!(report.output_eligible);
            }
        }
    }
    Ok(())
}
