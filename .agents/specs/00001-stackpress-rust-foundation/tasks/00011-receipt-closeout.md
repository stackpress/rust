# Task 00011: Slice 6 closeout receipt

2026-10-27. Implementer worktree `trial3-s6-m0`, branch `trial/3-s6-m0`.
The requested `git merge trial/3` fast-forwarded `472ec93` to
`7dbc2527797ad2a4417244d692031f35244a8afd`; updating the parent worktree's Git
metadata required sandbox escalation. The worktree started clean. Ready for
coordinator review, with the confirmed ordering follow-up below still open.
No task/status/decision record was changed; verification remains coordinator-owned.

## Four whole-crate review rounds

Scope: framework-schema; framework-sql with every feature; generated-workspace
tool/app/contracts; data-quickstart. Reviewed all production modules, generator
families and their expanded fixtures, manifests, relevant tests, watchdogs and docs.
Private metrics inventory covers 207 Rust files including tests, plus the quickstart
Python proof/controls. Generated variants were checked by template, fixture differences,
fresh generation and SHA-256; this is not a claim of manually rereading every embedded
serialized descriptor in every expansion.

1. **Complexity/branch risk:** separated the tool's compressed migration-plan branch
   into private `plan_migration`, `plan_schema` and `publish_plan` phases. Preserved
   target/dialect/options/error order, baseline handling, candidate/loss inspection,
   refusal reports and publication authority. Retained the planner's separate review
   and executable-construction passes: review must aggregate later losses even when
   construction stops early. Large record/include traversals retain one budget owner
   and named phases; further extraction did not justify scattering their invariants.
   Baseline eight-package suite: 596 passes; `round1-tool`: 8 passes after extraction.
2. **Responsibility/abstraction:** gave the tool's identical external-error wrapper
   one private owner in its library. Generation, schema preparation and maintenance
   retain their distinct static contexts, categories and concrete causes. The shared
   `storage_definition` already feeds both emitted DDL and revision snapshots; no
   competing storage conversion was added. Domain codecs, migration literals and
   public-input projection retain different contracts and remain separate.
   `round2-tool`: 8 passes. No shared-crate or public API change.
3. **Consistency/Rustdoc/comments:** added seven compiled examples for omitted/null
   input, omitted/null patch, explicit rename/reverse planning, eager loaded state,
   borrowed app reads, pure key serialization and tool history inspection. Documented
   the canonical-wrapper decoder, bounded serializer sink and nested policy cleaner;
   explained metadata derivation and planner review/construction boundaries. Reviewed
   #11 contexts/cause chains throughout handwritten runtime and emitted templates:
   no cause Display interpolation was found. `round3-doc`: 8 doctests pass; final
   tests, Clippy, strict Rustdoc and MSRV checks pass. No generated text changed.
4. **Safety/soundness:** reviewed borrowed executors, locks across await, cancellation,
   cleanup/error precedence, write/commit acknowledgement, preflight, finite traversal,
   exact hash/loss authority, cooperative file publication and native reconstruction.
   No unsafe code or memory/data-safety counterexample was found. The existing native
   lock/cancel/fault/unknown-commit, row/FK/sequence, alias/escape and disclosure proofs
   pass. The real quickstart, complete-file loss refusal and all four proof controls
   pass. The pre-IO ordering gap below is reported without changing behavior.

Only the tool has executable edits. All other Rust edits are documentation/comments;
public declaration lines are unchanged (`public-api-audit.json`). All 110 generated
Rust files are byte-identical; ownership manifests also remain fresh. Revision 2's
serialized contracts/metadata and all shared crates, manifests and lockfile are unchanged.
Data-quickstart required no source edit after its whole-crate review.

## Follow-ups requiring separate scope

**Confirmed pre-IO ordering gap:** `framework-sql/src/codegen/operations.rs` emits
the authorized public `search` handler's `engine.begin()` before `search_public`
checks the dialect-specific query capability. Trusted `search_atomic` performs its
known-capability preflight before BEGIN. The disposable `preflight-probe` uses the
actual generated Project handler, Server and policy with a recording Database that
refuses BEGIN; it opens no database and starts no listener. For both PostgreSQL and
MySQL, `q="Ship"` records **0 BEGIN entries** through trusted `search_atomic`, but
**1 BEGIN entry** through the authorized generated handler; both submit **0 statements**.
The latter reports acquisition failure before it can report Unsupported. Thus a universal
pre-IO refusal claim is unwarranted. Authorization still precedes both paths. No mutation,
unauthorized access or memory-safety failure was demonstrated; this is an acquisition/
error-order defect, not a soundness veto. Probe source, exact command and output are in
`/private/tmp/s6-closeout/preflight-probe/` and `preflight-probe-{result.json,log}`.

Recommendation: release a behavior-changing generator/runtime follow-up to preflight
the public exposure, includes/snapshot capability and selected executor limits before
BEGIN, preserving policy-before-decoding. Review detail's analogous transaction wrapper
in that scope. Do not reuse a trusted-exposure check as public validation. This closeout
leaves emitted outputs unchanged as instructed; it does not claim the gap is fixed.

