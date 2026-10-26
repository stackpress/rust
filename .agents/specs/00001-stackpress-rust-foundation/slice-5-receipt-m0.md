# Slice 5 M0 receipt

Task: [00008, shared Leptos UI and Tauri hosting](tasks/00008-slice-5-ui.md). Scope: exactly M0 steps 1–4 in [verification](slice-5-design-verification.md), under accepted [G25/G23](decisions.md). Date: 2026-10-26. Branch: `trial/3-s5-m0`, unchanged HEAD/base `c4d22ed5bd571437847fd0bf2b4f09fac59102da`.

Coordinator re-verification and Slice 5 visual acceptance remain pending. No commit, push, merge, publication, listener, or context promotion was performed.

## Implemented boundary

- `crates/framework/framework-view`: checked PageId, PageLoad, explicitly projected PageSnapshot, script-safe JSON, strict revision/build decoding, logical/encoded byte ceilings, RenderedPage and safe ViewError. It has no host or Leptos dependency.
- `crates/framework/framework-ui`: pinned Leptos/router/meta and separate SSR/hydrate/CSR modes. Synchronous component rendering uses a fresh RAII-cleaned Owner. Only owned Send HTML and leptos_meta output survive the buffered Head-assembly await. This is not general asynchronous component SSR.
- `examples/profile-ui/contracts`: imports the actual generated Profile source with `#[path]`; owns strict versioned SearchInput/SearchReply and the public search seed. No handwritten Profile fallback or native app dependency is present. Canonical query encoding preserves UTF-8 bytes.
- `examples/profile-ui/shared`: one component and controller across SSR/hydration/CSR. Hydration consumes the seed without initial search. Generation checks suppress stale/disposed completion, and previous useful rows survive loading.
- `examples/profile-ui/web`: named Send loader delegates `profile-search` within the existing lifecycle; child Abort skips rendering and loader error enters the existing error path. HTML and the fixed typed POST endpoint use existing HttpAdapter. The executable is a finite stdin/argument-to-stdout fixture, not a network gateway.
- `examples/profile-ui/client`: client-only WASM with explicit HTTP hydration or fixed IPC CSR adapters. Owner-local cancellation storage drops active transport on supersession/disposal; no custom Send wrapper, unsafe code, leaked Owner or detached native domain task is introduced.
- `examples/profile-ui/native`: the same generated registration, two fixed commands, explicit app-command ACL, local main-window capability, packaged-origin check, bounded 32-call admission, scoped IDs, five-second deadline, watch cancellation and RAII removal. Each admitted read uses `handle_with_options` once; nested dispatch stays raw.
- Root facade features are opt-in (`framework-view`, `framework-ui`, `framework-ui-ssr`, `framework-ui-hydrate`, `framework-ui-csr`); defaults remain unchanged. Generated assets/ACL schemas are ignored; command permission declarations are reproducible from build.rs.

The [finite build/proof instructions](../../../examples/profile-ui/README.md) describe the owned files and exact entrypoints. Shared Lib/server/HTTP/CLI/Idea and generated-profile sources were not edited.

## Approved dependency exception

Coordinator approval in this session permits `any_spawner = { version = "=0.3.0", features = ["tokio"] }` for buffered Head/metadata assembly. The failing predecessor is `/private/tmp/stackpress-s5-m0-head-failure.log`: `ServerMetaContextOutput::inject_meta_context` calls `leptos::task::tick`, which panicked without an initialized executor despite synchronous component rendering. S5-F6 remains deferred.

The direct edge is optional under native-only SSR dependencies. Browser trees still contain Leptos's own WASM any_spawner dependency, but contain no any_spawner Tokio feature or Tokio package. `initialize_ssr()` is documented and idempotent, treats AlreadySet as success, and requires an active Tokio runtime. Both prepare and finish check the requirement and return ViewError::ExecutorUnavailable rather than letting an uninitialized dependency panic. Separate-process tests cover uninitialized startup, repeated startup, absent runtime, cleanup and unwind.

## Actual proof outcomes

