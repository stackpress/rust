# Slice 5 architecture and shared-crate boundary

Proposed contract behind [the decisions](slice-5-design-decisions.md). Existing Rust APIs were read in `crates/lib/src/{future,event,request,response,data/value}.rs`, server builder/registration/context/lifecycle/http, CLI entry/execution, Idea crate entrypoints and the two profile examples. The KB's setup-era statements that no Cargo code exists are historical; this checkpoint uses current source as implementation evidence without editing context.

## Crates, source reuse and feature graph

| Package/source | Responsibility and dependencies |
| --- | --- |
| `stackpress-framework-view` at accepted path | Typed page identity/load outcome, public snapshot/config, head/asset contract, render outcome/errors. Client-safe contracts by default; optional `server` module for ServerBuilder extension and adapter. No Leptos, Tauri or drivers in its default graph. |
| `stackpress-framework-ui` at accepted path | Internal `components`, `state`, `routing`, `language` versus `render` modules. Leptos/router/meta; explicit `ssr`, `hydrate`, `csr` modes. Default has no renderer mode or host dependencies; SSR alone enables view/server integration. No mandatory separate renderer/components crate. |
| Example contracts package under `examples/profile-ui/` | Import the **same generated Profile source file** from generated-profile's contracts partition via a documented `#[path]` module; add app-owned versioned SearchInput/SearchReply codecs. Only serde/serde_json. Do not depend on generated-profile's whole app from WASM: that app currently imports server, CLI and Tokio. No copied Profile definition. |
| Shared example UI package | Search/list/detail components, typed operation identity and route definitions. Depends on contracts and selected framework-ui mode. No database/native host dependencies. |
| Example web/native host | Depends on the existing generated-profile app/registration, server/http and SSR UI. Reuses generated server operation source without copying or altering it. Example-only gateway, if approved, remains here. |
| Tauri host + CSR entry | Native host depends on existing generated registration, Tauri and server; frontend depends on shared UI/contracts only. A tiny app-owned JS module exposes fixed search/cancel wrappers over the documented global Tauri core API (`withGlobalTauri` enabled), imported via wasm-bindgen. The global invoke bridge is not an authorization barrier; native command allowlisting/ACL and CSP enforce admission. No Node/npm dependency is needed for this binding. |
| Root facade | Optional explicit `framework_view` and `framework_ui` re-exports/features; direct lower-level dependencies, never a back-edge through stackpress. Coordinator integrates manifest/lock/facade changes. |

The `#[path]` reuse is a proof-app boundary, not a proposed public package distribution strategy. The generator already emits standalone serde contracts. A separately publishable generated-contract crate is S5-F3 work; if source-file sharing proves unsuitable in M0, request that example-generator output change through the coordinator. Removing Profile should make a consumer that requires it fail compilation; no hidden handwritten fallback is allowed.

Cargo features are additive. Target-specific dependency declarations and separate native/client packages enforce graph separation; a feature name alone is insufficient. Build native SSR, browser hydrate and Tauri CSR in separate Cargo invocations/output directories. Explicitly reject invalid native render-mode combinations. Check `cargo tree --target wasm32-unknown-unknown --edges normal,features` on the **actual client package**, plus a real wasm build.

## Dependency budget for coordinator review

| Scope | Proposed dependency/feature budget |
| --- | --- |
| view contracts | serde derive, serde_json (already used); bounded explicit DTOs and data-only head metadata. Optional lib/server for native registration, not default/browser. |
| UI | pinned Leptos 0.8.22, router 0.8.17, meta 0.8.7; target wasm-bindgen/js-sys/web-sys features only for DOM, fetch/AbortController and host bindings actually used. No direct custom unsafe or unconditional Send wrappers. |
| Native example | existing lib/server/http/generated-profile app, serde/JSON, existing Tokio; no SQL. Optional example gateway: hyper 1 + hyper-util Tokio adapter + http-body-util and Tokio net/time, isolated from framework server/defaults. This extra budget depends on D07 approval. |
| Tauri | tauri 2.12.5, build-time tauri-build 2.7.4; no SQL/filesystem/shell/http/plugin bundle by default. Existing Tokio/futures cancellation primitives as justified by M0. |
| Build/tests | matching wasm-bindgen CLI, Tauri CLI; existing cargo-llvm-cov. Browser automation uses available tooling; no new test framework install is presumed. any_spawner Tokio is needed only if component async SSR is explicitly added. |

Dependency versions above are assessment pins, not an instruction to modify current manifests now. Exact transitive normal/build/dev trees and compile costs are recorded at M0. New rich-editor, sanitizer, localization/date dataset, CSS runtime, codegen or network libraries require an approved later milestone.

