# Design: public API and execution contract

Spec file of [00001 Stackpress Rust foundation](index.md). Load for Slice 1 package layout, handler/registration/ownership signature, execution semantics, and matcher grammar (design sections B1–B4). Moved losslessly from `docs/trial-3/slice-1-design.md` lines 187–446 at commit `e230359`; headings are demoted one level and links rebased.

### B. Proposed public API and behavioral contract

#### B1. Package/module layout and dependency policy

```text
Cargo.toml                    workspace + stackpress facade, resolver 3
Cargo.lock                    one reviewed workspace resolution
src/lib.rs                    explicit lib alias and selected primitive re-exports
crates/lib/Cargo.toml          stackpress-lib
crates/lib/src/lib.rs          explicit public exports
crates/lib/src/data/           Value, Nest, paths, buffered parsers/files
crates/lib/src/queue.rs        ordered items, task flow
crates/lib/src/event.rs        registration, invocation, dispatch, resolve
crates/lib/src/pattern.rs      exact/Express/regex and neutral route matching
crates/lib/src/request.rs      neutral request construction/load state
crates/lib/src/response.rs     neutral response/envelopes/output-attempt state
crates/lib/src/session.rs      revisions, cookies and codecs
crates/lib/src/headers.rs      checked case-insensitive multi-value headers
crates/lib/src/body.rs         owned buffered body representations
crates/lib/src/status.rs       code/message catalog
crates/lib/src/error.rs        errors, source locations/spans and diagnostic views
crates/lib/tests/              contract tests, compile-fail cases, watchdog cases
```

Use edition **2024**, `rust-version = "1.96"`, and resolver **3**. This is the owner-selected installed stable toolchain and a directly verifiable baseline. The 1.85 proposal is superseded; no older-compiler support claim or additional toolchain installation is in scope. Dependencies must support the declared 1.96 floor.

Candidate production dependencies: `serde`/`serde_json` for validated numbers and envelopes, `indexmap` for explicit ordered maps/sets, `regex` for bounded non-backtracking regex, `url` for URL values, and small percent-encoding/HTTP-date facilities if needed. Select exact compatible versions from the resolved manifests during implementation, recording MSRV and target constraints. No Tokio, executor, filesystem, network, database, UI, Tauri, dynamic loading or automatic logging dependency in production. Async test execution/timeouts can use Tokio and futures as dev-dependencies. No optional placeholder crate/features and no assumed all-target parity.

The facade explicitly exposes `stackpress::lib` and common imports such as `stackpress::{EmitterBuilder, EventEmitter, Request, Response, Flow, Error}`. Direct `stackpress_lib` use remains supported; neither foundation nor later libraries depend back on the facade.

#### B2. Registration, ownership and async signature

Recommended core handler shape:

```rust
async fn operation(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow>;

pub enum Flow { Continue, Abort }
pub enum DispatchOutcome { Completed, Aborted, NoListener }
pub enum ListenerPhase { Before, Action, After }
pub type Result<T> = std::result::Result<T, Error>;
```

`Invocation` is a borrowed immutable view of the dispatch table and invocation-local metadata. It does not implement Clone. Handlers borrow req/res exclusively and return `Result<Flow>`, permitting `?` without confusing error and cooperative abort. **Dispatch futures are Send now**, as the owner requires. The runtime remains executor-independent.

Named async functions register directly. A lifetime-indexed internal helper trait captures the returned future type without unstable AsyncFn associated-type syntax:

```rust
trait HandlerFn<'a> {
    type Future: Future<Output = Result<Flow>> + Send + 'a;
    fn call(&'a self, req: &'a mut Request, res: &'a mut Response,
        invocation: Invocation<'a>) -> Self::Future;
}
// Blanket implementation for Fn(&'a mut Request, &'a mut Response, Invocation<'a>) -> Fut.
// Fut must itself be Send + 'a. Requiring only the callable to be Send is insufficient.
```

The implemented simple public registration bound is `impl Handler`, with private lifetime-indexed adaptation and object-safe erasure to `HandlerFuture<'a, Result<Flow>>`. Internally require `H: for<'a> HandlerFn<'a> + Send + Sync + 'static`. The queue uses the equivalent sealed `TaskHandler<C>` bound. This contains generic/lifetime machinery inside the framework and requires no trait implementation from plugin authors. No unstable `AsyncFn::CallRefFuture`, unsafe lifetime extension, raw pointers or mutex-wrapped FnMut callbacks.

