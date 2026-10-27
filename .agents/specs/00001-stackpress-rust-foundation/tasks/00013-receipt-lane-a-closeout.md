# Task 00013 Lane A whole-crate closeout receipt

Ready for coordinator review of the released closeout only. Required `git merge trial/3` fast-forwarded `65b36fe` to `6f61f15`; all work remains uncommitted on `trial/3-s8-m0`. No next milestone or publication started. Scope: framework-ui across valid modes/features, framework-view, ui-gallery/shared/web/client, ui-format, and ui-stream/shared/web/client. Generated-admin was left alone.

## Review rounds and final behavior

| Round | Findings, behavior-preserving changes, and verification |
| --- | --- |
| Complexity | Reviewed bounded projection/admission, route construction, sort/filter/page/selection and value/draft state machines. Table filtering now clones only matching rows; membership checks borrow cells rather than cloning the whole dataset for each selected row. Route parameter uniqueness uses a borrowed iterator instead of allocating a Vec per parameter. First selected suite: 104 passed, 0 failed, 1 child-only ignored. |
| Responsibility/abstraction | One private Owner-restoration guard now serves sync, buffered async and streaming; cleanup/queue order stays renderer-owned. Shared private helpers own arrow/edge movement and local-admission-before-app-error precedence across tabs/menus/menubars, text/metadata collections and combobox. Separate render policies, locale-key grammars and syntax-only PhoneNumber versus metadata-valid E164Phone remain separate because admission/features differ. Both targeted reruns: 104 passed, 0 failed, 1 child-only ignored. |
| Consistency/Rustdoc/comments | Reviewed public names/exports and first-use examples; added executable examples for ordinary fields, choices, bounded lists, tables, popovers, all three renderer constructors and inert page snapshots. Updated stale synchronous Head docs to point to the available async renderer while preserving existing native cause text. Added missing timer destructor Rustdoc and ownership comments for overlay/stream state. Final native suite after docs: 117 passed, 0 failed, 1 child-only ignored; Rustdoc is warning-clean. An initially unresolved module-level ViewError link was corrected to its absolute crate path. |
| Safety/soundness | Reviewed cleanup/unwind/migration, cancellation/queue cycles, guarded native edits/IME, trusted HTML boundaries, replay/byte ceilings, listener/timer/observer leases and callback reentry. No unsafe or dependency/executor replacement introduced. Existing child watchdogs, cancellation, native browser undo/focus/selection, paging and nested/unmounted-overlay proofs pass after the private refactors. Example tests: 13 passed, 0 failed, 1 child-only ignored. Standalone upstream counterexamples below were rebuilt and run under independent kill/reap deadlines. |

Heuristic whole-source review signals are in `review-signals.json` (65 source files; no remaining function-without-Rustdoc signal after checking attributes). They are review aids, not numeric quality targets. Long Leptos markup lines remain where rustfmt does not reflow macros; the reviewed policy/state transitions have local purpose/ordering comments. Public Rust declaration names, feature defaults, serialized shapes, rendered DOM and existing light/dark/system behavior are unchanged. No private dead-code lint allowance or unused feature stub was added.

The stream build, browser driver, Node proof and native watchdog fixtures now honor `UI_STREAM_OUTPUT_DIR`, preserving `/private/tmp/s8-stream` as the default. A nested custom artifact path containing a space passed the actual six-case browser proof. Gallery/format/stream builders and gallery's browser host lookup now honor standard `CARGO_TARGET_DIR`, preserving their old defaults when absent; this enabled one compilation root for the whole task. Native gallery logs honor its existing output variable too. README instructions are updated.

## Findings requiring a later decision, not implemented

