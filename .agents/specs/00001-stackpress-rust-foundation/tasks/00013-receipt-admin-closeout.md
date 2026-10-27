# Task 00013 — admin closeout and submitted-draft safety fix

Status: implemented; stopped for coordinator review. No commit, push or publication. This supersedes the initial safety-stop status under the coordinator's explicit closeout exception; the original stop receipt remains in the evidence directory as `receipt-safety-stop.md`.

## Scope and provenance

Started with `git merge trial/3`, fast-forward `1ce84a8..8d63b7b`; the worktree was clean. Current base is `8d63b7b8785dca0cbba864081250b9d72848ffe8`. Reviewed the required context/style/testing workflows, Slice 8 architecture/inventory/verification/decisions and prior M4/catalog/M5/M6 receipts. Rust-adapted coding, technical documentation and Playwright skill guidance were applied; explicit coordinator rules govern permissions, installed Playwright and listener-free proofs.

Only framework-admin, examples/generated-admin and this pre-approved receipt changed. No framework-ui/view/schema/sql, SQL/server/Idea, generated-workspace, profile-ui, Cargo manifest or lockfile edits. Revision 2 remains frozen. Tracked generated output was **not regenerated**: both roots pass read-only freshness checks. Plugin/disable regeneration runs in disposable copies of the workspace with current source overlaid. The coordinator can reconcile the data-layer agent's separate generated-store changes at merge.

Evidence root: `/private/tmp/s8-admin/closeout-admin-agent/`. Other evidence directories were preserved. The runnable entry point and case inventory are in the [example README](../../../../examples/generated-admin/README.md).

## Initial safety stop and before/after evidence

The initial review obeyed “If you find a real safety bug, stop with a counterexample.” At the recorded head, `crud::complete` (lines 130–135) unconditionally selected Saved; the draft-only effect in `editor` (lines 175–192 and 225–234) did not rerun when an acknowledgement arrived. No fix was made before the coordinator released the exception.

The counterexample used actual generated Create/Get handlers and real SQLite:

1. Enter ID `race`, name **Submitted value**, and submit.
2. The finite native HTTP request completes the real write and closes its pool. Hold only delivery of the successful acknowledgement in the `.invalid` interceptor.
3. While Submitting remains visible and the input stays enabled, change the name to **Later unsaved value** and settle effects.
4. Release the old acknowledgement. SQLite contains **Submitted value**, the input displays **Later unsaved value**, but the form claims **Saved** with Save enabled.

It reproduced in hydrate/CSR at both `/` and `/admin/`, without browser errors or unexpected requests. This could cause a user to leave believing newer content was saved. It is separate from a database last-write-wins conflict: the later value was never submitted.

Before evidence is retained at the evidence root:

- `pending-edit-counterexample.cjs`, `counterexample-watchdog.py`, `pending-edit-counterexample.json`, `counterexample.log` and `counterexample-result.json` preserve the four-case reproduction, head/source/native hashes, stored/visible values, and 120-second parent / 105-second inner / 10-second native bounds. Its exit 0 means it observed the bug, not that the application passed.
- Four `pending-edit-<mode>-<mount>.png` images show the bug; hydrate/root was visually inspected. Relevant source was unchanged from accepted M6 `cec2ef5`; source SHA and native artifact SHA are in the JSON.
- The permanent regression, before the fix, failed with exit 1: actual `Saved`, expected `Saved the submitted version. You have newer unsaved changes.` See `write-races-before.log`, `write-races-before-result.json` and its watchdog. The accepted M6 artifacts were reused read-only from `/private/tmp/s8-admin/m6-admin-agent`; no build, new target, listener or install was needed for the original stop.

The original reproducer remains runnable through its watchdog with `ADMIN_PLAYWRIGHT` naming the installed module and `ADMIN_REVIEW_ARTIFACTS` naming those accepted artifacts. Its output accepts `ADMIN_PROOF_ROOT`, default `/private/tmp/s8-admin`.

The coordinator then explicitly released the fix and the remaining closeout work. The proposal in the stop receipt is implemented below; its deferred work is completed here.

## Exact submitted versions and retained drafts

[Private write tracking](../../../../examples/generated-admin/shared/src/write_version.rs) captures the admitted structural draft once at submission and observes current signals directly during acknowledgement rendering. It avoids a deferred-effect race. Structural edits reverted to the exact submitted values are the same version again. Invalid/uncommitted formatted edits also count as newer through the root's PendingEdits state.

