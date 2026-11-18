# Stackpress Rust project direction

Authority: current source/manifests, accepted owner decisions and verified receipts, as routed by [source catalog](source-catalog.md). [Source structure](source-structure.md) records the implemented workspace, features and public namespace.

## Identity and architecture

Stackpress is one Rust monorepo with independent libraries, a root `stackpress` facade and explicitly composed framework plugins. Retain the understandable event/config/plugin architecture while using idiomatic Rust; repository/npm boundaries and JavaScript syntax do not dictate Rust abstractions. Readability requires deliberate responsibility boundaries, consistent patterns, Rustdoc and explanatory comments.

The foundation is `stackpress-lib`. Convenient imports use valid Rust identifiers, such as `stackpress::sql_mysql` and `stackpress::framework_admin`; independent crate use remains possible. The facade's default selects only the foundation among Stackpress dependencies. Public exports and Cargo features make code available; runtime registration remains explicit.

Named capabilities share caller-owned, independently constructible request/response values across direct/in-process, CLI and HTTP entry points. The server supplies typed layered config, services, startup and a general `request → selected operation → response` lifecycle with `error` handling. Nested dispatch remains raw. Prioritized listeners are all-matching, sequential, and cooperative: `Flow::Abort` is the Rust abort signal, and equal priorities use registration order. See [runtime and portability](runtime-and-portability.md) for finalization and error behavior.

## Plugin and source-port contract

A registration function is the essential plugin contract. A plugin can be one module/file; reusable plugin crates may choose their library entry path. `events/`, `pages/`, `views/` and `tests/` are optional organization. No class, mandatory trait or directory scaffold is required. The [blog](../../examples/blog/README.md) demonstrates separately compiled plugin crates.

Plugins are selected before compilation; disabling one requires rebuild/redeploy. The host validates selected plugin IDs and declared dependency presence/order, preserves the application's order and freezes registrations at build. Enabled plugins share a process; crate boundaries do not provide crash isolation or reverse stored data/migrations on removal.

Preserve the original framework behavior (G42/G43). Duplicate route/event handlers are intentional: all matching listeners run by priority/FIFO until abort. App and plugin authors own route inventories and each process's protection; configured session/API/CSRF mechanisms do not imply an added global default-deny policy. Missing configuration follows each constituent's source activation rules. Do not add hardening, different security semantics or substitutes without an accepted decision. [Named source-port exceptions](runtime-and-portability.md#accepted-source-port-exceptions) are bounded changes, not permission to redesign adjacent behavior.

## Generation and models

The native Idea parser is implemented at [crates/idea/idea-parser](../../crates/idea/idea-parser/PROVENANCE.md), extracted and repaired from the recorded Rust parser core. Node/N-API distribution is separate and is not a Cargo dependency. Assess and extend this implementation before proposing another parser.

Idea composition and app-owned generation run before final application compilation. Framework packages ship their original `.idea` sources. Apps import, extend and generate the model families consumed by session/API/admin/aggregate plugins. No premade `framework-models` package or stock Profile/Auth/Application/Session family remains (G59). Use [runtime generation contracts](runtime-and-portability.md#generation-and-application-owned-families) for package imports and `register_with`.

## UI, targets and databases

Leptos supplies shared components and separate SSR, hydrate and CSR artifacts. `stackpress-leptos` is independently usable; `framework-view` adds page contracts and optional host adapters. Share applicable validation, contracts, UI and operation source across web and installed targets, isolating native/execution-host differences in adapters. Browser WASM receives deliberately public snapshots, without server secrets or database access.

Tauri remains the selected installed-app host and `examples/profile-ui/native` retains its native proof. The release blog is web-only: desktop packaging is deferred and MCP/`stackpress-ai` is unsupported, with original MCP config retained as reference (G52). Native/browser builds and Chromium proofs do not establish installed mobile/desktop acceptance. No offline persistence/synchronization promise follows from shared source.

The implemented database engines are MySQL, PostgreSQL and SQLite. Native adapter crates use SQLx; SQLite supplies the file-based option. PGlite is not required and no connector is planned (G46). Keep independent SQL contracts distinct from framework-generated persistence, migration tooling and source-order schema setup.

## Delivery and contributor rules

The 23 release packages use lockstep `0.2.1` beta manifests, edition 2024 and MSRV 1.96. See [release and licensing](release-and-licensing.md) for the 11/12 licence split, attribution and release checks. Publication remains an owner action.

Use [Rust style](rust-code-style.md), [testing and quality](testing-and-quality.md), [programming](../workflows/rust-programming.md) and [coordinated development](../workflows/coordinated-crate-development.md). Preserve unrelated changes; commits, pushes, deployment, publication and KB MCP publishing require explicit authorization. Development listeners use an available session installation of devmetrics.

Historical intent, rejected frontend recommendations and resolved setup questions are preserved in [the preimplementation direction and runtime reference](../references/00042-preimplementation-direction-runtime.md); load it only when auditing earlier rationale. Current claims come from this context and its linked source.
