# Slice 5 Leptos and Tauri assessment

Evidence for [checkpoint decisions](slice-5-design-decisions.md), accessed 2026-10-25. Compiler probes live only in `/private/tmp/stackpress-s5-probe/`, a standalone Cargo workspace with its own new target/lockfile. Existing repository target output and other trial branches were not used. Crates were fetched; no targets, binaries, system packages or Xcode components were installed. No server, WebView or listener ran.

## Current stable candidates and MSRV

Queried crates.io's `/api/v1/crates/<name>` API, selected the latest non-yanked version without a prerelease suffix, and retained each response as `<name>-registry.json` in the disposable directory. These are point-in-time versions; transitive resolution must be locked and checked again before implementation/publication.

| Package | Stable candidate | Declared Rust minimum | Evidence/limit |
| --- | --- | --- | --- |
| `leptos` | 0.8.22 | 1.88 | Native SSR compiled and ran on 1.96.0. [API](https://docs.rs/leptos/0.8.22/leptos/). |
| `leptos_router` | 0.8.17 | 1.88 | Compiled in the probe; actual route/refresh/base-path integration remains M0/M2. [API](https://docs.rs/leptos_router/0.8.17/leptos_router/). |
| `leptos_meta` | 0.8.7 | 1.88 | Compiled; Head collection/deduplication/hydration remains M0/M2. [API](https://docs.rs/leptos_meta/0.8.7/leptos_meta/). |
| `tauri` | 2.12.5 | 1.90 | Native Wry builder and async command compile; see probe receipt below. [Package](https://docs.rs/crate/tauri/2.12.5). |
| `tauri-build` | 2.7.4 | 1.90 | Compiled transitively; real app config/build-script/capability asset embedding remains M0. [Package](https://docs.rs/crate/tauri-build/2.7.4). |
| `tauri-cli` | 2.12.4 | 1.90 | Candidate only; not installed/run. [Package](https://docs.rs/crate/tauri-cli/2.12.4). |
| `cargo-leptos` | 0.3.11 | 1.82.0 | Candidate only; not installed/run or proved against the custom host. [Official tool](https://github.com/leptos-rs/cargo-leptos). |
| `trunk` | 0.22.0 | 1.96.1 | Not compatible with installed 1.96.0; do not follow an unversioned install guide. [Package](https://docs.rs/crate/trunk/0.22.0). |
| `wasm-bindgen-cli` | 0.2.129 | 1.86 | Not installed. Must match the probe-resolved `wasm-bindgen` 0.2.129 exactly when processing its WASM. [Tool](https://rustwasm.github.io/docs/wasm-bindgen/reference/cli.html). |

The full probe resolved with Cargo resolver 3 and `rust-version = "1.96"`; Cargo reported 490 packages selected for Rust 1.96 compatibility (including optional native dependencies). Native compilation is stronger evidence than declared MSRV alone. WASM execution, GUI behavior, packaging and cross-platform compatibility are not proved by it. No prerelease Leptos 0.9 adoption is proposed.

## Host inventory

- `rustc 1.96.0 (ac68faa20 2026-05-25)`, Cargo 1.96.0, LLVM 22.1.2; host `aarch64-apple-darwin`.
- Installed Rust target: **only `aarch64-apple-darwin`**. Components include clippy, rustfmt, rust-src and llvm-tools; no `wasm32-unknown-unknown` standard library.
- macOS 26.6.2, arm64; `xcode-select -p` → `/Applications/Xcode.app/Contents/Developer`; `xcrun --show-sdk-version` → 27.0. The sandbox emitted Xcode cache/file-event warnings, but SDK discovery succeeded.
- Cargo subcommand inventory: no leptos or tauri; PATH lookup: no trunk or wasm-bindgen. Node 26.2.0/npm and pnpm are present, but are unnecessary for the recommended direct build route.
- No assumption is made about signing identities, provisioning profiles, iOS simulator runtimes or Android SDK/NDK/JDK. They were not needed for the desktop design and were not installed or broadly inspected.

Host inventory commands: `rustc -Vv`, `cargo -V`, `rustup target list --installed`, `rustup component list --installed`, `cargo --list`, `command -v trunk wasm-bindgen node npm pnpm`, `xcode-select -p`, `xcrun --show-sdk-version`, `sw_vers`, `uname -m`.

## Disposable Cargo probes

The manifest pins Leptos/router/meta/Tauri above, uses existing lib/server/generated-profile path dependencies, and enables server's `http` feature. The first fresh build uses only the new disposable target directory. Test binaries deliberately use concise assertions/unwraps; they are not proposed production code or a source archive.

| Probe / exact invocation after `cargo` | Observed result | What it establishes |
| --- | --- | --- |
| `run --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --bin stackpress-s5-probe --offline` | PASS: HTML contains `Ada Lovelace`; `buffered-ssr.log`. | A named Send page handler delegates through `Invocation::delegate` to the real generated operation, creates/cleans an Owner after its final await, calls `to_html`, sets `Response::html`, and succeeds through the existing `HttpAdapter`. |
| `run --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --bin async_ssr --offline` | PASS: delayed Suspense resolves to `<p>Resolved</p>`; compile-time `T: Send` assertion passes; `async-ssr.log`. | In-order Leptos render streams can be fully collected into String by a Send future on the native Tokio executor. No network streaming is needed. This probe does not prove Resource snapshot serialization, request borrowing in `'static` resources or real hydration. |
| `check --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --bin non_send --offline` | Expected failure 101: future not Send because `Rc<String>` survives an await; `non-send.log`. | Existing ServerBuilder handler bounds reject a render-local value crossing an await. No unsafe Send, erased lifetime or weakened handler contract is needed. |
| `check --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --features native --bin native` | PASS, 41.30 seconds after fetch. | Tauri 2.12.5 default native Wry dependencies and an async `#[tauri::command]` builder compile on this macOS SDK/Rust combination. Does not validate generated context/ACL or launch a WebView. |
| `build --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --features native --bin native --offline` | PASS, native executable linked in 37.30 seconds; `tauri-link.log`. | Native linking assessment only; no `.app` bundle, CLI run or GUI launch. |
| `check --manifest-path /private/tmp/stackpress-s5-probe/Cargo.toml --target wasm32-unknown-unknown --no-default-features --features hydrate --bin stackpress-s5-probe --offline` | Expected failure 101, E0463 missing `core`; `wasm-missing.log`. | Confirms the installation gate. This binary also has host dependencies, so even after target installation it is not the intended browser build. A clean client-only manifest/tree is required by M0. |

Two initial probe-authoring errors were corrected: `Response::html` accepts just the HTML value; executor initialization is `any_spawner::Executor::init_tokio()` with its Tokio feature, not a public `leptos::task::Executor`. The final async probe handles the initialization Result. These corrections changed only temporary files. Any probe failure or limitation remains distinct from production integration results.

## Buffered rendering and integration model

The existing HTTP API is `http::Request<Vec<u8>>` → `HttpReply`/`http::Response<Vec<u8>>`; callers own network buffering. It supports no backpressure or streaming. `Response::html(String)` is already supported; the page adapter can render after its async loader and before returning its ordinary handler result. No server or HTTP signature change is required for this design.

Leptos supports synchronous, fully asynchronous and streamed rendering. Collecting a Leptos stream before native output remains a **buffered** response: no early TTFB, progressive Suspense delivery or infinite stream is promised. The recommended vertical awaits data first, then produces deterministic markup and one escaped explicit JSON snapshot; it does not depend on Leptos server functions or resource replay. [Official rendering modes](https://book.leptos.dev/ssr/23_ssr_modes.html).

The first render must preserve Leptos hydration markers. The browser reads and validates the seed before `hydrate_from`, renders the same root/branch/data and performs no initial HTTP fetch. Tauri builds the same root/components in `csr` mode and mounts packaged assets, with no website SSR process. Pin modes/features separately; Leptos warns against combining SSR and hydrate in one build target. [Leptos features](https://docs.rs/leptos/0.8.22/leptos/).

## Build tooling recommendation

For M0, explicit scripts run (1) generation freshness check, (2) native SSR Cargo build, (3) client Cargo build for `wasm32-unknown-unknown` with either hydrate or csr, (4) matching `wasm-bindgen --target web`, (5) content-addressed asset/seed-contract manifest checks. These are finite build commands. Build web and installed frontend separately; never use `--all-features` to combine render modes.

`cargo-leptos` can coordinate server/client feature sets, WASM processing, styles and asset output. It is optional orchestration, not the page router or a substitute for Stackpress lifecycle. Its custom-server metadata/site package and `LeptosOptions` assumptions need a real integration proof before replacing the direct scripts. Its watch/dev listener must be managed by devmetrics. [Official configuration](https://github.com/leptos-rs/cargo-leptos).

Tauri's Leptos guide uses Trunk but explicitly says it is accurate for Leptos 0.6. It establishes the packaged-frontend approach, not verified 0.8 configuration. For Tauri build, point `frontendDist` at the CSR asset directory and use a finite `beforeBuildCommand`; no localhost plugin/sidecar/server is needed. [Guide](https://v2.tauri.app/start/frontend/leptos/).

## Exact installation request for the recommended first target

Owner approval is required before any of these commands is executed:

1. `rustup target add wasm32-unknown-unknown` for the existing 1.96.0 toolchain.
2. `cargo install wasm-bindgen-cli --version 0.2.129 --locked` (or an equivalently pinned approved binary). Re-resolve this version if dependency locking changes before M0.
3. `cargo install tauri-cli --version 2.12.4 --locked` for the `.app` build/config validation workflow. An approved project-local `@tauri-apps/cli` is an alternative, not an additional requirement.

No additional desktop Xcode/system package is currently identified as missing; actual bundle build remains the verification gate. `cargo-leptos 0.3.11` is an optional later install only if D06-B is chosen. Trunk is unnecessary for D06-A; adopting its latest release requires a separate toolchain policy decision. No wasm-opt install is required for the first debug proof.

Tauri macOS desktop uses Apple development tools and WKWebView. iOS needs full Xcode, additional Rust targets, simulator/device tooling and possibly signing; Android needs its SDK/NDK/JDK/toolchain setup. These are later target-specific requests, not implicit installs for desktop. [Official prerequisites](https://v2.tauri.app/start/prerequisites/).
