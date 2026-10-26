# Task 00008: Slice 5, shared Leptos UI and Tauri hosting

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-25, to run in parallel with [task 00007](00007-slice-4-sql.md) under [G23](../decisions.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Bring the UI layer into the monorepo under the [accepted source structure](../../../context/source-structure.md):

- `crates/framework/framework-view` (`stackpress-framework-view`): the page and render contract;
- `crates/framework/framework-ui` (`stackpress-framework-ui`): Leptos SSR with hydration, plus reusable components, kept separate internally;
- a first Tauri client host that shares the UI, contracts and operation source. Platform and execution-host differences stay in adapters.

The first vertical can reuse the in-memory `profile-search` capability (Q01/G15) and the generated Profile contracts from Slice 3, so this slice doesn't depend on Slice 4. Browser builds must not acquire database drivers or native host services through UI dependencies.

This slice resolves these open questions in [decisions](../decisions.md):

- Q01 (first Tauri target);
- Q02 (which operations run remotely, locally or both; offline);
- Q05 (the Tauri top-level entry);
- Q11 (Leptos routing, metadata and Head, server props and hydration, the Tauri operation adapter);
- Q12 (which React/Frui/r22n behavior to reimplement).

Unlike the library slices, UI tasks define visual acceptance criteria.

## Implementation steps

1. Design checkpoint, with no production code. Write it as spec files `slice-5-design-*.md`. It covers:
   - the upstream inventory: Reactus, Frui, r22n, [view API contracts](../../../references/00027-upstream-view-api-contracts.md), [interfaces and experience](../../../references/00015-upstream-interfaces-and-experience.md), and the [frontend decision history](../../../references/00007-discussion-frontend-decision-history.md). Map each item to now, a named later slice, or not ported;
   - Leptos and Tauri version and toolchain assessment, using disposable probes outside the repository: the WebAssembly target, build tooling, Tauri CLI, and SSR/hydration through the existing HTTP adapter (which buffers whole responses). Identify any install that needs owner approval;
   - the API sketch, with DX examples;
   - the risk inventory: hydration mismatch, server/client contract drift, `Send` bounds across SSR, browser-dependency leakage, the Tauri IPC trust boundary, and buffered versus streamed rendering;
   - open decisions with recommendations;
   - proposed visual acceptance criteria;
   - the milestone plan, with lane ownership sized for about two lanes.
2. Coordinator review and owner decisions, then M0, the riskiest proof.
3. Milestones, a stop before rounds, rounds 1–4, and closeout, followed by visual acceptance by the owner.
4. Docs as a follow-up task.

Any change needed in `stackpress-server` (including the HTTP adapter), `stackpress-lib`, the CLI or the Idea crates goes through the coordinator and lands in sequence, per G23. Any preview or dev server runs only through the devmetrics skill.

## Verification process

As defined in the coordinated workflow, with coordinator re-verification at every stop. This includes:

- SSR output and hydration tests;
- a WebAssembly build of the browser target;
- a dependency-tree check that the browser build carries no drivers or native services;
- a Tauri build on the chosen first target;
- per-crate coverage of at least 70%.

## Acceptance criteria

Owner-accepted on 2026-10-26: controls 00008-001 to 00008-008 in [Slice 5 risks, milestones and acceptance](../slice-5-design-verification.md#proposed-owner-visual-acceptance). They cover:

- SSR content before JavaScript, with no-flash hydration;
- keyboard search with error, empty and retry states;
- navigation and refresh;
- 375 px and 1280 px layouts and 200% zoom;
- Head and locale updates;
- the packaged macOS app working with networking off;
- stale or missing asset recovery;
- one plugin customization staying consistent across every adapter.

Styling is neutral grayscale; a third-party widget is deferred to S5-F2.

## Implementation notes

Design checkpoint prepared on 2026-10-25. Start with [Slice 5 decisions and recommendations](../slice-5-design-decisions.md), which routes to the assessment, architecture, API, the Reactus/view, Frui and r22n inventories, and the risks, milestones and acceptance plan. Probes ran only in `/private/tmp/stackpress-s5-probe/`; no production code or manifests changed.

## Verification notes

**Design checkpoint review (coordinator, 2026-10-26): approved; M0 released.**

- **Reviewed:** decisions, assessment, the M0 plan and the visual acceptance criteria.
- **Probe evidence (implementer):**
  - buffered Leptos SSR through the existing `HttpAdapter` renders the generated `profile-search`;
  - Send checks pass;
  - native Tauri v2 compiles and links;
  - the WASM check fails with E0463 because the target is missing (an install gate).
- **Coordinator check of the probes:** they use only the in-process adapter, with no listener. The coordinator's independent tool-MSRV lookup was blocked by a transient tooling failure, so the Trunk 1.96.1 claim stands on implementer evidence. The chosen route doesn't use Trunk.
- **Owner decisions:** recorded as [G25](../decisions.md), with every recommendation S5-D01 to D08 accepted. Installs approved: `wasm32-unknown-unknown`, `wasm-bindgen-cli 0.2.129`, `tauri-cli 2.12.4`.

**M0 re-verification (coordinator, 2026-10-26): passed; merged as `2685caf`; M1 released.** Details are in the [M0 receipt](../slice-5-receipt-m0.md).

- **Checks rerun by the coordinator** in a fresh target outside the shared-crate rlib-mixing defect:
  - fmt passed, Clippy `-D warnings` clean, `cargo +1.96.0 check --workspace --all-targets` passed;
  - 274 workspace tests passed, 0 failed, and 430 doctests passed.
  - After merging with Slice 4, the integrated tree also passes: 284 tests, 432 doctests, Clippy clean, `--locked` against the regenerated lockfile, and `sql-sqlite` + `framework-ui-ssr` features checked together.
- **End-to-end proofs rerun by the coordinator:**
  - **Headless Chromium hydration** (request interception, no listener) reported `{"hydration":"unchanged DOM and focus","initialCalls":0,"secondTypedQuery":"Grace Hopper","disposed":true,"hostileSeed":"inert exact roundtrip","mismatch":"visible refresh required","errors":[]}`.
  - **Packaged macOS `.app`** reported `{"events":["denied-ok","main-ok"],"in_flight":0,"packaged_origin":"tauri://localhost"}`; the ungranted window gets Tauri's ACL rejection, and no process was left running.
- **Isolation:** `cargo tree` for the wasm32 client has no Tokio, Tauri, server, CLI, Idea transformer or SQL edge. The default facade gains no Leptos, Tauri or framework edge. No shared-crate edits.
- **Coordinator-approved dependency exception:** `any_spawner =0.3.0` with `tokio`, native SSR only, for buffered `leptos_meta` Head assembly.
  - `initialize_ssr()` is idempotent;
  - an uninitialized executor returns the typed `ViewError::ExecutorUnavailable` instead of panicking;
  - the browser trees carry no any_spawner Tokio feature.
- **Coverage:** the native per-crate range is 87.84–100%; see the receipt table.
  - **WASM disposition (coordinator):** numeric line coverage of the browser-only `client/src/browser.rs` adapter isn't measurable with native LLVM coverage, so it's excluded from the numeric gate. It's exercised behaviorally by the real Chromium and WKWebView proofs above.
  - Condition carried forward: browser-only modules stay thin transport and DOM adapters. Decision and state logic lives in natively tested shared crates. The closeout re-checks this.
- **Findings carried forward:**
  1. The shared-crate compile-fail harnesses in `crates/lib/tests/handler_signature.rs` and `crates/server/tests/ownership.rs` pick the newest rlib by mtime, so a target holding several feature builds can pair incompatible builds. This is scheduled as a G23 shared-crate fix.
  2. The browser proof depends on Playwright from a user-local npx cache, which is a reproducibility limitation.
  3. The design packet's bare `--check` is not CLI syntax; use `--check true`.

**Lane A M1 re-verification (coordinator, 2026-10-26): passed; merged as `d86c3e7`.** Details are in the [lane A M1 receipt](../slice-5-receipt-lane-a-m1.md).

- **Checks:** fmt passed and Clippy `-D warnings` is clean; `cargo +1.96.0 check` passed; 297 workspace tests and 433 doctests passed, 0 failed.
- **Proofs rerun by the coordinator:**
  - the hydrate build passed;
  - Chromium at the root and at the `/demo/v1` base both reported unchanged DOM and focus, `initialCalls: 0`, the typed second query returning Grace Hopper, disposal, the mismatch refresh, and no console errors;
  - the packaged `.app` reported `denied-ok`/`main-ok`, `in_flight: 0`, and no process left behind.
- **Defect found and fixed by the implementer:** native captures arrive percent-encoded. The detail loader now decodes once before validation and re-encodes for the public location.
- **Coordinator integration edit:** `proofs/launch.py` was aligned to lane B's `PROFILE_UI_LAUNCH_REPORT` line, so both branches are identical.
- **Pinned host error envelope:** `code`, `status`, safe `error`, `errors`, `total: 0`. The shared decoder is assigned to lane B as an M2 integration item.
- **Lane B (2026-10-26):** independent M2 work committed on its branch as `5e51ac3`. Lane B now merges trial/3, replaces its shims with lane A's real route, asset and snapshot APIs, finishes detail and back/forward navigation, and adds the shared error decoder.

**Lane B M2 integration (coordinator, 2026-10-26): merged as `70627b5`; M2 not yet complete.** Details are in the [lane B M2 receipt](../slice-5-receipt-lane-b-m2.md).

- **Delivered:**
  - lane A's real route, asset and snapshot APIs replace every shim;
  - the shared HTTP/IPC host-error envelope decoder (host rejection versus transport or protocol failure, safe message, retry);
  - newest-search-wins even after a late successful older one.
- **Checks rerun by the coordinator:** fmt and Clippy clean; 367 tests and 442 doctests passed. The hydrate build passed; browser.cjs at the root and `/demo/v1` and browser-m2.cjs passed (keyboard, invalid, empty, transport failure, host error and retry, loading retains rows, cancellation, newest wins, locale, Head, theme, and 375/1280 px at 100% and 200% zoom).
- **Gap found:** lane A's M1 lacked the return-query codec and detail seed, and the renderer always selected search presentation. The coordinator's M1 verification missed this, because those proofs exercised only search.
- **Correction assigned to lane B as a single owner,** with a scoped exception to edit lane A's route, web and render files: a shared detail-location codec, a `{ id, search_prefix }` detail seed, a canonical return query, a page root selected from the snapshot, navigation, and an initial French/dark SSR-to-hydration proof.

**M2 completion re-verification (coordinator, 2026-10-26): passed; merged as `440b333`; M3 (native, lane B) and M4 DX (lane A) released in parallel.**

- **Delivered (lane B, with a scoped exception into lane A files):**
  - the shared detail-location codec and strict return-prefix decoder;
  - the `{ id, search_prefix }` detail seed;
  - a canonical return query;
  - a snapshot-selected page root (native 404 renders detail not-found);
  - navigation and history, plus initial French/dark SSR-to-hydration equality.
- **Defects found and fixed:**
  - keyed rows reused across a prefix change kept stale return links, found by the packaged proof;
  - French and English count phrases ignored plural categories ("1 profils"), found by the coordinator in the M2 screenshots and now fixed with per-locale plural categories tested for 0, 1 and 2.
- **Checks rerun by the coordinator:**
  - fmt, Clippy and the 1.96 check clean; 399 tests and 444 doctests passed;
  - the hydrate and CSR builds passed;
  - browser.cjs (root and `/demo/v1`), browser-m2.cjs and browser-navigation.cjs (root and `/demo/v1`) all passed with `errors: []`;
  - the packaged `.app` passed the full control list (bootstrap, invalid, empty, search, native host error and retry, denied transport, detail, history, return, missing, locale, theme, disposal), with `in_flight: 0` and no process left behind.
- **Acceptance progress:** 00008-002 and 003 have implementer and coordinator evidence. 004 and 005 are covered for native geometry, locale and Head. 006 to 008 remain for M3 and M4. Owner visual sign-off is still pending.

## Acceptance notes

Pending.
