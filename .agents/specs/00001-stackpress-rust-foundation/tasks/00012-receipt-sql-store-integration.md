# Task 00012: generated-store integration receipt

2026-10-27, branch `trial/3-s7-sql-m0`, uncommitted work on HEAD `44aa10fc6490ce37d7ba72ee1d4e976a79173e1a`. The requested
`git merge trial/3` fast-forwarded from `44988ae`. This completes the released integration work for [task
00012](00012-slice-7-production-servers.md), pending independent coordinator review. Read the [Slice 6 M2
receipt](00011-receipt-m2.md), including stores, exact core additions and limits. This replaces the earlier stopped G23
checkpoint while preserving its evidence below.

## Implementation and exact authority

The coordinator granted the shared-code fix under G23 and explicitly allowed test-owned MySQL DDL. The MySQL allowlist and
PostgreSQL domain wire-type rule are also coordinator decisions. There is no new product capability or owner-decision
attribution here.

- `codegen/queries.rs::emit`: emitted search_atomic performs the existing pure trusted/active
  query/path/budget/snapshot checks before BEGIN. `codegen/writes.rs::emit`: emitted batch_atomic
  calls `DataSession::pooled(engine).require_returning()?` before BEGIN or input/provider preparation.
- Minimum runtime factoring: `stores/relations.rs::prepare_search` (new),
  `preflight_atomic_search` (new hidden codegen bridge), `search` (uses the same preparation),
  and private `SearchContext` (new). `stores.rs` only reexports the hidden bridge. Exact validation
  order and safe error contexts are preserved, including invalid include paths before snapshot refusal.
- Regenerated 14 store leaves through the existing tools: app default/matrix/names and admin stores.
  Four ownership manifests update only those leaves' hashes/bytes. Contracts, metadata, UI,
  schema fingerprints and existing API signatures remain unchanged; schema revision 2 and SQL
  runtime revision 1 remain unchanged. All generated output is byte-identical on repeat generation.
- Shared adapter test support `stores/` executes the actual emitted Profile/Project/Task methods
  through Engine/Database and the exact exclusive TransactionSession. MySQL path-includes the same
  physical suite; SQLite is the offline reference. Observation forwards operations unchanged and
  counts begin/run/commit/rollback without retaining statements, binds or driver handles.
- MySQL `OutputBudget::cell` now uses SQLx's public string compatibility check for admitted
  text/binary families: utf8mb4_bin text sets BINARY but has a text wire charset. Display-name-only
  classification returned Blob. Actual binary remains Blob, unsupported ENUM/SET remain rejected,
  NULL/budgets/UTF-8 checks remain strict. A native regression failed before and passes after.
- Both test-only M2 `report_error` functions walk at most eight causes and print only safe
  DataError/SqlError contexts and native structured codes/categories. Native messages/SQL/URLs
  and credentials are never formatted. Root Cargo.lock changes only six existing dev-dependency
  edges; manifests change only the two adapters' dev dependencies. No new package/version/fork.

SQL core, SQLite, framework-schema, migrations, root Cargo.toml/facade and unrelated sources are untouched. Documentation
uses the repository chrisai-docing skill. Testing guides, backend switch notes and MySQL codec docs distinguish actual proof
from retained refusals.

## Shared sequences and exact refusals

Each backend seeds 22 profiles (21 active), 40 projects and 80 tasks in one explicit transaction. PostgreSQL and SQLite
execute generated DDL unchanged. MySQL first proves all three generated TEXT-key plans return SqlError Unsupported without
executor entry, then installs clearly labelled harness-owned VARCHAR(191) key/index widths and InnoDB tables. This is no
generated width policy. MySQL seeding uses generated input preparation/encoding and harness inserts without RETURNING; it
does not rewrite an executing generated write or claim successful generated MySQL CRUD.

All three execute get/maintenance, nested JSON/enum/list/bool codecs, deterministic tied paging, scalar/list filters,
filtered counts independent of offset/take, find/find_all, selected rows, and supported base-only atomic reads.
PostgreSQL/SQLite additionally execute RETURNING CRUD, missing-row outcomes, FK refusal with conservative UnknownOutcome,
conflict insert/no-change/ update branches, targeted primary upsert and a core non-primary unique target, ordered batches,
whole-batch validation, rollback after a later native failure, cross-store commit/rollback and idempotent soft
delete/restore. No automatic retries or certainty after an uncertain write.

