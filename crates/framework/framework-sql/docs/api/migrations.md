# Revisions and migrations

Use `stackpress_framework_sql::migrations` for explicit schema history and reviewed maintenance. Enable `migrations` for pure planning/history and `sqlite-maintenance` for SQLite application. Importing generated stores or starting an app performs no migration or population.

```rust
use stackpress_framework_sql::migrations::{
    DialectId, MigrationPlanner, RenameMap, RevisionSnapshot,
};

fn plan_rename(
    before: &RevisionSnapshot,
    after: &RevisionSnapshot,
) -> stackpress_framework_sql::DataResult<String> {
    let mapping = RenameMap::new().field("project", "title", "heading")?;
    let plan = MigrationPlanner::between(before, after, mapping, DialectId::Sqlite)?;
    plan.artifact()
}
```

## History and plans

Create explicit field mappings through `RenameMap::field`; individual rename artifact
DTOs are private. Visibility does not change the persisted recipe or approval hash format.

New revision and plan artifacts use format **3**; an explicit frozen format-2 codec preserves existing bytes and hashes. The ledger remains format 2. See [storage lengths](storage-length.md) for target facts, complete narrowing checks and reverse refusal.

`RevisionSnapshot::new(app, revision, tables)` validates portable storage DTOs. `with_provenance` pins a semantic source fingerprint and an application definition revision. Snapshots contain no database path, credentials or raw plugin configuration. Dynamic SQL defaults/types are refused. The tool uses the same checked DDL definitions as the generated stores.

`RevisionStore::record(snapshot, parent)` writes a create-new immutable `revisions/<name>/schema.json` directory, or returns `RevisionOutcome::NoChanges` when the head's semantics match. The explicit parent must equal the current unbranched history head. `check()` verifies formats, hashes, companions and the parent chain without repairing anything. Missing parents, branches, modified ancestors, symlinks, special files and interrupted publication state refuse.

`MigrationPlanner::between` creates typed ordered operations, findings and an explicit reverse. It retains source physical slots and appends additions; declaration/UI order alone produces no SQL reorder. Plan recipes keep physical column order separately from immutable logical snapshots. `RevisionStore::revision` recovers a unique planned layout; conflicting proposed layouts refuse rather than guess.

`MigrationPlanner::suggestions` returns at most 4096 reviewable same-shaped candidates and errors above that bound. No match is accepted automatically. `RenameMap::field` and `table` authorize exact one-to-one identities. `drop_field` explicitly resolves a candidate as a deliberate removal; loss approval is still required. `backfill` accepts a checked `Literal` for an addition or nullable-to-required transition. Existing non-NULL values survive a required transition. Arbitrary expressions and unreviewed type conversions are unavailable.

`MigrationPlanner::review` aggregates forward loss/allocation findings before SQL rendering, including missing-backfill transitions that cannot produce an executable plan. The tool returns a `Blocked` inspection report without a plan hash when rendering is refused; it never publishes partial execution authority.

`MigrationPlan::findings` aggregates destructive changes and SQLite allocation warnings. `blocked_reasons` identifies conversions or capabilities that approval cannot waive. `reverse_plan` distinguishes `Available` from `Refused`; a dropped row/column cannot be recovered by recreating its schema. A nullable-add reverse checks that every discarded value is NULL. Required additions, backfills, seed effects and loss plans require custom recovery or a backup instead of an automatic reverse.

`RevisionStore::write_plan` publishes `migrations/<hash>/plan.json`, `<dialect>-up.sql` and an optional `<dialect>-down.sql`. An available reverse also gets its own checked plan directory and hash. SQL companions show bindings for inspection; execution loads structured recipes and re-renders them. Edited companions or incomplete reverse publication refuse. Hashes detect accidental edits and are not authentication against malicious rewrites.

Publication uses a cooperative writer lock and synced staging files. An interrupted `.pending-*` directory remains for investigation; no automatic deletion/repair occurs. Directory publication is not a portable crash-atomic transaction. The history root must be separate from disposable generated output and used only by cooperative writers.

## SQLite application and status

`MigrationRunner::apply` accepts nondestructive plans with exact hash approval. `apply_to(owner, plan, hash, target, loss)` additionally requires a `LossApproval::new(hash, target)` for destructive findings. A generic CLI force flag supplies no such authority. The target is a logical configuration label; private connection options remain app-owned.

The app must quiesce its normal runtime and close ordinary pools before opening maintenance. One dedicated exclusive transaction rechecks ledger/catalog state, applies all native/reconstruction phases, verifies row counts, schema and foreign keys, and publishes ledger/head together. Unknown objects on managed tables, unowned views/triggers, unknown dependent tables, namespace collisions, unmanaged existing schema and drift refuse. Unowned view/trigger dependencies are refused conservatively; single-quoted references are treated as possible dependencies. Unrelated objects without managed dependencies remain untouched.

`_stackpress_migrations` and `_stackpress_schema_state` use format 2. The applied chain is bounded to 1024 entries and includes app, sequence, parent, revision/plan hashes, dialect, state and execution time. The head and applied chain must agree. Execution time is excluded from deterministic artifact bytes. Existing unknown ledger formats are refused; there is no baseline-adoption or repair command.

