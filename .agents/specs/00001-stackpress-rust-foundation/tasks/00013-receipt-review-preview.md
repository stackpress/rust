# Task 00013 — example review preview

Status: **Part A accepted/merged; Part B implemented and verified without mobile launch, ready for coordinator review/build/launch.** Owner visual acceptance is separate. No listener, installation, new commit, push or publication was performed by the implementer.

## Part A — scope and artifacts

Implemented the [approved design](00013-review-preview-design.md) in [examples/review-preview](../../../../examples/review-preview/README.md), plus additive registration/origin seams in gallery, preferences, stream and generated-admin. Root workspace membership and lock entries are additive; no existing dependency version changed. No framework, generator or tracked generated output has an uncommitted change.

Before the final unresumed regressions, release isolation and coverage, merged `trial/3`: clean fast-forward `8b29f84..4903165`. Full head is `4903165e6c3e36c3b670e48fe5f58be96983f850`. Rebuilt all five examples and both gateway variants against schema/UI contract revision 3. No conflicts or hand-merged generated files.

The single debug/feature-selected HTTP gateway prepares one existing `stackpress-server-http` host. Only its explicit `serve` action binds `127.0.0.1`; preparation/tests/`pipe` bind nothing. Port and directory are required; base/render/compiled variant are checked. Host and devmetrics independently bound each session to two hours; existing native admission, body, asset, lifecycle, response, stream and drain limits apply.

Retained output root: `/private/tmp/s8-review`.

- `bundle/disabled/slice8-review` and `bundle/plugin/slice8-review`, with matching hashed browser assets for every mount; hydrate and CSR variants are retained. Streaming always uses streamed SSR and completion-gated hydration.
- Fresh explicit setup: `state.json` selects `review-28117-1793126285852886000.sqlite`, containing 25 profiles, one related project and 25 tasks. Profile `p0` contains `=1+1` for export guarding. Setup uses actual generated schema/operations, create-new files and path checks; serving attaches existing files and never seeds/resets them. The earlier implementation fixture was archived under `review/pre-G38-owner-fixture-*` before this separate setup step.
- `review/` holds command/result logs, browser receipts/screenshots, isolation artifacts/hashes, LCOV/JSON coverage and merged profiles. Other example proof evidence lives in the root's `gallery`, `preferences`, `stream`, `format`, `roles`, `crud`, `transfer` and `races` folders.

The review panel labels caller switching as **Review fixtures — not authentication**. Editor/Reader/Denied selection uses a scoped HttpOnly cookie; supplied actor headers never override it. All one-shot faults and hydration pause state are memory-only and reset on restart. Lost acknowledgement is injected after a real SQLite commit and produces UnknownOutcome, never Saved. Slow/failed reads, preference failure, public flash, hydration controls and CSV samples each name their owner control.

Gallery/preferences keep their legacy HTTPS fixture defaults. The review host supplies the actual loopback origin through existing cookie/action APIs, preserving plain-HTTP persistence without Secure. Public role/format pages reuse the same component sources. Ordinary streaming delays details 800 ms; Pending supports immediate Back/disconnect, and Late repeats the existing bounded failure/recovery case.

## Owner walkthrough and exact launch commands

Use [the owner checklist](00013-review-preview-owner-checklist.md): **main root/hydrate/disabled**, then **plugin root/hydrate**. It gives paths and 3–6 concrete steps for every control. CSR and `/demo` are optional, with a fresh browser profile for the mounted session. Controls 011 and the installed half of 013 wait for Part B.

The prepared fixture is already fresh. The commands below show the task directory; replace it consistently with a new caller-owned directory when reproducing. Build and then explicitly set up that directory; setup refuses an existing manifest:

```sh
python3 examples/review-preview/review.py build --directory /private/tmp/s8-review
python3 examples/review-preview/review.py setup --directory /private/tmp/s8-review
```

Coordinator launches the main session:

```sh
devmetrics list
devmetrics start --summary "Slice 8 main owner review" -- '/private/tmp/s8-review/bundle/disabled/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant disabled --duration-seconds 7200'
```