| Proof | Result / evidence under `/private/tmp/` |
| --- | --- |
| Real generated search -> Send page loader -> Head/seed/body -> HttpAdapter | PASS; `stackpress-s5-m0-workspace-tests-clean.log`; simultaneous distinct page inputs remain isolated; empty input and child Abort do not render a page shell. |
| Real hydration | PASS; `stackpress-s5-m0-browser-coverage.log`: unchanged SSR DOM and focused input, three initial rows, zero duplicate initial calls; typed second query returns Grace Hopper. Chromium uses request interception over browser pipes, not a listener. |
| Hostile public seed | PASS in real Chromium and native unit tests: `</script>`, script markup, quotes, ampersand and Unicode separators roundtrip exactly; no script executes. |
| Contract mismatch | PASS: incompatible revision is refused before hydration and visibly requests refresh; unit checks also reject an old build ID. |
| Browser disposal | PASS: barrier-held request is disposed/aborted, root is empty and late completion produces no signal write or console error. Controller tests separately complete B before A and ignore A. |
| Actual macOS ARM64 bundle | PASS; `stackpress-s5-m0-tauri.log`: unsigned debug `Stackpress Profile M0.app`, 30.12 MiB, built with Tauri CLI. |
| Packaged WKWebView | PASS; `stackpress-s5-m0-launch.log`: `main-ok`, `denied-ok`, zero in-flight entries, `tauri://localhost`. Main CSR obtains generated rows over local IPC. The ungranted window must receive Tauri ACL's `not allowed` response, not merely a command-body rejection. Wrapper waits for exit and kills/reaps after 25 seconds on timeout. |
| Native cancellation/lifecycle | PASS with parent watchdog and ready/go barrier: one request/response lifecycle, raw nested delegation, denied input before counters increment, duplicate/capacity/closed admission, cross-window cancel denial, cancellation drops suspended handler, disposed future removes metadata and weak registry reference expires. |
| Render ownership | PASS: staging drops reactive state before Head await; outside-runtime finish is typed error; child-process panic releases captured state and a subsequent independent render succeeds. No recovery is promised for panic=abort. |
| Non-Send rejection | PASS; `stackpress-s5-m0-non-send.log`: actual ServerBuilder rejects Rc<String> retained across await for its intended Send constraint. |
| Browser dependency isolation | PASS; `stackpress-s5-m0-{hydrate,csr,facade}-tree.txt`: no Tokio, Tauri, server/CLI, Idea transformer or SQL/driver edge. Real hydrate/CSR builds and facade hydration check pass. |

No preview was started. The packaged proof uses local assets/IPC with CSP blocking external resources; system networking was not toggled off. Full networking-disabled visual acceptance is still a later owner control.

## Verification commands and logs

All native work uses isolated target directories. Initial `cargo clean` and `cargo llvm-cov clean --workspace` ran before the first effort build; the final workspace suite used the fresh `/private/tmp/stackpress-s5-m0-verify` target.

- PASS: `cargo fmt --all -- --check` and `git diff --check`.
- PASS: `CARGO_TARGET_DIR=/private/tmp/stackpress-s5-m0-verify cargo test --workspace --all-targets --locked` (`stackpress-s5-m0-workspace-tests-clean.log`).
- PASS: same target, `cargo test --workspace --doc --locked` (`stackpress-s5-m0-doctests-clean.log`; 430 passed, zero failed/ignored).
- PASS: same target, `cargo clippy --workspace --all-targets --locked -- -D warnings` (`stackpress-s5-m0-clippy-clean.log`).
- PASS: native app `cargo clippy -p stackpress-example-profile-ui-native --all-targets --features app --locked -- -D warnings` (`stackpress-s5-m0-native-clippy.log`).
- PASS: `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` (`stackpress-s5-m0-doc.log`).
- PASS: `cargo +1.96.0 check --workspace --all-targets --locked` (`stackpress-s5-m0-msrv.log`).
- PASS: `python3 examples/profile-ui/build-client.py hydrate` and `csr`, each in its own target; matching `wasm-bindgen 0.2.129`; both target-specific Clippy invocations are warning-clean (`stackpress-s5-m0-{hydrate,csr}-clippy.log`).
- PASS: `cargo check -p stackpress --target wasm32-unknown-unknown --features framework-ui-hydrate --locked` (`stackpress-s5-m0-facade-wasm.log`).
- PASS: from `examples/profile-ui/native`, `cargo tauri build --debug --bundles app --features app --no-sign --ci`; final bundle at `/private/tmp/stackpress-s5-m0-native/debug/bundle/macos/Stackpress Profile M0.app`.
- PASS: generated freshness uses the existing CLI's actual long spelling `cargo run -p stackpress-example-profile-codegen --locked -- emit generate --check true` (`stackpress-s5-m0-generation.log`). Bare `--check` in the design packet is not the CLI's boolean syntax.
- Trees: normal/features for native SSR and browser hydrate/CSR/facade; normal/build/features for native app (`stackpress-s5-m0-{ssr,hydrate,csr,facade,native}-tree.txt`).

Tool inventory: stable rustc 1.96.0, macOS ARM64; wasm32-unknown-unknown installed on stable, wasm-bindgen 0.2.129, tauri-cli 2.12.4, tauri 2.12.5, tauri-build 2.7.4, cargo-llvm-cov 0.8.6. The separate named `+1.96.0` toolchain lacks the WASM target: its attempted facade WASM check reported E0463; the installed stable 1.96.0 target supplies the successful WASM builds. No tool/target was installed by this implementer.

