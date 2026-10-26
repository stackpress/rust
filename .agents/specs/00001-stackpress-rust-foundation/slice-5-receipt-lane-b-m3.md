# Slice 5 lane B: M3 native receipt

Task 00008; lane B; base `440b333`; branch `trial/3-s5-lane-b-ui`. This is implementation evidence for coordinator review, not task verification, owner acceptance, publication or a commit. It extends the [completed M2 receipt](slice-5-receipt-lane-b-m2.md) under the [accepted verification controls](slice-5-design-verification.md).

**M3 native, feature off: complete.** The approved feature-on customization proof is pending the M4 merge, as explicitly requested by the coordinator. No root manifest, lockfile, facade, web host, route contract, framework-view or framework-ui source was changed. No commit, install, listener, network toggle or agent message was made.

## Native boundary and G12

- Only `profile_search` and `cancel_profile_search` are registered. The existing `main-search` capability grants only those commands, only locally, only to `main`. No filesystem, shell, generic emit, HTTP client or additional test command is exposed.
- Admission checks authoritative native window/URL metadata (`main`, `tauri`, `localhost`), strict bounded request JSON, wire revision, typed prefix, an opaque bounded ASCII call ID, duplicate IDs, closed admission and capacity. Caller-supplied window/event fields cannot grant authority. Denial happens before the lifecycle.
- Search has one top-level `handle_with_options` execution path, without lifecycle re-entry. Trusted `NativeCallContext` carries window/call ID separately from request data and survives raw nested dispatch. The same generated operation and registration remain the data source.
- Native output uses private `OutputGuard` authority, honors stopped output, rejects foreign finalization and projects the final response rather than treating the original failure diagnostic as the final result. A recovered response succeeds; Abort with data retains its data; empty Abort is explicit no-content. No second native write is attempted.
- Valid host rejections remain successful IPC acquisition containing the shared five-field public error envelope. Bridge failures use fixed safe tokens. The native-tested contracts map deadline, busy, no-content/suppressed and unavailable to distinct safe UI messages; unknown/private rejection strings are never echoed. CSR remains a thin token/IPC adapter.

`proofs/native-parity.py` builds a disposable offline harness using existing path dependencies and its own lockfile; it does not modify the root dependency graph. `native-parity.rs` compares **the same ten cases** through the real NativeHost, CLI and HttpAdapter: success, Abort with data, empty Abort, request failure, operation failure, error-handler failure, response failure, error recovery, stop, foreign finalization. It asserts hook order, one request lifecycle, raw nested dispatch, public output equality, fresh error response/header replacement and terminal safe fallback without partial/private data. Compilation and execution are separate; the actual test executable is killed/reaped after a 10-second timeout, rather than merely killing Cargo.

## Cancellation, deadline and shutdown

- No detached task owns domain work. Search owns a borrowing lifecycle future; biased selection gives ready cancellation priority, then the fixed five-second deadline. Cancellation/deadline drops that future and its actual suspended stack.
- The bounded registry contains cancellation senders only, keyed by authoritative window plus call ID. RAII removes the ticket on completion, cancellation, timeout or disposal and notifies shutdown. Poison remains an operational failure; unwind cleanup removes its own metadata without clearing poison or panicking again.
- `close` seals admission before signalling all calls. `shutdown` registers its completion notification before inspecting the map, preventing a lost final-ticket wake; it awaits actual disposal with a finite deadline. It never reports success while entries remain.
- Tauri intercepts main-window close and app exit, prevents immediate exit, starts one bounded shutdown coordinator, then permits process exit after drain. Unexpected destruction seals admission as a fallback. State locks never cross await.
- Native watchdog tests prove deadline disposal, scoped cancellation, duplicate/capacity/closed admission, disposal, retained poisoning, acyclic ownership and **eight simultaneously suspended calls** drained by shutdown. Actual stack-drop counters accompany registry assertions.
- Real packaged runs exercise the production five-second deadline and clear timeout UI/retry, explicit cancellation and recovery, actual window-close and actual app-exit requests while a handler is suspended. Both shutdown modes observe one in-flight call, one entered stack, one dropped stack, zero final entries and successful shutdown.

## Packaged operation and network audit (00008-006)

CSP retains only packaged/self resources plus the Tauri IPC bridge (`ipc:` and `http://ipc.localhost`); external origins have no grant. It additionally denies objects and frames, constrains base URLs and form actions, and retains the existing required WASM/style allowances. Native navigation admits only the packaged origin. CSR's sole operation path is the fixed IPC command; no HTTP fallback exists.

Every final ordinary and instrumented launch records:

| Mode | Entered / dropped suspended stacks | Before shutdown / final entries | Network audit |
| --- | --- | --- | --- |
| navigation | 2 / 2 | 0 / 0 | zero non-IPC data fetches; one local WASM asset fetch; external image and WebSocket blocked by CSP |
| window-close | 1 / 1 | 1 / 0 | same |
| app-quit | 1 / 1 | 1 / 0 | same |

