# Slice 8 risks, milestone gates and visual acceptance

Verification proposal for [owner decisions](slice-8-design-decisions.md). This is the implementer's independent risk inventory, beyond the coordinator's named examples. Every new component family needs native state tests and real rendered behavior; coverage alone is insufficient.

## Risks and proving tests

| Risk | Prevention | Required evidence |
| --- | --- | --- |
| Overlay traps or loses focus | Native dialog where supported; explicit trigger, dismissal stack and deterministic labels/IDs; no hidden duplicate portals | Keyboard open/Tab/Shift-Tab/Escape/close, nested dialogs, removed trigger fallback, pointer dismissal policy, unmount and disabled background on Chromium and packaged WKWebView. |
| Table used as a broken grid | Native caption/th scopes, explicit row keys, labelled overflow and controlled sort/page | Screen-reader structure, sort-state announcement, 0/1/many rows, unique-key validation and retained row updates; keyboard scroll at narrow width/zoom. Virtualized editing grid is a separate task. |
| Hydration repairs or loses input | Deterministic semantic markup and stable state/IDs; closed overlay initially | Real SSR -> parse -> hydrate equality for each family, retained node/focus/draft, no duplicate listeners; zero initial data/persistence calls; hostile text and French/dark initial seeds. |
| Date/number/phone drift | Typed values, explicit locale/timezone/precision and pinned metadata | Native versus actual WASM strings for en/fr, Unicode separators, zero/negative zero, invalid/partial edit, bounds, paste, caret, IME and timezone/DST boundaries; invalid values never silently become zero. |
| XSS or sanitizer bypass | One narrow sanitizer before any HTML insertion, private checked HTML wrapper, bounds and fixed policy revision | Encoded schemes, script terminators, malformed/nested markup, mutation-XSS corpus, SVG/MathML/style/events/URL tests in native and real browsers; sanitizer repeat equality and SSR/hydrate/CSR equality. CSP is additional protection, not the sanitizer. |
| Content or metadata causes unexpected network/code | Closed resource/URL policy, no arbitrary iframe/eval; optional integrations isolated | Block external requests, javascript/data/protocol-relative URLs, stylesheet/events and remote image probes; malicious metadata/links never load a resource. |
| Editors/charts leak listeners or alter SSR | Dedicated client adapter, static accessible SSR fallback, explicit lifecycle and data contract | Before selection: module/version/license/bundle-size audit, mounted/unmounted listener/worker proof, selection/IME/undo/paste, CSP, no remote CDN, keyboard and data-table fallback. Read-only evaluation does not mean adopted. |
| Schema props become executable code | Validated typed definitions, source spans and escaped Rust token generation | Hostile quotes/import names/labels and unknown component/prop diagnostics; fixed-prop collision, alias, omitted role, nullable/default/false/zero behavior, generated compile/check/remove/rename. |
| Secret or unauthorized record disclosure | Explicit public DTO/field policy; authoritative edge and row/relation checks | Sentinels absent in HTML/seed/export; denied page/API/IPC/import/relation cases invoke no domain handler. Client menu changes cannot bypass host guards. |
| CSRF or accidental writes | Fixed typed mutation routes, origin/CSRF check, no GET writes, explicit confirmations | Missing/wrong/cross-origin token and action/row tampering denied; valid form and API agree; copy omits identity; direct event invocation has separately stated caller policy. |
| Lost/duplicate writes | No automatic retry; preserve drafts and expose outcome-unknown; use expected-version if Slice 6 offers it | Cancel before/after admission, late success, timeout after commit, stale edit, double click and navigation; never fabricate rollback or promise idempotency absent a domain contract. |
| CSV resource/formula abuse | Byte/row/column/field caps, strict parser, selected public fields and preview | Malformed UTF-8/quotes/newlines, inconsistent columns, cancellation/progress, per-row failures, denied writes; formula-leading cells exported under a documented spreadsheet-safe policy and no arbitrary client file access. |
| Locale routes loop or run lifecycle twice | Compiled locale routes, strict one-decode codecs, explicit precedence | Prefix/cookie/default/alias table, unknown/duplicate locale, base/return/query encoding, POST acknowledgement, refresh/history and exactly one top-level lifecycle. |
| Persistence flashes or corrupts | Host-authoritative initial settings; bounded checked preferences, scoped cookie deletion | No first-render mismatch; HTTP-only cookie flags/path; invalid locale/theme; blocked cookies, write failure, corrupt/truncated/oversized/symlinked native settings and restart recovery; unsaved edits preserved. |
| Flash duplicates/leaks across callers | Public bounded payload, clear same cookie scope, root ID dedupe and no-store HTML | Refresh and duplicate ID, hostile flash/error, concurrent tabs and different callers; document best-effort cookie consumption rather than exactly-once delivery. |
| Full session support inferred from menus | Safe allowed-action projection and explicit host policy; fail closed if absent | Guard-not-configured startup/exposure failure and denied deep link/forged mutation; no token/role snapshot used as authority. Full auth remains separately scoped. |
| SSR Owner/Head leaks or crosses await unsafely | Preserve old renderer; new async root owns render/resource life; restore thread-local context per poll/unwind | Outer Owner/context success/error/panic/cancel table, Weak/drop probes, reactive Head and actual Resource replay; Send compile assertions and real Rc-across-await rejection. No retry of failed metadata work. |
| Resource completion races a root | Owner-root publication queue or proven single-context scheduling; no arbitrary cross-thread signal mutation | Barriers for completion/disconnect/disposal, independent concurrent pages/locales, late A after B and same-root callbacks. If public Leptos APIs cannot meet this, stop for redesign. |
| Streaming changes G12 after commit | Explicit header commit and owned Send stream; no request borrow in stream | Precommit failure gives fresh safe response; postcommit failure terminates without status rewrite/recursive lifecycle; response hooks execute once; no private cause in fragments. |
| Slow consumer/cancel leaks async work | Pull-driven bounded buffers/admission/deadlines; Drop cancellation and tracked task ownership | Slow finite sink pauses producer; byte/chunk overflow; disconnect/deadline/graceful shutdown; root/resource count returns to zero inside watchdog; no detached task or global render mutex. |
| Tauri mobile trusts desktop origin assumptions | Exact per-platform origin/window metadata, fixed commands and local capability grants | Real packaged simulator/emulator allowed/denied calls, forged window/event/ID, malformed/oversized DTO, other-window cancel and background/resume/close; no localhost fallback. |
| Mobile plugin/secret exposure | Minimal capabilities; secret-free preferences; signing and update keys outside source/UI | Capability/CSP/package audit; no shell/arbitrary fs/emit; inspect artifacts/logs for sentinel secrets. Signing/updater cannot be marked complete without separate credential/release approval and signature-negative tests. |
| Native target overclaim | Target-specific build/packaged/interaction receipts | Native build is not simulator launch, launch is not device/signing acceptance. Report skipped/unavailable targets and OS/WebView versions explicitly. |
| Browser dependency/coverage leakage | Separate runtime modes, native tool packages and thin WASM adapters | Build actual hydrate/CSR/frontend facade trees with `--edges normal,no-proc-macro`; deny DB/server/CLI/Idea/Tauri/Tokio. Keep controller/decoder/policy branches in native coverage. |
| Panic/deadlock/poison/reentry | No locks/RefCell guards across await/callback; bounded admission; no implicit recovery | Child-process reentrant dispatch, poison, render panic and resource-drop fixtures under parent watchdog. Do not infer arbitrary destructor/panic-abort recovery. |