- An unchanged draft shows **Saved**. A newer draft shows exactly **Saved the submitted version. You have newer unsaved changes.** It remains editable and Save becomes available again.
- UnknownOutcome explicitly refers to the **submitted version**, preserves newer values and remains terminal for resubmission. There is no automatic retry.
- Validation, authorization and acknowledgement errors preserve controls. Chosen rule: field errors are marked **Submitted version** until the next admitted request clears them, rather than being silently attributed to newer values. Root and field feedback use the same rule; local formatted-input errors still describe the current edit.
- Response data never writes draft controls, so server echoes cannot overwrite newer values. Pending requests block duplicate submissions while inputs remain editable. The Pending/Unknown guard now precedes local formatted-field validation, so a later partial edit cannot clear that guard or replace the submitted snapshot.
- Create, update, copy, remove/restore and related create share the same editor and frozen-envelope preparation. A private native test exercises every editor write action's frozen envelope/version, identity omission and preservation after a later edit. Existing actual related-write proofs retain the parent scope.

[Import confirmation](../../../../examples/generated-admin/shared/src/transfer.rs) binds successful preview permission to file generation, exact CSV bytes and decode option. A replacement file, even with identical bytes, cancellation, or a decode change requires another preview. Submit rechecks that identity synchronously; toggling the option away and back cannot reuse permission. No pending request lock is released by editing.

Newer selections remain intact while requests finish. A stale read cannot approve the current file; a known successful write remains known and names the submitted file, rather than inventing UnknownOutcome from a selection mismatch. UnknownOutcome also names the submitted file and preserves the newer selection, with no imported claim or retry. The host still independently authorizes and revalidates the resent file through the actual generated atomic operation; no file storage or background job was added.

After evidence:

- `races/write-races.json`, its command/result log and 32 screenshots: create/update × success/validation/denial/uncertainty × hydrate/CSR × both mounts. Delivery is held after the actual native result; newer controls survive, only the submitted value reaches SQLite, inputs stay enabled, duplicate form submission is blocked, and no replay occurs. The eight-case success-only after run is also retained as `races/write-races-after.*`.
- `transfer/browser-*.json` and screenshots cover replacing a previewed file, decode toggle/back, known confirmation with a newer file, and uncertain confirmation with a newer file. Native Get proves the newer file was not applied. Every formerly committed M6 case remains present.
- A second permanent counterexample within the released exception covers formatted-field submit events: partial input previously changed Pending to Failed, allowing a second attempted submission to replace the snapshot while the transport still held the first request. `round4/formatted-guard-before.log` records submitted/stored **1**, current draft **2**, and a false **Saved** claim; the expected failing result, source/native/WASM hashes and `format/screenshots/guard-before-formatted-guard-saved.png` are retained. After moving the guard first, all eight formatted base combinations run both Saved and Unknown subcases (16 subcases), preserve Pending/Unknown, retain newer values and send only one write. See `format/browser-*.log` and `*-formatted-guard-*.png`.
- Native tests cover structural/reverted/partial versions, the actual signal-owning tracker, frozen operation envelopes and exact import preview identity. Both default and formatted coverage runs execute them.

## Four whole-crate review rounds

All rounds covered framework-admin's feature paths and generated-admin tool/shared/client/web, their generated boundaries, tests, bootstrap and proof scripts. Native SSR/codegen/format and independent WASM hydrate/CSR default/format configurations were checked; incompatible renderer selections were not combined blindly.

1. **Complexity and branch risk.** Reviewed drafts/search/presentation, every scalar/collection/formatted adapter, catalog/import preflight and role/page/table emission, all host/transport/transfer branches and proofs. Found the proven acknowledgement race, stopped, then implemented the released snapshot fix and exact preview guard. Direct signal observation replaced the stale deferred Saved-reset effect. Verification: `round1/native.log`, later default/formatted native suites and delayed-response matrices.
2. **Responsibility/abstraction and duplication.** Retained existing shared `control_source`, pure formatted configuration, numeric bounds and `sync::connect`; roles, pages and tables have different contracts and do not justify another generic layer. Consolidated repeated proof runners/build/compile/graph/coverage orchestration and HTTP/browser setup. Private controller tests live separately from shipped implementation. Verification: runtime watchdog tests, generation/compiler proofs and every former M5/catalog-2/M6 case through the common harness.
3. **Consistency, public API, Rustdoc/comments.** Reviewed all feature APIs and logical-block/state comments. Corrected the obsolete blanket “no database writes” shared-crate comment and Phase lock description; documented exact/reverted versions, submitted errors, file permission and proof ownership. Kept existing useful Rustdoc examples and added no ceremonial commentary. The final native run caught a duplicate test-module declaration while adding the envelope regression; tests were merged into the existing private module and native/Clippy reruns passed. Verification: fmt, scoped Clippy -D warnings, native tests/doctests, Rustdoc -D warnings, MSRV and WASM checks.
4. **Safety/soundness.** Rechecked public projection, escaped generated literals/import tokens, bounds before CSV allocation/append, authorization before decoding, related scope, unknown outcomes, disposal and request/child lifetime. The same released acknowledgement race had the formatted-submit guard path described above; it was reproduced and fixed within the explicit duplicate-submission requirement. No unrelated application safety bug was confirmed. All production unwrap/expect sites reviewed retain checked/static invariants; no unsafe code was introduced. Native #11 contexts retain concrete causes; public parser/rule text is bounded/static public information. Proof watchdogs kill/reap on timeout, cancellation and budget failure; even a hung disk measurement cannot extend the child deadline. Final browser/compile/coverage runs verify the resulting code.

