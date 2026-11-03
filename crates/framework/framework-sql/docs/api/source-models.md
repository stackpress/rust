# Native source-model contracts

`stackpress_framework_schema::source` and `stackpress_framework_sql::source` define
the native contracts for G50 generated framework models. Neither module requires
an optional feature. Both are absent on `wasm32`. They supply no default models,
handlers, engine, database setup or runtime model selection.

The [schema module](../../../framework-schema/src/source.rs) owns `SourceSchema`
and `SourceField<Name>`. Implementations choose an owned `Value` and ordered
`Record`; these types need not implement Serde. Native Dates and extension fields
therefore need no intermediate strict JSON DTO. `new(seed, offset_minutes)` only
constructs codec state. Defaults, filtering, assertions and serialization are
explicit methods; the contract does not implement their source semantics.

The [SQL module](../../src/source/mod.rs) associates each `SourceModel` with its
exact schema, store, actions and role. `SourceStore` prepares queries/DDL and
codecs without holding an engine. `SourceModel::actions` constructs immutable
actions from an application-owned `Arc<Engine>`. No query runs at construction.

`SessionModels` supplies Profile/Auth. `ApiModels: SessionModels` adds
Application/Session and inherits that exact Profile type. Role markers and
`SourceField<Id, FieldValue = String>` reject a wrong role or incompatible ID.
M0 checks IDs only; the remaining required core-field witnesses must be added
before plugins consume these families. See the compiled examples in the modules'
Rustdoc and the [negative compiler cases](../../tests/source-ui).

## Actions and executor ownership

[`SourceActions`](../../src/source/actions.rs) requires `Send` futures for CRUD,
batch and setup. Pooled CRUD methods forward to the same `_in` implementation
using a temporary `DataSession::pooled`. Borrowed methods retain the exact
exclusive executor/input borrow until completion or cancellation; only the caller
finishes a borrowed transaction. Returned records own their contents. Dropping a
future releases its Rust borrow; it is not evidence of rollback or remote outcome.

Implementations must not hold a model/config lock across await. The trait cannot
statically exclude every `Send` lock guard. The immutable-state
[SQLite ownership proof](../../tests/main/source_models/ownership.rs) suspends a
real provider, re-enters the same actions, cancels a borrowed call, then reuses
its executor. An external process watchdog includes a forced-kill control.
That proof covers the provided mechanics implementation, not arbitrary downstream
implementations or the later generated source codecs.

## Explicit emission selection

With `codegen`, schema's `emit_contracts_with_semantics` and SQL's
`emit_stores_with_semantics` take `EmissionSemantics::{Typed, Source}`.
`Typed` is the default and delegates to the existing emitters. At M0, `Source`
returns `SchemaErrorKind::Unsupported` without adding files. It never falls back
to typed handlers. Source normalization, source emitters and default families are
later stages; no existing emitter's behavior changes here.

The [test-only structural template](../../tests/main/source_models/model.rs.tpl)
emits default and app-extended families from a small composed scalar/Date corpus.
Its helper codec stores probe Dates as integers and does not implement full
Stackpress source preparation, hashing, encryption, query or batch semantics.
The [emission tests](../../tests/main/source_models/emission.rs) also compose the
actual shipped session input, keeping Application/Session links external and
generating no hidden API models. Source-mode normalization of those links remains
M1 work; M0 does not claim that the full shipped schema emits yet.
