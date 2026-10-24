//! M2 ordering, pattern, customization and diagnostic contracts against production.

use stackpress_lib::queue::{ItemQueue, QueueOutcome, TaskQueue, task};
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Error, EventPattern, Flow, Invocation, ListenerSpec,
    MethodPattern, Request, Response, Result, Status,
};

/// Append an observable event/phase receipt through public data operations.
fn record(req: &mut Request, entry: String) -> Result<()> {
    // Persist only ordinary receipts through the public data API so tests cannot mutate registry
    // internals.

    let mut log: Vec<String> = req.data.path_as("log")?.unwrap_or_default();
    log.push(entry);
    req.data.set(
        "log",
        stackpress_lib::Value::try_from(serde_json::to_value(log)?)?,
    )?;
    Ok(())
}
/// Read receipts rather than reach into registry/queue internals.
fn log(req: &Request) -> Result<Vec<String>> {
    // Decode receipts using the same typed accessor a real handler uses.

    Ok(req.data.path_as("log")?.unwrap_or_default())
}
/// A named callback records the selected label and current capture map.
async fn labeled(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Yield before recording the listener label to expose invocation metadata lifetime mistakes.

    tokio::task::yield_now().await;
    record(req, inv.listener().label.clone())?;
    Ok(Flow::Continue)
}
/// A replacement changes behavior without changing its stable registration metadata.
async fn replacement(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A different receipt makes stable-handle replacement distinguishable from duplicate
    // registration.

    record(req, "replacement".into())?;
    Ok(Flow::Continue)
}
/// Phase policy is controlled by explicit request input, avoiding three duplicate hooks.
async fn phase(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Record before suspending/failing so each stop/error case proves its exact executed prefix.

    let phase = inv.listener().phase.clone();
    record(req, format!("{phase}:{}", inv.listener().label))?;
    tokio::task::yield_now().await;
    if req.data.path_as::<String>("error")?.as_deref() == Some(&phase) {
        return Err(Error::new("private asynchronous failure"));
    }
    Ok(
        if req.data.path_as::<String>("abort")?.as_deref() == Some(&phase) {
            Flow::Abort
        } else {
            Flow::Continue
        },
    )
}

/// Scheduling compares priorities without subtraction and retains high/low watermarks.
#[test]
fn stable_item_queue_ties_extremes_and_drain() -> Result<()> {
    // Use equal and extreme priorities to expose unstable ties and arithmetic overflow.

    let mut queue = ItemQueue::new();
    assert!(queue.pop().is_none());
    queue
        .add("first", 7)?
        .add("second", 7)?
        .push_back("low")?
        .push_front("high")?;
    assert_eq!(
        queue.entries().iter().map(|v| v.item).collect::<Vec<_>>(),
        ["high", "first", "second", "low"]
    );
    assert_eq!(queue.pop(), Some("high"));
    assert_eq!(queue.len(), 3);
    // Drain completely before inserting extremes: retained front/back watermarks must still prevent
    // overflow.
    while queue.pop().is_some() {}
    queue.add("maximum", i64::MAX)?.add("minimum", i64::MIN)?;
    assert!(queue.push_front("overflow").is_err());
    assert!(queue.push_back("underflow").is_err());
    assert_eq!(queue.len(), 2);
    Ok(())
}