## First proof and stop

M0 has two independently reviewed proofs, not broad implementation authorization:

1. **Components/persistence:** real typed Table/Dialog/Form plus locale preference action. SSR/hydrate/CSR use one component source, preserve initial DOM/focus, enforce native-dialog behavior, sanitize one rich-content corpus, and prove locale refresh with no initial refetch. Keep existing synchronous Head/Owner rules and error causes.
2. **Async feasibility:** real component Resource + Suspense + reactive Head, escaped replay payload, zero duplicate initial fetch and Send top-level loader/renderer; success/cancel/unwind restore the caller Owner and drop resources. A delayed Suspend/collected stream alone does not pass it. Streaming stays a separate gate after Slice 7.

The coordinator may release proof 1 while proof 2 remains explicitly blocked/deferred. Do not label all M0 complete if either approved part fails. No mobile build until installation decisions are applied by an authorized operator.

## About two implementation lanes

| Lane | Exclusive files after coordinator assignment | Milestones |
| --- | --- | --- |
| A reusable UI/rendering | framework-ui components/controllers/rich/formatting; additive async renderer; gallery and own finite component/browser proofs | M1 semantic/presentation/input core; M2 overlay/navigation/table/collection breadth; M3 rich/formatting and specialized input behavior; M7 async buffered, then streamed integration after Slice 7. |
| B framework integration/generated/native | framework-language; framework-admin and UI generator; new generated-admin example contracts/shared/web/native and its proofs | M1 locale/theme/flash/policy projection; M4 role wrappers after Slice 6 handshake; M5 core admin CRUD/related; M6 bounded import/export and selected native target after approval. |

