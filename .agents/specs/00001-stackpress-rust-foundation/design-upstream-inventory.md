# Design: authority, scope, and upstream inventory

Spec file of [00001 Stackpress Rust foundation](index.md). Load for the Slice 0+1 design authority, scope, and the complete upstream `lib` export/type/test mapping (design sections preamble and A). Moved losslessly from `docs/trial-3/slice-1-design.md` lines 1–186 at commit `e230359`; headings are demoted one level and links rebased.

## Slice 0 + Slice 1 design checkpoint

Status: **approved with owner/coordinator changes; implementation authorized in milestones with explicit review stops**.
Prepared on 2026-10-24 in `trial/3`; revised after coordinator/owner approval. Design examples remain sketches until their milestone compiles and tests them; milestone receipts record actual verification separately.

### Authority, observations, and scope

The current task and [AGENTS.md](../../../AGENTS.md) govern this proposal. The accepted owners are [project direction](../../context/project-direction.md), [runtime and portability](../../context/runtime-and-portability.md), [source structure](../../context/source-structure.md), [Rust style](../../context/rust-code-style.md), [quality policy](../../context/testing-and-quality.md), and the [programming workflow](../../workflows/rust-programming.md). I also read the required references [00039](../../references/00039-open-design-questions.md), [00034](../../references/00034-foundation-source-map.md), [00019](../../references/00019-upstream-runtime-api-contracts.md), and [00008](../../references/00008-discussion-diagnostics-existing-parser.md).

The coordinator addendum is incorporated using the relevant readability/extension/proof sections of [00006](../../references/00006-discussion-crate-plugin-proposals.md), the lib inventory/redesign boundaries in [00005](../../references/00005-discussion-feature-feasibility.md), capability-call examples in [00030](../../references/00030-upstream-operational-examples.md), lifecycle ownership in [00012](../../references/00012-upstream-architecture-and-composition.md)/[00024](../../references/00024-upstream-cli-and-plugin-contracts.md), and the full documentation/round references [00036](../../references/00036-rustdoc-commenting-examples.md)/[00037](../../references/00037-refactoring-round-checklists.md). Historical crate proposals do not override the accepted current tree.

Source observations use only this branch, the project KB, and the current sibling `lib` checkout. This branch was clean at inspection, at `90dca8737b00cf0ded9dfe2f4a34d6538c9513b8`. Upstream `lib` was clean at `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a`. No other `trial/*` branch or stale `target/` output was inspected. At initial design inspection there was no Cargo manifest or Rust implementation on this branch. Installed compiler and Cargo report 1.96.0; the owner selected this installed compiler as the MSRV. No additional toolchain will be installed.

Slice 0 will create only the root workspace/package `stackpress`, its `src/lib.rs` facade, and `crates/lib` (`stackpress-lib`), with one lockfile. Slice 1 will implement transport-neutral data/parsing, ordered queues, event matching/dispatch, request/response/session values, status, and diagnostics. This document adds no empty crates.

Named later work below means a responsibility and acceptance boundary, not a scheduled commitment:

- **Slice 2 — capability host + CLI adapter:** `stackpress-server` and CLI in `tools/`; configuration/plugin/service composition, top-level lifecycle, native transport envelopes, output/streaming/routing adapters, Terminal/control/input and process/exit behavior. Neutral `resolve(event, input)` is in Slice 1.
- **Slice 3 — Idea/generation:** existing parser assessment, transformer, generation, host filesystem/FileLoader/NodeFS capabilities and templates. No new parser is proposed here.
- **Slice 4 — SQL:** independent query/dialect/backend capabilities and framework integration as implemented.
- **Slice 5 — UI:** shared Leptos UI and Tauri adapters, with their own target proofs.

### A. Upstream inventory mapping

The [upstream index](../../../../lib/src/index.ts) exports **49 runtime values and 64 types**. Each appears below. “Implement” means preserve the useful capability in Slice 1 under the indicated Rust API; it does not promise the JavaScript identifier, inheritance tree, callable-object syntax, or unchecked casts. “Not ported” identifies deliberately excluded behavior rather than an untracked omission.

#### A1. All 49 runtime exports

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

#### A2. All 64 type exports

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

#### A3. Behavior mapping for all 26 upstream test files

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
