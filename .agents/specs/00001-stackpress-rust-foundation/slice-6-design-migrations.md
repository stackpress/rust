# Slice 6 revisions, migration plans and maintenance

Proposed operator contract behind [the owner choices](slice-6-design-decisions.md). Load before implementing history, SQLite reconstruction or database tooling. Planning, writing artifacts, and applying them are three distinct actions. Generating Rust never changes a database.

## Durable history versus disposable output

The independent app tool owns a separate explicit `schema/` history root, outside the transformer's disposable generated root:

```text
schema/
  revisions/0001-initial/schema.json
  revisions/0002-project-summary/schema.json
  migrations/0001-to-0002/plan.json
  migrations/0001-to-0002/sqlite-up.sql
  migrations/0001-to-0002/sqlite-down.sql  # only when a reverse exists
```

Revisions are versioned normalized schema snapshots with logical app ID, parent revision, source fingerprint, storage signature, generator/definition revisions and safe metadata. Plugin config/environment values are excluded. A private full semantic snapshot is not shipped to browsers. Explicit names/order replace wall-clock epoch identity; canonical content hashes detect edits. Re-recording identical semantics returns NoChanges. Unknown format revisions, missing/duplicate parents, branches without explicit selection and modified ancestors error.

History uses create-new immutable files and a cooperative writer lock; no recursive deletion or stale generated-output cleanup touches it. Write all artifacts to staging, publish the complete revision/plan directory, and retain recovery state on interrupted publication. Explicit check validates completeness/hash/parent chain without repair. Filesystem publication is not a portable crash-atomic transaction; a database apply must reject incomplete history. Do not claim a hash authenticates a maliciously edited migration.

`RevisionSnapshot`, `RevisionId`, `RevisionStore`, `MigrationPlan`, `MigrationStep`, `RenameMap`, `RiskFinding`, `ReversePlan` and `AppliedRevision` are framework-owned types. Snapshots serialize framework storage DTOs; `TableDefinition`/`SchemaDiff` are reconstructed via checked conversion, since the current SQL core does not expose Serde snapshot derives. There is no need to add framework metadata/serialization to the independent SQL core.

## Plan before effects

`MigrationPlanner::between(from, to, RenameMap, dialect) -> Result<MigrationPlan>` compares every managed model. For retained tables it calls current `TableDefinition::diff`/`SchemaDiff::rename_field`; additions/removals are explicit table steps in dependency order. SchemaDiff's no-op, same-logical-table and physical-order restrictions remain binding. A framework storage-order map retains old physical slots and appends additions; declaration/UI order alone is not a request to reorder SQL columns. Explicit reordering or changed constraints uses reconstruction.

Plan includes source/target signatures, exact ordered operations and binds, dialect, capability/format/generator revisions, preconditions, explicit renames, destructive findings, reversibility and a canonical hash. Validate all steps, aggregate bytes/binds/statements and supported capabilities before acquisition. Raw `.sql` is an inspection/export companion, with bind/precondition metadata in plan.json; concatenated placeholder SQL alone is not executable migration authority. Execution reads the checked structured plan and re-renders it, never arbitrary modified SQL files.

Same-shaped removed/added fields produce suggestions. An explicit mapping must be one-to-one and refer to the actual old/new definitions; map-related indexes/FKs consistently and retain changed semantics after renaming. Ambiguity is not resolved by force. Model/table rename likewise requires explicit identity mapping; otherwise it is a destructive drop/create finding. Backfills are checked literal values or a named reviewed app-owned conversion with a pinned revision; no caller-authored expression evaluation.

Destructive findings include drops/removals, narrowing/retyping, nullable→required without complete backfill, key/unique/FK weakening or replacement, and purges. Aggregate all findings before refusing. A data-loss approval names the exact plan hash and target configuration; it does not waive validation, object ownership, conversion correctness, drift, missing predecessors, or FK/uniqueness failures. A generic CLI `-f` has no such authority.

Reverse planning is explicit. Safe rename/index changes and compatible additions can have a checked reverse with runtime preconditions; dropping an added column is allowed without data loss only if its values are provably discardable under the reviewed policy. Dropped rows/columns, lossy conversions and unconstrained seed effects have no automatic data-preserving reverse. Store an irreversibility reason and required recovery path. Tests must demonstrate both permitted reverse and refused impossible reverse, rather than claim every forward migration can recover data.

## Applied database state

Use reserved, namespaced tables such as `_stackpress_migrations` and `_stackpress_schema_state`, with format version, app ID, sequence, parent, revision hash, plan hash/dialect, application state and execution time. Execution time belongs to the ledger, not deterministic generated bytes. Namespace collision or unknown ledger format errors; never adopt an unrelated pre-existing table automatically. Ledger is the applied chain; revision directories are proposed history.

For SQLite, obtain one exclusive maintenance transaction, inspect ledger and managed schema, validate plan preconditions against actual tables/columns/constraints/indexes, apply all steps, verify resulting signature/FKs, insert the ledger record and update head, then commit once. The source state and plan are rechecked after obtaining the DB writer lock; a prior read-only check is not authority against another migrator. Lock/acquire/busy waits are finite. A competing migrator either observes the completed new head and returns AlreadyApplied for identical hashes or refuses its stale plan; it never executes twice.

The ledger and schema changes commit together. Failed work rolls back; cancellation before acknowledgement makes outcome uncertain and the next invocation reads ledger/head/schema to reconcile. Never retry automatically. An acknowledged ledger entry with conflicting artifact hash fails. No ledger with existing managed tables fails as UnmanagedExistingSchema; explicit baseline adoption is a separately reviewed future command. Empty install means no conflicting managed objects, not permission to erase other tables.