One coordinator-assigned integrator owns framework-ui exports/manifests, framework-view settings/async registration, root manifest/lock/facade, build/asset/wire revisions and shared example contracts. Lanes request patches through the coordinator; never both edit these files. Slice 6 owns schema/data generation and Slice 7 owns server streaming; G23 sequencing is mandatory. The coordinator schedules total agent capacity across the three slices; two lanes here is a plan, not a request to spawn now.

| Milestone gate | Evidence before advancing |
| --- | --- |
| M1 | Label/error/controlled values; deterministic SSR/hydrate locale/theme; failed persistence and absent host policy; no dependency leaks. |
| M2 | Every selected component row has state/accessibility/browser proof, including nested/unmounted overlay and paging. |
| M3 | Sanitizer and deterministic native/WASM formatters; full specialized edit-state behavior or an owner-approved explicit deferral; accepted optional dependency/bundle report. |
| M4 | Actual Slice 6 metadata, alias/precedence/omission/diagnostic fixtures; deterministic compileable generated role wrappers and stale/model removal checks. |
| M5 | Actual generated data layer, real SQLite, allowed/denied edge cases, forms/relations/errors/uncertain write behavior; no handwritten operation substitute. |
| M6 | Preview/limits/authorization/import-export outcomes; actual selected packaged native target with restart preferences and IPC denials. Uninstalled targets remain blocked, never passed. |
| M7 | Buffered actual Resource replay first; then Slice 7 stream/drop/commit/slow-sink proof. Resource/Head/Owner tests rerun with final types. |
| Before rounds | Component/inventory completion and named deferrals, exact feature/target matrix, per-crate production-line coverage and owner-visible artifacts. Stop for coordinator re-verification. |
| Closeout | Four distinct test-verified rounds: complexity; responsibility/abstraction; consistency/Rustdoc/comments; safety/soundness. Repeat targeted rounds for material findings. Then coordinator re-verification, owner visual review and docs follow-up. |

## Execution and coverage rules

Native unit/contract tests own meaningful state, input/error/security branches. Feature-specific `cargo test`, doctests, warning-denied Clippy/Rustdoc, rustfmt and pinned compiler checks precede the workspace gates. Clean only lane-owned target directories at a fresh effort; never clean shared active output.

Require at least 70% executable production-line coverage in each changed crate with tool/target/features/counts/exclusions recorded; publication later requires at least 90%. Generated shipped code needs compile/runtime behavior evidence and stated coverage. Preserve stronger existing tests. The accepted WASM exception applies only to thin DOM/fetch/IPC modules exercised in real Chromium/WKWebView; new policy/state/parser logic is not exempt by being placed under `cfg(wasm32)`.