Closures use a small documented identity helper that supplies the higher-ranked expected type; a plain borrowing closure without that context is not promised to infer correctly:

```rust
fn handler<F>(callback: F) -> F
where
    F: for<'a> Fn(&'a mut Request, &'a mut Response, Invocation<'a>)
        -> HandlerFuture<'a, Result<Flow>> + Send + Sync + 'static;

builder.on("feature-run", handler(|req, res, invocation| Box::pin(async move {
    invocation.emit("child", req, res).await?;
    Ok(Flow::Continue)
})))?;
```

No macro is needed. Owned captured configuration is copied into each returned future explicitly; an Arc capture is visibly shared. Named async fns are the recommended path. M0 must prove all three arguments, nested emit from named/helper handlers, mutable req/res across await, captured configuration, `?`, a genuinely Send future and execution in `tokio::spawn` with owned invocation inputs. A spawned future must own or retain Arc ownership of its table and values: a raw borrowed `emitter.emit(...)` is not automatically `'static` merely because it is Send. Rejection of a non-Send future must be proved as well. Any failure blocks this contract with compiler evidence.

Registry operations:

```rust
EmitterBuilder::new() -> EmitterBuilder
EmitterBuilder::on(&mut self, event: &str, handler: impl Handler)
    -> Result<ListenerId>                          // exact, priority 0
EmitterBuilder::on_priority(&mut self, event: &str, priority: i64, handler: impl Handler)
    -> Result<ListenerId>
EmitterBuilder::listen(&mut self, spec: ListenerSpec, handler: impl Handler)
    -> Result<ListenerId>                          // pattern, priority, label, plugin
EmitterBuilder::on_pattern(&mut self, pattern: EventPattern, handler: impl Handler)
    -> Result<ListenerId>
EmitterBuilder::route(&mut self, method: MethodPattern, path: &str,
    handler: impl Handler) -> Result<ListenerId>
EmitterBuilder::before(&mut self, handler: impl Handler) -> &mut Self
EmitterBuilder::after(&mut self, handler: impl Handler) -> &mut Self
EmitterBuilder::remove(&mut self, id: &ListenerId) -> Result<bool>
EmitterBuilder::replace(&mut self, id: &ListenerId, handler: impl Handler)
    -> Result<()>                                 // preserve identity/order/matcher/priority
EmitterBuilder::clear(&mut self, pattern: &EventPattern) -> usize
EmitterBuilder::extend(&mut self, other: EmitterBuilder) -> Result<()>
EmitterBuilder::build(self) -> EventEmitter

EventEmitter::listeners(&self) -> /* borrowed read-only descriptors */
EventEmitter::matches(&self, event: &str) -> Result<Vec<EventMatch>>
EventEmitter::emit(&self, event: &str, req: &mut Request, res: &mut Response)
    -> /* Send Future<Output = Result<DispatchOutcome>> borrowing all inputs */

Invocation::emit(&self, event: &str, req: &mut Request, res: &mut Response)
    -> /* Send Future<Output = Result<DispatchOutcome>> with inherited path */

EventEmitter::resolve(&self, event: &str, input: impl IntoRequest)
    -> /* Send Future<Output = Result<Resolution>> */
Invocation::resolve(&self, event: &str, input: impl IntoRequest)
    -> /* Send Future<Output = Result<Resolution>> with inherited path */
pub struct Resolution {
    pub outcome: DispatchOutcome,
    pub response: ResponseEnvelope,
}
```

All registration entry points capture the actual caller through synchronous `#[track_caller]` wrappers, including helper-to-listen forwarding; compile-time call-site tests protect this. Every listener has an opaque registry-qualified ID, default human-readable ID/site description, optional explicit label/plugin, priority, original matcher and registration location. IDs are not reused; checked allocation rejects exhaustion. Foreign removal IDs cannot remove unrelated registrations.