Public API narrowing candidates, **listed only; unchanged**:

- Tuple fields of `pages::{FormIssues, PendingEdits, LockedFields, EditorAction}` could use explicit construction/read/update methods before publication, while retaining the required cross-crate context seam.
- `AdminConfig` public fields currently allow bypassing its checked constructor; consider private fields/getters. Public PageSpec/Snapshot/ParentScope construction could also use validated builders without changing frozen serialized shapes.
- Review the stability promises for `presentation::{issues, submitted_issues}` and low-level `transfer::{write, fields, issue}`. They serve current consumers, but some may be implementation helpers rather than permanent extension points.
- Example-only `web::finite` harness input/fault controls, database setup and `host::{spec, related}` exports are integration/proof seams; consider a narrower feature-gated example surface before publication. Generated component functions and app registry entry points intentionally remain public.

## Proof consolidation and polish

[proof.py](../../../../examples/generated-admin/proof.py) is the parameterized driver. Cohesive `proofs/` modules own runtime/builds/generation/checks/graphs/coverage/browser/probe concerns; former Python entry points are thin compatibility wrappers. `report` re-exports coverage without rerunning tests. Three distinct HTTP scenario modules retain the CRUD, formatted/plugin and transfer assertions; one shared CJS harness owns SQLite fixtures, finite requests, interception, screenshots, errors and deadlines.

`ADMIN_PROOF_ROOT` configures all output and the sole target; default `/private/tmp/s8-admin`. Every suite has a subdirectory, and browser artifacts remain outside the target. Exact argv/cwd/status/time are recorded. Target usage is checked during gates against 8 GiB. No installed module path is hard-coded: Playwright comes only from `ADMIN_PLAYWRIGHT`. Browser cases have 120-second parents, 105-second inner deadlines and finite native-request deadlines. Three negative/runtime tests prove failure propagation, killed/reaped hung children, correct environment and a killed/reaped hung disk meter.

The consolidated single-target runner exposed a Cargo artifact-name collision: a fresh check reused a tool binary compiled in an already-deleted disposable workspace, producing NotFound. Selected tool/web/client packages are now cleaned before rebuilding across workspace selections. Coverage also removes only the two top-level example executables before each feature pass, preserving all instrumented objects for the combined report. A partial refresh exposed cross-feature executable reuse (45.04% web); that superseded result is not used. A complete forced-relink rerun restores coverage of both generated roots. Final runs exercise those clean/rebuild paths, plugin removal and successful live-root freshness checks; no tracked regeneration was used to conceal the failure.

Every former case runs, with current validation expectations:

| Suite | Final cases | Evidence |
| --- | ---: | --- |
| M4/catalog-1 roles | 24 (original 16 plus wide plugin/disabled cases) | `roles/browser.json` |
| M5 CRUD | 8 | `crud/browser-matrix.json` |
| Catalog-2 base/plugin | 16 | `format/browser-matrix.json` |
| M6 transfer | 8 | `transfer/browser-matrix.json` |
| Delayed write responses | 32 | `races/write-races.json` |

All 88 top-level combinations pass, including the 16 additional formatted-guard subcases, covering both mounts and hydrate/CSR; legacy suites retain 1280 px and 375 px/200% layouts. M5's stale generic validation assertion now checks the specific **Name </script> 🦀 is required**; write errors additionally carry the submitted-version prefix. Earlier gallery assertions were updated for hidden nonnullable creation selectors and the four current Task list columns; omission remains exercised through nullable controls/filters. The original omitted-slider/displayed-zero bridge assertion now runs through the current priority filter placement in both CRUD scenarios, rather than relying on a removed creation selector.

