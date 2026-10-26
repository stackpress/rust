# Slice 5 lane A — M4 web/DX receipt

Date: 2026-10-26. Implementer worktree: `trial3-s5-m0`, branch `trial/3-s5-m0`, base `440b3333027b6d949ade303712229df676060651`.

## Scope and authority

This receipt covers the coordinator-released M4 work independent of native M3: finite build/check scripts, the example-only HTTP gateway, managed preview instructions, version/error recovery, and the direct/CLI/HTTP/SSR/browser plugin-reuse demonstration. It does not claim full Slice 5 acceptance or packaged native parity.

Read the merged M2 changes before implementation and retained the detail seed, return-query codec, page root, navigation, plural/language and theme behavior. No G23 crate was edited. No native, client, shared UI or contracts source was edited. Root facade defaults/exports remain unchanged. No commits, messages to other agents, listeners or GUI applications left running.

Sources: [verification and controls](slice-5-design-verification.md), [accepted decisions](slice-5-design-decisions.md), [task 00008](tasks/00008-slice-5-ui.md). Coordinator owns milestone acceptance, task/index/ledger updates and integration with lane B.

## Implemented behavior

- [One check command](../../../examples/profile-ui/check.py): `python3 examples/profile-ui/check.py`. Stops on the first failure and retains command/output logs. Each child build/check has a 600-second bound. Verifies installed/locked bindgen `0.2.129`, the installed WASM target, generated freshness (`--check true`), native SSR, separate hydrate/CSR builds and facade features, manifests, dependency graphs, tests, doctests, Clippy and a finite service probe. No implicit tool/browser/package installation and no `--all-features`.
- [Client builder](../../../examples/profile-ui/build-client.py) requires an explicit mode, rebuilds only its owned mode output, removes stale output, uses the pinned bindgen, and produces the manifest. Native and WASM builds reuse exactly two target directories.
- [Manifest codec](../../../examples/profile-ui/asset_manifest.py) records wire revision/build, a fingerprint of the seven actual contract sources, bindgen version, exact per-file MIME/size/SHA-256, and a deterministic content identity. Tests reject source/byte/version/inventory/symlink drift. No wire revision/build change was needed: the M2 public shape is unchanged.
- [Gateway](../../../examples/profile-ui/gateway/src/lib.rs) is an example workspace crate. Hyper 1 + hyper-util + http-body-util + Tokio turn finite native bytes into the existing HttpAdapter. Generated operation/page registrations and exact checked static routes share that adapter. The reusable service and injected accept runner cannot bind a socket. Only the executable supplies `TcpListener::bind`, after startup validation, from explicit `--port` or `PORT`.
- Asset startup rejects bad versions/contracts, malformed/extra fields, unsafe names, missing/oversize files, symlinks, altered bytes and aggregate mismatch. Errors name the affected path and preserve I/O causes. It retains verified bytes; requests do not reread paths. Assets use `/assets/<content-id>/...`, immutable caching and `nosniff`; HTML/API use `no-store`.
- Default limits: 64 KiB request bodies, 16 KiB semantic header bytes/raw head buffer, 64 headers, 4096-byte URI, eight admitted connections; header/body/request/connection/drain deadlines of 3/2/5/30/2 seconds. Chunked and unknown-length bodies obey the same total byte/time bounds. Trailers cannot bypass header limits. Lowered header policies retain Hyper's minimum 8192-byte parser buffer, then enforce the semantic limit before HttpAdapter.
- Admission/body failures return safe fixed JSON, close the connection and do not echo parser/body/native causes. Envelope shape is `{"code":N,"status":"<HTTP reason>","error":"<fixed message>","errors":{},"total":0}`. Statuses cover 400/408/413/431/500/504. Existing typed search success and framework error envelopes are preserved; the shared decoder retains host-error versus transport-failure behavior.
- Runner owns all admitted tasks, stops admission, signals graceful shutdown, aborts/reaps beyond its drain allowance and releases permits/counts on disconnect. Executable lifetime is 1–7200 seconds, with Ctrl-C/Unix SIGTERM shutdown. Cancellation does not roll back completed work or retry operations.
- [External bootstrap](../../../examples/profile-ui/bootstrap.js) handles old seed/build and missing JS/WASM, preserves useful SSR where possible, and shows a focused, visible **Reload page** action without raw details or an automatic reload loop. Explicit reload recovers. No normal-startup DOM is added.
- [README](../../../examples/profile-ui/README.md) documents exact prerequisites, build/check/proof commands, deployment limits and managed preview:

```sh
devmetrics start --summary "Stackpress profile UI preview" -- 'CARGO_TARGET_DIR=target/profile-ui-native cargo run --locked -p stackpress-example-profile-ui-gateway --bin stackpress-example-profile-ui-gateway -- --port {port} --base /demo'
```

The coordinator performs the authorized live preview. This implementer did not run that command or any listener executable.

## Plugin parity contract for lane B

