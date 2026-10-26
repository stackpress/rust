# Slice 8 decisions for owner review

Design checkpoint for [task 00013](tasks/00013-slice-8-more-ui.md), 2026-10-26, `trial/3` at `61457c91b91c4743fb7891075be963237a02f5a6`. This packet completes the proposed design scope; it does not release M0, freeze APIs, claim implementation or visual acceptance, or authorize installation. [Context](../../context/index.md), [G23/G25](decisions.md), and the [coordinated workflow](../../workflows/coordinated-crate-development.md) govern.

## Read by question

- [Component inventory](slice-8-design-inventory-components.md): load for every Frui module, inherited exports/types/tests, breadth and intentional differences.
- [Framework inventory](slice-8-design-inventory-framework.md): load for Reactus, schema roles, admin, r22n, language/session and other access surfaces.
- [Architecture and dependencies](slice-8-design-architecture.md): load for Slice 6/7 handshakes, crate ownership, persistence, mutation policy, streaming and G23 requests.
- [Author experience](slice-8-design-api.md): load for table/dialog/form composition, generated admin CRUD and persisted locale examples. All new names are proposals.
- [Library and tooling assessment](slice-8-design-assessment.md): load for disposable evidence, exact candidate versions, platform limits and the owner installation request.
- [Risks and acceptance](slice-8-design-verification.md): load for proving tests, two lanes, milestone gates and plain-language visual controls.

## What can start, and what must wait

| Order | Work | Can start after design approval? | Blocking contract |
| --- | --- | --- | --- |
| 1 | Advanced components: semantic tables/paging, disclosure/navigation, overlays, basic and collection inputs, simple views | Yes. Build reusable behavior and a component gallery without generated data. | Existing Leptos/view contracts. Overlay SSR/hydration is the first risk proof. |
| 2 | Sanitized Markdown/HTML; deterministic decimal/date/phone presentation; strict input state and specialized parsing | Yes, in batches after each dependency proof. | Reviewed sanitizer/formatting policy, budgets and exact-version dependency approval. Editors/charts/media remain evaluated extensions. |
| 3 | Locale routing/persistence, theme and bounded flash, permission-aware layout projection | Yes. Requires application host guards for protected content. | No need to wait for a full identity implementation; do not invent one inside UI. |
| 4 | Generated field/filter/span/list/view wrappers and admin workflows | Contract design now; production generation waits for Slice 6. | Normalized role definitions, prop rules, typed CRUD/search/relations and disclosure/validation contracts. |
| 5 | Native expansion | Host inventory/design now; target builds wait for explicit installs and target approval. | iOS simulator first is recommended; Android can follow with its independent toolchain gate. |
| 6 | Component-owned async SSR, reactive Head, then streamed SSR | Isolated buffered async proof can start now. Production Resource replay needs a new M0 gate; streamed integration waits for Slice 7. | Owned render lifetime, single-writer scheduling, resource serialization; then owned Send stream/commit/cancellation contract. |

These are staged parts of Slice 8, not six simultaneous implementations. Do not hold component or locale work behind schema/streaming. Do not substitute handwritten CRUD or a collected String for the blocked generated or streamed deliverable.

## Plain-language owner choices

Every row remains open. Recommendations are not accepted decisions. The coordinator records the owner's answers in the shared ledger and releases only the next gate.