**Public API proposal, not applied:** make migration `FieldRename` private in a future
approved API revision. Repository references are confined to migrations.rs/steps.rs;
`RenameMap::field` is the checked caller entry, and its stored vector is already private.
Existing downstream users could name the exported DTO, so narrowing it is a public API
change. `MigrationStep`/`steps()` remain deliberate read-only inspection APIs; generated
cross-crate support helpers cannot simply become `pub(crate)`.

## Verification and reproducible commands

Evidence: `/private/tmp/s6-closeout/`; every command has exact argv, exit/duration and
its own log/result JSON. `final-*` labels are the initial checks; `post-*` are after the
last source edit. The normal and instrumented proofs retain complete commands, plans,
SQL/binds, read reports, source hashes and compiled literal README Rust. Cargo is offline
and locked except the disposable probe/README consumer, whose copied lock adds only its
local package. No dependency was installed or downloaded.

| Gate | Result |
| --- | --- |
| post-final-scoped | Eight packages: 164 runtime + 439 doctests = **603 passes**, 0 failures; 16 network-gated and 9 supervised/helper entries ignored. |
| post-final-fmt / clippy / rustdoc / msrv | fmt, all-target Clippy `-D warnings`, `RUSTDOCFLAGS=-D warnings`, explicit Rust 1.96.0 check pass. Quickstart baseline/evolved and app matrix/names Clippy also pass. |
| post-sqlite-default | Unchanged default SQLite suite: 14 runtime + 3 doctests pass; 9 helper entries ignored. Maintenance is in the scoped suite. |
| round4-proof / controls | All six README command blocks, SQL/Rust snippets, absent-output bootstrap, safe reverse, full-file loss refusal, and four failure/timeout/restoration controls pass. |
| generation checks | Original, admin, matrix, names and pure matrix are fresh; all quickstart roots reproduce byte-for-byte in both proofs. |
| feature-commands | 13 normal dependency trees, 9 separate feature checks, 3 real WASM contract builds pass; pure consumers have no driver/Server/CLI/Idea/Tokio normal edges. |
| final coverage cohort | **187 runtime passes**, 0 failures, 25 reported gated/helper entries ignored, plus the actual instrumented README CLI/query binaries. No added skips. |

All build/native groups use an external 600-second parent watchdog with kill/reap;
native Rust parents have their existing independent deadlines. Timeout is failure.
The deliberately stuck control is expected to fail its child proof and confirms reaping.
Feature trees cover schema default/codegen, SQL default/codegen/server/migrations/SQLite,
tool default/database-tools and pure/native consumers. SQL all features compose; app
matrix and names are mutually exclusive and run separately. No listener-capable workspace
suite or network database was run. Buffered HTTP tests use in-process adapters only.