/// Named task actions support question-mark and mutate caller-owned state sequentially.
async fn increment(state: &mut Vec<&'static str>) -> Result<Flow> {
    // An ordinary named async task proves registration without boxing ceremony.

    state.push("action");
    Ok(Flow::Continue)
}
/// An explicit task stop leaves unstarted work available.
async fn stop(state: &mut Vec<&'static str>) -> Result<Flow> {
    // Return cooperative Abort after a visible mutation; remaining tasks must stay queued.

    state.push("abort");
    Ok(Flow::Abort)
}
/// A phase error remains a failure and preserves prefix mutation.
async fn fail(state: &mut Vec<&'static str>) -> Result<Flow> {
    // Return an ordinary Error after a visible mutation; it must not become cooperative Abort.

    state.push("error");
    Err(Error::new("task failure"))
}
/// Queue before/action/after cancellation/error paths have the same precedence.
#[tokio::test]
async fn tasks_are_sequential_consume_once_and_honor_hooks() -> Result<()> {
    // Start empty, then run named actions with observable before/after receipts.

    // The first empty run must report Empty rather than Completed or a fabricated response status.
    let mut queue = TaskQueue::new();
    let mut state = Vec::new();
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Empty);
    queue.before(task(|state: &mut Vec<&'static str>| {
        Box::pin(async move {
            state.push("before");
            Ok(Flow::Continue)
        })
    }));
    queue.after(task(|state: &mut Vec<&'static str>| {
        Box::pin(async move {
            state.push("after");
            Ok(Flow::Continue)
        })
    }));
    queue.add(increment, 1)?.add(increment, 0)?;
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Completed);
    assert_eq!(
        state,
        ["before", "action", "after", "before", "action", "after"]
    );
    let mut queue = TaskQueue::new();
    // Abort consumes the selected task while preserving the unstarted successor for a later run.
    queue.add(stop, 1)?.add(increment, 0)?;
    state.clear();
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Aborted);
    assert_eq!(state, ["abort"]);
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Completed);
    let mut queue = TaskQueue::new();
    // A before abort skips both action and after phases for the consumed task.
    queue.before(stop);
    queue.add(increment, 0)?;
    state.clear();
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Aborted);
    assert_eq!(state, ["abort"]);
    let mut queue = TaskQueue::new();
    // An after abort preserves the action receipt but stops the next queued task.
    queue.after(stop);
    queue.add(increment, 0)?.add(increment, -1)?;
    state.clear();
    assert_eq!(queue.run(&mut state).await?, QueueOutcome::Aborted);
    assert_eq!(state, ["action", "abort"]);
    let mut queue = TaskQueue::new();
    queue.add(fail, 1)?.add(increment, 0)?;
    assert!(queue.run(&mut state).await.is_err());
    assert_eq!(queue.len(), 1);
    // Hook failures consume only the selected task and skip all later phases.
    for before in [true, false] {
        let mut queue = TaskQueue::new();
        if before {
            queue.before(fail);
        } else {
            queue.after(fail);
        }
        queue.push_back(increment)?.push_front(increment)?;
        state.clear();
        assert!(queue.run(&mut state).await.is_err());
        assert_eq!(
            state,
            if before {
                vec!["error"]
            } else {
                vec!["action", "error"]
            }
        );
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }
    Ok(())
}

/// All matching kinds share priority/tie ordering; composition never reparses regex.
#[tokio::test]
async fn all_match_order_composition_and_captures_stay_explicit() -> Result<()> {
    // Compose different matching kinds at equal/unequal priorities to prove one global
    // deterministic order.

    let mut parent = EmitterBuilder::new();
    parent.listen(
        ListenerSpec::exact("/users/42").priority(5).label("first"),
        labeled,
    )?;
    parent.listen(
        ListenerSpec::pattern(EventPattern::express("/users/:id", '/')?)
            .priority(5)
            .label("second"),
        labeled,
    )?;
    let mut child = EmitterBuilder::new();
    child.listen(
        ListenerSpec::pattern(EventPattern::regex(r"^/users/(\d+)$", false)?)
            .priority(10)
            .label("regex"),
        labeled,
    )?;
    child.listen(
        ListenerSpec::pattern(EventPattern::express("/users/**", '/')?)
            .priority(-10)
            .label("fallback"),
        labeled,
    )?;
    // Move contributions without reparsing compiled patterns or losing their source ordering.
    parent.extend(child)?;
    let emitter = parent.build();
    let matches = emitter.matches("/users/42")?;
    assert_eq!(matches.len(), 4);
    assert_eq!(matches[1].data.args.len(), 0);
    assert_eq!(matches[2].data.params["id"], "42");
    // Seed an existing request capture key; raw dispatch must leave it unchanged.
    let mut req = Request::new();
    req.data.set("id", "original")?;
    let mut res = Response::new();
    assert_eq!(
        emitter.emit("/users/42", &mut req, &mut res).await?,
        DispatchOutcome::Completed
    );
    // Observable callback order, not internal vector layout, proves priority/tie/composition
    // behavior.
    assert_eq!(log(&req)?, ["regex", "first", "second", "fallback"]);
    assert_eq!(req.data.path_as::<String>("id")?, Some("original".into()));
    assert!(res.result().is_none());
    Ok(())
}

