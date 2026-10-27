# Generated workspace schema and generation proof

This example compiles the accepted three-model Idea schema into real Rust contracts and stores. M0 storage and M1 schema/generation are accepted. M2 adds SQLite CRUD, native queries, bounded eager relations and explicitly selected generated operations; coordinator acceptance is separate from this implementation.

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

`contracts::public` contains `Public<Model>`, `Public<Fieldset>`, `PublicCreate<Model/Fieldset>`, `PublicPatch<Model>`, `Patch<Model>`, `<Model>Key`, `<Model>Filter`, `<Model>Sort` and `<Model>Query`. Public inputs reject unknown/generated/denied fields, expose `validate_client` and host `validate_create`/`validate_patch`, and prepare only approved literal defaults. M2 supplies checked host assembly and explicit database execution. Patch keeps omission, Set and Null distinct; empty updates fail. Public serialization rejects integers outside the JavaScript exact range.

The frozen public queries remain pure data contracts; M2 native stores execute them after allowlist validation. Predicates are Eq/Ne/Ge/Le/Like/Has/Hasnt/IsNull, checked against field eligibility. Query defaults are take=20, skip=0, total=false; limits are take 1–100, skip ≤100000, at most 100 filters and 16 sorts/includes, includes depth ≤3 and q ≤4096 bytes. Includes use approved relation paths. M2 adds query executors and named handler registration inside `stores`. Routes and lookup URLs stay app-owned; the frozen descriptor operations remain empty and lookup remains None.

Date is YYYY-MM-DD in years 0001–9999; local time includes seconds and at most nine fractional digits; datetime is normalized UTC RFC3339 at nanosecond precision. Leap seconds and excess precision fail. Relative rules use the supplied clock; stored validation does not repeat creation-time or host-only checks. Character counts use Unicode scalar values, words use Unicode whitespace; floating step uses a quotient tolerance of 1e-10 while Integer step requires a positive integer and remains exact over i64. JSON has a one MiB/10000-node/depth-32 budget and list values have at most 1024 elements. Model/field semantic failures retain winning Idea UTF-8 spans; normalized JSON reports structural paths without invented coordinates.

The migration library proves explicit baseline installation, non-key safe rename, nullable native addition, NULL-only addition reversal, exact plan approval, writer-locked ledger checks and reconciliation without replay. Dedicated adapter reconstruction is proved separately with checked table/copy queries and FK/ledger rollback. General reconstruction planning, durable revision directories and DB command registration remain later work. The `database-tools` feature selects maintenance dependencies; it does not register unfinished commands.

Maintenance consumes its dedicated owner/session and retires uncertain connections. Applications must quiesce and close ordinary pools first, await completion/rollback, and reconcile an unknown commit result from the ledger. Automatic integer identity and unmanaged objects are refused by the M0 migration subset.

## M2 native data and operations

Generated stores provide detached plans, complete scalar/compound-key CRUD, primary/unique targeted upsert, bounded batches and active-only remove/restore. Writes prepare once, use statement-local RETURNING and submit once on the exact borrowed executor. Update is last writer wins. `create_if_absent` returns `InsertOutcome::Inserted(record)` or `AlreadyExists`, which acknowledges no change and contains no fabricated existing row. MySQL targeted conflicts are Unsupported before effects; PostgreSQL rendering is covered while native generated-CRUD integration remains later work.

Typed queries implement scalar comparison, literal q, escaped Like, scalar-array membership, explicit NULL ordering, complete-key ties and bounded offset pages. Counts include matched base identities before paging. Includes are finite eager `Loaded` values, batched across parents using the selected executor's limits and bounded aggregate rows/keys/bytes/statements. Counts/includes require a caller-owned SQLite snapshot; no query retries or truncated collections are returned. Default detail has a distinct `Selected` projection and applies approved `@query` paths after recursive public projection.

The tool explicitly selects named generated handlers. The app selects operations in `stores::operations::<model>::register` after registering Engine and Policy services; one app-owned Registration marker prevents accidental duplicate selection. Direct stores remain trusted domain APIs. Strict public envelopes, policy, private/default assembly, completion guards, recursive projection and sanitized errors are proved through direct, CLI and buffered HTTP calls without a listener. Submitted write/commit failures conservatively report an unknown outcome; a display refetch is never permission to replay.

```sh
cargo test -p generated-workspace-app --test reads --test writes --test atomic \
  --test operations --test outcomes --locked
cargo test -p generated-workspace-app --features matrix --test matrix --locked
cargo test -p generated-workspace-app --features names --test names --locked
```

The alternate fixtures are generated independently because their combined source would exceed the existing generator output budget. Select one fixture feature at a time; defaults use the original three models. The full matrix covers storage/value/temporal/enum/compound-key/unique/repeated-relation behavior. The small names fixture covers Schema/Operations aggregate collisions, Rust keywords, active business rules and a fully private key-only model.

```sh
cargo run -p generated-workspace-tool --locked -- emit generate \
  --input "$PWD/examples/generated-workspace/app/native.idea" \
  --output "$PWD/examples/generated-workspace/app/src/matrix_generated"
cargo run -p generated-workspace-tool --locked -- emit generate \
  --input "$PWD/examples/generated-workspace/app/names.idea" \
  --output "$PWD/examples/generated-workspace/app/src/names_generated"
```

Read the [data-layer documentation](../../crates/framework/framework-sql/docs/README.md) for APIs, limits, transaction ownership and outcome semantics. M3 migration expansion and network generated-CRUD integration are outside this milestone. Schema revision 2, the original descriptors/DTOs/query shapes and the contracts/metadata/stores partition names remain unchanged.
