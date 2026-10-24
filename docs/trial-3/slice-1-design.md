# Slice 0 + Slice 1 design checkpoint

Status: **approved with owner/coordinator changes; implementation authorized in milestones with explicit review stops**.
Prepared on 2026-10-24 in `trial/3`; revised after coordinator/owner approval. Design examples remain sketches until their milestone compiles and tests them; milestone receipts record actual verification separately.

## Authority, observations, and scope

The current task and [AGENTS.md](../../AGENTS.md) govern this proposal. The accepted owners are [project direction](../../.agents/context/project-direction.md), [runtime and portability](../../.agents/context/runtime-and-portability.md), [source structure](../../.agents/context/source-structure.md), [Rust style](../../.agents/context/rust-code-style.md), [quality policy](../../.agents/context/testing-and-quality.md), and the [programming workflow](../../.agents/workflows/rust-programming.md). I also read the required references [00039](../../.agents/references/00039-open-design-questions.md), [00034](../../.agents/references/00034-foundation-source-map.md), [00019](../../.agents/references/00019-upstream-runtime-api-contracts.md), and [00008](../../.agents/references/00008-discussion-diagnostics-existing-parser.md).

The coordinator addendum is incorporated using the relevant readability/extension/proof sections of [00006](../../.agents/references/00006-discussion-crate-plugin-proposals.md), the lib inventory/redesign boundaries in [00005](../../.agents/references/00005-discussion-feature-feasibility.md), capability-call examples in [00030](../../.agents/references/00030-upstream-operational-examples.md), lifecycle ownership in [00012](../../.agents/references/00012-upstream-architecture-and-composition.md)/[00024](../../.agents/references/00024-upstream-cli-and-plugin-contracts.md), and the full documentation/round references [00036](../../.agents/references/00036-rustdoc-commenting-examples.md)/[00037](../../.agents/references/00037-refactoring-round-checklists.md). Historical crate proposals do not override the accepted current tree.

Source observations use only this branch, the project KB, and the current sibling `lib` checkout. This branch was clean at inspection, at `90dca8737b00cf0ded9dfe2f4a34d6538c9513b8`. Upstream `lib` was clean at `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a`. No other `trial/*` branch or stale `target/` output was inspected. At initial design inspection there was no Cargo manifest or Rust implementation on this branch. Installed compiler and Cargo report 1.96.0; the owner selected this installed compiler as the MSRV. No additional toolchain will be installed.

Slice 0 will create only the root workspace/package `stackpress`, its `src/lib.rs` facade, and `crates/lib` (`stackpress-lib`), with one lockfile. Slice 1 will implement transport-neutral data/parsing, ordered queues, event matching/dispatch, request/response/session values, status, and diagnostics. This document adds no empty crates.

Named later work below means a responsibility and acceptance boundary, not a scheduled commitment:

- **Slice 2 — capability host + CLI adapter:** `stackpress-server` and CLI in `tools/`; configuration/plugin/service composition, top-level lifecycle, native transport envelopes, output/streaming/routing adapters, Terminal/control/input and process/exit behavior. Neutral `resolve(event, input)` is in Slice 1.
- **Slice 3 — Idea/generation:** existing parser assessment, transformer, generation, host filesystem/FileLoader/NodeFS capabilities and templates. No new parser is proposed here.
- **Slice 4 — SQL:** independent query/dialect/backend capabilities and framework integration as implemented.
- **Slice 5 — UI:** shared Leptos UI and Tauri adapters, with their own target proofs.

## A. Upstream inventory mapping

The [upstream index](../../../lib/src/index.ts) exports **49 runtime values and 64 types**. Each appears below. “Implement” means preserve the useful capability in Slice 1 under the indicated Rust API; it does not promise the JavaScript identifier, inheritance tree, callable-object syntax, or unchecked casts. “Not ported” identifies deliberately excluded behavior rather than an untracked omission.

### A1. All 49 runtime exports

| Upstream export | Disposition and Rust owner | Reason / boundary |
| --- | --- | --- |
| `ArgString` | Implement: `data::parse_args`, `parse_arg_string` | Parse supplied arguments without owning a terminal or invoking a shell. |
| `FileData` | Implement: `data::FileData` | Preserve filename, MIME type, and binary/text input, with owned bytes. |
| `FormData` | Implement: `data::parse_multipart` | Bounded buffered multipart parsing belongs at the value boundary; streaming acquisition is Slice 2. |
| `PathString` | Implement: `DataPath` and `Nest` path mutation | Explicit path segments replace a processor retaining its parent store. |
| `QueryString` | Implement: `data::parse_query` | Preserve nested brackets, append/index paths, decoded values, and scalar coercion. |
| `ReadonlyMap` | Implement capability through `&DataMap<K,V>` and iterators | Rust borrowing provides actual read-only access without a second store/controller class. |
| `ReadonlyNest` | Implement capability through `&Nest` | Shared borrowing prevents mutation and preserves path reads/inspection. |
| `ReadonlyPath` | Implement: `DataPath` plus borrowed `Nest` reads | A parsed path is independent of the store and cannot hide an alias to mutable state. |
| `ReadonlySet` | Implement capability through `&DataSet<T>` and iterators | Borrowing provides the read-only collection contract. |
| `DataMap` | Implement: insertion-ordered `DataMap<K,V>` | Iteration order, indexed lookup, filter/find/map and conversion must remain available. |
| `DataSet` | Implement: insertion-ordered, unique `DataSet<T>` | Preserve uniqueness and insertion order; this is not an unordered `HashSet`. |
| `Nest` | Implement: owned `Nest` | Preserve nested CRUD, append paths, inspection and explicit overlays. |
| `FileLoader` | Defer host path/discovery to Slice 3; JS module loading not ported | Filesystem access and executable module resolution are not neutral value primitives. |
| `NodeFS` | Defer filesystem capability to Slice 3 | Use a Rust host boundary; do not depend on Node or expose a fictional portable filesystem. |
| `ItemQueue` | Implement: `queue::ItemQueue<T>` | Higher priority first, stable registration ties, consume/front/back operations. |
| `TaskQueue` | Implement: `queue::TaskQueue<C>` | Sequential async mutation, before/action/after hooks and cooperative abort. |
| `EventEmitter` | Implement: `EmitterBuilder` + `EventEmitter` | Preserve all matching listeners, priorities, hooks, composition, removal and nested dispatch. |
| `ExpressEmitter` | Implement capability: `EventPattern::express` / `on_pattern` | One emitter accepts exact, Express-style and regex matchers without subclassing. |
| `RouteEmitter` | Implement neutral capability: `EventPattern::route` / builder `route` | Method/path event patterns and route metadata do not require a transport router. |
| `Request` | Implement: `request::Request` | Construct independent values, merge query/post/data and load once; native envelope is Slice 2. |
| `Response` | Implement: `response::Response` | Preserve body/status/results/rows/errors/redirect/session and explicit output-attempt state. |
| `Router` | Split: implement neutral `resolve`/matching/req-res behavior in Slice 1; defer host wrapper to Slice 2 | Capability calls need plain-data ergonomics now; native/service context, transport helpers and lifecycle remain host work. |
| `Terminal` | Defer to Slice 2 | Command selection, branding, required arguments and execution/exit policy are CLI concerns. |
| `Exception` | Implement capability: `error::Error` and diagnostic report | Preserve typed failures, causes, status, validation/source spans and invocation breadcrumbs. |
| `Reflection` | Intentionally not ported | V8 call sites, function objects, `this`, eval and Promise frame inspection have no sound Rust equivalent. |
| `Status` | Implement: `status::Status` | Application/response status with a known-code catalog and explicit custom messages. |
| `codes` | Implement: documented immutable status catalog | Retain all upstream entries, including custom 419 and legacy 309; dispatch does not return a status. |
| `control` | Defer to Slice 2 | ANSI styling, branded console output and interactive input require a terminal boundary. |
| `input` | Defer to Slice 2 | TTY editing, validation, cancellation and cleanup need their own adapter proof. |
| `esmImport` | Intentionally not ported | Dynamic JavaScript evaluation/import is replaced by explicitly compiled registration. |
| `include` | Intentionally not ported | Automatic executable default-export unwrapping is a JS module convention, not a Rust contract. |
| `cookie` | Implement: `session::cookie` functions | Parsing and checked serialization are pure boundary operations. |
| `map` | Implement constructor capability through `DataMap::new` / `FromIterator` | Do not recreate callable objects or a redundant function wrapper. |
| `set` | Implement constructor capability through `DataSet::new` / `FromIterator` | Methods/iterators replace callable indexed collection syntax. |
| `nest` | Implement constructor capability through `Nest::new` / `from_object` | One owned store replaces callable store/controller aliases. |
| `formDataToObject` | Implement: `data::parse_body` | Dispatch by parsed MIME essence/parameters to strict JSON, URL-encoded or multipart parsing. |
| `isObject` | Implement: `Value::is_object` | Exhaustive value variants replace JS constructor/name introspection. |
| `makeArray` | Implement: explicit object-to-array helper | Preserve lexicographically sorted-key value extraction as an explicit operation, separate from numeric-path normalization. |
| `makeObject` | Implement: explicit array-to-object helper | Preserve numeric-string keys without pretending the operation is a deep merge. |
| `objectFromArgs` | Implement: `data::parse_arg_string` | Same argument grammar as supplied argv, with documented quoting and errors. |
| `objectFromJson` | Implement: `data::parse_json_object` | Parse into an object-root `Nest`; reject malformed/non-object JSON explicitly (change below). |
| `objectFromQuery` | Implement: `data::parse_query` | Accept optional leading `?` and an empty query. |
| `objectFromFormData` | Implement: `data::parse_multipart` | Require an explicit boundary, accept owned buffered input and return typed files. |
| `shouldBeAnArray` | Implement normalization predicate | Only nonempty, complete canonical numeric keys `0..n-1` become an array. |
| `withUnknownHost` | Implement explicit helper `request::with_unknown_host` | Preserve its literal prefix operation, including protocol-looking input; ordinary URL construction is separate. |
| `TemplateEngine` | Defer to Slice 3 | Build-time text rendering is unrelated to dispatch; choose an explicit safe grammar then. |
| `helpers` | Defer typed template helpers to Slice 3 | Conditions, loops, clipping/case and arithmetic require a generation contract. |
| `escapeRegex` | Implement matcher-internal escaping; no standalone public compatibility export | Correct literal matching requires escaping, not another general utility API. |
| `render` | Defer to Slice 3 | Do not conflate build templates with Leptos rendering. |

