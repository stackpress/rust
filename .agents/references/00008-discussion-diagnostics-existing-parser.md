<!-- kb-import:start -->
> Scope: preserved discussion history. Confirmed current Rust decisions live in [project direction](../context/project-direction.md). Proposed APIs, old paths, and historical status in this source are not implementation instructions.
> Source: `/Users/cblanquera/server/projects/stackpress/rust-notes/stackpress-rust-feasibility/diagnostics-and-existing-parser.md`; SHA-256 `b89e1469814b4db69dbbd91cc394b0865b8dae98d1de1488e5a36ff152c92be8`; captured 2026-10-23. [Exact original](../resources/conversation/2026-10-23-stackpress-rust/diagnostics-and-existing-parser.md). Prose links are rebased; source content otherwise retained.
<!-- kb-import:end -->

# Diagnostics and existing Rust parser

Source-reviewed notes as of 2026-10-23. Design proposals are not implemented APIs or test results.

## Confirmed updates from the author

- Leptos + Tauri is selected.
- Share application code across web/mobile/desktop, except hardware-specific integration; use adapters for necessary execution-host differences.
- Disabling a production plugin means rebuild and redeploy.
- Preserve Ingest's general request/response/error events.
- Rename the proposed foundation to `stackpress-lib`.
- Assess existing Rust Idea parser work before writing a parser.
- Existing Idea, Inquire, and Stackpress repositories are monorepos.
- Prefer idiomatic Rust, with no requirement for a 1:1 repository/package conversion.
- Plugin layout is recommended and unopinionated. A registration function is the essential framework contract.

## What the current Exception provides

Source inspected: [lib/src/Exception.ts](../../../lib/src/Exception.ts).

The current TypeScript Exception extends Error, carries code/status/type, validation errors and source start/end positions, upgrades ordinary errors while preserving their original stack, parses V8-formatted stack strings into frame-like data, and creates structured response output.

The Rust design should preserve the useful diagnostic contract while using Rust error values and async instrumentation. Parsing a particular engine's stack string need not remain the foundation.

## Four complementary diagnostic layers

1. **Typed error and source chain.** Implement `std::error::Error` and retain the underlying cause through `source()`. Preserve useful domain/status/validation/source-position fields. `Result` errors do not automatically acquire a backtrace just because they implement Error.
2. **Framework invocation metadata.** Record plugin identifier, selected event/route, matched pattern, lifecycle stage, listener identifier/name, priority, and registration location. Use per-invocation metadata so concurrent calls cannot overwrite each other.
3. **Async logical spans.** Instrument each event/listener future and propagate context to spawned work. This follows the operation path across awaits even when native thread frames do not show the whole history.
4. **Optional native backtrace.** Capture near the original failure when supported/enabled. Preserve that capture when wrapping an error rather than replace it with the later error-handler location. Release inlining/symbol stripping and target support affect detail.

`tracing` provides async-aware spans and instrumented futures. `tracing-error` offers captured span traces, but its own documentation labels it experimental; it is an optional candidate, not a selected required dependency. Stackpress can retain structured invocation breadcrumbs independently so the public diagnostic contract does not depend on this crate.

The exact error type name remains open: a Rust `Exception` struct is possible, while `Error`/`Report` naming and Result-based propagation are more idiomatic alternatives. No need to recreate JavaScript throw/catch semantics for ordinary failures.

## Anonymous handlers can remain allowed

Named functions improve source navigation, but requiring names would make the minimal plugin contract more opinionated than intended. The framework can report a closure using its plugin/event/registration location and an optional label.

A synchronous registration API can use `#[track_caller]` and `Location::caller()` to record the call site. `file!`, `line!`, and `module_path!` are also useful for explicit registration helpers. This identifies registration, not automatically the precise original failing expression. Error construction or instrumentation must capture the latter separately.

Do not infer a robust public handler name from erased Rust function pointers or require full runtime reflection. Once type erasure occurs, explicit metadata remains the reliable contract.

Illustrative report; not actual output:

```text
payment declined

Cause: provider returned a decline response
Invocation: web request → checkout-submit → payment-charge
Stage: selected operation
Plugin: payments
Listener: charge
Registered: plugins/payments/plugin.rs:18
Original error: plugins/payments/events/charge.rs:74
Request correlation: 7f...

Native backtrace: available in the developer report when captured
```

Optional `#[instrument(skip_all)]` on a named async function can provide further internal spans without recording all arguments. Dispatcher-level instrumentation supplies baseline reports even if a plugin author adds no tracing annotations. Do not hold an entered span guard across await; instrument the future. Explicitly propagate context into spawned tasks.

Error chains, invocation records, and correlation IDs can cross HTTP/IPC boundaries in an intentional diagnostic envelope. They are not a literal native stack spanning different processes/devices. Keep developer diagnostics distinct from the public response message and field errors.

## Lifecycle handling and error reporting

Source inspected: [Ingest Route](../../../ingest/ingest/src/Route.ts).