| Choice for the owner | Recommended choice | Alternative and consequence |
| --- | --- | --- |
| How much of the component library should this slice deliver? | Deliver all ordinary navigation, disclosure, overlay, table, input/collection and safe display families in tested batches, followed by the generated admin workflows. | Deliver only the table/dialog/basic form first and schedule the other explicitly inventoried families as another task. This is smaller but leaves much of the accepted breadth outstanding. |
| Which rich content should be enabled by default? | Offer optional sanitized Markdown/HTML with a closed tag/URL policy; omit scripts, style, SVG, embedded frames and remote images. Keep editors/charts/media as separately approved integrations. | Ship text/Markdown text only first and defer HTML insertion entirely. Arbitrary HTML/formula execution remains excluded under G25. |
| How much number/date/phone behavior should be included? | Typed decimal strings, pinned locale data, ISO date/time values, explicit timezone, E.164 phones; prove cursor/IME/bounds behavior for formatted inputs separately. | Only display formatting now; defer specialized editing. A native number/date field must not be presented as complete Frui parity. |
| Which generated admin workflows should ship? | Search/filter/sort/page, detail/create/update/copy/remove/restore and related records, then bounded CSV import/export as separate milestones after Slice 6. | Stop at search/detail/create/update/remove and schedule restore/related/import/export as a named later task. Missing schema roles still remain omitted. |
| What should happen if an app supplies no authorization or CSRF policy? | Keep generated protected pages and mutations disabled. Require explicit host policy; use a clearly identified test-only policy in the disposable demo. | Provide public read-only pages only. Neither option implements sign-in, JWTs or grants automatically. |
| How should website locale selection and persistence work? | Configured locale prefix wins, then saved cookie, then configured default. A picker persists through a validated same-origin action; legacy unprefixed routes stay readable. Start with English/French, explicit aliases, and no browser-language guessing. | Query-only selection/persistence is smaller; it lacks stable locale-prefixed links. No silent change to existing `en`/`fr` seeds or region aliases. |
| Where should installed applications remember locale/theme? | A small host-managed preferences file through fixed read/update commands, loaded before first CSR render; preserve unsaved edits if persistence fails. | Process memory only initially, with a visible statement that restart resets preferences. No generic filesystem IPC grant. |
| Where should generated CRUD execute in an installed app? | Use an explicitly configured remote typed host with application-supplied authentication; retain the existing local read-only search demo. No offline write/sync promise. | Local database writes require another policy/proof gate for storage, migrations, authorization and uncertain write outcomes on each target. |
| Which additional native targets should be proved first? | Approve iOS ARM64 simulator tooling first; make Android ARM64 emulator a separate approval. Keep signed distribution, updater and other-desktop release claims deferred. | Approve both mobile toolchains together, or retain macOS-only delivery for this slice. Exact installations are in the assessment. |
| How should async rendering be introduced? | Prove complete buffered async pages with resource replay and reactive Head first, then opt-in in-order streaming after Slice 7. Buffered pages remain the default. | Defer all async production support, or take on out-of-order streaming now with inline-script/CSP/serialization complexity. |
| What should a streamed page do if it fails after bytes were sent? | Stop the stream, cancel/drop owned work, retain diagnostics, and show a safe retry/navigation state where client recovery is possible; never claim a replacement HTTP 500 or retry a write. | Keep that page buffered so ordinary fresh-response error replacement is possible. This choice must agree with Slice 7's commit policy. |
| Are the proposed visual checks sufficient? | Accept the numbered plain-language controls in the verification file, retaining neutral grayscale. | Amend the list before M0. Acceptance still requires the owner's rendered review after implementation. |

## Installation approval request

No installation has been performed. Request approval separately for the exact iOS simulator set and, optionally, the Android set in [the assessment](slice-8-design-assessment.md#owner-installation-request). Library source downloads for disposable probes are not installed developer tools. Mobile initialization, license acceptance, device pairing, signing identities and release credentials are not authorized by this packet.

## Gate and handoff

The spec-only write set is `slice-8-design-*.md`. Task/status/index/accepted decisions, Slice 6/7 documents, Cargo files, production code and upstream sources remain outside this implementer's write scope. This decisions file routes every companion, so the coordinator can link one entry from the task after review.

After owner choices and coordinator dependency/G23 review, release M0 only. Recheck the blocked interfaces with their owners through the coordinator. No commit/push, context promotion, KB publishing, listener or mobile install follows from this checkpoint.