One final workspace attempt in the reused native/Tauri target failed the existing server ownership fixture with an unrelated `service`-method lookup error. Its harness independently picks newest Lib/server rlibs by mtime, allowing incompatible feature builds to mix. No shared change was made. A fresh workspace-only target passes the fixture and complete suite; retain separate native app/workspace/browser outputs when re-verifying. The failed attempt is retained in `stackpress-s5-m0-workspace-tests.log`.

## Coverage after clean

The native coverage scope includes production Rust src files in every new non-WASM crate, including the actual Tauri executable and web fixture main. Tests, dependencies, build scripts and non-Rust tooling are not production-line denominators. Generated operation source remains separately exercised (68/74 lines, 91.89% in the raw export); its public Profile declaration has no independently reported executable lines.

| Crate | Covered / executable lines | Percent |
| --- | --- | --- |
| stackpress-framework-view | 65 / 74 | 87.84% |
| stackpress-framework-ui (native SSR) | 43 / 43 | 100.00% |
| stackpress-example-profile-ui-contracts | 63 / 63 | 100.00% |
| stackpress-example-profile-ui-shared (native SSR/controller) | 87 / 94 | 92.55% |
| stackpress-example-profile-ui-web, including fixture main | 115 / 120 | 95.83% |
| stackpress-example-profile-ui-native, including app main | 189 / 207 | 91.30% |

Evidence: `stackpress-s5-m0-complete-coverage.json`, `stackpress-s5-m0-coverage-summary.txt`, `stackpress-s5-m0-coverage-export.log`. The raw LLVM export has no warnings. cargo-llvm-cov's built-in `examples/` exclusion hides the actual example crates, so `examples/profile-ui/proofs/coverage.py` exports their real profiles without that blanket exclusion. It does not omit uncovered production functions. The root facade adds only cfg re-exports/features, with no new executable lines; actual mode compilation supplies its evidence.

Reproduce in a fresh isolated target: clean, run scoped `cargo llvm-cov` for view/UI/contracts/shared/web/native with UI SSR, then run `proofs/coverage.py build-native`. Run `proofs/browser.cjs` using the instrumented web fixture and `LLVM_PROFILE_FILE` under that coverage target. Run `proofs/launch.py` using the instrumented native executable and the same profile directory. Finally run `proofs/coverage.py`. Exact target and Playwright module settings from this run are captured in the session tool receipts.

WASM limitation: `stackpress-example-profile-ui-client/src/browser.rs` (186 source lines: ActiveCall/drop, transport, search, seed and start) is WASM-only and has no native executable coverage denominator. Its hydration/CSR behavior is exercised by real Chromium/WKWebView checks, including hostile seed, mismatch, second typed call, abort/disposal and denied IPC, but that is not a numeric WASM line-coverage result. No numerical coverage exception has been assumed; coordinator disposition remains required. No unrun target is claimed supported.

## Test-verified M0 review passes

1. Logic/complexity: checked early Abort/error, logical versus escaped snapshot bounds, UTF-8 canonical query, current-generation publication and single lifecycle; added exact boundary/security/controller cases and reran native tests plus actual browser proof.
2. Responsibility/ownership: extracted app-owned controller state from event plumbing, kept local UnsyncCallback transport separate from Send native rendering, and removed per-submit arena callback retention. Drop/barrier/weak-reference tests prove Owner, handler and registry release; reran target builds and interaction proofs.
3. Consistency/Rustdoc/comments: documented public metadata/error/startup contracts, exposed explicit facade mode features, formatted code and resolved Clippy findings; reran tests, doctests, Rustdoc, native/target Clippy and Rust 1.96 checks.
4. Safety follow-up: checked no unsafe/custom Send wrapper, no lock across await, byte limits and safe disclosure; strengthened packaged denial to require ACL rejection and reran bundle/finite launch. No direct listener, shared-crate modification or hidden domain implementation was needed.

These are bounded M0 reviews, not completion of the later Slice 5 milestone/refactor/visual gates. Full PageSpec/ServerBuilder extension, routing/detail navigation, reusable controls, notification/localization/theme surfaces, example gateway, broad lifecycle/error tables and other platform proofs remain later M1–M4 work. S5-F1–F6 and production-network work remain deferred as designed.

## Agent Workspace validation

`python3 .agents/scripts/validate-agent-workspace.py` ran after receipt creation and returned 146 errors and 20 warnings. All errors are pre-existing upstream links resolving into absent sibling checkouts relative to this Paseo worktree. Every error-source document is unchanged from HEAD; no error or warning names this receipt. The line-cap warnings and prior link-label warnings are outside this write set. Log: `/private/tmp/stackpress-s5-m0-kb-validation.log`. No unrelated KB repair was attempted.

## Handoff

Stop for coordinator independent re-verification of M0. Work is uncommitted. The coordinator owns task verification/status and integration into trial/3. Owner visual controls 00008-001–008 are not marked accepted. The remaining coverage disposition is explicitly the WASM-only client scope, not a claim that native percentages measure browser paths.