The audit wraps WebView fetch before bootstrap, separately identifies the exact packaged WASM acquisition, rejects every other non-IPC fetch, and observes actual `img-src`/`connect-src` CSP violations for deliberate external-origin attempts. The native side acknowledges the audit before the shutdown fixture starts, so shutdown cannot race its receipt. No host networking was disabled. This proves local operation and the exercised WebView data/resource boundary; it is **not** an OS packet capture, a proof of all native libraries' socket behavior, or a physical offline-device test. That residual limit remains explicit.

The normal packaged proof also retains invalid/empty/search/host-error/retry, ungranted-window transport denial, unknown-command and malformed/forged/oversized-input rejection, detail/history/return/missing navigation, both locales' singular wording, theme and root disposal. It compares exact IPC names with visible rows. The opt-in fixture adds a selected rejecting/suspended listener after application registration; ordinary startup has no proof service/listener and no extra command.

Earlier launch attempts failed in the harness. WKWebView timer polling and a retained-row completion check could miss progress; the audit could also race immediate shutdown. Final proof waits use DOM/native progress barriers, require a different newly published recovery row, report JS errors, and acknowledge the audit. All three ordinary and all three instrumented final runs pass. No product success is inferred from those earlier failures.

## Customization integration (00008-008)

The native crate declares `demo-customization = []`. Its `register_application` registers generated profiles first, then feature-gated `customization::register(builder)` from the agreed `../customization.rs` shared source, with no native copy. Both ordinary and proof startup use that helper. The absent source is not resolved in feature-off builds; the module's temporary formatting skip permits that explicitly approved pre-merge state.

The packaged script requires exactly ordinary Ada/Grace/Margaret names when off and `Demo: `-prefixed names when on, compares the actual IPC reply to visible rows, and applies the same expected names to detail navigation. Its receipt records the compiled feature; the launcher rejects a feature/expectation mismatch. Feature-off is proved in all six final runs.

**Pending on merge, not attempted:** lane A's `examples/profile-ui/customization.rs` is absent on this branch and may not be created by lane B. After M4 merges, the coordinator must verify the agreed `register` interface, compile/package with `--features app,demo-customization`, run the launcher with `PROFILE_UI_CUSTOMIZATION=1`, and compare that IPC output with lane A's direct/CLI/HTTP/SSR results. Run all three `PROFILE_UI_PROOF_MODE` values if repeating the full native matrix. Remove the temporary `#[rustfmt::skip]` once the shared source exists. 00008-008 is not claimed complete here.

## Final verification and artifacts

Log prefix is `/private/tmp/stackpress-s5-b-m3-`. Compiler: rustc 1.96.0; target: aarch64-apple-darwin; wasm-bindgen 0.2.129; Tauri CLI 2.12.4 / SDK 2.12.5; cargo-llvm-cov 0.8.6. No tools/targets were installed. Builds set `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0`; separate feature phases clean/reuse the same two target paths.

| Gate | Final result / log suffix |
| --- | --- |
| Formatting/whitespace | `cargo fmt --all -- --check`, standalone parity rustfmt, JS/Python syntax, `git diff --check`; pass |
| Workspace tests | `cargo test --workspace --all-targets --locked --features stackpress-example-profile-ui-shared/ssr`; 403 Cargo tests pass, plus the native child watchdog execution; `workspace-tests.log` |
| Workspace doctests | `cargo test --workspace --doc --locked` with the same SSR feature; 853 pass; `workspace-doctests.log` |
| Workspace Clippy | Same workspace/all-targets/SSR scope, `-D warnings`; `workspace-clippy.log` |
| Explicit MSRV | `cargo +1.96.0 check` in that workspace/all-targets/SSR scope; `workspace-msrv.log` |
| Rustdoc | Workspace/SSR/no dependencies, `RUSTDOCFLAGS='-D warnings'`; `workspace-doc.log` |
| Native app | Tests/doctests, all-targets Clippy `-D warnings`, explicit +1.96.0 check with `--features app`; `app-tests.log`, `app-clippy.log`, `app-msrv.log` |
| Real adapter parity | Ten-case table passes with the direct test watchdog; `parity-coverage.log` |
| Generated freshness | `cargo run --locked -p stackpress-example-profile-codegen -- emit generate -c`; `freshness.log`. The documented older bare `--check` spelling initially rejected its arguments; the current README's `-c` check passes without rewriting output. |
| Hydrate and CSR | Fresh `build-client.py hydrate` / `csr`, actual wasm32 Clippy `-D warnings`; `hydrate.log`, `csr.log`, `hydrate-clippy.log`, `csr-clippy.log` |
| Client graph | Actual target normal-edge trees exclude server/CLI/Idea-transformer/Tauri/Tokio/SQL/driver edges; `hydrate-tree.txt`, `csr-tree.txt` |
| Five browser runs | root and `/demo/v1` browser.cjs, browser-m2.cjs, root and `/demo/v1` browser-navigation.cjs; all pass; `browser-{root,mounted,m2,navigation-root,navigation-mounted}.log` |
| Ordinary bundle | `cargo tauri build --debug --bundles app --features app --no-sign --ci`; 25.06 MiB, 35.49 s dev build in its clean phase; `package.log` |
| Ordinary packaged launches | All three modes pass; `launch-packaged-{navigation,window-close,app-quit}.{log,json}` |
| Instrumented packaged launches | Same three modes pass after final app tests/lint/MSRV and rebuild; `package-coverage.log`, `launch-coverage-{navigation,window-close,app-quit}.{log,json}` |

