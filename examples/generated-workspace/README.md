# Generated workspace schema and generation proof

This example compiles the accepted three-model Idea schema into real Rust contracts and create/get stores. It includes the accepted Slice 6 M0 storage proof and M1 schema/generation contracts. Later milestones supply the complete data layer.

## Generate and compile

Run these commands from the repository root. The tool has no dependency on the generated application and compiles when generated output is absent.

```sh
cargo run -p generated-workspace-tool --locked -- emit generate
cargo run -p generated-workspace-tool --locked -- emit generate --check true
cargo check -p generated-workspace-app -p generated-workspace-contracts --locked
cargo check -p generated-workspace-contracts --target wasm32-unknown-unknown --locked
```

The existing CLI grammar requires a value after long options: use `--check true` or the supported `-c` shorthand. A present `--check false` takes precedence over `-c`. Generation never changes a database or permanent schema history.

The tool composes and finalizes Idea through the existing parser/transformer, then runs schema and SQL functions in one registry. Their fingerprints match. Contract source is shared verbatim with the pure consumer; its normal dependency graph contains Serde, JSON and the default schema crate, including its pure value-validation dependencies. Neither SQL, server, native adapters nor codegen enter that graph.

## Execute and inspect

The native proof uses the generated `Profiles`, `Projects` and `Tasks` functions with one exclusive `DataSession::transaction(&mut transaction)`. The caller commits or rolls back. For `@active` models, ordinary `get` excludes inactive rows; create still returns the inserted record. An app-owned named `profile_name` handler uses the current typed service API, an explicitly registered policy and a name-only projection. No operation or route is registered automatically.

```sh
cargo test -p generated-workspace-app -p generated-workspace-contracts --locked
cargo test -p stackpress-sql-sqlite --features maintenance --locked
cargo test -p stackpress-framework-sql --features codegen,server,sqlite-maintenance --locked
```

Hang-class proofs execute in children under parent deadlines with kill/reap. The maintenance suite also checks a deliberately stuck negative control. SQLite files are independent temporary fixtures; no listener or network database is needed.

## Checkpoint boundaries

M1 supports String/Text, i64 Integer, finite Number/Float, Boolean, canonical Date/Time/Datetime, bounded Json/Object/Hash objects, homogeneous String/i64/Boolean enums, acyclic fieldsets and bounded non-model lists. Defaults retain missing/null/value distinctions and literal false/zero/empty values. Selected unknown semantics, unsupported crypto, unavailable providers, invalid relations and embedding cycles fail before output publication. Idea quoted strings retain the existing parser's raw syntax; normalized JSON can represent escaped quotes.

The schema-only matrix is generated independently of native stores:

```sh
cargo run -p generated-workspace-tool --locked -- emit generate \
  --input "$PWD/examples/generated-workspace/contracts/matrix.idea" \
  --output "$PWD/examples/generated-workspace/contracts/src/matrix"
```

`stackpress_framework_schema::CONTRACT_REVISION` is **2**. Generated `metadata::schema::FINGERPRINT` identifies canonical semantics; `metadata::ui::schema()` returns the app-approved `UiSchema`. Stable component keys and canonical Field/Filter/List/Span/View bindings contain public props and availability flags. The app's `DefinitionRegistry` supplies access policy, provider/validator revisions and tool-only `ComponentImport` entries. A role never grants access, and selecting an unavailable role for actual UI output fails.

Stored contracts expose `validate`, `validate_with` and fallible `project`. Trusted `Create<Model>` and `Create<Fieldset>` have tri-state `Input` fields and `prepare_create`/`prepare_with`; nested defaults share one budget, and each generated provider and business validator runs once per instance. `ValueContext` supplies the clock and host functions explicitly. Private literal defaults are requested through `default:Model.field`, keeping private values out of browser source. A trusted literal fieldset containing hidden members uses the same host path rather than embedding them.

`contracts::public` contains `Public<Model>`, `Public<Fieldset>`, `PublicCreate<Model/Fieldset>`, `PublicPatch<Model>`, `Patch<Model>`, `<Model>Key`, `<Model>Filter`, `<Model>Sort` and `<Model>Query`. Public inputs reject unknown/generated/denied fields, expose `validate_client` and host `validate_create`/`validate_patch`, and prepare only approved literal defaults. Host assembly of private/generated values and database execution belong to M2. Patch keeps omission, Set and Null distinct; empty updates fail. Public serialization rejects integers outside the JavaScript exact range.

Queries are typed data contracts only. Predicates are Eq/Ne/Ge/Le/Like/Has/Hasnt/IsNull, checked against field eligibility. Query defaults are take=20, skip=0, total=false; limits are take 1–100, skip ≤100000, at most 100 filters and 16 sorts/includes, includes depth ≤3 and q ≤4096 bytes. Includes use approved relation paths. There are no generated query executors, routes, operation registrations or inferred relation lookup URLs; descriptor operations are empty and lookup is None.

Date is YYYY-MM-DD in years 0001–9999; local time includes seconds and at most nine fractional digits; datetime is normalized UTC RFC3339 at nanosecond precision. Leap seconds and excess precision fail. Relative rules use the supplied clock; stored validation does not repeat creation-time or host-only checks. Character counts use Unicode scalar values, words use Unicode whitespace; floating step uses a quotient tolerance of 1e-10 while Integer step requires a positive integer and remains exact over i64. JSON has a one MiB/10000-node/depth-32 budget and list values have at most 1024 elements. Model/field semantic failures retain winning Idea UTF-8 spans; normalized JSON reports structural paths without invented coordinates.

The migration library proves explicit baseline installation, non-key safe rename, nullable native addition, NULL-only addition reversal, exact plan approval, writer-locked ledger checks and reconciliation without replay. Dedicated adapter reconstruction is proved separately with checked table/copy queries and FK/ledger rollback. General reconstruction planning, durable revision directories, broader CRUD, include loading, operation generation and DB command registration remain later work. The `database-tools` feature selects maintenance dependencies; it does not register unfinished commands.

Maintenance consumes its dedicated owner/session and retires uncertain connections. Applications must quiesce and close ordinary pools first, await completion/rollback, and reconcile an unknown commit result from the ledger. Automatic integer identity and unmanaged objects are refused by the M0 migration subset.
