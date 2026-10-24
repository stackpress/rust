# Slice 2 design: lifecycle, failures and diagnostics

Proposed Q05/Q14 contract for [task 00005](tasks/00005-slice-2-server-and-cli.md). Load when reviewing top-level versus raw dispatch, aborts, finalization, recovery and retained failures. [Upstream findings](slice-2-design-upstream-host.md#precise-upstream-lifecycle-findings) establish the source differences; owner approval is still required.

## Entry and result types

```rust
pub enum LifecycleStage { Input, Request, Operation, Response, Error, Output, Dispatch }
pub enum LifecycleOutcome {
    Completed,
    Aborted { stage: LifecycleStage },
    Failed { stage: LifecycleStage },
    Finalized { stage: LifecycleStage },
}
pub enum ErrorHandling { NoListener, Completed, Aborted, Failed }
pub struct CorrelationId { /* checked host-local sequence, optional configured instance prefix */ }
pub struct StageFailure {
    pub stage: LifecycleStage,
    pub error: Arc<stackpress_lib::Error>,
}
pub struct HandleReport {
    pub correlation_id: CorrelationId,
    pub outcome: LifecycleOutcome,
    pub stages: Vec<StageRecord>,
    pub failures: Vec<StageFailure>,
    /* error offer results, selected operation outcome, output eligibility */
}
pub struct HostReply {
    pub response: Response,
    pub report: HandleReport,
}
pub struct HandleOptions {
    pub adapter_context: Option<Arc<dyn Any + Send + Sync>>,
}

Server::handle(&self, event: &str, req: &mut Request, res: &mut Response)
    -> impl Future<Output = Result<HandleReport>> + Send
Server::call(&self, event: &str, input: impl IntoRequest)
    -> impl Future<Output = Result<HostReply>> + Send
Server::handle_route(&self, req: &mut Request, res: &mut Response)
    -> impl Future<Output = Result<HandleReport>> + Send
Server::handle_with_options(&self, event: &str, req: &mut Request, res: &mut Response,
    options: HandleOptions) -> impl Future<Output = Result<HandleReport>> + Send
Server::emit(&self, event: &str, req: &mut Request, res: &mut Response)
    -> HandlerFuture<'_, Result<DispatchOutcome>>
Server::resolve(&self, event: &str, input: impl IntoRequest)
    -> HandlerFuture<'_, Result<Resolution>>
Server::resolve_route(&self, method: Method, path: &str, input: impl IntoRequest)
    -> HandlerFuture<'_, Result<Resolution>>
```

The latter lifetime sketches abbreviate the shared borrow lifetime used by the existing lib API. `handle` mutates the exact caller pair and never performs native output; it defaults to empty HandleOptions. `handle_with_options` carries trusted adapter metadata separately from caller input, with typed access through the server Invocation extension. `call` owns conversion/fresh response and retains limits/private diagnostics by returning a Response, not merely an envelope. Input conversion belongs inside the top-level report boundary. `handle_route` derives `METHOD pathname` from normalized Request, ignoring query/headers/data as event selectors.

`Server::emit/resolve/resolve_route` are raw dispatch with host context, without lifecycle, automatic loading, default status or error-event fallback. They return the existing lib results. Their root context has stage Dispatch and no invented top-level correlation; `Invocation::emit/resolve` inherits the caller's actual stage/correlation/failure context. Nested errors annotate the deepest listener path and return to the parent normally. A child Abort only propagates if the parent returns Flow::Abort.

G17 clarifies the pre-normalization boundary: when `call` input conversion or adapter normalization fails before a normalized `Request` exists, allocate the correlation, retain an Input failure, and return a fresh sanitized status-400 response carrying the input error's public view. Run no callbacks and never fabricate a Request. Once a normalized Request exists, load/body-parse failures use the full Input error offer and optional Response table below. The shared crate-private Input runner keeps adapter loading inside this single lifecycle.

## Top-level state machine

Allocate a correlation before normalization/load and validate event length before handler entry. `handle` finalizes an already buffered Request using its existing load state; HTTP decoding supplies an adapter load operation through the same internal Input stage runner. The public HTTP API must not create a second lifecycle to offer load failures.

| Stage result | Next action while Response is Open |
| --- | --- |
| Input success | Request event. |
| Input error | Offer error; skip Request and Operation; optionally Response once under the error table below. |
| Request Completed or NoListener | Selected Operation. Missing optional lifecycle listeners are normal. |
| Request Abort | Stop all remaining lifecycle stages; preserve deliberate response; no synthetic not-found. |
| Request error | Offer error; skip Operation; optionally Response once. This deliberately avoids running a capability after a failed guard/preprocessor. |
| Operation Completed or NoListener, Body::Empty and status None | Synthesize public 404, offer error, then optionally Response. This includes a matched no-op with no response. |
| Operation Completed or NoListener, any explicit body or status | Preserve it; select 200 only if status is None; enter Response. A deliberate redirect/status-only reply is valid. |
| Operation Abort | Stop all remaining lifecycle stages, including Response; preserve response; no synthetic not-found. Fix the upstream discarded-result behavior explicitly. |
| Operation error | Offer error; optionally Response once. Never retry Operation. |
| Response Completed or NoListener | End lifecycle. |
| Response Abort | End lifecycle with Aborted at Response; do not undo its mutations. |
| Response error | Offer error once; end lifecycle; never re-enter Response. |

Typed body existence is `Body::Empty` versus any other variant; empty text/bytes, false, zero and explicit null are valid bodies. A nonempty body without status defaults to 200. Ordinary successful Operation with neither body nor status gets 404; early cooperative Abort with neither gets 204 at final adapter normalization. An aborted body without status gets 200. A response hook intentionally removing/replacing all output ends with 204 if empty/unset, rather than rerunning Operation or missing-result detection.

The report records actual stage/outcome observations in order, including missing listeners; it does not confuse lib before/action/after phases with host Request/Operation/Response/Error stages. No automatic response-finally behavior or hidden asynchronous cleanup is implied.

Report outcome precedence is explicit: any recorded failure yields Failed at the first failing stage, even if an error listener later deliberately recovers public output; otherwise an observed abort yields Aborted, a skipped already-finalized entry yields Finalized, and a complete normal path yields Completed. Later aborts/finalization and secondary failures remain in ordered stage records. CLI/wire success uses the independent final response/error-handling/output policy, never this diagnostic outcome alone.

## Error offer and nonrecursive fallback

For each caught Result error, retain `Arc<Error>` in the report **before** touching response state. Create an Error-stage CallContext containing that StageFailure; `inv.failure()` identifies the original failing stage and error while `inv.stage()` reports Error. Existing listener frames retain plugin, label/ID, matcher, priority, caller/replacement site and before/action/after phase.

If the response is Open, replace it with a fresh Response carrying the same limits and the error's public view. This intentionally discards partially built body, headers, sessions, response data, count and disclosed stack; an error handler can deliberately reconstruct safe output. It prevents a prior success body or cookie from leaking through a later failure. The original response values are dropped normally, with no rollback promise for external effects. This policy needs owner approval; it does not change lib `apply_error` semantics.

The report owns original diagnostics independently of response recovery. A small server-private error-source wrapper around Arc<Error> lets the new response retain a normal source chain without making Error Clone. Response mutation/replacement by an error listener cannot erase report failures.

| Error-event outcome | Host action |
| --- | --- |
| NoListener | Record unhandled offer; retain sanitized original public error; run Response once if it has not run and output remains Open. |
| Completed | Record processed offer; permit Response once if not already run. Completion alone is not proof of recovery; final public response decides output. |
| Aborted | Record aborted offer; stop remaining lifecycle, including Response; adapter may still output the chosen error/recovery response. |
| Err(secondary) | Retain original and secondary with their own frames/stages; stop all hooks; terminal fallback. Never automatically emit error for the secondary failure. |

If constructing the public error response fails (e.g. public validation details exceed limits), retain that failure too and use the same terminal fallback; do not lose the original or ask the error pipeline to repair its own failed setup. The error stage is not a catch around itself. A later Response failure can receive a new error offer; it cannot trigger another Response pass. The state machine therefore has a finite number of automatic offers even if every stage fails.

Terminal fallback on an Open response uses a fresh status-500 response with generic public text if budgets permit; otherwise status 500 with empty body. It drops all partial recovery headers/cookies/body. No original/secondary message, source file, backtrace, request/session data, or captured value is included in wire fallback. `HandleReport` preserves both errors. If creating this minimal state unexpectedly fails, return the failure report and let the adapter emit its fixed empty 500 fallback before any output attempt.

Explicit successful recovery is possible by replacing the error response with a new successful Response in the error handler. Merely setting status 200 or results while an error field remains does not count as recovery. The report still records the failed stage and error-offer outcome. CLI success after recovery requires final successful status, no public error and no terminal/delivery failure. It is not inferred from the dispatcher's Completed value.

## Finalized, attempted and failed responses

`delivery_state()`, not `sent()`, is authoritative. Inspect it at entry and after every stage/offer. It can also change inside a handler, where ordinary lib dispatch still governs per-listener abort/error behavior; the host can only inspect state after the event returns. Handlers that finalize should return Flow::Abort to stop later listeners immediately. Do not secretly change lib's per-listener dispatch semantics.

| Delivery state | Host/adapter rule |
| --- | --- |
| Open | Lifecycle may mutate; adapter may attempt output once after host completion. |
| Sent | Skip later lifecycle and adapter write; preserve existing output. |
| Stopped | Skip later lifecycle and output; explicit suppression. No fabricated transport success body. |
| Attempted | Incomplete/cancelled attempt; skip hooks/write, return failure diagnostic; never retry. |
| Failed | Prior output may be partial; skip hooks/write, return failure diagnostic; never retry. |

An error returned after finalization is retained but does not mutate the response or run error listeners against frozen state. Error-listener failure after it finalized output preserves both errors and never sends again. For a buffered HTTP adapter, an independent lib `dispatch()` marks no native resource as produced; record an adapter-state failure and return HttpOutput::Unavailable rather than manufacture a second HTTP output. `stop()` similarly returns an explicit suppressed result. The HTTP return type represents suppression/previous delivery; see [adapters](slice-2-design-adapters.md).

An exclusive `&mut Response` also permits safe Rust whole-value replacement. Its delivery guard therefore applies to that value instance; it cannot prove that arbitrary user code never wrote elsewhere before replacing it. The native adapter retains its own private one-attempt output state and writer outside handler reach, independent of any replaced Response. M0 must test replacement before/after low-level finalization and show adapter output remains at most once. Callbacks holding their own native writer/global stdout can perform external effects; the framework cannot roll those back or detect them through the current Response. Do not claim the immutable-field guard prevents whole-value substitution.

Cooperative abort is different from suppression: Open aborted output is still eligible for one adapter write, matching the source adapter's separation. Output conversion/write is outside the lifecycle. A serialization/native writer failure is Stage Output, is retained, and never reruns error/response hooks or native output. Preflight may choose one fixed sanitized 500 before beginning the sole delivery attempt; partial writes cannot be repaired by another response.

Dropping any handling/output future stops future work and preserves completed mutations/attempt states. Panic unwinding propagates; the framework does not catch arbitrary panics or claim panic=abort recovery. No background task exists to finish dropped work. Tests must demonstrate a subsequent independent invocation still works without framework poisoning.

## Correlation and disclosure

Use a checked per-Server AtomicU64 sequence, starting at one, allocated once for each top-level call. Sharing the Server with explicit Arc shares that allocator visibly; overflow returns Err before any handler runs. Optional configured instance prefix is bounded and printable. The default ID is unique within that host, **not globally unique across hosts/processes**, an authentication token, or distributed tracing. No raw incoming header can replace it. Cross-process propagation/tracing remains a separate Q14 extension.

Handle's outer Result is for inability to start/report (such as sequence exhaustion); normal input/handler/error-listener failures are represented in HandleReport with the chosen Response. Error Display/developer formatting must include correlation, host stage, selected event and failing listener/plugin/site when available. Errors before a listener runs honestly have no listener/plugin; never invent one.

Reports retain original Error/source/backtrace/path data. Automatic public output is only the existing public envelope. Default report Display names metadata without serializing Request, Response data, session, capture values or service/config contents. Developer cause detail is an explicit formatter/accessor; no automatic logging sink/subscriber is installed. Raw URLs in diagnostics use pathname only, without query values. Event strings are bounded; caller-provided labels/IDs are validated against control characters to prevent terminal log injection.

No tracing dependency is needed in this slice. A future optional instrumentation feature can consume stage reports without changing baseline behavior; it needs async span instrumentation, never an entered guard held across await. Remote request IDs, traceparent validation and trust policy are deliberately unresolved beyond local correlation.

The [verification plan](slice-2-design-verification.md) requires decision-table, dual-failure, partial-output, cancellation and concurrent-correlation tests. The [decision checkpoint](slice-2-design-decisions.md) lists the owner choices before M0.