The existing handled-request path has prepare (`request`), process (selected event/action), and shutdown (`response`). Stage failures are converted into response exception state and offered to `error`. Early aborts can skip later stages. Direct emit is lower-level dispatch and does not itself mean a whole transport lifecycle.

The Rust proposal retains those names and roles. Specify how CLI and Tauri top-level invocations enter the lifecycle and how nested calls bypass or participate. Do not run request/response recursively for every internal event.

An error handler that fails needs a terminal fallback preserving both failures, not infinite recursive error emission. Panic handling is separate from Result failure; aborting panics and memory/resource failures cannot be promised recoverable by an error event.

## Existing Idea Rust parser branch

User reference: [feature/rust-parser-2](https://github.com/stackpress/idea/tree/feature/rust-parser-2).

Verified branch head: `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`.

Verification method: remote `git ls-remote`, GitHub tree API, and `git show` against the existing local remote-tracking branch. The tree and remote head matched. The local Idea checkout stayed on main; no branch checkout, merge, code edits, dependency installation, or Rust tests were performed.

Observed structure:

```text
packages/idea-rust/
├── Cargo.toml                         # Cargo workspace
└── crates/
    ├── idea_parser_core/
    │   ├── Cargo.toml                 # serde + serde_json
    │   └── src/
    │       ├── lib.rs
    │       ├── token.rs
    │       ├── ast.rs
    │       ├── parser.rs
    │       ├── finalize.rs
    │       ├── error.rs
    │       └── types.rs
    └── idea_parser_node/
        ├── Cargo.toml                 # cdylib, napi, napi-derive
        ├── build.rs
        └── src/lib.rs                 # binding/error conversion

packages/idea-node/                    # JS loader/tooling
packages/idea-node-<platform>/         # native-addon distribution
```

The core exports tokenizer types/functions, `parse_ast`, schema compilation/finalization, schema/config types, and structured parser errors. The Node crate exposes those operations through N-API and translates error fields into JS errors. It does not own the parser implementation.

The Rust workspace uses edition 2024. The Node package contains napi build commands and optional platform package dependencies for OS/architecture/libc combinations. This confirms the author's publishing concern in the Node distribution design.

## Distribution consequence

For a Rust consumer:

```text
Rust Stackpress generation
  → direct Cargo dependency on Idea parser core
```

No Node binding or OS-specific npm package is inherently required. Use a path dependency during development, a Git dependency pinned to a reviewed revision, or a source crate publication if/when chosen. No publication or extraction was performed.

Native executable releases still need builds for their intended targets. This differs from publishing a family of npm packages to load a native addon. A generator/build parser ordinarily runs on the build host; it does not need to ship inside the final mobile/browser app. Editor/browser parsing, if later needed, could warrant a separate WASM wrapper and compatibility assessment; no WASM build is proven here.

No claim is made about full grammar parity, use/import composition, all diagnostics, environment behavior, generated schema parity, or production readiness. The core's source separation is verified; completeness must be audited against current fixtures and TypeScript behavior.

## Next proof additions

- Test the existing Rust parser core against current accepted Idea fixtures before deciding what to extract or extend.
- Preserve current schema/metadata semantics where wanted, while allowing deliberate idiomatic Rust API changes.
- Exercise a named listener and anonymous listener failure after an await.
- Check nested event context, concurrent invocations, spawned work, and errors raised inside lifecycle listeners.
- Verify a useful baseline report even with native backtraces disabled/unavailable and without plugin-specific annotations.
- Verify error-handler failure does not recurse and original cause information remains.

## Primary references

- [Rust Error/source chain](https://doc.rust-lang.org/std/error/trait.Error.html)
- [Rust Backtrace](https://doc.rust-lang.org/std/backtrace/struct.Backtrace.html)
- [Caller location](https://doc.rust-lang.org/std/panic/struct.Location.html)
- [tracing instrument](https://docs.rs/tracing/latest/tracing/attr.instrument.html)
- [Instrumented futures](https://docs.rs/tracing/latest/tracing/trait.Instrument.html)
- [tracing-error](https://docs.rs/tracing-error/latest/tracing_error/)
- [Cargo library paths](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)
- [Pinned parser core manifest](https://github.com/stackpress/idea/blob/6c289d1f3fba17c758baafb2d4dcefd87fd448ea/packages/idea-rust/crates/idea_parser_core/Cargo.toml)
- [Pinned parser core exports](https://github.com/stackpress/idea/blob/6c289d1f3fba17c758baafb2d4dcefd87fd448ea/packages/idea-rust/crates/idea_parser_core/src/lib.rs)
- [Pinned Node binding manifest](https://github.com/stackpress/idea/blob/6c289d1f3fba17c758baafb2d4dcefd87fd448ea/packages/idea-rust/crates/idea_parser_node/Cargo.toml)
- [Pinned Node package](https://github.com/stackpress/idea/blob/6c289d1f3fba17c758baafb2d4dcefd87fd448ea/packages/idea-node/package.json)