`ApplyOutcome::AlreadyApplied` reconciles an identical acknowledged operation. A failed submitted commit or lost acknowledgement can be `DataErrorKind::UnknownOutcome`; reopen and reconcile instead of retrying individual statements. Failure before commit rolls back, while cancellation retires the dedicated connection. A live async runtime and awaited completion/cleanup are required.

`MigrationRunner::status(options, snapshot)` opens an existing database read-only and reports ledger/schema agreement. It never creates the file, adopts tables or repairs drift. This is an observational report; application performs its own authoritative checks after taking the writer lock.

## SQLite integer identity allocation

Only `INTEGER PRIMARY KEY AUTOINCREMENT` tables maintain `sqlite_sequence`. Reconstruction captures the historical high-water mark before dropping the source and restores at least that value after copying/renaming the replacement. Surviving larger IDs win, empty rebuilt tables are supported, and no other table's sequence is changed. Both forward and reverse reconstruction preserve deleted-ID history.

A plain `INTEGER PRIMARY KEY` uses SQLite's `max(rowid)+1` allocation rule and can reuse deleted IDs. The planner reports `plain_integer_primary_key_may_reuse_deleted_ids`; it does not promise non-reuse for that schema. The adapter's `sequence_high_water` returns `None` for plain tables, and `preserve_sequence_high_water` refuses them.

## App-owned operator events

Use the independent application tool's existing `emit` grammar. Command options follow the event; long boolean options require an explicit value. Database events are registered only with `--features database-tools`. All commands accept `--history PATH`; the default is the example's separate `schema/` root.

| Event | Required options and behavior |
| --- | --- |
| `revision` | `--name ID`; optional `--input FILE`, `--app ID`, `--parent ID`, and the paired explicit `--mysql-collation NAME --mysql-page-bytes N`. Without parent, selects the checked unbranched history head. No DB access. |
| `history-check` | Validate all immutable history and companions. No repair. |
| `migration-plan` | `--to ID`; optional `--from ID`, `--dialect sqlite/postgres/mysql`. Omitted from uses the declared history parent; a root revision creates a baseline. Returns forward and optional reverse hashes, findings and blocked reasons. |
| `db-status` | `--database FILE`; optional `--to ID`, otherwise the latest proposed snapshot. Read-only; proposed and applied heads can differ. |
| `db-install` | `--database FILE --plan BASELINE_HASH`. May create the explicit file; never uninstalls first. |
| `db-upgrade` | `--database FILE --plan FORWARD_HASH`. Requires an applied predecessor. |
| `db-downgrade` | `--database FILE --plan REVERSE_HASH`. Requires the exact separately published reverse hash, not the forward hash. |
| `db-purge` | `--database FILE --plan HASH --target LABEL --allow-data-loss true`. Requires a reviewed purge plan; preserves schema/head and allocation sequences. |
| `db-uninstall` | Same approval options with an uninstall plan. Drops enumerated managed tables and records `uninstalled`; preserves ledger and unrelated objects. |
| `db-populate` | `--database FILE --seed ID --plan HASH`. Executes the matching reviewed named seed plan in one transaction; the same seed plan is not replayed. |

For destructive upgrades, also supply `--target LABEL --allow-data-loss true`. Every command rejects unknown options. Reinstall after an uninstall currently refuses pending an explicit reviewed reinstall protocol.

Plan mapping options use stored SQL identities: `--rename project.title=heading`, `--table-rename project=portfolio`, `--drop-fields project.obsolete`. Comma-separated mappings are supported. `--backfills` accepts a JSON object such as `'{"project.summary":{"type":"text","value":"pending"}}'`.

Operator plans use `migration-plan --to ID --operation purge` or `--operation uninstall`. For seeds use `--operation populate --seed ID --seed-file FILE`; the file is a bounded regular JSON file containing `SeedRow` objects. Each row names its managed `table`, typed `values`, and explicit `upsert` boolean. Upsert targets the complete primary key. These are database-only steps; arbitrary events and external side effects are not part of the atomic promise.

```sh
cargo run -p generated-workspace-tool -- emit revision --name initial
cargo run -p generated-workspace-tool -- emit migration-plan --to initial --dialect sqlite
cargo run -p generated-workspace-tool --features database-tools -- emit db-install \
  --database /private/tmp/workspace.sqlite --plan REVIEWED_BASELINE_HASH
```

Replace the hash placeholder with the reviewed command result. History/plan operations await blocking work; database operations await cleanup. Removing a model never deletes historical revisions. Do not place secrets in explicit seed/backfill literals: those reviewed private artifacts deliberately preserve their supplied values.

## Backend limits and evidence

SQLite execution is the M3 target. PostgreSQL and MySQL have checked rendered plans only; neither network backend is applied by this runner. MySQL application stays refused until a separate durable nontransactional DDL journal/repair protocol exists. Existing renderer restrictions, including MySQL indexed TEXT keys and targeted upserts, remain visible errors.

The M3 tests exercise real SQLite forward/reverse reconstruction, CASCADE/RESTRICT/self/cyclic references, sequence preservation, NULL/backfill/index/unique behavior, drift and operator safety. Parent watchdogs bound native locking, cancellation and fault tests. Post-commit response loss is tested at the framework boundary; the adapter separately tests cancellation inside native COMMIT. These are distinct evidence scopes.