Inspect only the configured target database and bounded managed-object catalog. Unknown dependent tables/views/triggers/custom indexes or unsupported/generated/virtual table features block reconstruction. Do not silently discard or attempt to recreate unknown objects by string substitution. Schema drift is a refusal, with a reviewable diagnostic; future repair tooling is separate scope. This is a cooperative application schema contract, not arbitrary third-party database migration support.

## SQLite reconstruction protocol

Current SQL builders reject SQLite type/nullability/key/FK alteration and raw PRAGMA/transaction policy. A new adapter-owned maintenance API is required; normal Engine SQL stays restricted. Proposed `SqliteMaintenance::open(options)` creates a dedicated bounded connection owner; `begin_rebuild()` returns a Send, non-Clone exclusive session. It can execute validated Query/QueryPlan data and perform typed catalog/FK checks. No raw native connection or arbitrary pragma setter is exposed. The app quiesces its runtime and closes normal pools before maintenance; the library does not discover or stop arbitrary processes.

1. Open the explicit file with normal options; verify journal/foreign-key policy and inspect bounded object features. Disable foreign-key enforcement **before** BEGIN on this owned connection; read back its value. A failed/cancelled transition retires the connection. Begin EXCLUSIVE with finite busy timeout; recheck catalog/ledger under the lock.
2. Pre-render all replacement definitions/copy mappings/checks. Use deterministic collision-checked temporary table names, create the new table under that name, and preserve target PK/unique/FK definitions. Defer secondary index creation until names are available. Never rename the old table first and accidentally rewrite inbound references.
3. Copy explicit columns with `INSERT INTO new(cols) SELECT mapped_cols FROM old`. Fill additions only with approved defaults/backfills. Reject silent casts/truncation or row omission; copy count and target integrity must match. Generated models initially have explicit IDs; for supported SQLite integer identity preserve a safe sequence high-water mark through adapter-owned typed inspection, or refuse that rebuild until proven.
4. Drop the old table and rename the replacement to the original name; recreate all owned indexes, including mapped constraints. Rebuild the affected managed-table group in dependency-aware order; FK cycles are permitted only within the checked group. All steps remain inside the one transaction, with enforcement disabled only on that connection.
5. Run typed `foreign_key_check` across affected managed references plus target schema/data checks before ledger write/commit. Any violation rolls back the entire group. Inbound cascade/restrict behavior must survive; disabling enforcement is not permission to ignore invalid data.
6. Commit once; restore/verify foreign keys outside the transaction or retire the connection unconditionally. Close/await native cleanup before returning the maintenance owner. Drop/cancellation retires it and initiates rollback; no uncertain connection returns to a normal pool. Commit acknowledgement loss is explicitly UnknownOutcome.

The local probe in [architecture](slice-6-design-architecture.md) supports steps 1–5's basic mechanism. It does not prove this proposed API, concurrent processes, full feature preservation, cancellation, crash recovery, sequence handling or production boundedness. M0 must prove those first with real adapter types and a parent watchdog. Physical files/WAL/SHM are managed by SQLite; do not copy or swap live DB files as a migration algorithm.

## Explicit operator commands

All commands below are proposed app-tool `emit` events, compiled/registered only in the maintenance tool. No maintenance handler is installed in the normal HTTP/Tauri/app host.

| Event | Effects and default refusal |
| --- | --- |
| `generate`, `generate --check` | Plan/install/check disposable generated source under G20; no revision/DB write. |
| `revision --name ...` | Record current normalized schema explicitly; no DB access. |
| `migration-plan --from ... --to ... --dialect ...` | Write reviewable structured plan/SQL and warnings, including unsafe findings; no application or force requirement just to inspect it. Omitted from means the explicit history parent, never guessed live state. |
| `db-status` | Read-only ledger/schema/artifact agreement report; no automatic adoption, repair or retries. |
| `db-install --plan HASH` | Install reviewed baseline only into a nonconflicting managed namespace, record head; unlike upstream, never uninstall first. |
| `db-upgrade --plan HASH` | Apply exactly the next approved plan from current applied state. Optional explicit `--allow-data-loss` required for destructive steps; unknown/ambiguous/drifted plans still refused. |
| `db-downgrade --plan HASH` | Apply checked reverse of current head with preconditions; refuse irreversible/data-bearing loss without a separately approved destructive reverse plan. Preserve history as ledger records. |
| `db-purge --plan HASH --allow-data-loss` | Delete only the enumerated managed tables' data in FK-safe order, one transaction. Preserve schema/revision head; sequence reset is a separate explicit plan item. No generic cascade into unmanaged data. |
| `db-uninstall --plan HASH --allow-data-loss` | Drop only enumerated owned objects, preserve a ledger record marking uninstalled. Unrelated objects/ledger namespaces remain. No unbounded directory/database removal. |
| `db-populate --seed ID --plan HASH` | Execute named bounded typed seed steps with one session/transaction; choose create/upsert semantics and idempotency keys explicitly. Never run arbitrary untrusted event names under an assumed outer transaction. |

Tool config selects database options/paths and logical target label; credentials and connection URLs are private, never revision/generated/public output. CLI uses existing operator flags before `emit`; command-specific data follows the event. Parse unknown options strictly in the handler. No new CLI parser, stock binary or command synonym is needed. Tool runtime awaits blocking file work and database cleanup; dropping the caller is not a detached successful migration.

## Other dialects

PostgreSQL plans can use current transactional DDL rendering; applying them still requires Slice 7's real adapter, typed catalog/drift checks and transaction/snapshot contract before a support claim. MySQL `execute_plan` currently refuses nontransactional DDL. Keep that refusal: a future S6-N migration task must design durable per-step journal, advisory locking, restart/repair and observed-partial-state handling before MySQL application. An adapter's CRUD support does not supply that protocol. Rendered plans and G26 tests are not production migration evidence.