| Refusal | Error type/kind | Operation-boundary observation |
| --- | --- | --- |
| MySQL generated TEXT primary/index keys | SqlError Unsupported | Pure plan render; zero executor entries |
| MySQL generated create/create_with/create_if_absent, update/update_key, upsert/upsert_key/upsert_on, delete; Profile remove/restore | DataError Unsupported | All model methods tested; no run/BEGIN/completion |
| MySQL borrowed batch / batch_atomic, all three models | DataError Unsupported | No run; atomic also no BEGIN/completion |
| MySQL core DoNothing/DoUpdate target, deliberately without RETURNING | SqlError Unsupported | Zero entries, independently of RETURNING gate |
| Network nonempty literal q | DataError Unsupported | Zero entries, including atomic BEGIN |
| Network includes or exact page total | DataError Unsupported | Zero entries, including atomic BEGIN; borrowed caller BEGIN lies outside the measured operation |
| Invalid includes plus total | DataError Validation | Original validation precedence, zero entries |

The zero-entry sentinel matrix uses both working and failing BEGIN implementations, both network dialects and every model: 36
q/total/include refusals, 12 invalid-path cases, six MySQL atomic batch refusals. Every case asserts
begins/runs/commits/rollbacks all 0. Real native owners repeat pre-effect checks, and subsequent reads confirm
unchanged/absent rows after refused MySQL writes. SQLite still proves actual literal q (%/_ literal, case-sensitive),
loaded/unloaded/loaded-empty relations, forward/inverse paths and nested BFS. One and twenty parents both take four
statements; Engine binds=2 chunks the same graph into 32 statements and Transaction/DataSession retain that limit. The
three-model fixture has no datetime ordering field; its existing network Unsupported gate is retained, with generated
temporal execution still SQLite-only. No network snapshot/read gate widened.

## Four test-verified review rounds

1. Logic/complexity: checked preflight error ordering, planned versus borrowed transactions,
   empty/missing/hidden states, whole-batch validation and no-effect assertions. Kept the pure
   preparation shared; no further production edit justified after the native codec fix.
   `round1-logic.log`: shared SQLite suite plus existing atomic/reads/outcomes pass.
2. Responsibility/abstraction: split MySQL mutation refusals into `writes/mysql.rs`; successful
   native write sequences stay in `writes.rs`, setup/reads/observation/preflight have separate owners.
   No support crate or core changes. `round2-abstraction.log`: adapter M2 and framework suites pass.
3. Consistency/Rustdoc/comments: preserved signatures, clarified atomic-read Rustdoc, accurate MySQL
   refusal reporting, codec charset explanation and generated-store limits. Default/matrix/names
   tests, Clippy -D warnings, fmt, strict Rustdoc and doctests pass after the edits.
4. Safety/soundness: checked exact-session forwarding, consuming completion, cancellation/close,
   bounded diagnostics, retained payload charging, charset versus binary metadata, native fixture
   authority and deadlines. No unsafe code or public instrumentation. No further edit justified;
   fresh instrumented M0/M1/M2 replay verifies final source and all cleanup paths.

## Verification, resource bounds and coverage

Rust/Cargo +1.96.0, aarch64-apple-darwin, local PostgreSQL 12.2/MySQL 8.0.27, explicit plaintext. All repository checks after
the dev-edge lock update use --offline --locked. Evidence lives at `/private/tmp/s7-sql-store-integration/run/`: exact
commands, exits and durations in each JSON; `checks.py`, `coverage.py` and `export-coverage.py` reproduce the matrix. No tool
installed.

| Check | Result / evidence |
| --- | --- |
| fmt / scoped all-target Clippy -D warnings | Pass; fmt-verified, clippy-final, matrix-clippy, names-clippy |
| Offline scoped tests including SQLite reference | 42 pass / 14 explicit ignored network cases; round3-offline |
| Separately selected generated matrix/names suites | 7 / 2 pass; matrix-test, names-test |
| Native M0/M1/M2, both gates set | Pass; native-baseline and final round4-native-coverage |
| Strict Rustdoc and doctests with gates unset/set | Pass; rustdoc-final, round3-offline, native-doctests |
| Explicit +1.96.0 facade check with both drivers | Pass; check-facade |
| Workspace --locked --all-targets / --doc | 675 pass / 78 expected ignored; 895 doctests pass; workspace-final/workspace-doc |
| Repeat all four generators / manifest audit | Pass; generation-idempotence.json, scope-audit.json |
| Admin inactive stores compiled in disposable consumer | Pass; admin-store-compile; no SQL added to the admin browser module graph |
| Normal dependency trees | Default/browser/admin-browser no SQLx/driver; each selected adapter has only its own driver; tree-*.log |