`FileLoader`'s npm/node_modules resolution, CJS/ESM/TS executable loading, `Reflection`'s V8 behavior, `Exception.try` arbitrary thrown values, and JS template method/eval behavior are intentionally not ported. Later Rust capabilities must use explicit contracts rather than simulate those mechanisms.

### A2. All 64 type exports

| Upstream type | Disposition / replacement | Reason |
| --- | --- | --- |
| `TypeOf` | Not ported; typed access/deserialization in Slice 1 | TS conditional widening is not runtime data validation. |
| `Key` | Implement: `PathSegment::Key` / `Index` | Distinguish literal keys, numeric indices and append operations. |
| `NestedObject` | Implement: `Value::Object` and `Nest` | Owned finite recursive data with checked access. |
| `UnknownNest` | Implement: `Nest` | Unknown reads return `Option<&Value>`, not an asserted caller-selected type. |
| `Scalar` | Implement: scalar `Value` variants | Null/bool/finite number/string remain distinct. |
| `Hash` | Implement: object-valued data | No separate duplicate wrapper is needed. |
| `ScalarInput` | Implement: `Value` and checked conversions | Preserve scalar/array/object inputs alongside distinct dates, bytes, files and typed native values. |
| `FileMeta` | Implement: `FileData` construction | Owned bytes, filename and MIME type without Node Buffer. |
| `CallableSet` | Callable syntax not ported; `DataSet` in Slice 1 | Borrowing/methods replace function-object mutation. |
| `CallableMap` | Callable syntax not ported; `DataMap` in Slice 1 | No hidden shared store/controller identity. |
| `CallableNest` | Callable syntax not ported; `Nest` in Slice 1 | Rust does not need a callable intersection type. |
| `ResponseStatus` | Implement: `Status` | Keep code/message together and separate from dispatch outcomes. |
| `Trace` | Implement purpose: `SourceLocation` / `InvocationFrame` | Preserve structured locations; do not parse V8 stack strings. |
| `ErrorResponse` | Implement: explicit public error envelope | Validation/status/public message remain separate from developer diagnostics. |
| `SuccessResponse` | Implement: success envelope | Results/total/status are serializable data, not output delivery. |
| `StatusResponse` | Implement: `ResponseEnvelope` with optional fields and `Resolution` | Preserve partial envelopes including false/zero/null; resolve also reports dispatch outcome separately. |
| `Item` | Implement: priority queue entry | Carry value, priority and stable insertion sequence. |
| `TaskResult` | Implement: `Result<Flow, Error>` | Continue/abort/error are distinct typed outcomes. |
| `TaskAction` | Implement: async task callback over `&mut C` | No arbitrary dynamically typed variadic arguments. |
| `TaskItem` | Implement: queued callback entry | Reuse ordered scheduling rather than a parallel queue policy. |
| `EventMap` | TS tuple-map mechanism not ported | This slice standardizes mutable `Request`/`Response`; typed domain extraction is explicit. |
| `EventName` | Implement: owned/borrowed event string | Named operations remain transport-independent. |
| `EventData` | Implement: `MatchData` | Separate named captures and positional wildcard/regex captures. |
| `EventMatch` | Implement: `EventMatch` | Preserve emitted event, original matcher and captures. |
| `Event` | Implement: invocation/listener metadata | Metadata is local to an invocation, not emitter-global state. |
| `EventHook` | Implement: before/after `Handler` callbacks | Same req/res/context contract and `Result<Flow>` as actions. |
| `EventExpression` | Implement: compiled `EventPattern` | Retain matcher kind/original expression/options without string-key reparsing. |
| `Body` | Implement: `Body::{Empty,Text,Bytes,Value}`; streams deferred to Slice 2 | Preserve buffered forms; native stream ownership/backpressure is adapter work. |
| `ResponseDispatcher` | Implement neutral `dispatch_with` callback; native resource return deferred to Slice 2 | Explicit one-attempt delivery contract without making `Response` generic over a socket. |
| `ResponseOptions` | Implement: `Response` constructors/setters | Fallible checked setters replace a permissive partial object. |
| `Headers` | Implement: validated multi-value `Headers` | Case-insensitive names and repeat values, particularly Set-Cookie, need deliberate semantics. |
| `Data` | Implement: `Nest` | One owned neutral data model. |
| `Query` | Implement: parsed `Nest` or explicit query text at construction | Parsed input is not coerced a second time. |
| `Session` | Implement: `Session` initial values | Cookie-backed string/string-list state, not authenticated identity. |
| `Post` | Implement: `Nest` | Preserve post data separately from the merged request data snapshot. |
| `LoaderResults` | Implement: `LoadResult` | Optional body/post updates preserve missing versus empty/null. |
| `RequestLoader` | Implement: `load_with` async callback | Isolated owned request snapshot in, staged `LoadResult` out; no partial framework mutation while pending. |
| `CallableSession` | Callable syntax not ported; `Session` methods in Slice 1 | Explicit mutations update revisions and keyed cookie options. |
| `RequestOptions` | Implement: `RequestBuilder` | Validate URL/headers and preserve initialization precedence. |
| `Revision` | Implement: `SessionRevision::{Set,Remove}` | Exhaustive state; removal retains the cookie scope needed to expire the original cookie. |
| `CookieOptions` | Implement: typed `CookieOptions` | Domain/path/expiry/max-age/security/partitioning/priority/same-site are explicit. |
| `Method` | Implement: request `Method` plus `MethodPattern` | Standard and extension methods are values; Any is a matcher, not a real request verb. |
| `Route` | Implement: method/path pattern metadata | Retain source route metadata without a transport selection policy. |
| `RouteMap` | Defer generic host/native tuple map to Slice 2 | Dispatch uses neutral req/res; transports own their native types. |
| `RouteAction` | Implement capability: `Handler` | One callback convention replaces duplicate route/event action aliases. |
| `RouterContext` | Defer host/service context to Slice 2 | Slice 1 supplies only invocation and nested-dispatch context. |
| `RouterArgs` | Defer compatibility tuple to Slice 2 | Prefer explicit borrowed arguments; no native generic tuple in the foundation API. |
| `RouterMap` | Defer host registry typing to Slice 2 | It is not required for a neutral dispatch table. |
| `RouterAction` | Implement neutral action as `Handler`; host enrichment deferred to Slice 2 | Avoid callback-shape inference or reflection. |
| `FileRecursiveOption` | Defer to Slice 3 | Host directory policy. |
| `FileStat` | Defer to Slice 3 | Host metadata boundary. |
| `FileStream` | Defer to Slice 2/3 streaming/file adapter | Node response piping is not a portable value. |
| `FileSystem` | Defer to Slice 3 | Select host capabilities when generation/file use exists. |
| `CallSite` | Intentionally not ported | V8 executable/frame reflection cannot be promised in Rust. |
| `TerminalInputConfig` | Defer to Slice 2 | Prompt/default/validation/transform behavior belongs to CLI. |
| `TerminalInputContext` | Defer to Slice 2 | Stream/cancellation/cleanup ownership belongs to CLI. |
| `TerminalTTYInput` | Defer to Slice 2 | Raw-mode input is platform-specific. |
| `TerminalTTYOutput` | Defer to Slice 2 | TTY rendering is platform-specific. |
| `TerminalPromptState` | Defer to Slice 2 | Interactive state machine has its own behavior proof. |
| `TerminalInputTheme` | Defer to Slice 2 | Validation display/clearing policy belongs to prompts. |
| `TemplateHelperOptions` | Defer to Slice 3 | Typed build-render context. |
| `TemplateHelper` | Defer to Slice 3 | Compiled helper registration. |
| `TemplateResolver` | Defer to Slice 3 | Build-time variable resolution. |
| `TemplateOptions` | Defer to Slice 3 | Delimiters/container/helper policy is generation work. |

