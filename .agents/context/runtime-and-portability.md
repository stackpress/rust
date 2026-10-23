# Runtime and portability contracts

Authority: author requirements, source-observed Stackpress behavior, and the explicitly marked portability boundaries. No Rust runtime is implemented yet.

## Ordered capability execution

Keep event matching/queue execution separate from transport routing. Stackpress's all-matching prioritized listeners must not silently become a conventional single-winner router. Preserve exact/pattern matches, extracted parameters, priorities, asynchronous sequential mutation, and cooperative false cancellation.

Define equal-priority order explicitly. The source queue's stated FIFO intent and comparator/testing coverage are not sufficient proof of a complete tie contract. Keep invocation metadata local, snapshot or otherwise safely access registrations, and do not hold a global registry lock while awaiting a handler.

An event dispatch outcome (completed, aborted, no listener) differs from response/application status. Native HTTP status, a JSON result envelope, and a terminal exit code are related mappings rather than interchangeable values.

## Request/response and lifecycle

Framework requests/responses are independently constructible with optional native resources. Preserve shared data, headers, request loading, cookies/session revisions, results/rows/totals, errors, redirects, MIME/body forms, and explicit dispatch state where useful.

The general handled-invocation sequence is `request → selected operation → response → output`. Errors from a stage are offered to `error`. Existing Ingest Route can exit early; response is not inherently a finally block. Direct `emit` is lower-level dispatch and must not recursively wrap every nested operation in general lifecycle hooks.

Define top-level CLI/HTTP/Tauri entry behavior, abort/finalization, sent responses, nested context propagation, and failed error-handler fallback before implementation. Keep these unresolved details visible instead of inferring a silent lifecycle redesign.

## Diagnostic intent

Make failure reports useful for named and anonymous handlers. Retain original errors/causes, validation/source-position details, plugin/event/listener/stage/registration metadata, and nested invocation paths. Capture optional native backtraces where supported and meaningful; a thread backtrace is not a complete asynchronous or distributed call history.

Use explicit framework metadata and async-aware instrumentation as appropriate. Result-based errors and `std::error::Error` are idiomatic; preserving an `Exception` name is optional. Panic/abort recovery is a separate problem. Do not turn raw request/session contents into automatic logs.

Read [existing Exception and Rust diagnostic options](../references/00008-discussion-diagnostics-existing-parser.md) when designing reports, and [the source assessment](../context/source-map.md) when locating current error/lifecycle code.

## Generation before compilation

Idea owns syntax, AST, composition, and transform inputs; Stackpress schema/SQL/view/admin/AI packages own interpreted metadata and their generated consumers. Preserve this distinction in the Rust design.

Build-time generator plugins must compile independently of the generated final application. Keep dependencies acyclic and output ownership explicit. Rust generation may emit Rust, SQL, UI, metadata, or docs. The selected Leptos UI changes component output, not necessarily Idea syntax.

Prefer inspectable generated source, deterministic transform order, declared inputs, clean/repeat generation, and intentional stale-file cleanup. A Cargo build script may write generated Rust into OUT_DIR; a pre-build CLI may prepare source/manifests. A running build script cannot retroactively add undeclared dependencies to Cargo resolution.

Revisions are schema history, not proof of applied database state. Keep query/dialect behavior, migration planning/execution, destructive policy, transactions, and population distinct.

## Shared targets

Use shared component and operation code with explicit input/output contracts. Web SSR obtains safe public data, renders HTML, and serializes data for matching hydration. Tauri loads packaged client assets, then uses remote API or permitted local IPC adapters. Choose target behavior through adapters rather than duplicated feature implementations.

Host-specific filesystem, native resources, device permissions, notifications, signing/packaging, and network/process APIs require target-specific integration. Do not assume all optional crates compile for browser WASM. Do not expose unrestricted event execution merely because Tauri or an HTTP endpoint can call the host.

Authentication, authorization, input validation, and output disclosure belong at each access surface and/or the capability's deliberate policy boundary. Internal event dispatch does not authorize a caller automatically; client-side permission display is not enforcement.

## Compatibility limits to decide

JavaScript callable-object syntax, dynamic imports, reflection, arbitrary ECMAScript regex, React components, Node/Electron APIs, and PGlite hosting require redesign or explicit bridging. SQL dialect support does not prove PGlite embedded runtime support. Backend crate/platform documentation is not tested Stackpress integration.

Leptos/Tauri and the [initial source/crate structure](source-structure.md) are selected. The latter records the transport-neutral `server`, independent `sql` family versus `framework-sql`, initial combined `framework-ui`, and build/runtime dependency boundaries. Transport implementation, async handler signature, first database/backend, first serverless adapter, offline/local execution, detailed feature/dependency wiring, and behavioral contracts remain unresolved; accepting a tree is not a Frozen implementation spec.