/// Separate display/typed identities preserve equivalent patterns with different names.
#[test]
fn express_regex_and_route_grammar_are_deliberate() -> Result<()> {
    // Use literal regex punctuation and rejected ambiguous tokens to pin the explicit Rust matching
    // contract.

    for text in ["/literal/file.txt", "/literal/(x)+[y]$^"] {
        let pattern = EventPattern::express(text, '/')?;
        assert!(pattern.captures(text)?.is_some());
        assert!(pattern.captures(&format!("{text}x"))?.is_none());
    }
    let custom = EventPattern::express("audit-**", '-')?
        .captures("audit-part-more")?
        .expect("match");
    assert_eq!(custom.args, [Some("part".into()), Some("more".into())]);
    assert!(EventPattern::express("/:id/:id", '/').is_err());
    assert!(EventPattern::express("/***", '/').is_err());
    assert!(EventPattern::express("/:id*", '/').is_err());
    assert!(EventPattern::regex(r"(?=x)", false).is_err());
    let regex = EventPattern::regex(r"^(?P<kind>item)(?:-(\d+))?$", true)?;
    // Repeated regex matching must have no JS-style global cursor or stale optional capture state.
    for _ in 0..3 {
        let captures = regex.captures("ITEM")?.expect("match");
        assert_eq!(captures.args, [Some("ITEM".into()), None]);
        assert_eq!(captures.params["kind"], "ITEM");
    }
    let any = EventPattern::route(MethodPattern::Any, "/health/:scope")?;
    assert!(any.captures("POST /health/live///")?.is_some());
    assert_eq!(
        any.route_metadata().expect("metadata").path,
        "/health/:scope"
    );
    assert!(
        EventPattern::route(MethodPattern::exact("get")?, "/health")?
            .captures("GET /health/")?
            .is_none()
    );
    assert!(EventPattern::route(MethodPattern::Any, "not/absolute").is_err());
    assert!(MethodPattern::exact("bad verb").is_err());
    let mut builder = EmitterBuilder::new();
    builder.on_pattern(EventPattern::express("/:id", '/')?, labeled)?;
    builder.on_pattern(EventPattern::express("/:name", '/')?, labeled)?;
    let matches = builder.build().matches("/42")?;
    assert_eq!(matches.len(), 2);
    assert!(matches[0].data.params.contains_key("id"));
    assert!(matches[1].data.params.contains_key("name"));
    Ok(())
}

/// Removal/replacement uses handles, preserves order/site and rejects foreign origins.
#[tokio::test]
async fn handles_and_replacement_are_explicit_not_duplicate_order() -> Result<()> {
    // Keep two similar registrations and customize only the chosen stable handle.

    let mut first = EmitterBuilder::new();
    let registered = line!() + 1;
    let id = first.on_priority("event", 7, labeled)?;
    let original = first.on("event", labeled)?;
    let mut foreign = EmitterBuilder::new();
    let foreign_id = foreign.on("event", labeled)?;
    assert!(first.remove(&foreign_id).is_err());
    let replacement_line = line!() + 1;
    first.replace(&id, replacement)?;
    assert!(first.remove(&original)?);
    assert!(!first.remove(&original)?);
    assert!(first.replace(&original, labeled).is_err());
    first.extend(foreign)?;
    let emitter = first.build();
    let description = emitter.listeners().next().expect("listener");
    assert_eq!(description.id, id);
    assert_eq!(description.registered.line, registered);
    assert_eq!(
        description.replaced.as_ref().expect("replacement").line,
        replacement_line
    );
    let mut req = Request::new();
    emitter
        .emit("event", &mut req, &mut Response::new())
        .await?;
    assert_eq!(log(&req)?.first().map(String::as_str), Some("replacement"));
    let mut cleared = EmitterBuilder::new();
    cleared.on_pattern(EventPattern::express("/x/:id", '/')?, labeled)?;
    assert_eq!(cleared.clear(&EventPattern::express("/x/:id", '/')?), 1);
    assert!(cleared.build().matches("/x/1")?.is_empty());
    Ok(())
}