[Stable shared native customization source](../../../examples/profile-ui/customization.rs), feature `demo-customization`, is selected as plugin `profile-ui-labels` **after** generated plugin `profiles`. Its `profile-search` listener has priority `-10`, after the generated operation at priority zero. Each public row name gains the exact prefix **`Demo: `** (including the trailing space); IDs, row order, totals, validation, original generated type and original operation remain unchanged. Empty results retain zero total.

[Reuse test](../../../examples/profile-ui/web/tests/reuse.rs) compares actual direct dispatch, CLI Terminal/CliSinks output, fixed typed HTTP rows/total, and SSR visible text/public seed. Both feature-off and feature-on runs pass. Real gateway hydration/search shows `Demo: Ada Lovelace` and `Demo: Grace Hopper` from the same operation output.

Coordinator routed the matching feature-gated registration to lane B's native M3 work. `customization.rs` was kept stable after that handoff; no native files were edited. Packaged Tauri parity with this exact behavior/order remains an integration proof after lane B merges, not a claim of this receipt.

## Dependency and ownership audit

- Root manifest adds only workspace member `examples/profile-ui/gateway`. New gateway dependencies stay in the accepted example gateway budget; web adds an explicit dev edge to the existing CLI for real adapter parity tests.
- Coordinator-approved exception: direct gateway-only `sha2 = "=0.10.9"` for fail-closed startup asset hashing. This is the already locked version. No external package/version was added or removed versus `440b333`; only the new workspace package is new.
- Lockfile edges additionally enable existing `httpdate` under Hyper's server and existing `signal-hook-registry` under Tokio signal support. The gateway package entry and web CLI dev edge are the remaining additions.
- Runtime trees use `--edges normal,no-proc-macro`; hydrate/CSR additionally select `--target wasm32-unknown-unknown`. No SHA-256/gateway/server/SQL/CLI/Idea/Tauri/Tokio browser runtime edges appear. Framework UI SSR, framework view server and default facade runtime trees contain neither the gateway nor SHA-256; the gateway normal graph contains `sha2 0.10.9`.
- Existing unchanged compile-time edge: `Leptos -> wasm_split_helpers -> wasm_split_macros (proc-macro) -> sha2 0.10.9`. The unfiltered inverse tree is retained separately; it is not browser runtime code.
- G23/shared-core sources and lane B native/shared/client/contract sources have zero diff. The only existing lane B proof touched is the owned `browser-m2.cjs` screenshot path parameter, allowing lane A evidence filenames without overwriting another lane's output.

## Test-verified refactoring rounds

1. **Logic/complexity and branch risk:** checked aggregate asset arithmetic, body frames/trailers, total deadlines, GET MIME, disconnect and already-stopped watches. Fixed overflow rejection, GET's accidental JSON MIME, and shutdown waiting for a second notification. Expanded meaningful adversarial and barrier tests. Gateway tests and subsequent full script/browser runs pass.
2. **Responsibility/ownership:** separated checked asset loading, bounded HTTP service, injected connection/task runner and executable-only bind/signal adapter. Extracted configurable web registration for hashed module URLs; plugin customization remains one plain native registration source. Reused the same service in duplex and browser probes. Gateway/reuse/workspace tests and graph checks pass.
3. **Consistency/Rustdoc/comments/DX:** documented public examples, limits, startup and error ownership; preserved I/O causes and asset paths. Visual review found recovery below the fold, so bootstrap prepends/focuses it. Script review found customization tests leave a feature-enabled web fixture, so the sequence restores the default web binary afterward. Default/custom browser proofs, final Clippy, docs/doctests and full sequence pass after these fixes. No unsafe lifetime tricks or weakened Send bounds.

## Verification evidence

All evidence paths below begin `/private/tmp/stackpress-s5-a-m4-`. Logs/evidence are retained; build output is disposable.

| Check | Evidence/result |
| --- | --- |
| Workspace all-target tests | `workspace-tests.log`: pass; includes the existing intentional ignored child-process tests whose parent tests execute them. |
| Workspace doctests and warning-denied Clippy | `doctests.log`, `clippy-final.log`: pass. |
| Final MSRV and Rustdoc | `final-msrv.log`: `cargo +1.96.0 check --workspace --all-targets --locked --offline` passes; `final-doc.log`: workspace no-deps Rustdoc with warnings denied passes. |
| Full finite script after final source changes | `final-check.log`, `final-check/summary.json`: all 21 named steps pass, `listener_started:false`; separate SSR/hydrate/CSR and generated freshness included. |
| Final gateway unit/doctests | `round3.log`: 9 tests plus the real options doctest pass. |
| Default real-browser proofs | `browser-{gateway,hydration,interaction,navigation,mounted-navigation}-final.log`: all pass. |
| Customized gateway real-browser proof | `browser-customized-final.log`: pass; content-addressed assets, zero initial calls, seed/build mismatch, missing JS/WASM, explicit recovery, typed second query, no unexpected console/hydration errors. |
| Actual HTTP connection/runner proof | Gateway tests over `tokio::io::duplex`: Hyper handshake/raw chunked input, slow/incomplete heads, acquisition errors, 408/504, EOF/error/stop/admission cap and graceful cleanup. A Notify barrier proves handler entry before disconnect; retained handler/connection counts then return to zero. |
| Asset/dependency proof | `final-check/{manifest-tests,runtime-hydrate,runtime-csr,existing-build-hash,default-facade,gateway-runtime}.log`; `tree-stackpress-framework-{ui,view}.log`: pass. |
| Formatting/patch hygiene | `cargo fmt --all -- --check`, `git diff --check`: pass. |

