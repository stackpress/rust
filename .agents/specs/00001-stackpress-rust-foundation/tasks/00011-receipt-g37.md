# Task 00011: G37 / G38 implementation receipt

2026-10-28. **Implemented; awaiting coordinator review.** No commit, push, install or listener.
Authority: [G38 / G23](../decisions.md) and the [approved design](00011-g37-design.md).
`git merge trial/3` fast-forwarded first to `74319ac`, then to admin closeout record **8b29f84**
(including merge **114d76a**). The approved design remains byte-identical; its proposal header
is historical. G38 explicitly supersedes G34's revision-2 freeze.

The coordinator's 2026-10-28 G23 extension additionally grants exactly
`crates/sql/sql-mysql/src/tests/m2.rs` and new `crates/sql/sql-mysql/src/tests/g37.rs`:
opt-in `--g37-ddl`, unchanged generated DDL, own-database metadata and existing G26 ownership.
No adapter production or manifest edit. SQL-core edits stay inside the original grant:
`src/ddl.rs`, `src/network_ddl.rs`, `src/diff.rs` and tests; no SQL-core lib/parser edit.
Framework-admin and its generated roots were touched only after the closeout release/merge.

## Result and public surface

- Schema contract **3**, `Field.storage_length: Option<u32>`, exact `@store.length(n)` on
  stored scalar String/Text model fields only. Range `1..=i32::MAX`; flags, floats, strings,
  null, multiple arguments and ineligible fields refuse. JSON/range checks, diagnostics,
  fingerprints and canonical `cle(n)` agree. Stricter assertions remain; incompatible static
  counts refuse. Equal explicit rules retain their message. No inference from `@max`.
- One resolved storage catalog feeds emitted builders and revisions. VARCHAR/length survives
  conversion; PostgreSQL/SQLite render VARCHAR(n). SQLite length is declaration-only;
  generated preparation/default/provider, patch, batch, trusted encode and decode guards enforce
  it. Private host guards do not publish private storage intent/rule arguments. Existing bounded
  validation precedes new trusted-encode scans; policy-before-decoding remains unchanged.
- Explicit MySQL InnoDB DYNAMIC/utf8mb4/collation/page expectations, checked whole compound-key
  bytes, FK-support keys, row/prefix/null overhead and secondary-record PK overhead. Full catalog
  FK family/width/charset/collation agreement; missing widths are never copied. No prefix indexes,
  trimming or default 191. Plain unselected VARCHAR and unannotated TEXT behavior are preserved.
- New revision/plan artifact **3**, frozen explicit format-2 readers/serializers preserving old
  bytes and hashes; namespaced ledger stays **2**, SQL runtime contract/StorageColumn stays **1**.
  Widen/removal is reviewed; narrowing requires exact hash/target approval and complete locked
  value checks. Reverse performs fresh checks and refuses later longer values. No truncation,
  casts, automatic recovery or network migration application.
- SQLite scans eight-row stable batches under the maintenance writer lock after ledger/drift
  checks, before reconstruction: aggregate 10,000 scanned values / 16 MiB text / five seconds.
  Incomplete scans refuse. Exact values, keys, FK groups, row counts and deleted AUTOINCREMENT
  high-water survive forward/reverse. Plain INTEGER PRIMARY KEY retains max(rowid)+1 reuse.
- Admin pins UI contract **3**, refuses 2 and future mismatches. Its permanent compile proof
  verifies E0080 for a mixed revision-2 metadata partition, restoration, stale renamed consumers,
  hostile JSON/labels, empty/omitted models and formatted generation. All nine tracked generated
  roots are regenerated and fresh, including workspace default/matrix/names, pure contracts,
  quickstart v1/v2/v3 and admin default/format. Operations are generated through the same registry.

Public additions: schema field/revision; Core Create target-selection methods and opaque policy
inspection on CreateData/TableDefinition; `ddl_policy::mysql_definitions`,
`codegen::mysql_storage_definitions`; generated `stores::schema::mysql_definition`; paired
app-owned `emit revision --mysql-collation NAME --mysql-page-bytes N` options. Both options
validate before source/history IO. No new parser, stock binary or connection escape hatch.
See the [storage-length reference](../../../../crates/framework/framework-sql/docs/api/storage-length.md).

## Proof and limits

The [permanent generated fixture](../../../../crates/framework/framework-sql/tests/proofs/lengths/README.md)
uses the existing app-owned tool, absent-output generation, full freshness and compiled actual
contracts/stores/operations on SQLite. ASCII, emoji, combining marks, NUL and trailing spaces;
default/provider outputs; private guards independent of a permissive host validator; patches,
whole-batch preflight, three related models, includes, search and paging all pass. Raw SQLite
accepts overlong VARCHAR; generated decode refuses it. All-dialect DDL snapshots are checked fresh.
Resize tests cover widening, bound removal, approved narrowing refusal, later-value reverse
refusal, typed-precondition forgery, FK groups, sequence history, writer locks, row/byte ceilings,
faults, cancellation, scan-deadline refusal and unknown commit reconciliation under watchdogs.
The frozen v2 fixtures retain original revision/plan/reverse bytes and hashes.