Core commands (the JSON command inventories contain every variant and exact argument):

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_LLVM_COV_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2
export TMPDIR="$PWD/target/tmp" LLVM_PROFILE_FILE="$PWD/target/trial3-s6-m0-%p-%m.profraw"
export PYTHONDONTWRITEBYTECODE=1
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
python3 /private/tmp/s6-closeout/post-checks.py
python3 /private/tmp/s6-closeout/features.py
python3 /private/tmp/s6-closeout/round4.py
python3 /private/tmp/s6-closeout/coverage.py
python3 /private/tmp/s6-closeout/run.py coverage-tool-default cargo llvm-cov --no-report -p generated-workspace-tool --offline --locked
python3 /private/tmp/s6-closeout/run.py coverage-report cargo llvm-cov report --json --output-path /private/tmp/s6-closeout/coverage-default.json -vv
python3 /private/tmp/s6-closeout/export-coverage.py
python3 .agents/scripts/validate-agent-workspace.py
```

## Fresh coverage and the publication floor

Rust/Cargo 1.96.0, aarch64-apple-darwin, LLVM 22.1.2, cargo-llvm-cov 0.8.6. Measurement
starts with cargo clean and llvm-cov clean, then the complete instrumented cohort,
default tool and all valid generated variants. Doctests run separately. Export removes
only cargo-llvm-cov's default filename exclusion, exposing every generated/example source.
No production file/branch is excluded. Separate test-only modules, dependencies and
disposable tooling are outside production; all compiled first-party totals include tests.
Source-include origins attribute the pure consumer's shipped modules to that consumer;
physical paths identify test helpers. Python has four real controls, not Rust coverage.

| Crate / measured scope | Production covered / executable | Percent | All compiled first-party | Percent | More covered lines to reach 90%* |
| --- | ---: | ---: | ---: | ---: | ---: |
| framework-schema, pure/codegen | 3218/3546 | 90.75% | 4087/4415 | 92.57% | 0 |
| framework-sql, all features | 4722/5295 | 89.18% | 7037/7639 | 92.12% | 44 |
| generated-workspace-app, default/matrix/names | 10376/13589 | 76.36% | 13686/16930 | 80.84% | 1855 |
| generated-workspace-contracts, original/pure matrix | 1463/2083 | 70.24% | 1999/2621 | 76.27% | 412 |
| generated-workspace-tool, default/database-tools | 543/652 | 83.28% | 1166/1277 | 91.31% | 44 |
| data-quickstart, pure/native/evolved/expanded | 8891/11861 | 74.96% | 9292/12265 | 75.76% | 1784 |
| shared SQL core, unchanged | 3016/3154 | 95.62% | 6056/6899 | 87.78% | — |
| shared SQLite, maintenance, unchanged | 736/808 | 91.09% | 4982/5098 | 97.72% | — |

*Arithmetic minimum at the current denominator, not a proposal for threshold-only tests.
All six Slice 6 crates clear 70% independently. Raw exports, per-file counts and uncovered
segment sites are retained in `coverage-{all-sources,files,publication-gaps}.json`.
Publication needs meaningful cases covering these observed gaps:

- **Schema:** already clears 90 numerically; remaining metadata applicability/provider,
  nested record/rule failures, direct-config accounting and codec/enum emission branches
  still need release-risk assessment. Largest gaps: metadata 51 lines, record 42.
- **SQL:** host assembly and public issue-path filtering, host/input failure mapping
  (operations.rs has 102 uncovered lines); DDL conversion errors, migration DTO/action
  decoding and catalog/ledger defensive paths. Resolve the demonstrated ordering defect.
- **App:** richer public input validation/preparation for the type matrix, malformed
  nested/provider cases, and prior-completion/batch/error branches across reserved-name
  and matrix operations. Largest gaps: matrix public contracts 423, names operations 414.
- **Contracts:** pure nested default/preparation/null/provider and browser/query-validation
  cases, especially matrix.rs (359 uncovered) and embedded/public inputs. Preserve pure/WASM
  isolation while adding cases that assert business outcomes.
- **Tool:** table renames, deliberate drops and backfill CLI decoding, invalid command/plan
  purpose, seed-file bounds and filesystem/worker failure handling. Gaps: maintenance 54,
  generator library 36, operator schema 19; some OS/runtime failures need controlled seams.
- **Quickstart:** public validation/client preparation and patch/query error cases across
  the three schemas, plus operation prior-completion/batch/projection failures. V3 public
  contracts alone have 389 uncovered lines; SQL observation poison/overflow/close failures
  also remain unproved. Repeating successful getters alone would not establish these paths.

Publication additionally needs its declared release feature/target/integration scope,
the separately released docs work and critical branch review. These line totals certify
neither every branch/platform nor publication readiness. No 90% completion is claimed.

## MySQL string-key width: landing analysis only

The three [SQL closeout options](00012-receipt-sql-closeout.md) remain an owner decision.
For explicit storage length, the declaration belongs to app-owned build-host storage
policy keyed by source model/field, interpreted by schema codegen/registry normalization;
it does not belong to UI role props or an inferred 191 default. A host-only resolved
storage-intent sidecar could preserve frozen revision 2. Adding a serialized `Field`
member requires a separately approved contract/version change. Syntax, scalar/enum
eligibility, portable versus backend-specific scope and character/byte semantics need
freezing first. Deriving max instead would consume canonical scalar-string `cle` rules
and make validation edits DDL edits; named profiles would resolve through the compiled
app catalog with explicit precedence and a pinned profile revision. None is implemented.

Both framework-sql `storage_definition` and the Rust `table` emitter must consume the
same resolved intent; the emitter currently maps its supported families back to TEXT.
Core already supports `SqlType::Varchar`/`Length::Size(i32)`. Revision `FieldDto` already
captures varchar/length, so resolved widths flow through snapshot hashes, source
provenance, diff, findings and plan approval. The planner currently blocks length/type
conversions: narrowing/widening application needs separately reviewed policy and value/
recovery proofs, not an approval bypass. Keep the strict text codec; validate width,
aggregate index byte budgets and matching FK width/charset/collation before IO. SQLite
VARCHAR length is not native enforcement. Native generated DDL/DML/relation evidence
would belong in task 00012's harness after the decision. Existing [network-store evidence](00012-receipt-sql-store-integration.md)
is referenced, not rerun; network migrations remain render-only and MySQL application refused.

## Resources and handoff

One owned worktree target: sampled peak 2,731,605,492 bytes (2.54 GiB), capped at 8 GiB
by the later verification runner; earlier normal checks sampled below that bound.
All evidence was exported before deleting the final 2,238,625,576-byte target. It is
absent; build/proof commands have completed. The 136 inherited ignored profiles outside
the target are preserved. No installs, listeners, network DBs, shared-crate edits,
other-trial/sibling-Idea access, coordinator messaging, commits, pushes or GitHub writes.
Final scope/whitespace/index/frozen-source audits pass: 12 Rust files plus this receipt,
195 other Rust files unchanged, index empty and target absent (`scope-audit.json`).
KB validation exits 1: the same 146 inherited errors and 24 inherited warnings, plus
one preferred-length warning for this coherent receipt (below the 500-line hard cap).
No new hard/link error; keeping review findings and reproduction evidence together is intentional.
Stop for coordinator review; the pre-IO follow-up and MySQL product decision remain open.