The admin shared crate deliberately does not include stores in its module graph; regenerated inactive leaves are
compile-checked by an external consumer. Its compiled UI/contracts are unchanged and retain workspace verification. Runtime
coverage below covers every changed executable crate, including all shipped/executed app default/matrix/names source; no
generated model is excluded. Framework-SQL scope is server/codegen (migrations are unchanged and outside this released edit).
Coverage uses installed cargo-llvm-cov 0.8.6 and LLVM 22.1.2, a fresh cargo/coverage clean, --no-report default/native plus
separate matrix/names runs, then a raw JSON export removing only the tool's default example exclusion. Exact production src
paths are aggregated; actual test modules, probe/m1/m2 executables and dependencies are excluded. Child profiles are
included. Doctests and compile-only borrowing checks pass separately and are not counted as runtime coverage.

| Executable production scope | Covered / lines | Percent |
| --- | ---: | ---: |
| stackpress-sql-postgres | 777 / 839 | 92.61% |
| stackpress-sql-mysql | 914 / 980 | 93.27% |
| stackpress-framework-sql (server/codegen) | 2229 / 2562 | 87.00% |
| generated-workspace-app (default/matrix/names) | 10420 / 13589 | 76.68% |

All four exceed the 70% development floor. Raw/report/file data: coverage-all-sources.json, coverage-export-command.json,
coverage-files.json and coverage-summary.json. Three profiles emitted by the final Rustdoc process were moved into the sole
target and merged; the final export includes them and all native child profiles. No coverage artifact remains outside target.

 Native gates are exactly STACKPRESS_TEST_POSTGRES_URL and STACKPRESS_TEST_MYSQL_URL. Unset gates report ignored; the SQLite
suite actually runs. Every runner CREATEs a unique bounded ASCII name without IF NOT EXISTS, arms cleanup only after
acknowledgement, closes owned connections and DROPs only that exact name, including failure/unwind. Scenario watchdogs
kill/reap after 30 s; total fixture budget is 120 s, administration 9 s IO/10 s wall, SQLite cleanup 10 s. No listing,
adoption, prefix sweep, unrelated-session termination, role/global mutation or FORCE drop. SIGKILL/power loss cannot
guarantee remote cleanup; this is not claimed.

Final native contract child times: MySQL 225 ms / PostgreSQL 514 ms. Clean pool-size 1/2/4 stress completed 5709/9085/12963
requests (MySQL) and 3544/5954/8429 (PostgreSQL), about 2 s per phase. Native peaks matched 1/2/4, all clean acquisitions
returned with zero retirements; permits restored, cleanup drained. Deliberate acquire/reuse/retire counts remained MySQL
78/24/54 and PostgreSQL 78/48/30. Overflow was 3.25/3.875 us; acquire timeouts 202/203 ms, distinct from Closed; 64 waiters
cancelled per control. Final native/idle/cleanup counts were zero. resource-observations.json and round4-native-coverage.log
retain exact counts, durations and native identity sets. These are logical observations, not a hard heap/socket/load-capacity
guarantee.

One target directory, CARGO_INCREMENTAL=0/CARGO_PROFILE_DEV_DEBUG=0, two build jobs. It was observed at 4.6 GB before the
fresh coverage clean and 1.1 GB afterwards, below the approximately 10 GB cap.

## Exact native database ledger

All 17 names below have one acknowledged exact-name DROP, including the failed runs. No catalog scan was used to confirm
absence; successful DROP is the confirmation. database-ledger.json retains machine-readable pairings. Every database name
created by this integration stage is listed.