Open `http://127.0.0.1:<assigned-port>/review`; select Editor and keep the panel in a second tab. After recording the disabled label for 013, stop the assigned port before rebuilding or replacing binaries. The bundle driver uses generated-admin's existing disposable generation helpers: plugin enabled/build, then disabled/regenerate/build. Tracked generated outputs stay unchanged.

```sh
devmetrics stop <assigned-port>
python3 examples/review-preview/review.py bundles --directory /private/tmp/s8-review
devmetrics start --summary "Slice 8 plugin owner review" -- '/private/tmp/s8-review/bundle/plugin/slice8-review serve --port {port} --base "" --render hydrate --directory /private/tmp/s8-review --variant plugin --duration-seconds 7200'
```

Stop that assigned devmetrics port when finished. No third web session is required. The restored Idea label is `Name </script> 🦀`; the replacement is `Plugin field label`. `--variant` checks compiled artifact selection; it cannot replace a label dynamically.

## Verification on the merged tree

Installed tools only: Rust 1.96.0 / native Apple ARM64, cargo-llvm-cov 0.8.6, wasm-bindgen 0.2.129, Node 26.2.0 and Playwright 1.64.0 at the coordinator-specified module path.

```sh
python3 examples/review-preview/review.py build --directory /private/tmp/s8-review
python3 examples/review-preview/verify.py regressions --directory /private/tmp/s8-review
python3 examples/review-preview/checks.py --directory /private/tmp/s8-review
python3 examples/review-preview/review.py run --directory /private/tmp/s8-review --name pre-isolation-clean -- cargo clean --offline
python3 examples/review-preview/isolation.py --directory /private/tmp/s8-review
python3 examples/review-preview/review.py run --directory /private/tmp/s8-review --name pre-coverage-clean -- cargo clean --offline
python3 examples/review-preview/coverage.py --directory /private/tmp/s8-review
python3 examples/review-preview/verify.py smoke --directory /private/tmp/s8-review
```

- Full example builds pass: gallery/preferences hydrate+CSR, stream hydrate, formatting's eight optional browser combinations, and generated-admin plugin/disabled hydrate+CSR plus both native review gateways.
- The final `regressions` command ran from the beginning with **no resume flag**. Existing gallery browser/breadth, preferences matrix, streaming, formatting and formatting-editor proofs pass.
- Existing `generated-admin/proof.py` builds for roles/CRUD/plugin/transfer and browser suites for roles (24), CRUD (8), formatting (16), transfer (8) and delayed writes (32) pass: **88 combinations**. The plugin proof verifies removal restores generated output. Compile proofs pass, including the expected stale-consumer rejection and renamed/hostile/empty/formatted consumers.
- Existing admin `checks`/`graphs` pass: workspace Clippy/tests/doctests, formatting Clippy/tests, Rust 1.96 check, warning-clean Rustdoc, all four browser lint/graph combinations and facade/tool graphs. The existing driver skips `tcp_watchdog`; no TCP test/listener was started.
- New package: **seven host tests + one finite CLI test pass**, with Clippy `--all-targets -D warnings`, warning-clean Rustdoc and workspace fmt. Tests cover actual SQLite writes, durable unknown outcome, denied/forged callers, exact HTTP cookie persistence/confirm, CSV preview/explicit confirm/export, origin/mount rejection, asset tampering/limits, symlink refusal and pre-bind CLI failures.
- Native duplex proof observes the useful shell before 700 ms, ahead of the 800-ms Resource; disconnect releases captures. Ordinary completion is distinct from interrupted late output, recovery and absence of a completion marker.
- Final composed Chromium smoke passes **disabled/root/hydrate**, **plugin/root/hydrate**, and **disabled/mounted/CSR**, with no browser errors or unexpected requests. It ran again **after target deletion**, proving retained bundles work independently. Panel and late-recovery screenshots were inspected.

