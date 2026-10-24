# Concepts

An operation should keep its business rules when a caller changes from a command to HTTP. The host
supplies shared resources and a finite lifecycle around the same Lib handler. Keep [Getting
started](getting-started.md) as the small working example while reading these rules.

## Selection before startup

A plugin is a compiled `fn(&mut ServerBuilder) -> Result<()>`. Its ID identifies a selected
contribution; it is neither a package loader nor a version requirement. `requires` demands a
selected predecessor, while `after_if_present` constrains a predecessor only when it is selected.
The host checks the whole graph before calling any registration function and rejects missing
requirements, self-dependencies, cycles, duplicate IDs, and wrong order. It never reorders plugins.

Direct builder contributions exist before queued plugin callbacks run. Plugins then contribute in
supplied order. Higher listener/task priority runs first; equal priorities retain actual
registration order. For exact ordering of related contributions, one application plugin is often
easiest to read. Omitting registration does not remove compiled code; Cargo `cfg`/features must
exclude that code.

## Defaults, layers, and typed sections

A plugin claims a literal top-level section name and a concrete Rust type with `config_section`.
Serialized defaults form the lowest layer. Explicit layers apply afterward in supplied order:
objects merge recursively; arrays, scalars, and null replace earlier values. Null is a value, not
deletion. This config merge is separate from Lib's shallow `Nest::merge`.

For example, defaults `{"profiles":{"limit":3,"enabled":true}}` followed by a layer
`{"profiles":{"limit":1}}` retain `enabled` and change `limit`. Each type is decoded and validated
once before any startup callback. Section names and concrete types are each unique; newtypes
distinguish roles. Unclaimed top-level sections fail the build. Field-level unknown-key policy
belongs to the owning serde type, commonly `#[serde(deny_unknown_fields)]`.

Layers are bounded data. Server code reads no environment, files, or current directory implicitly.
Application code explicitly converts those sources into layers; config strings cannot enable
plugins. See [typed configuration](guides/typed-config.md) for a complete example.

## Three startup phases

The host consumes three ordered task queues after registration and config decoding. Empty queues
succeed, and each task uses `async fn(&mut StartupContext) -> Result<Flow>`.

| Phase | Allowed work |
| --- | --- |
| Config | Read typed config; insert, name, or replace services. |
| Listen | Read config/services; register or customize ordinary operation listeners. |
| Route | Read config/services; register or customize host routes. |

Direct registration can add both operations and routes before these phases. Config cannot register
listeners/routes; Listen cannot register host routes; Route cannot register ordinary listeners.
Services freeze after Config. Any startup error or Abort fails the consuming build: no partial
Server or retryable builder escapes. Dropping build stops future tasks and drops owned resources;
external effects and asynchronous cleanup remain application responsibilities.

Runtime events named `config`, `listen`, or `route` are ordinary events. Emitting them cannot reopen
startup. [Startup reference](api/startup.md) describes phase permissions and error provenance.

## Services are explicit shared ownership

A Config task registers `Arc<T>` by concrete type, or under an explicit name. Runtime lookup returns
`&T`; `shared_service` explicitly clones the Arc, preserving the same allocation. Neither operation
calls `T::clone`. Named slots and typed singleton slots have separate namespaces. Missing or
wrong-typed lookup returns an error; replacement requires an existing typed singleton.

The registry adds no runtime lock. A service may use its own synchronization, and `&T` permits the
interior mutability chosen by T. Applications own transaction, cancellation, and concurrency policy.
Host state contains no emitter back-reference; handlers obtain it through
[ServerInvocationExt](api/context.md), outside caller-controlled request data.

## One lifecycle per top-level call

`call` converts owned input; `handle` mutates your exact Request/Response pair; `handle_route`
derives `METHOD pathname` from a normalized Request. They enter the same Input → Request → Operation
→ Response path. Native conversion/delivery is a later Output stage owned by an adapter.

| Result | Next step while output remains Open |
| --- | --- |
| Input succeeds | Dispatch `request`. |
| Normalized Input fails | Offer `error`; skip Request/Operation; permit Response under the error rules. |
| Request completes or has no listeners | Run selected operation. |
| Request fails | Offer `error`; skip Operation even after successful recovery. |
| Request or Operation aborts | Stop remaining lifecycle, including Response; preserve chosen output. |
| Operation completes with neither body nor status | Retain a synthetic 404 and offer `error`. |
| Operation completes with explicit body/status | Supply 200 only if status is absent; enter Response. |
| Operation fails | Offer `error`; never retry Operation. |
| Response completes/aborts | End; retain its mutations. |
| Response fails | Offer `error` once; never run Response again. |

These are the accepted G12 rules. Body presence means a variant other than `Body::Empty`: null,
false, zero, and empty text/bytes count as supplied bodies. Final Open output with no status gets
204 if Empty, otherwise 200. Thus empty early Abort gets 204, while an ordinary no-op gets 404. A
response hook that deliberately clears output gets final normalization, not another missing-result
check. Optional Request/Response/Error listeners may be absent without causing dispatch failure.