Browser proofs use existing Chromium via Playwright, synthetic `https://profiles.invalid`, request interception and browser pipes. A finite subprocess invokes the actual gateway service for each request, including real checked WASM/static bytes; it opens no port. The shared persistent connection/cleanup semantics are tested separately through duplex. All browser processes close in `finally`.

Tools/target: Rust stable and explicit `+1.96.0` on `aarch64-apple-darwin`, `wasm32-unknown-unknown`, wasm-bindgen CLI/locked crate `0.2.129`, cargo-llvm-cov `0.8.6`, Python 3 and existing Playwright/Chromium. No tool was installed. One approved cache write unpacked existing locked dependencies offline after a sandbox registry-cache denial; the broad offline fetch also reported uncached unrelated `etcetera`, so selected builds were used without installing it. Browser launches were approved finite GUI operations.

## Coverage after clean

Measured executable production Rust lines with cargo-llvm-cov `0.8.6` after `cargo clean`, using its `show-env` instrumentation in the same native target. Selected view/server, UI/SSR, contracts, shared/SSR, web and gateway tests; gateway `demo-customization` also enables web customization. Built real web/gateway/probe binaries with the same instrumentation, exercised finite web default/hostile/search/detail/French-dark/invalid requests, and reran the customized real browser gateway proof. Doctests passed separately; stable doctest coverage is not claimed.

| Crate scope | Covered / executable lines | Percent |
| --- | --- | --- |
| framework-view | 530 / 547 | 96.89% |
| framework-ui | 700 / 709 | 98.73% |
| contracts | 286 / 297 | 96.30% |
| shared | 429 / 466 | 92.06% |
| web (changed) | 405 / 453 | 89.40% |
| gateway (new) | 515 / 569 | 90.51% |

`coverage.json` is the full LLVM export; `coverage.profdata` preserves the merged profile outside disposable build output. `coverage-run.log`, `coverage-{tests,bins,browser,export}.log` and `/private/tmp/stackpress-s5-a-m4-run-coverage.py` retain the scoped procedure. The checked-in coverage exporter includes real example sources, shared route/customization sources and both executable entry points. Exclusions are dependencies, non-Rust tooling, dedicated test files and out-of-scope workspace crates; thin WASM-only browser adapters retain the coordinator-approved real-browser/WKWebView numeric exception.

No listener code is excluded: gateway main contributes 0/31 lines because launching it is prohibited; the reusable runner, service and probe are executed. Other gateway gaps are a few OS I/O/error/fallback branches. Web gaps include defensive route/detail/loader/error conversion and invalid finite fixture paths; route source is counted in each crate that compiles it. Both changed crates exceed the 70% development floor; web is below the 90% publication threshold. Publication is not requested.

## Observed build/asset sizes and limits

The final full check reused the two targets (warm dev builds, not benchmarks): native SSR 1.56 s, hydrate Cargo build 2.32 s, CSR Cargo build 1.31 s, gateway 2.53 s, selected-plugin reuse compilation 0.23 s, restored default SSR 0.16 s. Bindgen/whole-script duration is not represented by these Cargo timings; no performance target is claimed.

Uncompressed debug outputs: hydrate inventory 5,621,204 bytes across four files (WASM 5,579,067, JS 39,034, bootstrap 1,432); CSR inventory 4,413,756 bytes across six files (WASM 4,374,502, JS 35,581, bootstrap 1,432). Remaining inventory bytes are emitted chunks/index. These are measured proof artifacts, not release bundle claims.

This host is buffered and example-only: 64 MiB/file and 128 MiB total retained assets, potentially one response copy per admitted connection; separate existing snapshot/HTML ceilings remain. It provides no production streaming, TLS, authentication, arbitrary file serving, CPU preemption, persistent offline storage or acceptance evidence for other operating systems. Startup/signal binding is built, not live-run; coordinator owns that run. Native M3 and packaged plugin parity remain separate.

## Closeout

Final formatting and patch-whitespace checks pass. Owned native/WASM target directories were removed after copying merged coverage evidence; no other lane's output was deleted. Logs, screenshots, coverage JSON/profdata and ignored checked client artifacts remain for review.

The KB validator baseline has 146 errors and 21 warnings, predominantly historical upstream/sibling links unavailable in this worktree and existing line-length/link-label warnings. The after-write run is compared against that exact baseline; no unrelated KB repair is in scope. Coordinator review/merge and live devmetrics/native parity runs are next; no commit was made.