The builder owns callbacks and changes only through `&mut self`. Build consumes it into an immutable table, with no runtime registry locks and no public `Clone` implementation. `extend` consumes/moves another builder; it does not silently share callback state or rebuild compiled patterns from display strings. Incoming registrations follow existing registrations for equal priorities, keeping their internal order and original sites, with recipient dispatch order distinct from identity. Receiver hooks apply; source hooks are not implicitly imported, matching the upstream listener-composition distinction. Metadata inspection cannot expose writable callback storage.

`replace` is a deliberate construction-time customization operation, not a second listener racing the first by priority. It validates the handle before mutation, preserves ID/matcher/priority/position, and records the replacement call site while retaining original registration provenance. Failed or foreign/stale replacement is an error and changes nothing. `remove` followed by new registration obtains a new ID and order. No implicit replacement by label/function name, function-pointer equality, or duplicate-registration order. Labels need not be globally unique. The core API needs no control-flow macro; type erasure and lifetime machinery remain inside the framework.

This intentionally restricts upstream clear/unbind/register to construction time. Reconfiguration builds a new table under later host ownership. Dynamic plugin enable/disable is not smuggled into runtime dispatch.

Example plugin and invocation (proposed API):

```rust
use stackpress::lib::{
    DispatchOutcome, EmitterBuilder, Flow, Invocation, Request, Response, Result, Status, ListenerSpec, handler,
};

async fn profile_search(
    req: &mut Request,
    res: &mut Response,
    _invocation: Invocation<'_>,
) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_else(|| "unknown".into());
    res.results(&serde_json::json!({ "id": id }))?;
    Ok(Flow::Continue)
}

fn register(builder: &mut EmitterBuilder) -> Result<()> {
    builder.on("profile-search", profile_search)?;
    builder.on_priority("profile-search", 10, handler(|req, _res, _invocation| Box::pin(async move {
        req.data.set("observed", true)?;
        Ok(Flow::Continue)
    })))?;
    builder.listen(ListenerSpec::exact("profile-search").priority(5).label("audit"), profile_search)?;
    Ok(())
}

async fn call() -> Result<()> {
    let mut builder = EmitterBuilder::new();
    register(&mut builder)?;
    let emitter = builder.build();
    let mut req = Request::builder().data(serde_json::json!({ "id": "abc123" })).build()?;
    let mut res = Response::new();
    let outcome = emitter.emit("profile-search", &mut req, &mut res).await?;
    assert_eq!(outcome, DispatchOutcome::Completed);
    assert_eq!(res.status(), Some(&Status::OK));
    Ok(())
}
```

All final Rustdoc examples will be complete standalone consumers, including serde_json when used. This checkpoint does not claim doctest success.

Nested emit uses the invocation context rather than closing over a mutable emitter:

```rust
builder.on("checkout", handler(|req, res, invocation| Box::pin(async move {
    // Child dispatch inherits this listener's path and reborrows the values.
    let child = invocation.emit("payment-charge", req, res).await?;
    // A child's cooperative abort is a result the parent explicitly handles.
    if child == DispatchOutcome::Aborted {
        return Ok(Flow::Abort);
    }
    // The parent invocation metadata is unchanged after the child returns.
    assert_eq!(invocation.event(), "checkout");
    Ok(Flow::Continue)
})))?;
```

Concurrent child dispatch is allowed with two distinct owned req/res pairs, using `join!`/`try_join!` in the caller. Each child receives its own immutable path and captures. Concurrent borrowing of the same mutable req/res pair is a compile error; an async mutex is not introduced to disguise that constraint. No child is spawned or detached by the foundation. Explicit cross-table dispatch can call another emitter, but only `invocation.emit` promises inherited breadcrumbs in this slice; a later explicit host context can carry correlations across tables/processes.

M2 represents ancestry as an explicitly named `SharedAncestry` Arc-linked immutable parent chain. Creating a sibling/child copies the parent handle and adds its current frame, without copying every ancestor string. `Invocation::path()` and error annotation materialize an owned root-first diagnostic vector only when requested. No mutable global/thread-local current invocation exists. The production Send/nested/spawn/compiler proofs now use the real API; `tests/support/signature.rs` contains only production re-exports.

