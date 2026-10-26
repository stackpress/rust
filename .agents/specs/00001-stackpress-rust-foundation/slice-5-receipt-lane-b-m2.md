# Slice 5 lane B M2 progress receipt

Task: [00008](tasks/00008-slice-5-ui.md), under [G25/G23](decisions.md) and the accepted [M2 plan](slice-5-design-verification.md). Date: 2026-10-26. Worktree: `trial3-s5-lane-b-ui`; branch `trial/3-s5-lane-b-ui`; unchanged base/HEAD `2c3189fdde48caba37c75d1590462977c510cce0`.

**M2 remains in progress.** Independent component/state/language, search/controller and adapter work is implemented and verified below. Final route-catalog/detail-seed integration is pending lane A. No acceptance control is marked accepted, and no refactoring round or later milestone is claimed complete.

## Implemented scope

| Accepted Now surface | Implementation and evidence |
| --- | --- |
| Button, Input, FieldControl | Typed native button/link/disabled/type behavior; controlled text; label/help/error associations; safe href checks; SSR semantics tests and real keyboard/Enter proof. |
| Alert, Loader, LoaderContainer | Escaped safe status/error text, announced loading and aria-busy; previous useful rows remain visible. Native markup and actual browser state transitions pass. |
| Link, List, explicit scoped conditions/iteration | Safe native anchors, keyed For rows, escaped labels, visible empty state and explicit Show branches. Exact detail presentation has found/not-found/return states and hostile-text tests. Routing is not yet connected to that presentation. |
| Basic layout and scoped theme | LayoutProvider/Blank/Panel plus Head/Left/Main/Menu/Right semantic slots; responsive document order, neutral CSS, visible focus, reduced-motion rule; explicit light/dark root state without browser inference or persistence. |
| Immediate scoped notifications | Checked public NotificationOptions, bounded store, duplicate coalescing, stable keys, dismiss/clear and root cleanup. No global singleton or timer/cookie behavior. Real browser error/dismiss/retry proof passes. |
| Language/context/provider | Checked serializable/deserializable locale/map/placeholder, scoped replacement, missing-provider diagnostic and dictionary clearing. Native tests cover byte/map limits, concurrent roots and disposal. |
| Phrase, tagged-call equivalent, string and Translate behavior | Named phrase/template/text APIs and explicit ViewFn slots; source fallback, exact Unicode/whitespace, custom/repeated placeholders, missing-empty/excess-ignored policy, zero/false/empty preservation, escaped strings and real nodes. No raw HTML/formula execution. |
| Search state | Valid/invalid/empty/error/retry/loading/cancel states; invalid input cancels/revokes old publication rights; stale, duplicate, disposed and exhausted-generation completions cannot publish. No initial hydration fetch; explicit packaged bootstrap performs one local read. |
| Client boundary | Native-tested seed/reply/settings/asset admission is separate from browser DOM/fetch/IPC acquisition. Form and fixed HTTP operation paths use the decoded mount through the coordinator-authorized temporary M1 shim. |

The shared UI uses the actual generated Profile DTO and existing operation. No domain search implementation, lib/server/HTTP/CLI/Idea source, manifest, feature, facade, route-contract or generated-profile source was changed. The only framework-ui lib.rs edit is the coordinator-approved three documented module declarations. Shared/example module declarations are lane-B-owned.

## Compatibility changes and integration requests

The coordinator authorized small local shims until lane A merges. They are deliberately named and documented:

- `examples/profile-ui/shared/src/m1_compat.rs`: BasePath/location/context subset. Replace uses with `framework_ui::routing::{BasePath, provide_route_context, use_route_context}` and remove the shim/tests at integration. Call sites use joined locations through `as_str()`.
- `examples/profile-ui/client/src/assets_compat.rs`: PageAssets::decode subset. Replace its import with `framework_view::PageAssets` and remove the shim at integration. Bootstrap requires `#stackpress-assets`, checks BUILD and uses the checked mount for routing and the fixed HTTP search endpoint; there is no missing-element fallback.
- The original M0 web host does not emit M1 asset metadata. The finite browser harnesses explicitly inject the trusted root metadata fixture **only when the host lacks that element**. This proves client behavior against the planned shape; it is not evidence that the unmerged host emits it. Remove that compatibility helper and rerun against the integrated host.
- The snapshot constructor still targets today's PageSnapshot. The coordinator-owned merge must add `status: PublicStatus::ok()` once that field exists and retain lane A's build-ID update. Lane B did not add a speculative field or edit that contract.