### A3. Behavior mapping for all 26 upstream test files

This matrix maps observed assertions, including tests importing non-index session types. It is a porting checklist, not a claim that upstream tests were executed. Upstream tests alone do not establish untested edge semantics.

| Test source | Behavior inventory and disposition |
| --- | --- |
| `ArgString.test.ts` | Slice 1: argv and quoted-string forms, `--key value`, grouped short flags, positional values, scalar coercion, nested brackets, append arrays/arrays of objects, skip offset, supplied arguments containing spaces, CLI filter/span/roles cases. Bare `-`/`--` and dangling options need the explicit grammar in B/D; no shell execution. |
| `QueryString.test.ts` | Slice 1: empty input, scalar/multiple pairs, numbers/booleans/null, URI decoding, nested/indexed paths, empty values, malformed JSON-like value retained as text. Constructor/controller identity becomes ordinary function input/output. |
| `FormData.test.ts` | Slice 1: nested/multiple fields, numeric coercion, prefix paths, malformed empty multipart rejected, missing Content-Disposition ignored. Explicit boundary and valid termination replace first-line boundary guessing. |
| `FileData.test.ts` | Slice 1: byte/string file input, empty file and filename/MIME metadata; strings become their exact UTF-8 bytes. |
| `Nest.test.ts` | Slice 1: CRUD/deep paths, object/array shapes and append, dot/custom path access, missing/default reads, query/multipart integration, root overlays, empty object retention, clear/delete/size/keys/entries/values and pretty/compact JSON. Borrowed iterators plus a sequential visitor preserve early-stop/empty iteration. TS progressive structural inference and callable wrappers are intentionally not ported; checked typed deserialization replaces unchecked generic reads. The “invalid data” test has no assertion when nothing throws; Rust root-shape tests must assert rejection directly. |
| `Helper.test.ts` | Slice 1: sorted-key values to array, array to numeric object, mixed/empty values and contiguous-key detection; missing differs from null. Add 12+ numeric entries to catch lexicographic normalization errors. |
| `Map.test.ts` | Slice 1: map CRUD, presence/missing, object/JSON conversion, filter/find key/value/entry, map values, first/last/index and ordered iteration; set uniqueness/order, corresponding search/map/filter/array/JSON behavior, empty/out-of-range and clear/delete. Use Rust collection methods/iterators rather than callable wrappers. |
| `ReadonlyMap.test.ts` | Slice 1: borrowed lookup/size/ordered keys/values/entries, find/filter/map and object/JSON conversion; compile-fail proof that a read-only borrow cannot mutate. |
| `ItemQueue.test.ts` | Slice 1: empty consume, add/push/shift priority extremes, size and drain. Add stable ties, signed priorities and overflow tests absent upstream. |
| `TaskQueue.test.ts` | Slice 1: empty, sequential async/sync-adapted work, completion, task abort, before abort, after abort, consumed count and remaining queue. Typed outcomes replace numeric 200/309/404 dispatch status. |
| `EventEmitter.test.ts` | Slice 1: registration, exact match, all listeners, priorities/ties, missing/clear/unbind, composed registrations, before/after hooks, nested dispatch and error propagation after awaits. Emitter-global `event` inspection moves to invocation-local metadata. |
| `ExpressEmitter.test.ts` | Slice 1: custom separators, exact/parameter/wildcard/regex matches and captures, duplicate registrations, global/case-insensitive regex intent, all-match priority order, abort, hooks, removal/missing, composition preserving differing separators, nested errors. JS regex flags/stringified keys are replaced by explicit immutable matcher options. |
| `RouteEmitter.test.ts` | Slice 1: method/path events, uppercase methods, Any verb matching, parameterized trailing slashes, retained route metadata, plain-event composition without fictional routes, and no recompilation of canonical regex patterns during merge. Capturing a particular source emitter explicitly is still possible by reference/Arc; implicit self-binding is not recreated. |
| `Request.test.ts` | Slice 1: GET/unknownhost/text/plain/empty defaults, object/collection input, URL query, cookie initial session, separate query/post/data, buffered body kinds, successful load-once and precedence. `withUnknownHost` remains literal even for `https://...`. Native `resource` access is Slice 2. Buffer/Uint8Array distinction becomes `Bytes`. |
| `Response.test.ts` | Slice 1: initially unset status/empty body, checked manual setters, basic/advanced errors, HTML/JSON/XML, structured results/rows+total, redirects/Location, buffered body kinds, headers, no-output finalization, dispatch callback once, repeated finalization and conversion to diagnostic/error/envelope. Native `resource` is Slice 2; V8-formatted stack reconstruction is not ported. |
| `Session.test.ts` | Slice 1: initial read values and string lists, set/delete/clear revisions, per-key options, options retention on value-only set, options removal from live state on delete, ordered iteration and revisions. Callable session delegates become methods. Removal revision deliberately retains old scope (B/D). |
| `cookie.test.ts` | Slice 1: empty/invalid fragments, trimming, first duplicate wins, percent-decode/raw fallback and custom codec, checked name/value/options, integer max-age, domain/path/expiry, httpOnly/secure/partitioned, priority and SameSite variants, custom encoder output validation. Wrong option variants become unrepresentable; invalid data still yields `Result::Err`. |
| `Exception.test.ts` | Slice 1: message formatting via `format!`, status (including 419 and unknown code), field errors, source spans, explicit public JSON/envelope conversion and ordinary error causes. JS throwing of strings/objects/numbers, `.try().catch()`, mutable stack-string parsing and malformed V8 frame behavior are intentionally not ported. Structured metadata must work with no backtrace. |
| `Router.test.ts` | Slice 1 covers match collection, capture metadata, ordered request/response mutation, plain-data/request-input resolve, nested resolve, response envelope transfer and merge preservation (exact/root/parameter/Any/fallback). Slice 2 owns native resource wrapping, method/path resolve conveniences and host context. Route-to-request capture injection is explicitly deferred to the host adapter; raw event dispatch keeps captures in context. No MD5 authentication behavior is adopted from the illustrative test. |
| `CaseStudy.test.ts` | Slice 1 can prove a captured application service/increment operation returning a resolve envelope. Slice 2 owns a generic host service context rather than subclass inheritance. |
| `FileLoader.test.ts` | Slice 3: filesystem boundary/cwd, absolute/relative/base path, `@/` alias, required/missing path results, extension and directory-index resolution. npm package discovery and CJS/ESM/TS execution/default export unwrapping are intentionally not ported. |
| `NodeFS.test.ts` | Slice 3: injected filesystem, actual write/read/stat/realpath/delete, recursive directories and file streaming; prove separately on supported native targets. |
| `Reflection.test.ts` | Intentionally not ported: exact V8 frame function names/URLs and callable objects. Slice 1 instead proves explicitly captured registration/origin/invocation metadata. |
| `Terminal.test.ts` | Slice 2: command/args/brand/controls, argument lookup/default, command dispatch to success/error envelope. Slice 1 already supplies parser and event/value primitives. |
| `control.test.ts` | Slice 2: branded/plain/styled outputs, placeholder substitution and required/default prompt wiring. |
| `input.test.ts` | Slice 2: typed/default/required input, pattern and sync/async validation, keep/clear invalid input, cancellation before/during prompt, editable/tab prefills, backspace, Ctrl-C, cleanup, piped output without TTY escapes, final transformation and public wiring. |