The native MySQL extension executes the fixture's **three generated CREATE statements verbatim**.
It checks every string width/charset/collation, InnoDB/DYNAMIC, complete indexes (no SUB_PART),
FK endpoints and exact bound text joins. Missing-parent insertion refuses. A separate trusted
boundary fixture accepts VARCHAR(768)'s 3,072-byte key and verifies native error **1071** at 769.
The unchanged MySQL M0–M2 regressions pass, retaining generated TEXT-key refusal and the existing
harness-owned VARCHAR(191) integration. No generated MySQL CRUD or migration-apply claim is made.

| Owned MySQL database | Exact-name DROP acknowledged |
| --- | --- |
| `stackpress_s7_m0_11d5_18e2717acdef94d0` | yes |
| `stackpress_s7_m1_11d7_18e2717b1fe59b40` | yes |
| `stackpress_s7_m2_11f7_18e2717b81def760` | yes |
| `stackpress_s7_m2_120f_18e2717d45ec6650` (annotated DDL) | yes |

MySQL target facts are operator expectations, not global-setting discovery. Only admitted
all-inline Integer/Real/Boolean/DateTime/bounded-VARCHAR layouts are planned; unknown/off-page
TEXT/JSON/Blob/Decimal/trusted layouts refuse under this optional policy. Native evidence is the
owner's local server only; PostgreSQL remains render-only. Configured facts must be pinned at
baseline: introducing/changing them across a diff refuses, including for another dialect. No network migration apply, MySQL
RETURNING write, arbitrary server version, installed target or publication readiness is claimed.

## Commands and gates