**Concrete route-contract request to lane A/coordinator:** provide the compiled search/detail PageRoute definitions and canonical return-query codec before lane B wires the Router and detail navigation. Suggested callable contract, using lane A's real types:

```rust
pub fn profile_routes(base: BasePath) -> Result<RouteCatalog, ViewError>;
pub fn search_location(base: &BasePath, prefix: &str) -> Result<PublicLocation, ViewError>;
pub fn detail_location(base: &BasePath, profile_id: &str, search_prefix: &str)
    -> Result<PublicLocation, ViewError>;
```

These are requested signatures, not implemented APIs. The agreed detail seed must retain both the exact detail ID and originating search prefix so links, back/forward, direct refresh and return-to-search agree without duplicated path/query rules. Current SearchPage contains input/reply only; lane B has not invented a new seed or route query convention.

Also ensure the native SSR caller passes snapshot locale/theme into the new optional ProfileSearch `initial_language` / `initial_theme` props, and that the real snapshot policy accepts the approved locale/theme fixture. The M0 policy permits only en/light. A real French/dark initial SSR-to-hydration proof remains pending that integration; client locale/theme changes and concurrent synchronous component renders already pass.

## Behavioral evidence

- Native scope: **26 tests and 2 doctests pass**, no failures/ignored tests. This includes 4 native client decoder tests, 8 shared/controller/compatibility tests, 10 component/language/state tests and 4 existing render/executor cleanup tests.
- Chromium M2: unchanged SSR app markup, same focused input, zero initial calls; Tab/Shift-Tab/Enter; associated blank-input error; zzz empty state; safe transport error with retry/dismiss; loading retains prior rows; cancel; B wins over late A; locale/title/description/language updates; one style/meta owner; theme toggle; disposal with no browser errors.
- Chromium M0 regression: hostile script-looking/Unicode seed is inert and roundtrips exactly; mismatch requests refresh; second typed query and disposal pass. Both browser harnesses use intercepted requests over browser pipes, not a listener.
- Responsive evidence: 375 and 1280 px, 100% and 200% **CSS zoom**, no horizontal overflow. Screenshots `/private/tmp/stackpress-s5-b-m2-{375,1280}.png` were visually inspected. CSS zoom is not claimed as a separate OS/browser-toolbar zoom proof.
- Packaged macOS ARM64: unsigned debug `Stackpress Profile M0.app`, 24.55 MiB. WKWebView performs automatic local bootstrap, invalid/empty/next-query states, locale/theme changes and disposal; ungranted window receives Tauri ACL denial. Receipt reports `main-ok`, `denied-ok`, `in_flight: 0`, `tauri://localhost`; launcher exits or kills/reaps after 25 seconds.
- Networking was not disabled at the OS level. Packaged assets/IPC require no listener and CSP limits network/resource access. Full networking-disabled owner acceptance and packaged detail navigation remain future integrated proof.

Evidence logs use prefix `/private/tmp/stackpress-s5-b-m2-`: `tests.log`, `browser.log`, `browser-m0-regression.log`, `launch.log`, `launch-receipt.json`, `tauri.log`, `coverage-summary.txt`, `coverage.json` and the check logs below.

## Checks and reproducible commands

All commands use locked dependencies. Installed stable rustc is 1.96.0; explicit +1.96.0 native check also passes. No tools were installed. Targets are isolated under `/private/tmp/stackpress-s5-b-m2-{native,hydrate,csr,tauri,cov}`. Only this lane's output was cleaned when disk space briefly ran out; no other lane's target was touched. The CSR retry succeeded with development debug symbols disabled.