Final unsigned, ordinary macOS ARM64 app: `/private/tmp/stackpress-s5-b-m3-final.app`. Its inherited product name/identifier still uses M0; it contains the current M3 code and rebuilt CSR assets. This is a debug proof bundle, not a signed release or other-platform claim. A read-only process check after all launches found **zero** remaining M3 app processes. Chromium closes in each proof's `finally`; no listener was used.

Only `/private/tmp/stackpress-s5-b-m3-native` and `/private/tmp/stackpress-s5-b-m3-client` were Cargo target directories. Both were deleted after their final results, as requested. Temporary raw profiles and archived coverage objects were also deleted after export. Logs, the final app, `coverage.json`, `coverage.profdata`, `coverage-summary.txt`, `coverage-diagnostics.log`, `coverage-dump.log` and tool-version evidence remain. Earlier completed M2 target directories were removed at M3 start; its app was preserved separately as `/private/tmp/stackpress-s5-b-m2-final.app`. No other lane's target was touched.

## Per-crate coverage

Stable LLVM instrumentation (`RUSTFLAGS='-C instrument-coverage'`, `LLVM_PROFILE_FILE=<evidence>/%m-%p.profraw`) covers the real SSR tests, watchdog/native library tests, ten-case adapter harness, finite web browser fixture and real instrumented packaged app. `coverage.py` merges profiles with the installed LLVM tools and exports the actual first-party objects, including the native executable and parity executable. Separate native/SSR/WASM/app phases avoid sharing feature-build artifacts. Dedicated tests/dependencies/tooling are outside the production line totals; native app source, including its opt-in proof module, remains counted.

| Crate / scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 703 / 709 | 99.15% |
| framework-view | 535 / 547 | 97.81% |
| contracts | 297 / 308 | 96.43% |
| shared | 429 / 466 | 92.06% |
| client, native seed/codec scope | 55 / 62 | 88.71% |
| native, library plus app | 446 / 481 | 92.72% |
| web, including finite main/route contract | 369 / 415 | 88.92% |

All exceed the 70% development floor. The shared reply codec file is 96/96; native lib is 190/206, app main 153/166, opt-in proof module 103/109. Remaining native lines include unprovoked projection/serialization/OS/build failures and timeout/error fallbacks; successful drain and the actual deadline/cancel/denial/error-replacement paths are directly proved. Browser-only client/browser.rs and the thin DOM/IPC adapters remain outside native numeric coverage under the coordinator's M0 exception and are covered by actual browser/WKWebView behavior proofs. No native coverage claim is made for WASM execution, feature-on customization, mobile or other desktop targets. Publication's per-crate 90% requirement is not claimed for the whole slice.

LLVM reported 391 mismatch records. A `--dump` audit identifies all of them as 38 distinct third-party symbols from foldhash, memchr, Tokio, tachys or brotli; none is a first-party function. The complete diagnostics are retained, and no warning suppression or production exclusion was added to hide them. Per-crate totals come from first-party production source in the raw JSON.

Reproduce with the commands above in two fresh target paths. Set the instrumentation/profile environment for native tests, the parity harness and finite web/app executables; archive their instrumented executables under `<evidence>/debug/deps` and the web/app mains under `<evidence>/debug` before cleaning a feature phase. Run all five browsers against the instrumented finite web main and all three packaged modes against the instrumented app. Export with `PROFILE_UI_COVERAGE_TARGET=<evidence> PROFILE_UI_COVERAGE_REPORT=<report> python3 examples/profile-ui/proofs/coverage.py`. The exporter includes `native_parity-` objects rather than excluding application examples.

## Stop and remaining scope

M3 feature-off native implementation is ready for coordinator re-verification. The coordinator owns task state and acceptance decisions. The feature-on customization proof is the sole explicitly approved pending integration item above. Acceptance 00008-006 uses the coordinator's revised local-operation/network-audit proof and the stated residual limit, rather than claiming the host network was disabled.

Per coordinated development §5.3, this is the milestone stop **before** the separately gated refactor/safety rounds and crate-doc closeout; no ceremonial round completion or source-of-truth promotion is claimed. Lane A's M4 gateway/web work remains independent.

KB baseline validation had 146 errors and 21 warnings, chiefly pre-existing sibling-source links and document-size notices. The post-write validation must be compared against that baseline; this receipt does not authorize repairing unrelated KB material.