Browser interception relays real request headers, cookies and bytes through the same native HTTP host over duplex. It buffers response frames for Playwright fulfillment; progressive timing/drop are proven separately by native duplex tests. Fixture POSTs use manual redirect handling in this interception harness; native tests prove the real redirect response. The owner still performs the real managed-listener visual review.

## Test-verified review rounds

| Round | Findings and resulting change | Evidence |
| --- | --- | --- |
| Logic/complexity | Added finite manifest/input bounds, symlink/traversal refusal, exact one-shot state and committed-unknown assertions. | `review/round-1.log`: six host tests pass. |
| Responsibility/abstraction | Separated panel admission/chrome hooks from fixtures, kept transport/setup/configuration separate, and froze compiled variant selection. | `review/round-2.log`: six host tests pass. |
| Consistency/Rustdoc/comments | Documented public/example boundaries and errors, added real CSV/native and finite CLI coverage, applied warning-clean lint/docs/format. | Final `review/round-3-*`: seven host + one CLI tests, lint/docs/fmt pass. |

Follow-up contract review found the first extraction had scoped a legacy preferences lifecycle header outside its mount. Private registration now preserves the original finite behavior, while public composition scopes its response chrome. The five existing preferences tests and the final full regressions pass. Safety review also ensured browser-close errors cannot skip native child cleanup; the final smoke verifies normal closure/reaping. Earlier failed-scope and pre-lint logs remain retained. No framework fix was needed.

## Coordinator correction — unchanged admin default graph

Removed the unnecessary preferences/view dependencies and their generated-admin-web lock edges. Dependency declarations, default features and lock edges now match `HEAD` exactly (`review/fix-dependency-audit.json`). [The local origin check](../../../../examples/generated-admin/web/src/host/origin.rs) uses the existing `http` dependency: at most 512 bytes, lowercase HTTP(S), a nonempty URI host and optional decimal u16 port, with no userinfo/path/query/fragment. Bracketed hosts require IPv6. Validation precedes startup registration; `build()` retains the exact `https://admin.invalid` default. The two shape tests cover accepted and rejected inputs.

After this correction, reran generated-admin-web tests/Clippy, admin `proof.py checks`/`graphs`, fresh CRUD/transfer builds and all 16 corresponding browser combinations, and the review package tests. Rebuilt both review bundles and passed all three composed smoke cases. Refreshed native coverage below; the new validator itself measures 38/38 executable lines. Logs/results use `review/fix-*`; the earlier 88-case unresumed run and release-isolation proof remain the initial Part A baseline. Logic, responsibility/dependency and Rustdoc reviews found no further required changes. Part B remains unreleased until the coordinator confirms the owner has begun web review.

## Coverage

Fresh native `review-preview` scope includes all six changed crates, their existing tests/finite subprocesses and instrumented composed-browser requests. **LLVM LCOV DA records** count physical executable source lines once, using the existing `ui-format/coverage.py` convention. Exact test files/inline-test ranges are excluded; implementation and compiled generated-store paths remain included. Build scripts and uncompiled WASM adapters are outside this native numerical scope; matching WASM builds and real Chromium proofs cover the browser boundary.

| Changed crate | Covered / executable | Coverage |
| --- | ---: | ---: |
| review-preview | 739 / 795 | 92.96% |
| ui-gallery-web | 234 / 259 | 90.35% |
| ui-preferences-web | 273 / 291 | 93.81% |
| ui-preferences-shared | 91 / 106 | 85.85% |
| ui-stream-web | 264 / 270 | 97.78% |
| generated-admin-web | 5313 / 6068 | 87.56% |

All meet the 70% development floor. `review/coverage-production.json` retains per-file counts, uncovered lines and exact test exclusions; `coverage-lcov.log`, `coverage-raw.log` and merged `.profdata` retain underlying evidence. The initial conservative calculation subtracted all physical inline-test lines only from the numerator, producing a false floor failure; `coverage-production-conservative.json` preserves it. The corrected full driver then passed end to end. No runtime function was excluded to raise coverage. TCP accept/serve is deliberately unexecuted and remains in the measured implementation; ordinary loopback serving awaits the coordinator. No publication-coverage or whole-WASM/iOS percentage claim.