| Backend | Database | Log (DROP confirmed) |
| --- | --- | --- |
| mysql | `stackpress_s7_m1_10ff2_18e255f50c845f88` | binary-collation-before.log / yes |
| mysql | `stackpress_s7_m2_104f0_18e255e4984a95c0` | mysql-diagnostic.log / yes |
| mysql | `stackpress_s7_m0_12c14_18e255fb3a0637c8` | native-baseline.log / yes |
| mysql | `stackpress_s7_m1_12c20_18e255fb858d8f70` | native-baseline.log / yes |
| mysql | `stackpress_s7_m2_12c28_18e255fbd5437d68` | native-baseline.log / yes |
| postgres | `stackpress_s7_m0_12ced_18e255fd827901f0` | native-baseline.log / yes |
| postgres | `stackpress_s7_m1_12d06_18e255fe2cf49ba8` | native-baseline.log / yes |
| postgres | `stackpress_s7_m2_12d6a_18e255fec2777ad8` | native-baseline.log / yes |
| mysql | `stackpress_s7_m2_fc14_18e255da54a114e8` | native-diagnostic.log / yes |
| postgres | `stackpress_s7_m2_fc93_18e255dc1fde6b78` | native-diagnostic.log / yes |
| mysql | `stackpress_s7_m2_e29b_18e255cf82056a98` | native-first.log / yes |
| mysql | `stackpress_s7_m0_1668_18e256bbed96ae38` | round4-native-coverage.log / yes |
| mysql | `stackpress_s7_m1_1672_18e256bc4203e2b0` | round4-native-coverage.log / yes |
| mysql | `stackpress_s7_m2_1693_18e256bc95c2f8f0` | round4-native-coverage.log / yes |
| postgres | `stackpress_s7_m0_17f1_18e256be7bbf47b8` | round4-native-coverage.log / yes |
| postgres | `stackpress_s7_m1_18bb_18e256bedbc140a8` | round4-native-coverage.log / yes |
| postgres | `stackpress_s7_m2_19b7_18e256bf7a266e80` | round4-native-coverage.log / yes |

All eight parent-owned SQLite files and exact -wal/-shm sidecars were removed and checked absent; full paths/log pairings are
in sqlite-ledger.json. Exact basenames follow:

- `stackpress_s7_m2_sqlite_a20a_18e255a97cbb8200.sqlite` (first-offline.log; removed).
- `stackpress_s7_m2_sqlite_12d6a_18e255feb37d6f38.sqlite` (native-baseline.log; removed).
- `stackpress_s7_m2_sqlite_fc93_18e255dc101a87f8.sqlite` (native-diagnostic.log; removed).
- `stackpress_s7_m2_sqlite_13b34_18e256102a8ddf48.sqlite` (round1-logic.log; removed).
- `stackpress_s7_m2_sqlite_17d6a_18e2561f3874a470.sqlite` (round2-abstraction.log; removed).
- `stackpress_s7_m2_sqlite_4a56_18e2563d8307c8f0.sqlite` (round3-offline.log; removed).
- `stackpress_s7_m2_sqlite_19b7_18e256bf6c6af590.sqlite` (round4-native-coverage.log; removed).
- `stackpress_s7_m2_sqlite_11632_18e2569170330448.sqlite` (workspace-final.log; removed).


## Historical G23 checkpoint and limitations

The original disposable consumer used actual emitted functions with an observation-only Database whose BEGIN deliberately
failed. Nine atomic cases (Profile q/total/includes on both network backends plus each MySQL model batch) returned DataError
Sql with begin=1/run=0, masking known Unsupported. Four controls (borrowed q on both, borrowed MySQL create, core conflict
without RETURNING) had Unsupported and begin/run=0. All three MySQL generated table plans were also refused. No native
database/SQLite file was opened then. The stopped checkpoint alone changed the worktree; its target peaked at 329 MiB and was
removed (1074 files / 326.0 MiB).

Preserved evidence one level above run/: probe/src/main.rs, its Cargo.toml/private lock, probe-build.log,
preflight-probe.log, preflight-replay.log, preflight-result.json and replay.py. The original 10-second kill/reap replay
captured exit 101 in 0.0217 s; that red-only replay script is historical and intentionally expects failure. The same consumer
now exits 0 with all nine atomic cases Unsupported and zero entries (`run/preflight-after.log`). Durable tests additionally
count all four executor methods and cover every model and both BEGIN modes. Shared grant functions and regenerated outputs
above implement the exact requested proposal, without enabling reads.

The initial native integration exposed the binary-collation decoder defect; native-first, native-diagnostic and
mysql-diagnostic retain safe findings, and binary-collation-before proves Blob versus expected Text. All failed-run databases
were dropped. Final native/coverage replay passes with strict values. TLS/auth variants, external poolers, hidden
session-mutating MySQL triggers, other versions/targets, production load capacity and network snapshot/q/datetime support
retain their prior limitations. Line coverage is not branch proof or publication's 90% acceptance.

The sole target directory was deleted after the final export; final-artifact-audit.json records absence of target/stray
profiles and final source hashes. KB validation retains 146 existing errors and 22 existing warnings, plus the permitted
201–500-line warning for this coherent receipt. Other sibling-source links and length/link-text findings are unchanged and
outside this task. No listeners, installs, commit, push or publication. Stop for independent coordinator review; this receipt
does not claim task or slice acceptance.