## Page, routing and rendering ownership

1. Stackpress's existing all-matching prioritized method/path listeners remain native routing authority. A `ServerBuilder` extension supplied by view registers an ordinary page handler; it does not add a competing HTTP router or special lifecycle.
2. Use `/ui/profiles?id=abc` and `/ui/profiles/:id` for UI. Preserve existing generated `/profiles/:id` JSON route. Add only a fixed typed `/ui-api/v1/profile-search` endpoint; caller data never selects an event or page module.
3. A named async page loader validates inputs and delegates the existing capability within the current top-level invocation. Child Abort returns PageLoad::Abort; errors stay Result failures. There is no second `Server::call` lifecycle inside a page.
4. After loading, explicitly project approved fields into an owned PageSnapshot. Render synchronously in a new per-request Leptos Owner, collect Head and hydration-marked body, then clean the Owner before returning. The resulting String crosses the HTTP boundary; Request/Response/native services never enter reactive state.
5. Ordinary `response` hooks still run per G12 and may deliberately replace output. Changing response results after page rendering does not implicitly rerender HTML. Plugins that customize page data do so in the loader/delegated operation; rendering does not freeze or bypass lifecycle policy.
6. The same compiled route catalog gives Leptos Router its client search/detail routes and Tauri a memory/history adapter compatible with packaged URLs. Native Stackpress matching still executes every matching native listener. Client route selection does not change that contract.

Client navigation uses ordinary anchors/GET search forms before hydration and intercepted router navigation afterward. Back/forward, query encoding, non-root base paths, unknown/detail-not-found paths and full refresh must agree. Tauri must not rely on a remote server to resolve history paths: use the approved packaged route adapter and test reload/navigation in WKWebView. Never interpret user paths as dynamic Rust/JS module imports.

Head uses `leptos_meta` context per render and per client root: title, language, description, stylesheet/asset links and optional public canonical URL. Collect after page data is available; provide one Head owner, stable dedup keys and identical initial metadata. No raw Head HTML from input. Asset URLs/base paths are trusted build metadata; missing/mismatched assets produce an explicit version error/reload path, not guessed paths.

## Snapshot and wire contract

`PageSnapshot<T>` contains contract revision, build/asset ID, PageId, canonical public path/query, explicit locale/theme defaults, optional **public** caller display fields only when a future feature supplies them, page data and a safe public status/error. No request headers/cookies/session map, full URL credentials, native handles, arbitrary config or developer reports are serialized. The demo has no auth/session requirement.

Serialize a bounded JSON value into a non-executable `<script type="application/json" id="stackpress-page">`. Escape `<` (including `</script>`), `>`, `&`, U+2028/U+2029 using JSON Unicode escapes; preserve roundtrip data. Read with textContent and a strict typed decoder. HTML text/attributes use Leptos escaping. This is an explicit correction to upstream's raw JSON marker replacement, not a claim that JSON.stringify alone is safe.

Bounds initially: 64 KiB search request, 1 MiB public snapshot and 2 MiB final HTML, lowerable by app policy; measure/enforce serialized lengths and bounded page component inputs. Output buffer limits alone cannot prevent arbitrary trusted component allocation/CPU loops; the framework is not a sandbox. M0 boundary tests fix exact defaults before public API stabilization.

Search input is a nonempty, bounded UTF-8 ID prefix; public validation is shared, with host validation authoritative. SearchReply decodes the existing success/error envelope into `Vec<Profile>`, total, public error fields and contract revision. Revision/build mismatch refuses hydration/call with a visible refresh-required state. Typed Rust reuse does not replace wire validation. No automatic retries or fallback between local/remote execution.

## One operation boundary, three adapters

- **SSR:** a borrowed native page adapter delegates `profile-search` with current Request/Response/Invocation. No self-referential host service, Arc<Server> cycle or `'static` capture of borrowed Invocation. The resulting typed reply becomes the seed.
- **Hydrated browser:** a local UI client calls the fixed HTTP endpoint; a concrete route handler maps it to the same named operation. Same-origin by default, bounded input/output, strict revision, sanitized status. No dynamic event names or unrestricted CORS.
- **Tauri:** packaged CSR calls `profile_search` with a versioned DTO. The native command chooses the fixed operation after checking trusted window/origin policy and input. Register the same generated operation source; run `handle_with_options` exactly once, attach trusted adapter context and publish once. HTTP result codes are mapped to a typed public reply, not mistaken for IPC transport success.