/// Before/action/after abort and error precedence is exercised as a decision table.
#[tokio::test]
async fn phase_abort_error_and_async_diagnostics() -> Result<()> {
    // Vary the stopping phase and compare exact receipt prefixes, including async error provenance.

    let mut builder = EmitterBuilder::new();
    let before_line = line!() + 1;
    builder.before(phase);
    builder.after(phase);
    let line = line!() + 1;
    builder.listen(
        ListenerSpec::exact("event").label("one").plugin("payments"),
        phase,
    )?;
    builder.listen(ListenerSpec::exact("event").label("two"), phase)?;
    let emitter = builder.build();
    for (phase_name, expected_len) in [("before", 1), ("action", 2), ("after", 3)] {
        let mut req = Request::new();
        req.data.set("abort", phase_name)?;
        assert_eq!(
            emitter
                .emit("event", &mut req, &mut Response::new())
                .await?,
            DispatchOutcome::Aborted
        );
        assert_eq!(log(&req)?.len(), expected_len);
        let mut req = Request::new();
        req.data.set("error", phase_name)?;
        let error = emitter
            .emit("event", &mut req, &mut Response::new())
            .await
            .expect_err("phase failure");
        assert_eq!(error.frames().len(), 1);
        let frame = &error.frames()[0];
        assert_eq!(frame.phase, phase_name);
        assert_eq!(frame.label, "one");
        assert_eq!(frame.plugin.as_deref(), Some("payments"));
        assert_eq!(frame.registered.line, line);
        if phase_name == "before" {
            assert_eq!(
                frame.phase_registered.as_ref().expect("hook site").line,
                before_line
            );
        }
        let public = serde_json::to_string(&error.public_view())?;
        assert!(
            !public.contains("payments")
                && !public.contains("private asynchronous")
                && !public.contains("registered")
        );
        assert_eq!(log(&req)?.len(), expected_len);
    }
    let mut req = Request::new();
    assert_eq!(
        emitter
            .emit("event", &mut req, &mut Response::new())
            .await?,
        DispatchOutcome::Completed
    );
    assert_eq!(
        log(&req)?,
        [
            "before:one",
            "action:one",
            "after:one",
            "before:two",
            "action:two",
            "after:two"
        ]
    );
    assert_eq!(Status::lookup(309).expect("legacy code").message, "Aborted");
    assert_eq!(Status::from_code(9999).message, "Unknown");
    assert!(Status::find("OK").is_some());
    Ok(())
}

/// Diagnostic registration metadata rejects terminal controls before publishing a listener.
#[test]
fn registration_metadata_rejects_controls_atomically() -> Result<()> {
    let mut builder = EmitterBuilder::new();
    // Label and plugin IDs can reach host reports; neither may inject terminal controls.
    assert!(
        builder
            .listen(ListenerSpec::exact("safe").label("bad\nlabel"), labeled)
            .is_err()
    );
    assert!(
        builder
            .listen(
                ListenerSpec::exact("safe").plugin("bad\u{1b}plugin"),
                labeled
            )
            .is_err()
    );
    assert_eq!(builder.build().listeners().count(), 0);
    Ok(())
}