## Release isolation, cleanup and remaining gates

`isolation.py` passes both required proofs after the merge:

1. `cargo build --release -p stackpress-example-review-preview --features review-preview --offline --locked` fails with the exact explicit debug-only guard, exit 101. Release opt-level/debug were set to 0 to bound build cost; it remains a release-profile refusal. The build-script guard also refuses release profiles with debug assertions enabled.
2. Nine normal facade/example dependency graphs exclude the review package; the package's default graph is empty. **61** ordinary native/browser artifacts, including a native release gallery build and optimized formatting browser variants, contain none of seven exact review markers/package names. `review/isolation.json` retains artifact hashes; native evidence binaries are copied outside target.

One target `/private/tmp/s8-review/target` was monitored under an eight-GiB cap throughout sequential compilation phases. Reused that same target for the coordinator correction, then deleted its final **2,158,896 KiB** after retaining artifacts/coverage (`review/fix-cleanup.json`). The final browser proof uses only copied binaries/assets; target remains absent. Owned subprocesses use bounded watchdogs and kill/reap cleanup. Final audit/provenance are retained with the evidence. Setup/startup assumes the operator exclusively owns the directory during filesystem checks/open; no hostile concurrent filesystem guarantee is claimed.

KB validation retains the known **146 errors / 25 warnings**, with no new preview-document diagnostic. Scope/lock/diff checks pass; no existing dependency version changed. The coordinator launches actual devmetrics sessions and records the owner's web acceptance separately.

## Part B — installed iOS review handoff

Started with the requested clean fast-forward `cfd5257..1c67552` from `trial/3`; no conflict, stash, reset or hand merge. Read the owner findings and M4 receipt. All new source changes are in profile-ui's native example, review-preview tooling, this receipt and the consolidated checklist. Framework crates, generators, tracked generated roots, normal Tauri config/ACL, dependency versions and Cargo.lock are unchanged.

**Manual launcher:** existing `build-ios.py` accepts `--review-variant plugin|disabled --directory DIR`. It uses one eight-GiB-capped `DIR/target` for Cargo/WASM/Xcode caches, retains the actual built app separately per variant, selects checked bundled assets and enables `app,review-preview` in a debug unsigned simulator build. The normal build path keeps its ordinary frontend selection. `launch-ios.py --manual-review` uses that variant's successful build receipt, refuses a live options service or existing owner device/display, and creates one disposable simulator. Lifetime is 30–7200 seconds, default 1800, including setup. Restart retains the same app container; stop/deadline/Ctrl-C/SIGTERM/failure enter cleanup. The parent reaps its worker and retries recorded cleanup; a unique device name is recorded before creation so an interrupted create can be recovered. Display tracking claims only the launched process and observed descendants. Failed cleanup preserves its ownership record for inspection.

The two labelled review buttons reuse fixed existing Profile commands. Denial requires the exact native `denied` reply to a fixed extra-field DTO. Cancellation uses a four-second read fixture through the real ticket/deadline registry and reports success only on native `cancelled`. Lost acknowledgements remain uncertain. The fixture requires the debug-only `review-preview` feature **and** explicit manual-launch environment selection; no new command, permission or mutation is added. Old automatic proof modes remain separate.

**Packaged catalog:** `installed.py assets` builds the matching generated-admin CSR component from M4's isolated review-owned **Full name** schema, plugin first then disabled, using the accepted generator and shared/client WASM cache isolation. `ios-assets/{plugin,disabled}` contain `admin-review.html`, exact CSR bytes, styles, schema provenance and checked SHA-256 inventories. The local Generated admin review link returns to Profile UI. Only the explicit review build selects these folders; ordinary frontend paths exclude them. There is **no admin mutation or IPC service; this is not installed admin CRUD**. Local fixture drafts use the shared generated role wrappers. The web fetch/CSV bootstrap and admin seed are absent.