`resolve` is a first-class capability call, preserving the common [00019](../../references/00019-upstream-runtime-api-contracts.md)/[00030](../../references/00030-upstream-operational-examples.md) shape. The sealed `IntoRequest` input adapter accepts an owned `Request`, `Nest`, or object-root `serde_json::Value`. Plain data becomes `Request::builder().data(input).build()`; an owned Request is moved with its current state, never cloned. It creates a fresh Response, dispatches without a top-level lifecycle, and moves the response into an envelope. A nested `Invocation::resolve` inherits the path and uses a new request/response pair, so it can run concurrently with another child resolve. Retain `emit` when mutation of the caller's exact req/res is desired.

```rust
let created = emitter.resolve("article-create", serde_json::json!({
    "slug": "first-post", "title": "First Post",
})).await?;
assert_eq!(created.outcome, DispatchOutcome::Completed);
assert_eq!(created.response.code, Some(200));
let article = created.response.results;

builder.on("publish", handler(|_req, res, invocation| Box::pin(async move {
    let created = invocation.resolve("article-create", serde_json::json!({
        "slug": "first-post", "title": "First Post",
    })).await?;
    if created.outcome == DispatchOutcome::Aborted {
        return Ok(Flow::Abort);
    }
    res.apply_envelope(created.response)?;
    Ok(Flow::Continue)
})))?;
```

The illustrative `article-create` listener is not a generated model implementation in this slice. `ResponseEnvelope` exposes `code`, `status`, `results`, `errors`, `total`, public `error` and an explicitly disclosed optional `stack`. `Resolution.outcome` remains separate: no-listener does not pretend to be an HTTP 404 response, nor does completed dispatch manufacture a 200 when no handler set a response status. Ordinary failures remain `Err(Error)` with their cause/path rather than a silently swallowed error envelope. This small Rust wrapper makes both decisions inspectable. Field conversion to JSON/remote output is explicit; in-process envelopes can retain the richer Value model. Method/path overloads, caller authorization, host services, and automatic lifecycle/error fallback remain Slice 2.

#### B3. Execution semantics

1. Validate bounded event input and collect **all** matching registrations once. Sort the collected listeners by descending signed integer priority, then ascending registration sequence across all matcher kinds. Exactness/specificity adds no hidden priority. Registering the same callback twice runs it twice.
2. For each listener, derive immutable listener-local captures and an invocation frame. Execute before, action, after **sequentially**, awaiting each phase. Before abort skips action/after; action abort skips after; after abort skips later listeners. A phase error skips remaining phases/listeners and returns the original error with diagnostic context. After is not a finally hook.
3. No matches yields `NoListener`; completion yields `Completed`; cooperative abort yields `Aborted`. None sets response status, auto-sends a body or emits general lifecycle events. No-listener can coexist with a response status explicitly supplied by the caller.
4. A child error propagates through `?` retaining the deepest frame/cause. A child abort does not automatically abort its caller. Partial req/res and external side effects remain; dispatch is not a transaction. Dropping a future stops future phases without rolling back completed mutations, and emits no async cleanup hooks.
5. Recursive dispatch is supported and boxed at the internal recursion boundary. A configurable depth limit, proposed default 64, rejects overflow before entering the next child. Sibling invocations do not consume a shared global depth counter. Error paths are bounded and owned; the library cannot prevent user code from looping or blocking inside a handler.

No top-level `handle` is implemented here. The future server host must separately settle sent-response handling, lifecycle aborts/error fallback and output mapping. Naming an event `request`, `response` or `error` in Slice 1 registers an ordinary event; the dispatcher does not recursively wrap every child in those hooks or catch its own errors by emitting `error`.

Likewise `config` → `listen` → `route` and generation's `idea` conventions belong to Slice 2 orchestration, as [00012](../../references/00012-upstream-architecture-and-composition.md)/[00024](../../references/00024-upstream-cli-and-plugin-contracts.md) describe. An immutable **runtime** table does not require an immutable bootstrap: `TaskQueue<StartupContext>` can run ordered startup work with a mutable registration accumulator, then the host builds the runtime table after contributions finish. This is an available primitive, not a selected Slice 2 implementation or an automatic lifecycle inside emit/resolve. Do not preclude later host-owned stage/correlation context or require a global mutable registry.

Bootstrap mapping example (documentation/test only; no server crate in Slice 1):

