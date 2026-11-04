# Default generated framework models

`stackpress-framework-models` contains the native source schemas generated from the shipped
`stackpress/stackpress.idea` → `stackpress-api/schema.idea` →
`stackpress-session/schema.idea` graph. It exports the four Schema/Model/Store/Actions families and `Models` from checked-in
generated output. Importing this package registers nothing. Session/API plugin binding
remains M3 work.

```rust
# #[cfg(not(target_arch = "wasm32"))] {
use stackpress_framework_models::ProfileSchema;
use stackpress_framework_schema::source::SourceSchema;

let schema = ProfileSchema::new("app-owned-seed".into(), 0);
assert_eq!(ProfileSchema::describe().name, "Profile");
let defaults = schema.defaults_at(42)?;
assert_eq!(defaults["type"], "person");
assert_eq!(defaults["created"], stackpress_framework_schema::source::date(42));
# }
# Ok::<(), stackpress_framework_schema::SchemaError>(())
```

The seed and local Date offset are explicit native constructor arguments. Construction
runs no defaults or application IO. `defaults_at` supplies a clock; `with_providers` supplies
an app-owned `ValueProviders`. Missing members, explicit null, native Dates and insertion
order remain distinct. The complete private metadata includes all source fields, attributes,
relations, selectors and presentation bindings. It is not a browser publication or a route
policy. Source schemas are absent on wasm32; browser UI integration is a later stage.

[Native tests](tests/native.rs) check the original Node column and crypto vectors, every
core accessor, defaults, partial assertions, filter/populate and codec phases. [Generator
controls](tests/generation.rs) prove check-only drift rejection and app extension. Defaults
and schemas depend on the lower schema source runtime, without session/API/Idea runtime dependencies. Server is selected only by the optional
`server` feature. `serde_json` is a test dependency; emitted code uses the stable
`stackpress_framework_schema::source::__private::serde_json` support path.

## Regenerate and check

From the workspace root:

```sh
cargo run -p stackpress-framework-models --features generate --bin framework-models-generate --locked --offline
cargo run -p stackpress-framework-models --features generate --bin framework-models-generate --locked --offline -- --check
```

The finite tool uses the public `Schema::normalize_source` and
`codegen::emit_source_contracts` and SQL `codegen::emit_source_stores` APIs with the existing `PackageLoader`, `Registry` and
`OutputWorkspace`. Checked-in files beneath `src/generated/` and their managed manifest
are generated output: never edit them by hand. `--check` reports stale output and writes
nothing. `--output <path>` selects another managed root; `--input <file>` replaces the
single app entry source while retaining the package catalog. Relative auxiliary app files
need the application's own loader rather than this default-package tool.

An application extends a model by importing its package Idea file and redeclaring it with
additional fields under the existing G19 composition rules. The public emitter accepts the
composed graph; it does not repair overridden core declarations. The independent M1 probe
compiles a Profile `nickname String?` default with only the schema runtime dependency.
Session-only generation explicitly declares Application/Session external, retaining their
links without generating hidden API models or tables.

## Native stores and optional events

The [SQL source guide](../framework-sql/docs/api/source-models.md) describes NativeStore,
NativeActions, SourceStatement and pooled/exclusively borrowed action signatures. The
public `ProfileModel`, `AuthModel`, `ApplicationModel`, `SessionModel` and `Models` paths
are generated-family conveniences. `generated::stores::<model>::register` is available
only with `server`; an application emitting this output declares its own `server` feature
forwarding `stackpress-framework-sql/server`. No default feature installs a handler.

```rust
# #[cfg(not(target_arch = "wasm32"))] {
use stackpress_framework_models::{ProfileStore, Models};
use stackpress_framework_sql::source::{ApiModels, StoreQuery};

fn family<T: ApiModels>() {}
family::<Models>();
let store = ProfileStore::new("explicit-seed".into(), 0);
assert_eq!(store.table(), "profile");
let query = StoreQuery::eq("id", "application-owned-id");
let _ = store.select(&query, stackpress_sql::DialectId::Sqlite)?;
# }
# Ok::<(), stackpress_framework_sql::source::ActionError>(())
```

[SQLite and original SQL oracles](tests/sql.rs) prove source CRUD/query/results/batch and
setup behavior. [Live opt-in tests](tests/live_search.rs) execute generated Profile/Auth
and Application/Session search on owned MySQL/PostgreSQL throwaway databases; unset URLs
report skipped, never passed. The same scenario pins MySQL partial DDL progress,
PostgreSQL/SQLite rollback, adjacent upgrade and foreign-key purge.
[Events](tests/events.rs) prove source skips/order/errors without a global guard.
[Ownership tests](tests/ownership.rs) prove generated action reentrancy and cancellation
under a parent watchdog, including a forced-kill control.

The [maintained extended app](tests/extended/schema.idea) adds a Profile column, enums,
fieldsets, every list family and collision-prone model names. Its output is generated by
the same tool; [extension tests](tests/extensions.rs) execute the added column/uniqueness
and pin source conversion quirks. Source schema validation follows G51's named-rule repairs,
including exclusive cgt/clt, while preserving original messages. `@hashed` uses SHAKE256 and
`@encrypted` deterministic AES-CBC, independently of authentication password hashing.
The [schema guide](../framework-schema/docs/source-models.md) lists conversion/provider
boundaries. The accepted lower SQL type mapping governs physical DDL; complete source
metadata does not imply identical driver DDL spellings. Unknown columns are deliberately
omitted by the original SQL setup helper. Concrete renderers and plugin-family binding
are not supplied at this stage. Typed generated applications remain byte-stable.