The UI-facing call future may be local on WASM; native page loading must remain Send. Do not impose a Send trait-object future on browser fetch/JS values or use unsafe lifetime/Send coercions to unify them. Shared call names/types and controller behavior are the DX contract; native versus local future storage belongs inside adapters. The initial component reads the seed on SSR/hydration, then invokes the same typed search method for user actions. General async component/server-resource fetching is S5-F6.

## Tauri admission and finalization

Expose only `profile_search` and `cancel_profile_search`, with explicit generated app-command permissions via `tauri_build::AppManifest::commands` and capabilities restricted to the local main WebView. Listing an invoke handler alone is insufficient ACL evidence. No remote URL capability, wildcard invoke, filesystem/shell grant, raw event execution or caller-supplied identity is allowed by this demo's exposure policy. The native command also enforces its allowlist; capabilities protect an edge, not the operation in every process context. [Capability model](https://v2.tauri.app/security/capabilities/).

Once admitted, normalize to Request; attach trusted window/call metadata using existing HandleOptions; enter existing host lifecycle; use OutputGuard for one wire conversion/publication. Denied IPC does not trigger domain lifecycle. Invalid payload before Request follows the same no-fabricated-request principle as G17; if a common sanitized Input report is required, raise the small public-helper change listed below. No recursive error offers, leaked HandleReport or second finalization after stopped/sent/interrupted output.

Native commands must not hold a Tauri State guard or registry mutex across await. Clone an explicit Arc<Server>, take bounded metadata under a short lock and drop the guard first. Panic remains distinct from a recoverable handler Result: fail the invocation and dispose render/call state; never retry a poisoned owner or claim rollback. No blanket catch_unwind that fabricates success.

## Cancellation and bounded work

The component controller owns a request generation and cancellation handle. A new search or route disposal cancels the prior request, suppresses its stale result and releases listeners. Web uses AbortController; cancellation requests may race completion, so generation checks remain necessary. Dropping fetch does not prove a remote operation was rolled back.

IPC promises do not cancel native work on navigation automatically. Use a scoped call ID and a native bounded in-flight map (suggest 32/window), explicit cancel command, finite deadline (suggest 5 seconds), and RAII map removal. A cancel ID is valid only for its originating trusted window; duplicates error, unknown completed IDs are harmless. The native select/abort boundary drops the still-borrowing lifecycle future; no spawned detached domain task owns borrowed requests. No rollback or exactly-once domain side effect is promised; the first capability is read-only.

Global registry and Owner locks are unnecessary. If a small cancellation map mutex is introduced, contention/poisoning returns a sanitized error with native diagnostics; never unwrap or silently clear poison. Shutdown closes admission, cancels and awaits finite in-flight work before releasing host state. Test this with barriers and a parent process deadline.

## G23 shared-crate change list

| Shared owner | Required change in the recommended design | Justification/sequence |
| --- | --- | --- |
| `stackpress-lib` | **None.** Existing Send Handler, owned Response/HTML, Result, public wire conversion and explicit OutputGuard integration suffice. | Any proposed new native value/response mode must stop for coordinator review. Do not weaken Send or make Request/Response Clone. |
| `stackpress-server`, including HTTP | **None for buffered SSR and valid admitted IPC.** Use existing registration, `handle_with_options`, delegate and typed services. | No hidden streaming, UI dependency or render registry is added to server. |
| server Input reporting (conditional) | If IPC must allocate the host's standard Input-stage HandleReport before Request normalization, expose a narrow checked adapter failure helper; currently `input_failure` is crate-private. | Coordinator decides whether an adapter-owned sanitized rejection is adequate or a reusable public helper is warranted. This is a potential public API change, never a lane-local edit. |
| server HTTP trusted context (conditional) | Existing HttpAdapter always uses default HandleOptions. No change for this public unauthenticated demo. A future authenticated gateway needs explicit trusted-option propagation. | Report now so a UI lane does not smuggle caller authority through JSON. Authentication work is S5-F4/another task. |
| CLI | **None.** Existing app-owned generation CLI and `emit generate --check` remain. | Build scripts are finite app tooling; no new stock binary or grammar synonym. |
| Idea crates | **None.** Consume existing generated Profile source and registry. | Generic schema UI semantics/generation is S5-F3. If M0 needs additional independent contract output, coordinator sequences an example-codegen change, not an unreviewed parser/runtime dependency. |
| root Cargo/lock/facade | Add actual view/UI/example packages, optional re-exports and reviewed locked dependencies. | Coordinator-assigned single owner integrates with Slice 4. Both lanes use --locked after each serialized dependency update. |
| existing generated-profile app/codegen | None preferred; use a new profile-ui example and source-file reuse. | Any required change is coordinated because generated outputs share ownership and other agents document Idea. Do not hand-edit owned generated artifacts. |