Polish: singular **1 row**, plural otherwise; visible limits **1 MB** and **64 KB**, with exact binary admission quantities documented. Import preview cells stack at narrow widths with visible Raw cells/Decoded values/Errors labels; column headers remain in the accessibility tree. Search and gallery tables have labelled keyboard-focusable scroll regions and a visible narrow-width scroll hint. An older gallery textarea also needed max-width. Assertions cover no page overflow, reachable overflowing columns, readable labels, row/unit wording, action gaps (including Previous/Next), no JSON result-link text, specific errors, selector visibility and retained formatted-entry/precision/width/hint behavior. Narrow preview/search and before/after acknowledgement images were visually inspected.

## Gates and coverage

- Workspace fmt and Clippy -D warnings passed; workspace `--locked` suite reported **714 test passes and 937 doctest passes**, no failures. As explicitly authorized, `tcp_watchdog` was skipped for coordinator verification; unrelated existing external/integration ignores remain visible in the log. The final additional private envelope regression is included in subsequent scoped native/Clippy and both coverage suites.
- Final scoped native tests/doctests and Clippy -D warnings pass for admin/tool/shared/web with SSR, codegen and format. Rustdoc -D warnings and `cargo +1.96.0 check` pass. Four independent WASM Clippy/normal-edge graphs pass (hydrate/CSR, default/format), with no server/DB/SQL/Tokio edges.
- `graphs/default-graph-comparison.json` compares exact normal-edge package/version sets against trial/3 `8d63b7b`: default hydrate and CSR graphs are identical. Existing URL/IDNA ICU stays; no formatting ICU or phonenumber package is added by default.
- `compile/compile-proof.json` records byte-stable ordinary/formatted generation, actual rename/removal, expected obsolete-consumer exit **101**, repaired consumer, hostile JSON/formatted labels, omitted roles, all-model removal and warning-clean empty consumer. Both tracked roots have empty freshness differences (`round4/generated-*-fresh.log`). Compiled replacement/disable builds restore the full managed formatted root byte-for-byte.

Coverage: cargo-llvm-cov **0.8.6**, Rust **1.96.0**, native default + SSR/codegen/format union. Raw compiler JSON is retained after removing the tool's automatic examples exclusion, then selecting each crate's implementation sources. Generated production code and native path inclusions count. Integration tests are outside src; dedicated `_tests.rs` files are excluded, and remaining inline test physical lines are conservatively subtracted from covered counts while retaining their denominator. No production exclusion or test-only assertion is used to inflate the result.

| Crate | Conservative covered / executable | Development result | Additional covered lines for 90% at this denominator |
| --- | ---: | ---: | ---: |
| framework-admin | 2392 / 2739 | 87.33% | 74 |
| generated-admin-tool | 219 / 238 | 92.02% | 0 |
| generated-admin-shared | 7251 / 10202 | 71.07% | 1931 |
| generated-admin-web | 9895 / 11969 | 82.67% | 878 |

All measured crates exceed 70%. `coverage/coverage-production.json` contains exact ranges, raw counts, conservative bounds and per-file gaps; `coverage-raw.json` retains counters. Native path-included contracts/stores count in web; both generated frontend roots count in shared. The browser client is WASM-gated and has no native executable implementation lines: this is **not** a 100% or measured WASM line-coverage claim. Browser evidence covers its live mount/transport/disposal paths.

The 90% publication floor is not met overall. Admin needs meaningful additional controller/error/reset/adoption branches (largest gaps: formatted, ordinary and catalog adapters, sync and presentation). Shared needs generated role, page and strict DTO/validation success/error matrices, especially formatted Task and both RoleProof roots, plus native exercise of appropriate controller callbacks. Web needs generated operation/store/public-contract error and capability branches across both roots. Tool already exceeds 90%, but its remaining normalization/selection failures should still be reviewed. WASM executable-line measurement and platform integration scope must be established before a publication claim; browser scenarios alone do not supply a percentage. These are publication needs, not work silently claimed complete or new M5/M6 capabilities.

## Final disposition

The four rounds, released safety exception, polish, consolidated/configurable proofs and development coverage are complete. Original safety-stop evidence and the failing pre-fix regression remain preserved alongside passing final runs. The receipt and scope audit record final cleanup/KB validation below. Stop for coordinator review; no further milestone, commit or deployment was started.

Final cleanup: the sole owned target was deleted (2,822,420 KiB immediately before deletion). Removed 2,212 coverage-created profile files and new proof bytecode caches; all 264 pre-existing profile paths remain. No owned fixture/workspace directories remain. `cleanup.json`, `final-evidence.json` and the final scope audit retain the checks. The final watchdog fixture was hardened to capture the real child PID in the parent, so a timeout during child startup is still verified as kill/reap rather than misreported as a missing handshake; all three tests pass.

KB validation: inherited baseline **146 errors / 25 warnings** (missing upstream references); the exact final diagnostic comparison is unchanged (`kb-closeout-comparison.json`). No unrelated KB repair or context promotion is included.
