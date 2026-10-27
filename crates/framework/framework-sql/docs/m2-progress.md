# Historical M2 progress at executor-budget shared request

This note preserves the earlier shared-request stop. The accessor request was subsequently
approved and implemented, and CRUD/relations/operations are now implemented. The M2 receipt
supersedes this historical status; use the crate README for the current callable surface.

M2 is **incomplete**. This is a resumable progress note, not the pre-approved final M2
receipt. Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0`, branch
`trial/3-s6-m0`. M2 began by merging `trial/3`, fast-forwarding to
`cce3a314d64f95f962238b0984b636dd3581911c`. All source changes remain uncommitted.
No listener, network database, install, commit, push or publication has occurred.
No Slice 8 files or frozen revision-2 descriptor/DTO/query/partition contracts were changed.

## Implemented partial M2

The coordinator accepted [the targeted conflict proposal](m2-upsert-proposal.md) and granted
exactly four shared paths. Those are the only SQL-core paths changed:

- `crates/sql/sql/src/dml.rs`: additive `InsertConflict::{DoNothing, DoUpdate}`,
  private optional Insert clause, `on_conflict` and `conflict`, validation and native rendering.
- `crates/sql/sql/src/lib.rs`: re-export only (rustfmt wraps the existing export list).
- `crates/sql/sql/tests/dml.rs`: three new rendering/inspection/limits tests.
- `crates/sql/sql/docs/api/insert.md`: API, dialect and empty RETURNING outcome documentation.

Targets and assignments are nonempty, bounded, identifier-checked and duplicate-checked.
Conflicted inserts require one explicit VALUES row. SQLite/PostgreSQL render a clause before
RETURNING; insert binds precede patch binds. MySQL returns Unsupported with the restricted
conflict-target reason before the native seam. Existing InsertData and adapter traits are
unchanged. SQLite's effective DEFAULT VALUES omission case is refused for conflicted inserts.

Framework-sql now has `InsertOutcome::{Inserted, AlreadyExists}`, bounded/allowlisted
`encode_patch`, and an explicit statement-local RETURNING capability check. Generated stores
add pure `insert` and `update_plan`, plus `create_with`, `create_if_absent`, `update` and
`upsert`. Existing `create` delegates to `create_with` using the default ValueContext.
Preparation and host values are evaluated once before writes. Patch binding follows declared
column order, rejects immutable/unknown fields and retains omission versus null. A separately
supplied upsert key must agree with the prepared input identity. No check-then-create,
automatic retry, inferred inserted/updated branch flag or fabricated no-change row is used.

The real app output was regenerated. Only generated `stores/{profile,project,task}.rs` and
the ownership manifest changed; contracts/metadata did not. The app's Tokio test dependency
now explicitly selects sync for the barrier fixture. README describes this partial state.

## Evidence and review of this partial increment

Logs and result JSON: `/private/tmp/s6-m2/`. The runner fixes the worktree target dir,
CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, TMPDIR and LLVM profile paths; unsets network
DB URLs; and applies a 600-second process-group kill/reap ceiling. Native hang/concurrency
proofs also run in children under the existing parent watchdog with kill/reap. Target is
removed before this shared-request stop, and no test/build command remains running.

New real SQLite tests prove insert/update/no-change RETURNING, preservation of the existing
row on DoNothing, unrelated unique violations, statement rollback after update-branch
constraint failure, explicit transaction rollback, and two barrier-started writers with
exactly one inserted row. A MySQL refusal-only Database panics if native execution or begin
is reached; both conflict variants return Unsupported without reaching either seam.

Actual generated calls prove no-change InsertOutcome, last-writer-wins patching, native
upsert's create/update branches, absent update, empty patch refusal, invalid create refusal,
identity mismatch refusal, caller rollback and concurrent insert-if-absent with one Inserted
and one AlreadyExists. Patch codec tests cover partial updates, ordered binds, explicit null,
immutable/unknown members, empty/non-object/over-limit inputs and incompatible values.

Review rounds for this partial increment (full M2 rounds still follow the completed matrix):

1. Logic/resource/atomicity: checked conflict scope, clause/bind order, DEFAULT/multi-row,
   hostile identifiers, aggregate bind ceilings and no-effect gates. Added update-branch
   uniqueness failure and explicit rollback proofs. `round1-writes` passed. Source extraction
   started before its Cargo process completely finished, so this run is not the final snapshot
   evidence; `round2-writes` and `round3-writes` were run after all source edits/regeneration.
2. Responsibility: separated generated write-plan emission into `emit_mutations`, consolidated
   create preparation through `create_with`, and shared capability validation and authoritative
   row decoding. Moved patch shape ceiling before serialization. `round2-writes` passed.
3. Consistency/docs/comments: reviewed Rustdoc, explicit already-exists outcome, borrowed
   executor ownership, declaration comments, error causes and API docs. No further semantic
   refactor was needed. `round3-writes`, warning-clean Clippy, MSRV and formatting passed.

Exact commands (all from worktree root with the runner environment):

```sh
cargo test -p stackpress-sql --locked
cargo test -p stackpress-framework-sql --features sqlite-maintenance --test conflict --locked
cargo test -p stackpress-framework-sql --features sqlite-maintenance --test executor_budgets --locked
cargo test -p generated-workspace-app --test writes --locked
cargo test -p stackpress-sql -p stackpress-framework-sql -p generated-workspace-app \
  --features stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance --locked