1. **Existing framework-view asset-JSON round-trip mismatch.** Checked module and stylesheet paths containing 4000 ampersands each pass `PageAssets::validate`; `script_json` succeeds and emits 48,063 escaped bytes, then `PageAssets::decode` rejects the same data as InvalidPolicy at its 16 KiB encoded check. Native bounded reproduction: `assets-repro.rs` / `assets-repro.log`. Recommended next fix: align encoded allowance plus logical revalidation with PageSnapshot; alternatively reject expanded output before publication. Either changes accepted behavior, so no fix was made under this brief.
2. **Potential narrower public API:** `components::place_popup` and `effective_zoom` are used only by the overlay adapter and crate tests in this workspace. Making them crate-private would reduce implementation exposure but break outside callers; listed only, unchanged.
3. **Naming proposal:** Popover's `initial_focus` versus Dialog/Drawer's `initial_focus_id` is an avoidable prop inconsistency. Harmonize only with an approved compatibility/major-version plan. Existing Input/FieldControl versus FormattedInput label composition is intentional: the formatted control owns its complete draft/value associations. Legacy NotifierContainer/Notifications names are retained rather than silently renamed.

## Verification and artifacts

Exact commands/status/durations: `/private/tmp/s8-closeout/commands.json`; finite command driver: `run.py`, full matrix: `final.py`. All Cargo invocations use `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two build jobs, one test thread, and **one** target root `target/s8-closeout`. Native SSR/stream and WASM hydrate/CSR are separate valid builds, never `--all-features`.

| Check | Result |
| --- | --- |
| fmt; native default/no-mode and async-only checks | PASS |
| UI baseline SSR regression | 71 passed, 0 failed |
| UI/view `ssr,rich,format,stream`, tests/doctests, Clippy `-D warnings`, Rustdoc `-D warnings`, `cargo +1.96.0 check` | PASS |
| Gallery/format native and real hydrate/CSR builds; stream native/hydrate build | PASS |
| Eight optimized format optional-feature variants and normal-edge graphs | PASS; exact JS/WASM bytes, deterministic gzip and SHA-256 in `format/sizes.json` |
| Hydrate/CSR Clippy and facade check; facade/stream `cargo tree --edges normal,no-proc-macro` | PASS; no Stackpress server/SQL/DB/CLI/Idea/Tauri/Tokio normal-edge browser leaks |
| `cargo test --workspace --all-targets --locked -- --test-threads=1` | 674 passed, 0 failed; 64 explicitly ignored child/platform/external cases |
| `cargo test --workspace --doc --locked` | 908 passed, 0 failed |
| Workspace Clippy and Rustdoc, warning-denied | PASS |
| Existing gallery core and breadth Chromium | 4 + 4 cases PASS |
| Existing ui-format corpus and edit Chromium | 4 + 8 cases PASS |
| Existing ui-stream Chromium at custom output path | 6 cases PASS |
| Unchanged profile check/browser compatibility | 8 browser cases PASS with unchanged client builder/browser scripts in the single root. Full checker stops at its distinct-target preflight; it was not bypassed or claimed green (`profile-check.log`). |

Browser commands use installed Playwright via environment variables, intercepted `.invalid` origins only, finite parents with process-group kill/reap and per-action deadlines. No listener, install or devmetrics instance was started. Synthetic composition evidence remains distinct from real OS IME. Browser evidence does not establish packaged WebView, manual assistive technology or real network flush/backpressure.

The existing proofs retain strict hydration DOM/node/focus/draft comparisons, escaped hostile text/replay, zero initial refetch, native dialog/overlay keyboard behavior and paging/selection. `artifact-manifest.json` records source/screenshot hashes and artifacts; 152 review screenshots are retained. Representative [gallery narrow overlay](/private/tmp/s8-closeout/gallery/screenshots/root-hydrate-m2-popup-375-zoom2.png), [French editor 375 px/200%](/private/tmp/s8-closeout/format/screenshots/edit-fr-hydrate-root-375-zoom2.png), [stream settled page](</private/tmp/s8-closeout/stream artifacts/success-root-settled.png>). These temporary files can disappear after restart; the measured conclusions and counts in this receipt are durable.

## Per-crate coverage and publication work

Fresh installed cargo-llvm-cov 0.8.6 / Rust 1.96.0, native `ssr,rich,format,stream`, after the normal/browser runs. Reproduction driver: `/private/tmp/s8-closeout/coverage.py`. It cleans the single root, instruments all owned native crates/tests plus the actual finite format executable, then merges fresh profiles with the matching sysroot LLVM tools. Actual gallery/stream CLI binaries executed by their watchdog tests are included. Dedicated test source and actual inline test-module ranges are excluded; production following external `mod tests;` declarations is retained. `coverage-summary.json`, full JSON/LCOV and objects/diagnostics retain per-file uncovered lines.

| Crate | Covered / native executable production lines | Coverage |
| --- | --- | --- |
| framework-ui | 4754 / 4950 | 96.04% |
| framework-view | 507 / 524 | 96.76% |
| gallery-shared | 481 / 554 | 86.82% |
| gallery-web | 205 / 229 | 89.52% |
| gallery-client | 0 / 0 | N/A: WASM-only |
| ui-format | 229 / 239 | 95.82% |
| stream-shared | 34 / 34 | 100.0% |
| stream-client | 5 / 5 | 100.0% |
| stream-web | 233 / 238 | 97.9% |

Every measured native crate exceeds the 70% development floor. Gallery-client is entirely WASM-only, with **no native executable denominator**; N/A is not 100% or a passed native percentage. Its startup/adoption/disposal paths run in actual hydrate/CSR Chromium. Other WASM adapters/client bootstrap are also outside native line coverage. A measured Rust/WASM coverage strategy is still needed for any numeric claim about those lines.

The native 90% gaps are gallery-shared: 18 additional real covered lines to reach 499/554, and gallery-web: two to reach 207/229. Shared gaps include the native corpus helper and real gallery reset/collection/dialog callbacks in core/breadth; web gaps include the finite corpus CLI branch, theme fallback and native failure wrapper. Add meaningful instrumented corpus/protocol cases and a reviewed Rust/WASM coverage strategy for browser-only callbacks instead of fabricating native event execution. Other native scopes already exceed 90%, but stream replay delegation/error-boundary branches and edit/error-policy paths remain visible in the report. No percentage covers the omitted WASM adoption/DOM/IME adapters; publication needs target-appropriate measurements and assessment of remaining critical paths. Real OS IME/packaged WebView remain separate unproved platform evidence. No tests were added just to chase 90%.

LLVM diagnostic export reports 277 unmatched dependency function records (20 quick_xml, 252 tachys, 3 memchr, 2 tokio), zero first-party mismatches. Warnings are retained, not suppressed. Coverage is executable line evidence, not branch, concurrency, accessibility or all-platform proof.

## Leptos upstream draft 1 — text only, not filed

**Title:** Leptos 0.8.22 SSR ordinary textarea/style/script child strings are emitted raw

**Affected version:** Leptos 0.8.22, resolved tachys 0.2.19. Native SSR; ordinary title children are escaped. Reproduction uses only the existing pinned dependency:

```toml
leptos = { version = "=0.8.22", default-features = false, features = ["ssr"] }
```

```rust
use leptos::prelude::*;
fn main() {
    let payload = "</textarea></style></script><script>globalThis.pwned=1</script>&\"'\u{2028}";
    let html = view! {
        <textarea>{payload}</textarea><style>{payload}</style>
        <script type="application/json">{payload}</script>
    }.to_html();
    assert_eq!(html.matches(payload).count(), 3);
}
```

**Actual:** all three ordinary interpolated child strings appear verbatim; terminators can escape the intended element. Application/json MIME does not change HTML-parser script termination. This is a string-output counterexample; its unsafe output was never served.

**Expected/request:** textarea text should survive document parsing as text without element breakout. Please clarify/document style/script's trusted raw-body contract and provide a context-correct safe path for data children such as JSON. Blind HTML-entity escaping for executable JS/CSS would change those languages, so this report does not propose applying ordinary HTML escaping universally.

**Source/evidence:** tachys `src/html/element/elements.rs` lines 374/390/406 set ESCAPE_CHILDREN=false; ordinary title at 416 is true. `src/html/element/mod.rs:426-450` passes that flag to children. [Accepted M1 finding](00013-receipt-lane-a-m1.md#textarea-security-fix-and-pinned-source-audit). Fresh standalone `upstream-raw.log`: RAW_CHILDREN=3. Stackpress's checked textarea/replay/title workarounds and real browser containment regressions still pass.

## Leptos upstream draft 2 — text only, not filed

**Title:** A resolved Suspense child panic in the internal effect leaves the SSR stream pending

**Affected version:** Leptos 0.8.22, tachys 0.2.19, hydration_context 0.3.2, any_spawner 0.3.0; Tokio 1.54.1 / futures 0.3.34 in this repro. Minimal standalone dependencies:

```toml
leptos = { version = "=0.8.22", default-features = false, features = ["ssr"] }
hydration_context = "=0.3.2"
any_spawner = { version = "=0.3.0", features = ["tokio"] }
futures = "=0.3.34"
tokio = { version = "=1.54.1", features = ["rt", "macros", "time"] }
```

```rust
use leptos::prelude::*;
use futures::{FutureExt, StreamExt};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = any_spawner::Executor::init_tokio();
    let owner = Owner::new_root(Some(Arc::new(hydration_context::SsrSharedContext::new())));
    let calls = Arc::new(AtomicUsize::new(0));
    let child_calls = calls.clone();
    let stream = owner.with(|| {
        let data = Resource::new(|| (), |_| async { tokio::task::yield_now().await; 1 });
        view! { <Suspense fallback=|| "loading">{move || {
            child_calls.fetch_add(1, Ordering::SeqCst);
            data.get().map(|_| -> &'static str { panic!("resolved child sentinel") })
        }}</Suspense> }.to_html_stream_in_order()
    });
    let caught = std::panic::AssertUnwindSafe(stream.collect::<String>()).catch_unwind();
    let result = tokio::time::timeout(std::time::Duration::from_millis(100), caught).await;
    println!("OUTER_TIMEOUT={} CHILD_CALLS={}", result.is_err(), calls.load(Ordering::SeqCst));
    assert!(result.is_err());
    owner.cleanup();
}
```

**Actual:** native panic diagnostic is printed, then OUTER_TIMEOUT=true CHILD_CALLS=2. The child closure runs initially pending and is reevaluated after Resource resolution by an internal isomorphic effect; the latter panic does not reach the outer catch_unwind, and the stream completion remains pending. The issue is the detached failure/unfinished completion, not a promise that child closures evaluate only once.

**Expected/request:** expose a way for the SSR consumer to observe that failure promptly (original unwind or a render-error path), rather than requiring a separate deadline to stop an indefinitely pending stream. No custom global executor/hook or unstable Tokio policy is requested.

**Source/evidence:** `leptos-0.8.22/src/suspense_component.rs:351-430`, especially the later `children.dry_resolve()` at 399-405. `any_spawner-0.3.0/src/lib.rs:178-185` discards spawn handles. [Accepted stream finding](00013-receipt-stream.md#late-panic-finding-and-bounded-resolution). Fresh `upstream-panic.log` under a 10 s parent kill/reap deadline. Stackpress's existing stream deadline/private-runtime teardown remains unchanged and passes its cancel/unwind watchdog.

## Boundaries and handoff

No new dependency/version, root manifest/lock/facade change, public Rust rename/removal/narrowing, generated-admin change, commit, push or publication. `public-inventory.json` and `scope-audit.json` retain the export/scope comparison. The single target (maximum observed about 5.8 GiB) was deleted after coverage; generated metadata/empty fixtures were removed and target/ is absent. Workspace child logs were preserved. Final process audit found zero active owned compilers/proof parents; successful browser parents awaited finally cleanup. Nothing is staged. The KB validator retains its pre-existing 146 ERROR / 22 WARN baseline with no added or removed diagnostics; kb-before.log, kb-final.log and kb-comparison.json retain that comparison. This pre-approved receipt is the only .agents write. Stop for coordinator review, including the explicitly deferred behavior/API decisions and unmeasured WASM limitation above.