Coordinator finding: raw fixture dumps are hidden in the packaged owner view and mirrored with `textContent` into collapsed **Debug state (review only)**. The original reactive output nodes and exact-state proof assertions remain intact. A root-scoped observer updates the mirror and disconnects on disposal. Both variants prove that every rendered hostile `</script>` text, including Rows-style labels, belongs to the labelled **Safety samples** section. Ordinary sections use Full name / Plugin field label.

### Coordinator commands and owner walkthrough

Run from this worktree root. The coordinator executes mobile builds and launches: the existing mobile build starts a temporary Tauri options service, which the implementer is forbidden to start. No mobile build, simulator or installed app was launched here. The [README](../../../../examples/review-preview/README.md#installed-ios-review) explains displays, restarts, cleanup and both sessions; [checklist 011 and 013](00013-review-preview-owner-checklist.md#00013-011--installed-ios-app) supplies exact actions and expected results.

```sh
python3 .agents/scripts/build-benchmark.py run --label "00013 installed review assets" --kind build --scope selected --cache unknown -- python3 examples/review-preview/installed.py assets --directory /private/tmp/s8-review
IOS_REVIEW_VARIANT=plugin
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS $IOS_REVIEW_VARIANT build" --kind build --scope selected --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=4 python3 examples/profile-ui/native/proofs/build-ios.py --directory /private/tmp/s8-review --review-variant "$IOS_REVIEW_VARIANT"
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS $IOS_REVIEW_VARIANT manual review" --kind test --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --duration-seconds 1800
# In a second terminal with the same IOS_REVIEW_VARIANT:
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS restart" --kind other --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --control restart
python3 .agents/scripts/build-benchmark.py run --label "00013 iOS finish" --kind other --scope other --cache unknown -- env PROFILE_UI_IOS_LOGS="/private/tmp/s8-review/ios-logs/$IOS_REVIEW_VARIANT" python3 examples/profile-ui/native/proofs/launch-ios.py --manual-review --variant "$IOS_REVIEW_VARIANT" --control stop
```

Use the plugin installed session for 011 and Plugin field label in 013. After cleanup, set `IOS_REVIEW_VARIANT=disabled` in both terminals and repeat build/launch/finish; Full name must return. No third web session is required. On the existing Duo device use **primary / LCD 1**. Stop deletes the disposable device; use restart within a session to check preference persistence. Real SDK/build compatibility, WebKit/IPC, settings restart/background behavior and actual simulator/display cleanup remain coordinator-run gates. Android stays unverified by owner decision.

### Verification and three review rounds

All Cargo/proof/clean invocations were benchmark-wrapped once at the outer command; nested Cargo commands were not double-counted. Initial native/default cancellation and packaging policy tests passed before the rounds. Final evidence is retained under `/private/tmp/s8-review/review` and `installed-coverage`; matching frontend assets remain outside target.

| Round | Finding, change and test-verified result |
| --- | --- |
| 1 — complexity/branch risk | Made control publication atomic; restricted display ownership to observed descendants; preserved uncertain outcomes. Native cancellation, denied identity, normal reads, temporary-tree packaging and timeout/cleanup tests pass. |
| 2 — responsibility/ownership | Extracted build-command selection so feature and frontend move together; scoped restart/stop to the matching owned variant. Checked normal/native/browser graphs, stale asset rejection and child reaping. Both actual CSR variants pass Chromium with no catalog IPC or unexpected requests; app-feature Clippy passes without launch. |
| 3 — consistency/docs/safety | Completed contracts/comments and exact owner steps. Added the requested collapsed debug state, preserved exact snapshots, checked all hostile text stays in Safety samples, tightened denial acknowledgements and recovered interrupted device creation. Final **13** Python policy/file tests, real native tests, default native regressions, formatting, warning-denying app Clippy/Rustdoc and both packaged browser cases pass. |

The unchanged composed review Rust package also passes **7** host tests plus **1** CLI integration test and doctests. Chromium checks plugin/disabled at 375/1280 px, actual Full-name/plugin labels, editing, exact hidden snapshots, live collapsed debug disclosure, root disposal, no catalog IPC and zero unexpected requests/page errors. Panel IPC is a closed browser adapter; the separate Rust test proves real denied-DTO admission and cancellation. These are port-free proofs, **not installed acceptance**.

Fresh `cargo-llvm-cov 0.8.6` / `rustc 1.96.0`, native `aarch64-apple-darwin`, `review-preview` without `app`: **239 / 272 executable physical lines = 87.87%**, including the new fixture **10 / 13 = 76.92%**. Exact test files/inline ranges are excluded from both quantities; LCOV/JSON, uncovered lines and merged profdata remain in `installed-coverage`. App event-loop/platform code and build tooling are not claimed as native runtime line coverage; the app feature is warning-clean compiled, and real mobile execution remains pending. The review-preview Rust implementation is unchanged by Part B. Additional stdlib trace evidence on the new Python policies is **170 / 179 = 94.97%** for the manual launcher and **54 / 57 = 94.74%** for packaging; OS effects are mocked, with uncovered lines retained in `review/installed-python-coverage.json`.

**Isolation:** a real release build with native `review-preview` fails with the explicit profile guard, exit 101 (release opt-level/debug 0). Normal debug/release native RLIBs contain no installed-review marker, `review-wait` or catalog filename. Normal native and CSR graphs contain neither the review package nor generated-admin. The actual normal CSR source, normal debug iOS staging and freshly compiled release CSR staging omit all review files/markers, including nested/binary bytes; ordinary Tauri frontend paths and ACL remain unchanged. These are frontend/native-library proofs, not a release mobile archive build. Retained artifacts and `review/installed-isolation.json` expose the scope.

### Benchmarks, retained artifacts and cleanup

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. At receipt write, global attempts **64 → 90 (+26)**; Part B-owned **26** (25 passed, final audit writer in progress); other new attempts **0**. Observed-compilation attempts **8 → 8**; successful declared cold workspace rebuilds **0 → 0**. Completed owned outer time **445.929923s**; global completed outer time **8146.053313 → 8591.983236s**. Recorder Cargo stages **8 → 8**, stage time **303.05 → 303.05s**. Context captures inner Cargo output, so unavailable outer Cargo-stage timing does not mean no compilation occurred. Exact inner durations/logs, kind/scope/cache counters and all owned records are in `review/part-b-benchmarks.json`; `benchmark-part-b-after.json` retains finalized counters after the writer exits. The initial coverage-clean scope label was corrected to workspace with its original value/reason retained in `metadata_corrections`; no command, timing, outcome or log was changed. Final writer record: `20261028T084953.570548Z-76230cdf8f2a`.

Relevant terminal records: round 1 `20261028T081449.457441Z-0bfc895ed8f6`; round 2 `20261028T082653.987993Z-f79c9f577d30`; final coverage `20261028T084006.266207Z-17cf80740574`; final packaged smoke `20261028T084020.423480Z-42f69d68b85b`; final app/default checks `20261028T084211.022959Z-255ce5f1d9af`; final isolation `20261028T083332.283412Z-a7e83f2c6ce5`. Full attempts, exact argv, logs and outcomes are retained in the shared store and `review/part-b-benchmarks.json`.

Deleted the sole owned `/private/tmp/s8-review/target`, **3,786,240 KiB**, after all compilation/coverage. The existing Context watchdog measured it against **8,388,608 KiB** during long children. `review/part-b-cleanup.json` records target absence, no matching fixture processes and no remaining disposable workspaces. Cleanup record: `20261028T084508.882097Z-d5a332ec35ea`. Every long child has a deadline and kill/reap path; new simulator policy tests substitute platform effects, so no real simulator was created or cleaned here.

Inspected `review/installed-{disabled,plugin}-{375,1280}.png`; final `installed-browser.json`, asset inventories/schema provenance, `part-b-provenance.json`, native/Python coverage and source/cleanup audits remain outside target. Final KB validation retains the known **146 errors / 25 warnings**; no new task diagnostic. No owner data or managed preview was reset, no install/listener/commit was performed, and no other target was used. The coordinator can now build and launch the two installed sessions and record owner findings separately.