There is no template-specific test file in this upstream suite. Template breadth is inventoried from its source rather than inferred from a green test suite.

## B. Proposed public API and behavioral contract

### B1. Package/module layout and dependency policy

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

### B2. Registration, ownership and async signature

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

`resolve` is a first-class capability call, preserving the common [00019](../../.agents/references/00019-upstream-runtime-api-contracts.md)/[00030](../../.agents/references/00030-upstream-operational-examples.md) shape. The sealed `IntoRequest` input adapter accepts an owned `Request`, `Nest`, or object-root `serde_json::Value`. Plain data becomes `Request::builder().data(input).build()`; an owned Request is moved with its current state, never cloned. It creates a fresh Response, dispatches without a top-level lifecycle, and moves the response into an envelope. A nested `Invocation::resolve` inherits the path and uses a new request/response pair, so it can run concurrently with another child resolve. Retain `emit` when mutation of the caller's exact req/res is desired.

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

### B3. Execution semantics

1. Validate bounded event input and collect **all** matching registrations once. Sort the collected listeners by descending signed integer priority, then ascending registration sequence across all matcher kinds. Exactness/specificity adds no hidden priority. Registering the same callback twice runs it twice.
2. For each listener, derive immutable listener-local captures and an invocation frame. Execute before, action, after **sequentially**, awaiting each phase. Before abort skips action/after; action abort skips after; after abort skips later listeners. A phase error skips remaining phases/listeners and returns the original error with diagnostic context. After is not a finally hook.
3. No matches yields `NoListener`; completion yields `Completed`; cooperative abort yields `Aborted`. None sets response status, auto-sends a body or emits general lifecycle events. No-listener can coexist with a response status explicitly supplied by the caller.
4. A child error propagates through `?` retaining the deepest frame/cause. A child abort does not automatically abort its caller. Partial req/res and external side effects remain; dispatch is not a transaction. Dropping a future stops future phases without rolling back completed mutations, and emits no async cleanup hooks.
5. Recursive dispatch is supported and boxed at the internal recursion boundary. A configurable depth limit, proposed default 64, rejects overflow before entering the next child. Sibling invocations do not consume a shared global depth counter. Error paths are bounded and owned; the library cannot prevent user code from looping or blocking inside a handler.

No top-level `handle` is implemented here. The future server host must separately settle sent-response handling, lifecycle aborts/error fallback and output mapping. Naming an event `request`, `response` or `error` in Slice 1 registers an ordinary event; the dispatcher does not recursively wrap every child in those hooks or catch its own errors by emitting `error`.

Likewise `config` → `listen` → `route` and generation's `idea` conventions belong to Slice 2 orchestration, as [00012](../../.agents/references/00012-upstream-architecture-and-composition.md)/[00024](../../.agents/references/00024-upstream-cli-and-plugin-contracts.md) describe. An immutable **runtime** table does not require an immutable bootstrap: `TaskQueue<StartupContext>` can run ordered startup work with a mutable registration accumulator, then the host builds the runtime table after contributions finish. This is an available primitive, not a selected Slice 2 implementation or an automatic lifecycle inside emit/resolve. Do not preclude later host-owned stage/correlation context or require a global mutable registry.

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

### B4. Matcher grammar and captures

- `EventPattern::exact(text)` always means literal bytes. `on("event", ...)` uses exact matching; `on_pattern(EventPattern::express(...), ...)` is the explicit patterned form. This avoids interpreting a literal operation name containing `*`, `:` or regex punctuation accidentally.
- Express-style patterns use a declared nonempty single-character separator (default `/`, including custom space or `-` use). A `:name` token accepts the upstream ASCII alphanumeric/underscore/hyphen name vocabulary and captures one or more non-separator characters. Duplicate parameter names are rejected at construction; adjacent ambiguous tokens and invalid star runs are rejected rather than guessed. `*` captures one nonempty segment; `**` captures zero or more characters, including separators. Literals outside those tokens are regex-escaped. Patterns match the whole event.
- `MatchData.params` contains named parameters; `args` contains positional wildcard captures in token order. Double-star captures are split on the declared separator (including documented empty edge segments), replacing upstream's hard-coded `/` split. Regex positional captures retain their original captured text; optional unmatched groups remain `None`, not a fabricated empty string. Named regex groups are also exposed in `params`.
- Raw regex uses Rust `regex` syntax with explicit options such as case-insensitive matching; anchors are caller-controlled. Use the first match/captures per registration, not one invocation per global occurrence. No mutable regex cursor/global `lastIndex`; repeated emits are identical. JS `g` is thus unnecessary; arbitrary ECMAScript lookaround/backreferences and JS flag identity are intentionally unsupported. Compilation rejects unsupported syntax with pattern/site diagnostics and bounded compiled size.
- A neutral route matcher combines `MethodPattern::{Exact(Method),Any}` with an Express-style path and retained route metadata. Request verbs are normalized/validated tokens and extension verbs are allowed. Any matches a nonempty method token, not the fictional request verb ALL/ANY. Parameterized/Any routes tolerate trailing separators as upstream does; an exact literal route remains strict. `GET /users/42` is still an event value, not a transport endpoint. Percent decoding and URL security policy remain outside the matcher.
- Captures remain per listener in `Invocation::captures()`. Raw emit does not merge captures into request data. Slice 2's router adapter can inject them deliberately with an explicit precedence rule; this prevents different pattern listeners or nested events silently contaminating one another's req.data.
- `matches` preserves one descriptor per registration in execution order, including duplicate listeners. It does not return a display-string-keyed map that can collapse distinct capture-name definitions with equivalent compiled regex. Matches with zero active listeners cannot survive builder removal/clear.

### B5. Data and boundary parsing