G17 distinguishes failure before normalization: if `call` conversion or HTTP normalization cannot
produce a real Request, the host allocates correlation, retains Input failure, and returns fresh
public status 400. It invokes no callbacks and fabricates no Request. Once normalization succeeds,
load/body-parse failures use the ordinary error offer above.

## Error offers and deliberate recovery

A caught error is retained before the host replaces Open output with a fresh public error response
under the same limits. Partial success body, headers, cookies, response data, total, and disclosed
stack are discarded. During `error`, `inv.stage()` is Error and `inv.failure()` identifies the
original stage and Arc-owned error; nested dispatch inherits both.

| Error listener result | Consequence |
| --- | --- |
| NoListener | Preserve public error; permit one Response pass if not already run. |
| Completed | Permit Response; completion alone does not establish recovery. |
| Abort | Skip later hooks; Open output remains eligible for adapter delivery. |
| Error | Retain original and secondary; stop hooks; never recursively offer Error. |

Error-response setup failure follows the same terminal rule. While Open, terminal fallback is a
fresh sanitized 500 with generic text if budgets permit, otherwise empty output with status 500. A
later Response failure may get a separate Error offer, but cannot restart Response. To recover,
explicitly rebuild a successful Response; changing status/results while retaining its public error
is insufficient. [Errors and recovery](guides/errors-and-recovery.md) demonstrates that replacement.

## Delivery authority and cancellation

Lifecycle progress, response delivery state, and adapter output are different observations. The host
checks delivery at entry and after stages; Lib dispatch still controls listeners within a stage. A
handler that finalizes should return Abort to stop later matching listeners immediately.

| DeliveryState | Meaning to host and adapter |
| --- | --- |
| Open | Further lifecycle mutations and one adapter attempt are eligible. |
| Sent | Skip later host stages; a foreign send supplies no native value to this adapter. |
| Stopped | Explicit suppression; no fabricated native body. |
| Attempted | Interrupted/incomplete delivery; retain failure and never retry. |
| Failed | Output may be partial; retain failure and never retry. |

Abort does not suppress Open output. `stop()` does. A private adapter `OutputGuard` retains attempt
state even if a handler replaces its Response. Arbitrary application writers remain outside that
authority. Output conversion/write failure never re-enters lifecycle hooks. Dropping a future stops
future work and retains completed mutations/attempts; no hidden task finishes it. Handler panics
propagate. Cancellation and Rust ownership do not roll back external effects.

## Reports and correlation

Every top-level invocation reserves a checked per-Server sequence starting at one, optionally with a
configured prefix. Raw host dispatch has stage Dispatch and no correlation. Nested Invocation calls
inherit their parent's stage, correlation, trusted adapter metadata, and current failure. IDs are
local to one Server, may repeat across hosts, and are neither identity credentials nor distributed
trace IDs. Incoming data cannot replace them.

HandleReport records stages, raw operation outcome, Error offers, and original failures
independently from replaceable public output. First failure takes precedence over Abort, finalized
entry, and normal completion. A successful public recovery therefore still has diagnostic outcome
Failed. `is_success` separately requires status 200–399, no public error, no terminal failure, and
no Attempted/Failed delivery. It may succeed while output is Open; that is no native-delivery proof.
Default report formatting shows metadata; `developer()` explicitly exposes messages and causes.

## Raw dispatch and route captures

`Server::emit`, `resolve`, and `resolve_route` provide host lookups but no automatic input load,
status default, Error event, or lifecycle. Invocation `emit`/`resolve` are nested raw dispatch;
`delegate` maps child Completed to Continue, Abort to Abort, and NoListener to a public 404 error.
Use these inside a handler to avoid starting a second lifecycle. See the [Lib nesting
model](../../lib/docs/concepts.md#emit-resolve-and-nesting).

Host routes default to NamedIntoData: before each matched route action, named captures atomically
overlay literal keys in aggregate `req.data`. Route values win collisions; query/post stores remain
unchanged. Positional captures remain Invocation-only. Injection failures leave the prior aggregate
unchanged; later handler mutations, Abort, or cancellation do not roll earlier successful injection
back. `InvocationOnly` disables injection. Raw Lib listeners keep their existing capture behavior.

## The buffered HTTP boundary

Feature `http` consumes an already buffered `http::Request<Vec<u8>>`, selects only method/pathname,
and returns HttpOutput plus a report. Host/forwarded headers and request data cannot choose an
event. The adapter rejects targets whose URL normalization changes the path, so it cannot quietly
route a different resource. It parses supported structured bodies and preserves other bodies as
bytes.

It prepares native headers/status/body before its sole publication. Text and bytes stay exact;
structured/status-only replies use the public envelope. HEAD suppresses bytes without falling back
to GET. Suppression, foreign finalization, and native Response are explicit output variants.
Conversion failure yields one fixed empty 500 and retains Output failure. See [HTTP
reference](api/http.md) for budgets, framing, cookies, and representational limits.

The host now gives one operation shared resources and one lifecycle across supported entry paths.
Use the [task guides](README.md#common-tasks) to apply that model and the [API index](api/README.md)
to check a particular signature.
