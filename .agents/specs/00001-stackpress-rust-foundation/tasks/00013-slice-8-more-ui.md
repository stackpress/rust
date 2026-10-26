# Task 00013: Slice 8, more UI

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-26, in parallel with [task 00011](00011-slice-6-generated-data-layer.md) and [task 00012](00012-slice-7-production-servers.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Grow the UI layer beyond the first vertical. This covers the deferred work named S5-F1 to S5-F6 in [Slice 5 decisions](../slice-5-design-decisions.md):

- **S5-F1:** advanced components (disclosure/overlay/table/navigation and the remaining input/view families), each with accessibility and behavior proofs.
- **S5-F2:** rich content (sanitized Markdown/HTML, date/number/phone, and evaluated integrations such as editors, charts and media).
- **S5-F3:** schema-to-UI generation (admin CRUD screens from the generated data layer). **Depends on task 00011.**
- **S5-F4:** language/session integration (locale routing/persistence, theme/flash cookies, permission-aware layouts).
- **S5-F5:** native targets (Tauri mobile and other desktops, signing, updater).
- **S5-F6:** async UI rendering (component-owned SSR resources, suspense, and streamed rendering, which **depends on task 00012's streaming server**).

The design must propose an order and a breadth that fit the owner's preference for broad functionality without compromising safety. It must say plainly which parts can start now and which must wait for tasks 00011 or 00012.

## Implementation steps

1. Design checkpoint, with no production code. Write it as `slice-8-design-*.md`, per workflow §5.1. It covers:
   - the upstream inventory (references 00015, 00027, 00028, 00031, 00032, 00041 and the Slice 5 inventories);
   - a library and tooling assessment, using disposable probes, including any new installs (for example mobile toolchains) as an explicit request for owner approval;
   - the API sketch with DX examples;
   - the risk inventory: accessibility, XSS and sanitization, hydration and streaming, IPC on mobile, signing secrets;
   - **proposed visual acceptance criteria**;
   - open decisions written in plain language;
   - dependencies on tasks 00011 and 00012;
   - the G23 shared-crate requests;
   - a milestone plan for about two lanes.
2. Coordinator review and owner decisions, then M0 and the milestones, rounds 1–4, closeout, owner visual acceptance and a docs follow-up.

## Verification process

As in the coordinated workflow and task 00008: browser and packaged-app proofs, accessibility checks, the WASM dependency isolation, per-crate coverage of at least 70%, and owner visual acceptance.

## Acceptance criteria

To be proposed at the design checkpoint and accepted by the owner.

## Implementation notes

Design checkpoint delivered on 2026-10-26: start at [Slice 8 design decisions](../slice-8-design-decisions.md). It awaits owner decisions and the installation approval; M0 is not released.

## Verification notes

Pending.

## Acceptance notes

Pending.
