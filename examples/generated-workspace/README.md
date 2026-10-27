# Generated workspace M0 proof

This example compiles the accepted three-model Idea schema into real Rust contracts and create/get stores. It is the Slice 6 M0 checkpoint; later milestones supply the complete data layer.

## Generate and compile

Run these commands from the repository root. The tool has no dependency on the generated application and compiles when generated output is absent.

```sh
cargo run -p generated-workspace-tool --locked -- emit generate
cargo run -p generated-workspace-tool --locked -- emit generate --check true
cargo check -p generated-workspace-app -p generated-workspace-contracts --locked
cargo check -p generated-workspace-contracts --target wasm32-unknown-unknown --locked
```

The existing CLI grammar requires a value after long options: use `--check true` or the supported `-c` shorthand. A present `--check false` takes precedence over `-c`. Generation never changes a database or permanent schema history.

The tool composes and finalizes Idea through the existing parser/transformer, then runs schema and SQL functions in one registry. Their fingerprints match. Contract source is shared verbatim with the pure consumer; its normal dependency graph contains Serde, JSON and the default schema crate only.

## Execute and inspect

The native proof uses the generated `Profiles`, `Projects` and `Tasks` functions with one exclusive `DataSession::transaction(&mut transaction)`. The caller commits or rolls back. For `@active` models, ordinary `get` excludes inactive rows; create still returns the inserted record. An app-owned named `profile_name` handler uses the current typed service API, an explicitly registered policy and a name-only projection. No operation or route is registered automatically.

```sh
cargo test -p generated-workspace-app -p generated-workspace-contracts --locked
cargo test -p stackpress-sql-sqlite --features maintenance --locked
cargo test -p stackpress-framework-sql --features codegen,server,sqlite-maintenance --locked
```

Hang-class proofs execute in children under parent deadlines with kill/reap. The maintenance suite also checks a deliberately stuck negative control. SQLite files are independent temporary fixtures; no listener or network database is needed.

## Checkpoint boundaries

M0 implements supplied scalar IDs, String/Text/Integer/Boolean values, string enums, acyclic fieldsets, bounded lists, literal defaults and the fixture's nonempty assertion/explicit relations. Other selected type/validation/provider semantics fail generation until their later milestone implements them. The full UI descriptor is not frozen by this checkpoint.

The migration library proves explicit baseline installation, non-key safe rename, nullable native addition, NULL-only addition reversal, exact plan approval, writer-locked ledger checks and reconciliation without replay. Dedicated adapter reconstruction is proved separately with checked table/copy queries and FK/ledger rollback. General reconstruction planning, durable revision directories, broader CRUD, include loading, operation generation and DB command registration remain later work. The `database-tools` feature selects maintenance dependencies; it does not register unfinished commands.

Maintenance consumes its dedicated owner/session and retires uncertain connections. Applications must quiesce and close ordinary pools first, await completion/rollback, and reconcile an unknown commit result from the ledger. Automatic integer identity and unmanaged objects are refused by the M0 migration subset.
