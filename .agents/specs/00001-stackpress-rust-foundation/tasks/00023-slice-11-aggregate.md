# Task 00023: Slice 11 aggregate framework plugin

Task of [00001 Stackpress Rust foundation](../index.md), tracked in [task status](status.md). Released by coordinator on 2026-10-31 after Slice 9 docs merge ce80fa1 / record af9cc01. Worktree confirmed at af9cc01, initially clean. **Only the design checkpoint is released.** Follow [coordinated development](../../../workflows/coordinated-crate-development.md) and the later [fast build workflow](../../../workflows/fast-build-and-test.md).

## Task summary

Design the aggregate stackpress-framework at the accepted framework/framework location with optional root facade access. Cover the whole original packages/stackpress, not just its plugin list: source order, config/types/client, scripts/events, re-exports, backends, browser-safe exports, tooling/styles/assets and package metadata. Compose directly usable constituents with compile-time plugin selection. G42 multiple-handler semantics and G43 source behavior remain binding.

## Implementation steps

1. Design checkpoint, no production edits: [design entry](../slice-11-design-decisions.md), with [inventory](../slice-11-design-inventory.md), [all named exports](../slice-11-design-exports.md), [Rust overlap](../slice-11-design-constituents.md), [API](../slice-11-design-api.md), [targets/config](../slice-11-design-targets.md), [risk/milestones](../slice-11-design-verification.md) and [provenance/checks](../slice-11-design-provenance.md).
2. Coordinator review of full inventory, real versus missing registrars, exact native/renderer/build-host graph selection, static-client/admin binding, source phase ordering and owner questions. No new crate, root change, lock resolution or M0 build before release.
3. If released, follow the proposed M0 proof → M1 real adapters → M2 consumers/integration → M3 full gate → rounds → docs stops. Future file grants are explicit; this design does not authorize those stages.

## Verification process

Design gate: whole-package file/hash and exact export ledger; local link/anchor/line-cap coverage; frozen tracked source/manifests/locks; one appended status row; KB zero errors with canonical sibling-path resolution, raw worktree-only path diagnostics reported separately. No Rust compile evidence is claimed for sketches.

Later implementation: source-derived nine-stage/phase traces; source optional config/client/no-check modes; real SQLite/native and actual shared SSR/hydrate/CSR views; default/disabled/direct/facade/full/build-host standalone consumers; lock tuple subsets; full compatible tests/lints/docs and >=70% changed-crate implementation coverage. Deadlock tests fail fast in externally supervised kill/reap children. All Cargo/proofs recorded and locked/offline; workspace nextest excludes coordinator-owned tcp_watchdog. No implicit listener or live-provider permission.

## Acceptance criteria

Acceptance criteria: none for the library aggregate. Current scope creates no new UI behavior; existing visual behavior is regression evidence, not a new acceptance claim. If new UI becomes necessary, define separately released owner-visible acceptance scope first.

## Implementation notes

Design drafted only from af9cc01 and read-only upstream. Source order server/schema/language/CSRF/SQL/view/session/API/admin; source session composite auth/email/identity reused. Missing server/schema/language/command/client bridges are explicit work rather than empty placeholders. Pure libraries stay plugin-free, generated admin stays application-owned. Stock and compiled-client registration forms are proposals requiring M0 proof.

Open owner questions are carried embedded PGlite support and contradictory source license metadata before any copied-asset distribution. No security choice reopened; seven existing Slice 9/API source questions remain open at parity. Design evidence and benchmark details are in provenance. **Stop for coordinator checkpoint review; no implementation claimed.**

Implementer checkpoint checks: record **20261030T164203.385096Z-9edb5088444d**; 43 upstream file hashes, 30 package export keys, 77 exact export declaration rows / 2144 symbol occurrences, all nine new Agent Files within 200 lines, relative links/anchors/fences and frozen tracked-file scope passed. Raw KB remains 146 sibling-path errors / 28 warnings; canonical layout is 0 errors / 28 warnings, unchanged from baseline. No Cargo build or production edit occurred. This is checkpoint evidence; coordinator verification remains pending.

## Verification notes

Coordinator-owned; pending checkpoint review. Implementer mechanical/source-scope checks are recorded in [provenance](../slice-11-design-provenance.md#checkpoint-validation), not a verified status transition.

## Acceptance notes

None: library work, no new visual artifact or delegated product acceptance.

## Coordinator checkpoint review

Reviewed 2026-10-31 and merged into trial/3 as `6b52f58` (design commit `96e9830`). Location `crates/framework/framework` / `stackpress-framework` / `stackpress::framework` matches the accepted [source structure](../../../context/source-structure.md). Accepted coordinator recommendations: static compiled-client seam (`register` / `register_with`), new native source adapters live in the aggregate (schema/language stay plugin-free), `default = []`, `framework-aggregate-*` facade prefix that leaves existing root feature meanings unchanged, generated admin stays application-owned and last. Source order, G42 and G43 framing are correct; missing command/client/locale bridges are explicit, not placeholders.

Owner questions carried (non-blocking for M0): PGlite embedded runtime (default: separate later follow-up S11-PGLITE, no dummy feature) and the upstream MIT/Apache metadata conflict (default: copy no source assets until decided). M0 released with the grants recorded in the release message: new crate under `crates/framework/framework/**`, one root `Cargo.toml` workspace-member line and the resulting `Cargo.lock` workspace-edge entry only (no new external tuples), fresh `target/slice11` with a 24 GiB cap (guard 23.4).