Evidence root: **`/private/tmp/s6-g37`**. Exact argv/exit/duration/peak bytes are retained in
`*-result.json`, logs and `admin/<suite>/*-result.json`. Every build/native command has a parent
kill/reap watchdog and an 8 GiB physical-target cap. `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, offline/locked Cargo; one capped worktree target tree, TMPDIR beneath it. Existing bootstrap/trybuild subtargets
also stay beneath that tree.
Both database URLs stay unset except the explicit MySQL G26 runs. No PostgreSQL test/application.
`admin-run.py` invokes the unified proof driver, sets ADMIN_PROOF_ROOT, selects the installed
Playwright, aliases its target to the sole physical target and recreates TMPDIR after cache cleans.

```sh
python3 /private/tmp/s6-g37/integrate.py
python3 /private/tmp/s6-g37/full-cov.py
python3 /private/tmp/s6-g37/run.py native-mysql-coverage python3 /private/tmp/s6-g37/mysql-coverage.py
python3 /private/tmp/s6-g37/coverage-admin-final.py
python3 /private/tmp/s6-g37/coverage-quickstart.py
python3 /private/tmp/s6-g37/final-export.py
python3 /private/tmp/s6-g37/normal-gates.py
python3 /private/tmp/s6-g37/browser-gates.py
python3 /private/tmp/s6-g37/last-fresh.py
python3 /private/tmp/s6-g37/check-databases.py
```

The underlying native extension is `cargo test -p stackpress-sql-mysql --test m2 --offline
--locked -- --g37-ddl` with the explicit existing MySQL gate. Off by default; unset URL reports
ignored groups. The native measurement additionally runs unchanged MySQL M0–M2 cases.
`full-cov.py` records exact package/feature lists; separate matrix/names, admin default/SSR/format,
quickstart native/evolved/expanded and pure defaults are measured. The disposable generated proof
uses `cargo llvm-cov show-env --sh` with ordinary Cargo; its owned executable is explicitly
included in export, as are all quickstart feature-selected binaries.

- Final workspace: **728 tests / 940 doctests**, no failures; 81 existing/helper/native ignores.
  `tcp_watchdog` alone is explicitly skipped because this lane starts no listeners.
- Full-feature scoped four-package suite: **141 tests / 437 doctests**, no failures; 16 helper ignores.
  Matrix **7**, names **3**, admin default **48** and format **52** instrumented tests pass.
- SQLite unchanged default **14 + 3 doctests**, maintenance **16 + 3**, nine helper ignores each.
- Workspace/scoped/format all-target Clippy `-D warnings`, fmt, MSRV and Rustdoc pass.
  Four admin WASM Clippy/normal-edge graphs and pure-contracts WASM compile pass.
- Unified admin compile proof, all README quickstart commands/SQL/literal Rust, three quickstart
  native feature suites, generated guard proof and Python watchdog controls (3 admin / 4 quickstart) pass.
- Chromium roles **24**, format **16**, CRUD **8**, transfer **8**, races **32**: all **88** top-level
  combinations and 16 formatted guard subcases pass; representative narrow/wide screenshots inspected.
- Final nine-root `--check` results have empty differences; matrix/names and all three quickstart
  native feature all-target Clippy checks pass. No generated output was hand-merged.
- Feature trees: defaults/codegen/server/migrations/sqlite-maintenance and pure contracts retained in
  `trees-final.json`; pure/default/codegen/migration planning have no driver/server edges.

Exploratory failures remain in evidence: one stale contract-2 matrix assertion, one test-only
forgery assumption for bound removal, and an incorrectly nested coverage-wrapper command/cache.
The first aggregate admin gate also failed its 600-second outer budget during WASM compilation
after completed checks; retained as failure, then rerun with a 1,800-second aggregate budget
(the unified driver retains its per-command watchdogs). All were corrected and affected gates rerun. The first coverage export exposed
missing quickstart native feature tests; those existing meaningful suites were then run for all
three revisions. No test or production branch was removed to improve coverage.

## Four distinct review rounds

1. Complexity: normalization/rules, one DDL conversion, codec dispatch and resize derivation/
   scan accounting. Frozen v2 wire duplication is intentional; no additional abstraction needed.
2. Responsibility: pure target declarations vs adapters, private/public projection, migration
   authority vs execution and G26 ownership. No inference/discovery or production adapter change.
3. Consistency/Rustdoc/comments: current revision docs/assertions, static error contexts and
   typed causes, explicit limits, API examples and generated comments. Corrected stale revision
   references; no cause Display interpolation. Public additions are listed above.
4. Safety/soundness: checked numeric conversions/byte sums, whole keys/FK groups, scan identity/
   completeness/deadline, pre-IO refusals, exact cleanup and cancellation/unknown outcomes.
   Retained bounded validation before trusted scans; no unresolved safety bug found.

`reviews.json` maps each round to its scoped tests; final fmt/Clippy/Rustdoc/workspace/browser
checks verify the integrated result. No ceremonial refactor or silently widened capability.

## Per-crate coverage and publication work

Rust/Cargo **1.96.0**, aarch64-apple-darwin, LLVM **22.1.2**, cargo-llvm-cov **0.8.6**.
The measurement began with a clean target/profiles. `coverage-all-sources.json` removes only
cargo-llvm-cov's automatic examples exclusion. No production file is excluded; generated code
and native source inclusions count. `coverage-final.json` records source origins, raw counters
and conservative production bounds (inline-test physical lines subtracted only from covered
numerators, denominators retained). The all-first-party column also retains compiled test/helper
files. Dependencies/disposable proof tooling are separate. Doctests, branches and WASM lines
are not measured; browser execution and WASM compilation are separate evidence.

| Crate | Conservative production | All first-party, including tests | Additional lines for 90%, production / all |
| --- | ---: | ---: | ---: |
| framework-schema | 3328/3620 = 91.93% | 4364/4656 = 93.73% | 0 / 0 |
| framework-sql | 5435/5979 = 90.90% | 8428/9164 = 91.97% | 0 / 0 |
| sql | 3195/3351 = 95.34% | 6365/7226 = 88.08% | 0 / 139 |
| sql-sqlite | 736/808 = 91.09% | 4982/5098 = 97.72% | 0 / 0 |
| tool | 580/687 = 84.43% | 1029/1138 = 90.42% | 39 / 0 |
| framework-admin | 2401/2739 = 87.66% | 3803/4064 = 93.58% | 65 / 0 |
| mysql | 928/999 = 92.89% | 4417/5136 = 86.00% | 0 / 206 |
| generated-app | 10702/13887 = 77.06% | 14282/17516 = 81.54% | 1797 / 1483 |
| contracts | 1463/2083 = 70.24% | 1999/2621 = 76.27% | 412 / 360 |
| quickstart | 9191/12159 = 75.59% | 9592/12563 = 76.35% | 1753 / 1715 |
| admin-tool | 219/238 = 92.02% | 319/338 = 94.38% | 0 / 0 |
| admin-shared | 7251/10202 = 71.07% | 7961/10916 = 72.93% | 1931 / 1864 |
| admin-web | 10135/12205 = 83.04% | 11431/13505 = 84.64% | 850 / 724 |

Every measured crate clears 70%; **90% publication is not achieved overall**. Source/line gaps
are retained in `coverage-files.json`. Tool needs option/history failure branches; contracts need
strict DTO/query/validation error matrices; generated app/quickstart need operation/store success,
refusal and codec edge matrices across all revisions. Admin needs controller/error/reset/adoption
branches; shared needs generated role/page/DTO matrices; web needs operation/store/public-contract
error/capability branches across both roots. Crates already above 90% still need branch/platform
review; WASM executable-line scope must be established before publication.

## Cleanup and disposition

The owned target is deleted (peak **7,294,202,328 bytes**, 6.79 GiB, below 8 GiB); both server namespace listings are empty and all four owned MySQL databases are absent.
No new ignored file remains outside the target; inherited artifacts are preserved. KB diagnostics stay at inherited **146 errors / 25 warnings** (missing upstream references).
`cleanup.json`, `scope-final.json`, `leftover-databases.json` and exact command evidence are retained.
Stop for coordinator review. No next milestone, commit or publishing action started.