Following [00005's lib redesign boundary](../../.agents/references/00005-discussion-feature-feasibility.md), `Value` is richer than JSON. Owned variants are null, bool, validated finite JSON number, string, array, insertion-ordered object, **DateTime**, **Bytes**, **File**, and **Shared**. Missing is represented by absent lookup, never by a null variant.

| Non-JSON variant | Ownership and representation | Boundary behavior |
| --- | --- | --- |
| `DateTime(Timestamp)` | A distinct checked UTC instant, proposed signed epoch seconds + nanoseconds; no implicit conversion to a string/number | A caller selects a wire representation explicitly. Calendar/time-zone presentation is outside this primitive. |
| `Bytes(Vec<u8>)` | Owned bytes distinct from strings and files | Preserve arbitrary bytes, including invalid UTF-8; no implicit lossy text/base64 conversion. |
| `File(FileData)` | Owned bytes plus filename/MIME metadata | Empty files remain files; never write the untrusted filename to disk. |
| `Shared(SharedNative)` | Explicitly shared `Arc<dyn Any + Send + Sync>` with typed construction and borrowed downcast | Cloning shares only this named variant. Payload access is immutable; users explicitly choose any interior synchronization in their payload type. |

Default JSON conversion of DateTime/Bytes/File/Shared returns a path-qualified unsupported-value error. A caller may deliberately construct a JSON date/byte/file envelope before output. Shared native payloads never serialize through Debug or an inferred type name, and cannot enter an external request from JSON. No recursive user-defined wire codec framework, arbitrary JS object coercion, NaN/Infinity or executable function reflection is introduced. Owned Value containers are acyclic; a native payload may have application-defined internal sharing/cycles and the framework never traverses it.

`Value` and `Nest` implement Clone: scalar strings, arrays/objects, dates, bytes and files clone independently; only `Value::Shared(SharedNative)` shares its immutable native payload. `SharedNative` names that sharing in the public type and exposes no unchecked mutable access. No ordinary data node/controller aliases through Clone. Generic application payloads may deliberately contain synchronization or internal sharing; that is explicit at Shared construction.

Query/post→data initialization can clone uniformly, including explicitly Shared native values, removing the rejected `try_copy`/native-copy limitation. Request and Response themselves will **not** implement Clone: load/delivery attempt state and output ownership must not become a second independently usable invocation. Their data-bearing members are clonable values, so callers can deliberately construct a fresh Request from cloned data without duplicating an attempted native read/send. Invocation/registries/queues/owned error causes likewise have no blanket Clone.

Key data signatures:

```rust
Nest::new() -> Nest
Nest::from_object(value: serde_json::Value) -> Result<Nest>
Nest::get(&self, key: &str) -> Option<&Value>
Nest::has(&self, key: &str) -> bool
Nest::set(&mut self, key: &str, value: impl Into<Value>) -> Result<&mut Self>
Nest::remove(&mut self, key: &str) -> Result<Option<Value>>
Nest::path<'p>(&self, path: impl IntoPath<'p>) -> Option<&Value>
Nest::path_as<'p, T: DeserializeOwned>(&self, path: impl IntoPath<'p>) -> Result<Option<T>>
Nest::has_path<'p>(&self, path: impl IntoPath<'p>) -> bool
Nest::set_path<'p>(&mut self, path: impl IntoPath<'p>, value: impl Into<Value>) -> Result<&mut Self>
Nest::remove_path<'p>(&mut self, path: impl IntoPath<'p>) -> Result<Option<Value>>
Nest::merge(&mut self, other: Nest) -> Result<&mut Self>
Value::to_json_with(&self, limits: &ParseLimits) -> Result<serde_json::Value>

SharedNative::new<T: Any + Send + Sync>(value: T) -> SharedNative
SharedNative::downcast_ref<T: Any>(&self) -> Option<&T>
Value::as_str(&self) -> Option<&str>                // analogous checked scalar accessors
DataPath::dot(text: &str) -> Result<DataPath>
DataPath::separated(text: &str, separator: char) -> Result<DataPath>
DataPath::from_segments(segments: impl IntoIterator<Item = PathSegment>) -> Result<DataPath>
pub enum PathSegment { Key(String), Index(usize), Append }

parse_query(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_args(args: &[String], skip: usize, limits: &ParseLimits) -> Result<Nest>
parse_arg_string(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_json_object(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_multipart(body: &[u8], boundary: &str, limits: &ParseLimits) -> Result<Nest>
parse_body(content_type: &str, body: &[u8], limits: &ParseLimits) -> Result<Nest>
```

Mutation prepares the full path and structural changes read-only, measures incoming/replaced subtrees, and checks a delta against cached node/byte totals before committing in place. Unrelated payloads are not cloned or revalidated. Container normalization/deletion may scan or move the affected container. All returned errors leave the tree unchanged. Replaced/cleared native values are dropped only after tree and totals agree, preserving invariants through a user destructor panic. Missing is `None`; present null is `Some(Value::Null)`. Empty objects remain objects. `merge` is an incoming-wins **top-level overlay**, matching `Nest.set(object)`; it is not a speculative recursive config merge. Intermediate scalar/null/date/byte/file/shared-native traversal returns a path/type error rather than silently replacing data or panicking. Append uses the next available canonical numeric key, never object key count where that could overwrite an existing sparse index. Nonempty dense canonical numeric children normalize in numeric order; sparse/noncanonical/negative keys remain objects without huge index-based allocation. Removing an array element preserves remaining index identities by converting that container to a numeric-keyed object, rather than shifting values or equating a hole with null. Empty path is rejected for mutation; root read/clear are distinct methods. `path_as` checks conversions; it does not stringify shared-native/date/byte data to pretend deserialization succeeded.

Ordered `DataMap`/`DataSet` may use documented aliases/re-exports of `indexmap` collections, with Rust iterator equivalents for filter/find/map and explicit conversion examples. No duplicate bespoke collection hierarchy is necessary. Borrowed entries/keys/values and `len` replace controllers/`size`; Rust's `Option<usize>` replaces a set-search `-1` sentinel. A sequential async nested visitor preserves upstream early-stop behavior, with empty/missing collections producing an explicit empty outcome. JSON object key order is retained by the configured serializer; no promise of byte-identical whitespace except explicit pretty/compact modes.

Approved parser rules (D3), including the deliberate differences below:

| Boundary | Proposed contract |
| --- | --- |
| Common limits | Finite defaults: 1 MiB buffered input, 64 nesting levels, 10,000 entries/parts, 8 KiB multipart part headers. Count before allocation/insertion and bound outputs; callers can pass explicit alternatives. Depth is checked for supplied JSON and already-parsed values too. No unbounded recursive parsing or sparse index allocation. |
| Query / URL-encoded | Optional leading `?`; empty input gives empty Nest; split first `=` only; decode percent escapes once, `+` to space for both names and values; reject malformed encoding/UTF-8. Bracket paths allow explicit indices and append; repeated ordinary keys replace, `[]` appends. Parse finite decimal numbers, exact true/false/null and valid object/array JSON values; malformed JSON-like scalar text stays text. Empty values stay empty strings. |
| Args | Parse supplied argv directly; never join it into a query string. Support tested long-key/value, grouped short flags, numeric positional keys, bracket paths and skip offset. Also support explicit `--key=value` without allowing `&`, `=`, `+` or `%` inside a value to become new fields. Ignore lone `-`; bare `--` ends option parsing (a final bare marker adds nothing). A dangling long option is an error. Values immediately following a long key are taken as values, including a leading minus. |
| Argument string | Small documented whitespace/quote tokenizer, single/double quotes and quoted empty values; no expansion, commands, environment access or claim to full shell syntax. Unclosed quotes are errors. Share only scalar/path policy with query parsing, not delimiter interpretation. |
| JSON object | Validate JSON after permitted leading whitespace; require an object root, preserve false/0/null, reject malformed/non-object JSON. Use the same depth/entry/input limits. |
| Multipart | Caller supplies parsed Content-Type boundary; recognize delimiters only in valid positions, require final termination, preserve exact file bytes (including CR/LF and boundary-like content), validate text UTF-8, ignore parts without a usable field disposition as upstream tests expect, apply nested names/scalar policy to text only. Empty named files remain files. Support quoted boundary and repeated/append fields; reject truncation/invalid limits without partially mutating an existing Nest. No disk spooling/streaming support claim. |
| MIME dispatch | Parse MIME essence/parameters case-insensitively; support JSON, URL-encoded and multipart deliberately, including JSON charset parameters. Unsupported media types return a typed error rather than silently manufacturing empty data. |

### B6. Request/response/session values

Request sketch (loader signatures return borrowing futures whose output is `Result<()>`):

```rust
Request::new() -> Request
Request::builder() -> RequestBuilder
RequestBuilder::build(self) -> Result<Request>
Request::load(&mut self) -> Result<()>             // finalize with no loader
Request::load_with(&mut self, loader: /* one-shot callback producing a Send future of LoadResult */)
    -> /* borrowing Future<Output = Result<()>> */
pub struct LoadResult { pub body: Option<Body>, pub post: Option<Nest> }
pub enum LoadState { Unloaded, Attempted, Loaded, Failed }
```

Builder inputs include URL, method, headers, body/MIME, query override, post, explicit data, initial session and parse limits. Defaults preserve GET, `http://unknownhost/`, text/plain and empty body/collections. Prefer explicit `Body` variants rather than JS truthiness/type probing; bytes unify Buffer/Uint8Array. `Body::Empty` and `Body::Value(Null)` remain distinct. URL parsing does not trust forwarded headers or grant network access. Explicit initial session wins over cookies; explicit query wins over URL query. Parsed object values are not re-coerced. Initialization overlays query → post → explicit data; successful load later overlays loaded post into post/data, matching upstream sequence. Later direct edits of query/post do not magically resynchronize data.

`load_with` accepts an owned, isolated `RequestView` data snapshot and returns staged data: `F: FnOnce(RequestView) -> Fut + Send`, `Fut: Future<Output = Result<LoadResult>> + Send`. The view is Clone but contains no load-attempt state; ordinary snapshot data is independent, with sharing only through Value::Shared. This permits named async functions and closures borrowing a caller-owned Send native reader without a static callback bound. Snapshot edits cannot partially mutate the original. Exclusive mutable borrowing prevents simultaneous loads. Successful load/no-loader finalization occurs once. Before awaiting, record Attempted; success atomically installs validated output and records Loaded; error records Failed. A cancelled/dropped attempt stays visibly Attempted, and a later load returns an incomplete-attempt error rather than repeating a potentially consumed native reader. No implicit retry; the owner accepted this one-attempt contract (D4). Native transport request/stream holders remain outside this value in a later adapter-owned envelope; typed in-process values use Value::Shared. Successful load clones staged post values into aggregate data according to the explicit Shared semantics. Final validation enforces the request's selected budgets even if a caller replaced its public Nest stores.

Response sketch:

```rust
Response::new() -> Response
Response::status(&self) -> Option<&Status>          // None replaces code 0/empty message
Response::status_code(&mut self, code: u16, message: Option<String>) -> Result<&mut Self>
Response::set(&mut self, mime: &str, body: Body) -> Result<&mut Self>
Response::html(&mut self, text: impl Into<String>) -> Result<&mut Self>
Response::xml(&mut self, text: impl Into<String>) -> Result<&mut Self>
Response::json<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>
Response::results<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>
Response::results_value(&mut self, value: Value) -> Result<&mut Self>
Response::rows<T: Serialize + ?Sized>(&mut self, rows: &T, total: u64) -> Result<&mut Self>
Response::redirect(&mut self, location: &str) -> Result<&mut Self>
Response::set_error(&mut self, error: PublicError) -> Result<&mut Self>
Response::apply_error(&mut self, error: Error) -> Result<&mut Self>
Response::into_error(self) -> Error                // retains an owned original cause when present
Response::apply_envelope(&mut self, envelope: ResponseEnvelope) -> Result<&mut Self>
Response::to_envelope(&self) -> ResponseEnvelope    // infallible rich-value clone
Response::into_envelope(self) -> ResponseEnvelope  // move owned in-process values
ResponseEnvelope::try_to_json(&self) -> Result<serde_json::Value>
Response::dispatch_with(&mut self, dispatcher: /* one-shot callback producing a Send future of () */)
    -> /* borrowing Future<Output = Result<()>> */
Response::dispatch(&mut self) -> Result<()>         // explicit no-output finalization
Response::stop(&mut self) -> Result<()>
pub enum DeliveryState { Open, Attempted, Sent, Stopped, Failed }
```

HTML/XML/JSON/results/rows/set default to OK; redirect defaults to Found and sets validated Location. `json` stores serialized text, while results/rows retain structured JSON; results sets total=1 and rows uses the supplied total including zero. Raw JSON text, if needed, is explicit text plus MIME rather than double-guessing strings. Status/total overrides use explicit setters after a convenience method. Headers are validated, case-insensitive and multi-value; never comma-fold Set-Cookie. Mutable state is private behind checked methods/accessors so finalization cannot be bypassed by public field writes.

Envelope application distinguishes absent fields from present zero/false/empty/null. It stages validation before mutation, and does not copy delivery state/native transport resources. `results_value` supports richer in-process results, including dates/bytes/explicitly Shared native values. `into_envelope` moves those results without Clone; borrowed `to_envelope` clones ordinary data independently and retains only explicitly Shared native identity. In the envelope, Empty becomes absent results, Text becomes a string value, Bytes stays a byte value, and Value moves unchanged. The envelope is a result/status view, not a full copy of headers/session/MIME/body delivery state. The explicit JSON conversion rejects non-JSON values with their exact path, without silently erasing type distinctions. `apply_error` owns and retains the original Error for developer inspection/consuming `into_error`; only its explicit public view reaches an envelope. If no original cause was supplied, `into_error` constructs an error from the response's status/public fields and does not invent a source chain. The upstream status catalog includes custom application codes; an unknown `u16` code retains its value with `Unknown` unless a custom label is supplied. A later HTTP adapter must validate native status legality rather than assume every application status is HTTP-valid. `Status::lookup`/message lookup return Option; custom 309 remains a legacy catalog entry, never the event abort result.

`dispatch_with` passes an owned, isolated `ResponseView` data snapshot to a supplied async callback: `F: FnOnce(ResponseView) -> Fut + Send`, `Fut: Future<Output = Result<()>> + Send`. Like RequestView it is Clone without delivery/native state or owned private error causes. Native ownership/results stay with the caller, including a borrowed Send native writer. It revalidates the response budgets before snapshot/output, then records Attempted before awaiting, Sent on success, Failed on error. Repeated success is a no-op; failed/interrupted attempts are not automatically retried. `dispatch()` finalizes without native output. `stop()` records Stopped without invoking a dispatcher. All terminal/attempted states reject subsequent value mutation; no error path claims bytes were never partly sent. `sent()` retains the legacy “finalized” convenience for Sent/Stopped, while `delivery_state()` is authoritative. Drop cancellation preserves Attempted. These are the accepted D4 choices, not guarantees inferred from upstream's bool flag.

Session contains ordered string/string-list values, per-key validated cookie options and an ordered latest-revision-per-key map. Initial values produce no outgoing revisions. Value-only set retains that key's options; explicit options replace them. Delete/clear remove options from live state but **copy their prior domain/path scope into the removal revision**, allowing the later adapter to expire the same cookie instead of an unrelated default-path cookie. Removing a missing name still creates a tombstone. Arrays require an explicit adapter serialization policy; the foundation will not silently collapse a string list.

Cookie parsing keeps upstream first-value-wins, whitespace trimming, ignored malformed fragments and raw fallback for undecodable values; query decoding remains strict. Checked serialization supports domain/path, expiry, signed integer max-age, httpOnly, secure, partitioned, typed Priority and SameSite, and optional custom codecs whose output is revalidated. Validate option combinations separately from application security policy; no “session” value is proof of authentication. Session/Request Debug output must avoid dumping credential values.

### B7. Error and diagnostic model

`Error` implements `Display` and `std::error::Error`, owns an `ErrorKind`, developer message, optional application Status/public message, validation errors, optional bounded byte `SourceSpan`, origin/registration locations, an optional boxed `Send + Sync + 'static` original cause and invocation breadcrumbs. It does not implement Clone or serialize itself implicitly. Known framework/parser conversions support `?`; arbitrary external errors use `map_err(Error::from_source)?` without a conflicting blanket `From<E>` implementation.

Typed kinds distinguish invalid path/type/encoding, parser/limit failures, invalid matcher/registration, incomplete load/delivery, finalized response mutation, dispatch-depth exhaustion and handler/domain failure. Returned ordinary failures are not panics. Native backtrace capture is optional and original-capture preserving; absence does not remove useful reports. No tracing subscriber, raw request/session logging or V8 stack parsing is needed for baseline diagnostics.

For every failed phase, report the actual emitted event, matcher/captures' identity (not automatic sensitive values), listener ID/label, plugin when supplied, priority, original registration location, phase and complete nested event/listener path. Preserve the deepest failure and cause while outer propagation adds context without duplicating the same path. Anonymous listeners are identified by ID/site; do not infer stable function names from erased Rust types. Caller/source locations describe what was actually captured, not an invented precise failing expression inside arbitrary third-party futures.

Public error/envelope conversion requires an intentional public message/field-error view and excludes paths, plugin internals, causes, native backtraces and registration locations by default. A separate developer diagnostic view renders those fields. Transport correlation and lifecycle stage enrichment are later host responsibilities. Panics propagate normally; this slice neither promises panic recovery nor turns aborting panics into an error event. Application-owned locks may still deadlock/poison; the engine does not acquire or unwrap them.

## C. Risk inventory and proofs

These are design risks I would review even without the prior-attempt failure list. The engine can exclude internal lock/reentrancy bugs; it cannot guarantee arbitrary user callbacks terminate or are transactionally safe.

| Risk | Prevention / explicit limit | Required proof |
| --- | --- | --- |
| Registry lock held across await; reentrant deadlock | Immutable built table; no registry/handler mutex, no executor blocking | Parent→child→grandchild and same-event bounded recursion under watchdog; inspect lock-free execution path. |
| Concurrent nested dispatch corrupts global current event | Immutable per-dispatch/per-listener paths/captures; no task/thread-local mutable current slot | Barrier-controlled concurrent roots and concurrent siblings with different req/res; parent metadata correct after success/error/abort. |
| Framework lock poisoning after panic | No poisonable production registry/data locks | Panic/unwind in child process followed by a new independent dispatch; audit all lock/unwrap sites; do not claim user-lock recovery. |
| Clone silently aliases mutable state | Value/Nest clone ordinary data deeply; only Value::Shared(SharedNative) aliases. Request/Response/engine/queues/error are not Clone. | Mutate cloned nested objects/arrays/bytes/files independently; Shared retains typed identity; no mutable payload accessor; compile-fail Clone on stateful owners. |
| Unsound borrowed futures/reentrancy | Safe lifetime-bound erasure; shared callable contract, exclusive req/res borrows | Named and borrowing async closures compile; same req/res concurrent emit and escaping Invocation compile-fail; forbid unsafe in first-party crates. |
| Mistaking Send handler storage for Send futures | Require each lifetime-indexed returned future itself to be Send, then erase to a Send boxed future | Named/helper nested dispatch, positive future Send assertion and multithreaded tokio::spawn execution; non-Send future rejected by the same registration bound. |
| Handler mutable capture races | Shared Fn callbacks with lifetime-indexed Send futures; no shared FnMut wrapper | Immutable captures and explicitly synchronized payloads; FnMut-only and non-Send callbacks rejected; concurrent invocations share no ordinary framework mutation. |
| Tie nondeterminism or arithmetic overflow | Signed priority compare then registration sequence; checked counters/front/back extremes | Cross-kind equal priorities, negative/min/max priorities, duplicate callback, compose/remove/readd order and counter exhaustion unit path. |
| Composition or customization corrupts identity/order | Move typed matcher/metadata; explicit handle replacement; receiver sequence separate from ID | Preserve exact/pattern/regex/custom-separator matches across extend, original/replacement sites, foreign/stale handle rejection, replacement keeps position, no stale matches after clear. |
| Literal regex punctuation accidentally broadens a pattern | Typed exact vs Express vs regex; escape all literal Express fragments | `. + ( ) [ ] $ ^ \\` literals, malformed token forms, separator metacharacters, duplicate params. |
| Capture precedence or leakage between listeners | Captures only in per-listener context; no automatic req.data mutation | Conflicting parameter names/values across all-match listeners and nested dispatch, equivalent regex with different param labels, duplicate registrations. |
| Stateful/global regex repeats inconsistently | Immutable regex first captures, no global cursor | Repeated emits, optional/named groups, empty matches, case sensitivity and rejected unsupported JS regex. |
| Deep recursion, oversized input or huge sparse allocation | Dispatch/data/parser depth, bytes/entry/header/regex-size limits; checked indices and sparse objects | Boundary and limit+1 cases, recursive event cycle, huge index, deeply nested JSON/brackets, oversized multipart headers/file. |
| Inconsistent before/action/after stop/error behavior | One documented phase sequence; after not finally | Decision table for Continue/Abort/Err in each phase, zero/multiple listeners, child abort handled/propagated. |
| Cancellation replays an external side effect | No implicit task/load/delivery retry; visible attempt state; partial mutations documented | Drop futures while suspended before/inside/after handler and during load/delivery; unstarted phases never run; second attempt rejected. |
| Double send or mutating already emitted output | Exclusive Response borrow and delivery state guards | Successful repeated dispatch no-op, failure/cancel no retry, stop without output, post-finalization writes rejected. |
| Partial load or parse mutation after errors | Stage results before applying; isolated owned loader/dispatcher snapshots without attempt state; validate chosen budgets at commit/output | Fault after valid prefix, invalid post result, loader error/cancel; snapshot edits do not mutate original data/body; no retry after interruption. |
| Silent missing/null/false/0 loss | Option tracks presence; Value variants and explicit envelope option fields | Query/post/data/load precedence with null/false/zero/empty, envelope round trips including explicit null and total=0. |
| Dates/bytes/shared native collapse into JSON; sharing becomes implicit | Rich Value variants, explicitly named SharedNative and explicit wire conversion | Distinguish null/string/date/bytes/file/Shared, typed downcast success/mismatch, clones alias only Shared, native survives resolve, serialization errors name the path, payload drops after the final Shared owner. |
| Resolve loses outcomes or starts general lifecycle recursively | Neutral plain-data/owned-request input adapter, fresh response and inherited child path, explicit Resolution wrapper | Named/closure operations via resolve, code/status/results/errors/total/stack field presence, NoListener/Aborted/Err distinct, concurrent nested resolve with isolated values, no automatic request/response/error event. |
| Numeric keys normalized in lexicographic order; deletes shift identities | Canonical numeric ordering distinct from explicit sorted-key helper; sparse objects | 0..12 array, insertion order variations, 01/-1/large keys, append after gaps and deletion without shifting. |
| Args/query delimiter injection or double decode | Parse argv directly; decode query once | Values containing `&`, `=`, `+`, `%`, Unicode, quoted empty values, malformed escapes and percent-encoded percent signs. |
| Multipart truncation/binary corruption | Byte parser, explicit boundary, delimiter-position/termination checks and limits | Binary bytes including NUL/CRLF, boundary-like file content, zero-byte file, quoted boundary, missing final delimiter, malformed headers/dispositions. |
| Header/cookie injection and wrong cookie deletion scope | Typed/validated values/options and multi-value headers; scope in tombstone | CR/LF names/values/options, custom codec output, repeated Set-Cookie, case variants, nondefault-path/domain delete/clear. |
| Session mistaken for identity/authorization | Pure cookie state, no implicit trust decision | Docs and API review; no automatic authorized state from a cookie or event name. Access-surface enforcement belongs to Slice 2+. |
| Lost source/cause after nested await; leaking diagnostics | Owned source chain, immutable path, separate public and developer views | Named/anonymous async failure, nested propagation and simultaneous failures; public JSON excludes file/site/secret/cause/backtrace. |
| Panic/failed callback assumed rollback-safe | No recovery promise or automatic replay; invocation values remain caller-owned | Unwind/cancellation receipt shows partial state honestly; fresh invocation still works; no unwinding catch in engine. |
| Request URL/method used as transport authority | Value parsing only, explicit Any matcher, no forwarded-header trust | Extension/invalid verbs, literal URL helper cases and route method separation. Adapter security remains unproven. |
| Neutral foundation acquires native/runtime dependencies | Two real packages only, dependency graph review | cargo tree/metadata review; direct-lib/facade consumers compile without Tokio/Tauri/UI/network/filesystem production dependencies. |
| False MSRV/target/coverage claim | Actual toolchain/target checks and per-crate reports; no stale artifacts | Clean build before testing, installed 1.96 baseline verification, separate coverage per changed crate and explicit untested targets. |

## D. Decisions applied

The owner/coordinator approved implementation with the following changes on 2026-10-24. These replace the previous open recommendations; no further product choice is pending for M0.

| Decision | Accepted contract |
| --- | --- |
| **D1** | Send dispatch futures now. Named async fns register directly through lifetime-indexed Fn/Future adaptation; closures use one small higher-ranked boxing helper. No unstable associated-type bounds or unsafe workaround. Stop with compiler evidence if the real three-argument contract cannot be proved. |
| **D2** | Builder-time mutation/removal/composition and handle replacement, immutable runtime table; listener-local captures for raw emit. |
| **D3** | Approved explicit parser/matcher grammar, limits and documented bug corrections. |
| **D4** | Approved staged one-attempt load/delivery and finalization guards; no implicit retries. |
| **D5** | Approved breadth and neutral resolve; use corrected later slices: 2 server+CLI, 3 Idea/generation/filesystem/templates, 4 SQL, 5 UI. |
| **D6** | Preserve cookie removal scope; default public error views omit developer diagnostics. |
| **D7** | Edition 2024, resolver 3, rust-version 1.96; verify installed stable, install no toolchain. |
| **D8** | Value/Nest/data-bearing values are Clone, ordinary nodes deeply independent; only the explicitly named Shared(SharedNative) variant shares an Arc payload. Request/Response remain non-Clone to avoid duplicating attempt state. |
| **DX** | Literal get plus common string dot-path reads/typed reads/setters, scalar accessors; Serialize-based results/rows/json plus explicit Value APIs; one-line priority/label registration; bootstrap mapping example. |

String dot-path reads parse internally. `path` returns None on invalid/missing paths; `path_as` and mutation return an invalid-path error rather than silently discarding it. Parsed DataPath remains optional for reuse/literal segment addressing. `results`/`rows`/`json` accept borrowed Serialize values including typed structs and serde_json::Value; serialization errors return Result before any state mutation. Value implements Serialize for its JSON variants and deliberately rejects non-JSON variants with useful paths; `results_value`/body-value methods preserve those values in-process without forcing serialization. `rows` verifies an array result rather than trusting arbitrary Serialize output.

No HTTP framework/database/application/UI/lifecycle implementation is added to Slice 1. The soundness veto and the separately scheduled reviews remain in effect.

M1–M3 implementation refinements: one-shot loaders/senders take isolated owned data snapshots so caller-borrowed native readers/writers remain possible; envelope cloning/moving is infallible and wire conversion is explicitly fallible; optional `errors: Option<Value>` distinguishes absent errors from an explicit empty object. Data/path recursion has a hard ceiling of 64 in addition to caller-lowered budgets. These details and current verification are recorded in the [M1–M3 checkpoint receipt](m1-m3-receipt.md); post-M3 rounds are still pending.

## E. Test and verification plan

### E1. Approved milestone order and review stops

Each milestone must have passing relevant tests before the next begins:

1. **M0 — scaffold/clean/signature proof.** Create only the two real packages and root facade. Before the first build run `cargo clean` and `cargo llvm-cov clean --workspace`, retaining receipts. Prove the exact named/helper three-argument handler signature, returned-future Send bound, nested emit and tokio::spawn with private request/response proof fixtures if necessary; keep fixture limitations explicit rather than presenting a partial value API as finished. **Stop and report after M0.**
2. **M1 — data.** After the coordinator's reply: Value/Nest/paths/ordered collections/parsers/files and clone/Shared/serialization proofs.
3. **M2 — queue/events/status/diagnostics.** Integrate the proved handler mechanism into the real emitter and exercise priority/ties, matching, hooks, handles/composition, recursion, concurrency and useful failures.
4. **M3 — request/response/session/cookie/resolve/facade.** Prove the real value, capability-call, typed Serialize, cookie, attempt-state and direct/facade consumer contracts. **Stop and report after M3, before refactoring rounds, with tests and current coverage.** Continue M1–M3 without intermediate approval stops after M0 is released.
5. After the M3 review, run all four E3 rounds and close with E5. The process stops do not waive the final development coverage gate.

Recheck branch/worktree and preserve unrelated edits throughout. No other trial branches, listener, commit/push or publication. Crates.io fetching for listed dependencies is authorized; sandbox network approval is requested normally if required. Test observable behavior and the A3/C failure classes rather than private implementation shape. The relevant 00006 checklist proof items 2–4 and 15 remain explicit; host/UI/generation proofs are not inferred from a foundation milestone.

### E2. Deadlock-class tests must terminate

Use both async deadlines and an **external process watchdog**. An executor timer alone cannot interrupt a callback that blocks its executor thread on a synchronous lock or never yields.

- Ordinary async ordering/concurrency tests use barrier/oneshot coordination rather than timing guesses, with a 2-second per-scenario timeout on a test executor.
- Reentrant/concurrent nested, same-event recursion, unwind/no-poison and cancellation probes run in a dedicated test subprocess. A parent test uses `try_wait` with a short bounded polling interval and a generous fixed deadline (proposed 10 seconds), kills **and reaps** a stuck child, and reports the scenario as failure with captured bounded output. Never join a possibly deadlocked worker thread as the only timeout strategy.
- Include a deliberately nonterminating helper in a test-only watchdog self-test so the kill/failure mechanism is itself demonstrated. Child selection uses an exact marker/argument and cannot recursively launch its parent.
- Run no network/listener and leave no child/executor/background work alive after success or failure. For legitimate slower instrumented runs, set an explicit larger bounded watchdog deadline and record it; do not remove deadlines.

### E3. Required review rounds

After the baseline meaningful suite passes, record distinct rounds with actual findings and relevant tests rerun after each:

1. **Logic/complexity:** phase/abort/error/cancel precedence, matching, input normalization, mutation atomicity, branch-risk tables and uncovered meaningful paths.
2. **Responsibility/abstraction:** separate matcher compilation, registry construction, invocation execution, values/parsing, delivery state and diagnostics; review borrow/ownership/coupling and remove unnecessary wrappers.
3. **Consistency/Rustdoc/comments:** consistent Result/Flow/state vocabulary, idiomatic names/imports, explicit public API, real standalone compiling examples for every public entry point, `//!` module docs, `///` on every function/method including meaningful private helpers, field/invariant docs, `# Errors`/`# Panics` where applicable, purposeful explanations per logical block and detailed branching/state/async comments. Use conventional `// ` spacing. Apply [00036](../../.agents/references/00036-rustdoc-commenting-examples.md)/[00037](../../.agents/references/00037-refactoring-round-checklists.md); JS no-space comments, JSDoc/tag restrictions, class-per-file and JS import grouping are not Rust rules.
4. **Dedicated safety/soundness:** independently trace callback lifetimes/erasure, mutable alias prevention, concurrency/recursion, cancellation/drop, panic behavior, source-chain ownership, integer/length bounds, dependency features and absence of unsafe/lock-over-await. Rerun the concurrency/watchdog/compile-fail/error suites plus affected tests.

Repeat targeted rounds for material findings. A justified no-change round records scope, conclusion and rerun results; no ceremonial edits or skipped verification. The standing instruction authorizes these routine rounds once implementation is approved.

### E4. Final commands, coverage and target limits

Planned commands once code/manifests exist (adapt only to actual compatible features):

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo llvm-cov --workspace --lcov --output-path target/coverage/workspace.lcov
cargo llvm-cov -p stackpress-lib --fail-under-lines 70
cargo llvm-cov -p stackpress --fail-under-lines 70  # only if the facade has executable lines
```

Use isolated/clean profile collection per coverage command as needed so previous measurements cannot contaminate results. Preserve text/LCOV reports and assess each changed crate separately, plus workspace integration. Record compiler, target, features, tool version, covered/total executable lines and justified exclusions. Doctests run separately; do not silently rely on unstable/nightly doctest coverage. Dependencies/tests/fixtures/build output are not production coverage. No broad source exclusions or artificial executable facade code to manufacture a denominator.

The facade may be entirely re-exports and have **no executable lines**. Report that as N/A/unmeasurable, not 100%; its proof is real import/example/doctest/consumer compilation. If executable facade code exists, its measured scope must independently reach 70%. `stackpress-lib` must reach at least **70% executable line coverage** for development, and critical risk tests remain mandatory at any percentage. Publication preparation would require 90% per published crate and a separately authorized release scope.

Verify `cargo +1.96.0 check --workspace --lib --locked` and relevant tests/doctests using the installed 1.96 compiler. Install no other toolchain and claim no 1.85 support. Inspect production dependency trees for the facade and direct-lib paths. Initial verification target is native aarch64 Apple Darwin. Other OS/architectures, WASM, Tauri and real server-adapter integration remain unverified; a Send foundation future alone does not prove those integrations.

### E5. Closeout receipt

The implementation receipt will list implemented/deferred mappings, deliberate deviations approved in D, baseline tests, each review round and tests rerun, final format/lint/doc results, per-crate coverage evidence, MSRV/target/feature scope, and unresolved limitations. Passing those checks is code evidence, not owner acceptance, publication, deployment, commit or push.

Use [00037's receipt template](../../.agents/references/00037-refactoring-round-checklists.md), extended with the dedicated safety round:

```text
Task / crate / feature / target:
Intended behavior and approved design decisions:
Baseline tests and result:
Round 1 logic/complexity — findings, changes/no-change reason, tests/result:
Round 2 responsibility/abstraction — findings, changes/no-change reason, tests/result:
Round 3 consistency/Rustdoc/comments — findings, changes/no-change reason, checks/result:
Round 4 safety/soundness — findings, changes/no-change reason, watchdog/compile tests/result:
Follow-up rounds:
Coverage tool / command / crate / features / target:
Covered lines / total lines / result / applicable 70% floor:
Excluded or unmeasured code and rationale:
MSRV result and exact toolchain:
Remaining branch/adapter/platform risks:
Unresolved user decisions:
```

**Current status:** M0 and M3 were independently reviewed and approved. All four post-M3 rounds are complete; see the [final Slice 1 receipt](slice-1-receipt.md) for findings, current checks/coverage and limitations. Earlier [M0](m0-receipt.md) and [M1–M3](m1-m3-receipt.md) receipts retain their historical checkpoint scope.

The final naming scheme uses literal `get/has/set/remove` and path `path/path_as/has_path/set_path/remove_path`, accepting either dot strings or reusable DataPath through sealed IntoPath. Standard Method/MethodPattern constants and Invocation::param cover common route/capture calls. DataPath Display quotes literal punctuation/numeric keys. Selected Nest JSON conversion and Response JSON text conversion honor their chosen budgets; Value's default wire conversion remains finite, with to_json_with for an explicit policy.