cargo clippy -p stackpress-sql -p stackpress-framework-sql -p generated-workspace-app --all-targets \
  --features stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance --locked -- -D warnings
cargo +1.96.0 check -p stackpress-sql -p stackpress-framework-sql -p generated-workspace-app \
  --features stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance --locked
cargo fmt -p stackpress-sql -p stackpress-framework-sql -p generated-workspace-app --check
cargo run -p generated-workspace-tool --locked -- emit generate --check true
```

`core-tests`: 516 passed, 16 ignored including doctests. Latest `round3-writes`: **531 passed,
0 failed, 16 ignored** including doctests/compile-fail cases. `round3-clippy`, `msrv-writes`
and `fmt-check`: exit 0. Generation check: exit 0, differences empty. `git diff --check`: clean.
No PostgreSQL/MySQL native execution was attempted. The 16 ignored cases are existing opt-in
network SQL tests. Intermediate standalone builds without codegen reported the pre-existing
private `RecordSchema::validate_defaults` dead-code warning; the declared unified feature scope
above is warning-clean. The first native conflict fixture used the wrong Engine::diff shape,
failed to compile, and was corrected to explicit `table.create().render`; reruns passed.

## Shared request / precise next step

See [the exact executor-limit API proposal](m2-executor-limits-proposal.md). A real reduced-bind
SQLite test passes and demonstrates the gap: renderer defaults accept three keys, while an
Engine and its owned Transaction configured for two binds reject them; 2+1 chunks through
the exact borrowed session return all identities. Both selected budgets are private. A
second copied app config or query-based probing is not the accepted exact-executor contract.

Before editing any SQL-core file beyond the existing grant, ask the coordinator to sequence:
`engine.rs`, `tests/engine.rs`, `docs/api/engine.md`, `docs/api/transaction.md`, adding only
`Engine::limits(&self) -> QueryLimits` and `Transaction::limits(&self) -> QueryLimits` plus
copy/no-IO/frozen-budget tests/docs. This request does not change schema revision 2 or any
adapter trait. Do not write those files until granted.

## Remaining M2 work and final gates

M2 is not an accepted/generated complete data layer yet. Still required:

- Complete executable scalar/compound identity matrix and update timestamp preparation;
  current emitter retains its earlier single String/Integer executable-key gate.
- Full supported filter/sort/q/page/count plans and deterministic/null ordering;
  approved pure Query DTOs are still not executed.
- Bounded explicit includes, inverse paths, required-relation errors, reduced-budget chunks,
  whole-result limits, real N+1/count tests and one-transaction SQLite snapshot proofs.
- Hard delete, @active remove/restore and maintenance selection, fully validated finite batches,
  two-store atomic workflow and atomic conveniences with retained rollback failures.
- Unknown write/commit outcome classification and no-automatic-retry proofs. Current new
  generated writes still inherit the earlier generic DataErrorKind::Sql execution mapping;
  do not claim final outcome semantics from these partial methods.
- Named generated handlers, explicit operation selection/registration/services, duplicate/startup
  guards, public input/private assembly and recursive public projection, policy/disclosure,
  direct/nested-delegation/CLI/buffered-HTTP parity. No listener is needed or authorized.
- Final real matrix, three complete M2 review rounds, appropriate additional safety rounds,
  per-crate coverage (including generated shipped code and shared SQL core), complete scoped
  tests/doctests, Clippy, fmt, Rust 1.96 check, pure-contract/WASM dependency check, generation
  checks, locked workspace suite and KB-validator baseline. These final gates are **pending**.
- Write the pre-approved `.agents/specs/00001-stackpress-rust-foundation/tasks/00011-receipt-m2.md`
  with exact shared-file diff, evidence and honest limitations, delete the target and stop M2.
  No M3 work is released.

On resume, recreate `/private/tmp/s6-m2` and the owned target/tmp directories as needed.
The runner and old logs may survive; do not depend on that. If the accessor grant arrives,
implement precisely that proposal, rerun the core suite and reduced-budget fixture, then
continue the native query/include layer and generated operation matrix. No final M2 receipt
or `.agents` file was written at this partial stop.