```rust
// Upstream: ctx.on("listen", ({ ctx }) => ctx.on("feature-run", feature_run)).
fn feature_plugin(builder: &mut EmitterBuilder) -> Result<()> {
    // The host calls this registration function during its listen phase.
    builder.on_priority("feature-run", 10, feature_run)?;
    Ok(())
}

fn bootstrap() -> Result<EventEmitter> {
    let mut builder = EmitterBuilder::new();
    // Host-owned config/listen/route sequencing finishes before build.
    feature_plugin(&mut builder)?;
    Ok(builder.build())
}
```

A later host may schedule `feature_plugin` through `TaskQueue<StartupContext>` rather than call it directly. Contributions modify a construction-time builder; operations receive the immutable finished table. Runtime emit/resolve never runs bootstrap automatically. Lifecycle coordination is Slice 2; actual Idea parsing/transforms/generation are Slice 3.

`ItemQueue<T>` has `add(item, priority)`, `push_back(item)` below the retained priority minimum, `push_front(item)` above the retained maximum, `pop`, `len`, `is_empty` and read-only ordered iteration. Use checked priority/sequence arithmetic, never negation/subtraction comparisons that can overflow. Front/back operations return `Result` at extremes. Preserve upstream retained high/low watermarks while a queue exists; document that draining does not silently reset them.

`TaskQueue<C>` accepts borrowing async callbacks over `&mut C`, uses the same priority policy, and `run(&mut self, &mut C) -> Result<QueueOutcome>` drains one task before its before/action/after sequence. `QueueOutcome::{Completed,Aborted,Empty}` stays separate from response status. Abort/error/cancellation consume the selected task, retain unstarted tasks, and do not replay side effects. Before/after are optional single callbacks; hook error/abort semantics match events. Callback erasure follows the same Send-future contract as handlers.

#### B4. Matcher grammar and captures

- `EventPattern::exact(text)` always means literal bytes. `on("event", ...)` uses exact matching; `on_pattern(EventPattern::express(...), ...)` is the explicit patterned form. This avoids interpreting a literal operation name containing `*`, `:` or regex punctuation accidentally.
- Express-style patterns use a declared nonempty single-character separator (default `/`, including custom space or `-` use). A `:name` token accepts the upstream ASCII alphanumeric/underscore/hyphen name vocabulary and captures one or more non-separator characters. Duplicate parameter names are rejected at construction; adjacent ambiguous tokens and invalid star runs are rejected rather than guessed. `*` captures one nonempty segment; `**` captures zero or more characters, including separators. Literals outside those tokens are regex-escaped. Patterns match the whole event.
- `MatchData.params` contains named parameters; `args` contains positional wildcard captures in token order. Double-star captures are split on the declared separator (including documented empty edge segments), replacing upstream's hard-coded `/` split. Regex positional captures retain their original captured text; optional unmatched groups remain `None`, not a fabricated empty string. Named regex groups are also exposed in `params`.
- Raw regex uses Rust `regex` syntax with explicit options such as case-insensitive matching; anchors are caller-controlled. Use the first match/captures per registration, not one invocation per global occurrence. No mutable regex cursor/global `lastIndex`; repeated emits are identical. JS `g` is thus unnecessary; arbitrary ECMAScript lookaround/backreferences and JS flag identity are intentionally unsupported. Compilation rejects unsupported syntax with pattern/site diagnostics and bounded compiled size.
- A neutral route matcher combines `MethodPattern::{Exact(Method),Any}` with an Express-style path and retained route metadata. Request verbs are normalized/validated tokens and extension verbs are allowed. Any matches a nonempty method token, not the fictional request verb ALL/ANY. Parameterized/Any routes tolerate trailing separators as upstream does; an exact literal route remains strict. `GET /users/42` is still an event value, not a transport endpoint. Percent decoding and URL security policy remain outside the matcher.
- Captures remain per listener in `Invocation::captures()`. Raw emit does not merge captures into request data. Slice 2's router adapter can inject them deliberately with an explicit precedence rule; this prevents different pattern listeners or nested events silently contaminating one another's req.data.
- `matches` preserves one descriptor per registration in execution order, including duplicate listeners. It does not return a display-string-keyed map that can collapse distinct capture-name definitions with equivalent compiled regex. Matches with zero active listeners cannot survive builder removal/clear.