Deadlock, poisoned registry, panic, never-resolving Resource and shutdown tests run in a separate child with ready/go barriers, a parent timeout (normally 5–10 s), and kill/reap on expiry. Browser wrapper has a 120 s parent deadline plus 10 s action deadlines and closes in finally; mobile/app launch wrappers have explicit shutdown/watchdog/reap. A timeout is failure, never ignored green. Finite HTTP tests use the real service with duplex or request interception, never a listener.

Keep byte/chunk/field/row and time budgets explicit and lowerable. Proposed initial rich input/output caps are 64/256 KiB; admin pages cap 100 rows and collection inputs 100 entries, import 1 MiB/1,000 rows/100 columns. Exact limits are reviewed during M0 and checked at both edges. No arbitrary component/library CPU sandbox or automatic write rollback is claimed.

## Proposed owner visual acceptance

These controls are recommendations pending the owner's answer; retain accepted Slice 5 controls as regressions. Styling remains neutral grayscale. Automated geometry/interaction/axe checks aid review but do not replace owner acceptance or screen-reader inspection.

| Control | What the owner should be able to see and do |
| --- | --- |
| 00013-001 | Open the component gallery and generated list before JavaScript finishes. Labels, rows and useful content are visible; hydration keeps typed text, focus and scroll without a flash. |
| 00013-002 | Use only the keyboard to open/edit/close a dialog. Focus stays inside it, Escape follows its stated policy, and closing returns focus to the opener. Nested or removed dialogs leave the page usable. |
| 00013-003 | Read the table headers/caption, sort, filter and change pages. Loading keeps useful rows; empty, denied and failed states explain the next action. Sort/page state agrees with refresh and Back. |
| 00013-004 | Create, edit, copy, remove and restore a generated record where the model supports it. Invalid fields are named and focused; drafts survive errors. Destructive actions require confirmation; uncertain writes are visibly uncertain. |
| 00013-005 | Open related records without losing the parent context. A user without permission cannot reach private data by typing its URL or forging a request; hidden buttons are consistent with those denials. |
| 00013-006 | Preview a CSV import, fix row errors and explicitly submit. Progress/failure/cancel states are clear. Export contains only approved fields and opens without executing spreadsheet formulas. |
| 00013-007 | Switch between English and French while a form is dirty. Text, date/number presentation, title, document language and links agree; refresh/restart remembers the choice. A failed save explains the failure and preserves the draft. |
| 00013-008 | Change theme, refresh, and receive/dismiss one public flash notice. There is no initial light/dark flash, unreadable state or repeated notice in the same root. |
| 00013-009 | View malicious-looking Markdown/HTML as inert safe content. No popup, broken layout, hidden form or unexpected network load occurs. Code and unsupported content have a legible fallback. |
| 00013-010 | At 375 px and 1280 px and 200% zoom, reach every action and error without page-wide clipping. A wide table scrolls within its labelled area. Keyboard focus stays visible; motion can be reduced. |
| 00013-011 | In the selected installed target, navigate, use permitted operations and restart with saved preferences. Denied commands and background/resume/cancel behavior are clear. The packaged UI needs no development server. |
| 00013-012 | On an opted-in streamed page, useful shell content appears before a deliberately delayed section. The section settles without duplicate loading; leaving the page cancels work. A late failure gives safe recovery without a false success. |
| 00013-013 | Replace one registered component or label through an app plugin, regenerate, and see the same change in SSR, hydrated web and installed UI. Disabling it and rebuilding removes it. |

Controls involving generated CRUD, mobile or streaming remain pending their named dependency/target gates. A gallery screenshot does not satisfy them. Produce a review packet at root and mounted base paths with exact screenshots, artifact/tool versions and known limitations; owner sign-off remains a separate step.