```sh
cargo fmt --all -- --check
git diff --check
CARGO_TARGET_DIR=/private/tmp/stackpress-s5-b-m2-native cargo test -p stackpress-framework-ui -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client --features stackpress-example-profile-ui-shared/ssr --locked
CARGO_TARGET_DIR=/private/tmp/stackpress-s5-b-m2-native cargo clippy -p stackpress-framework-ui -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client --all-targets --features stackpress-example-profile-ui-shared/ssr --locked -- -D warnings
CARGO_TARGET_DIR=/private/tmp/stackpress-s5-b-m2-native cargo +1.96.0 check -p stackpress-framework-ui -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client --all-targets --features stackpress-example-profile-ui-shared/ssr --locked
CARGO_TARGET_DIR=/private/tmp/stackpress-s5-b-m2-native RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-framework-ui -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client --features stackpress-example-profile-ui-shared/ssr --no-deps --locked
```

Both `build-client.py hydrate` and `csr` pass with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` and mode-specific PROFILE_UI_TARGET. Mode-specific wasm32 client Clippy `-D warnings` passes. `cargo tauri build --debug --bundles app --features app --no-sign --ci` passes from the native directory with its isolated target. Native app code is unchanged; M2 extends its packaged proof script only.

Actual wasm32 client normal/feature trees are saved in `hydrate-tree.txt` / `csr-tree.txt`; neither contains Tokio, Tauri, Stackpress server/CLI, Idea transformer or SQL/driver edges. No manifest/dependency expansion was needed.

## Native coverage

Tool: cargo-llvm-cov 0.8.6, stable rustc 1.96.0, aarch64-apple-darwin. Scope: changed Rust crates, shared crate's **own ssr feature** plus its framework SSR dependency. Native tests execute actual production decision/state/render code. Browser-only `client/src/browser.rs` remains outside numeric coverage under the coordinator's M0 exception and stays a thin DOM/transport adapter.

| Changed crate | Covered / executable lines | Result |
| --- | --- | --- |
| framework-ui, native SSR | 547 / 551 | 99.27% |
| profile-ui shared, native SSR/controller | 273 / 292 | 93.49% |
| profile-ui client, native decision/codec scope | 77 / 80 | 96.25% |

Reproduce with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/private/tmp/stackpress-s5-b-m2-cov cargo llvm-cov -p stackpress-framework-ui -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client --features stackpress-example-profile-ui-shared/ssr --no-report --locked`, then run `proofs/coverage.py` with PROFILE_UI_COVERAGE_TARGET set to that target's `llvm-cov-target` and PROFILE_UI_COVERAGE_REPORT set to a lane-specific JSON path. The exporter includes examples without cargo-llvm-cov's blanket examples exclusion. Dedicated tests, dependencies and tooling are excluded; uncovered production functions are retained.

The first coverage attempt enabled only framework-ui's ssr feature, so shared's cfg-gated SSR tests did not run. That failed measurement was corrected using the shared crate's actual ssr feature in a cleaned lane-local target; the table is the corrected result, not the failed attempt.

## Findings, gates and remaining work

Two defects were caught and fixed during natural implementation: invalid submission failed to revoke an older completion; dynamic SSR title metadata retained a locale closure after Owner cleanup. Tests now protect invalid/stale/duplicate/disposed completion and the real buffered render/hydration path. This is implementation evidence, not the formal refactor rounds.

Stop at the route-contract boundary. Remaining M2 work: replace compatibility shims with real M1 APIs; wire result/detail navigation from the compiled catalog; preserve query/results across back/forward/refresh; rerun base-path, locale/theme initial-snapshot and packaged navigation proofs; obtain coordinator review before rounds 1–4. Control 00008-003 is not satisfied yet; full 00008-005 needs the integrated initial-locale proof. No whole-slice or owner-acceptance claim is made.

KB baseline validation before adding this receipt returned pre-existing errors from unavailable sibling-source links. Post-write validation is reported separately. No unrelated KB repair, commit, push, merge, listener, publication, deployment or MCP indexing was performed.
