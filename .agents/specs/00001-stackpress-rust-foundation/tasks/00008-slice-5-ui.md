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

## Acceptance notes

Pending.
