# Native source models, stores and actions

With `source`, `stackpress_framework_sql::source` supplies `NativeStore`, `NativeActions`,
`StoreQuery`, `SourceStatement` and the shared `ActionError`/`SourceResult`, backed by the
schema owner's native Value/ordered Record. Dates, missing/null and application-added
members survive without a strict JSON DTO. Native source modules are absent on wasm32.
No model, engine, route policy or registrar is selected merely by importing this module.

`SourceModel` associates one schema, store, actions and role. `SessionModels` supplies
Profile/Auth; `ApiModels` inherits exactly that Profile and adds Application/Session.
Generated witnesses reject wrong core declarations without repairing overrides.
[Compiler cases](../../tests/source-ui) pin field/declaration and family mismatches.
The [generated default package](../../../framework-models/README.md) supplies concrete
models without depending on session/API plugins.

## Generate and use

With `codegen`, call schema `emit_source_contracts(&SourceGraph, plan)` and SQL
`emit_source_stores(&SourceGraph, plan)` in the same managed output plan. The structural
SQL templates emit ordinary per-model Model/Store/Actions types and the applicable family.
Schema and SQL module names escape infrastructure collisions independently of physical
SQL column/table spelling. Typed emitters and existing generated output remain unchanged.
The old semantics selector refuses Source when passed a Typed Schema; it never falls back.
See [normalization](../../../framework-schema/docs/source-models.md) for source types,
assertions, enums/fieldsets, external references and native codecs.

`NativeStore::new(seed, offset_minutes)` constructs immutable metadata/codec state without
IO or defaults. `NativeStore::from_schema` keeps explicit application providers.
`SourceModel::actions(Arc<Engine>, seed, offset)` attaches an application-owned executor.
Pooled calls and their `_in` counterparts use the same action implementation. Borrowed
calls retain the exact exclusive DataSession borrow until completion/cancellation; only
the caller commits/rolls back. Returned records own their contents. No model/config lock
is held across await. [Generated ownership tests](../../../framework-models/tests/ownership.rs)
suspend the actual SQLite provider, reenter the same actions, cancel a borrowed read and
reuse its transaction under an external watchdog with a forced-kill control.

## Query, results and maintenance

`StoreQuery` preserves source q/eq/ne/ge/le/like/has/hasnt/sort/columns/skip/take.
Missing paging defaults to take=50/skip=0; zero take is unlimited. Unrepresentable native
paging fails explicitly. Nested maps and arrays flatten to indexed source paths.
Terminal relation/unknown-child selectors retain the last source selector and can fail
at SQL execution. Positive `hasnt`, active=-1 and native interpolation follow upstream.
G46 uses ILIKE only on PostgreSQL; SQLite/MySQL use LIKE. Inherent native store methods
return `SourceStatement`, which renders that spelling. The generic SourceStore trait
returns bare lower SQL ASTs; wrap them in SourceStatement when rendering directly.

Actions implement original preparation, ordered uniqueness checks, create-return behavior,
timestamp input mutation, ID-first upsert, physical deletion and @active removal/restoration.
`batch_values` retains dynamic null/primitive inputs; per-item statuses describe attempts.
It continues after caught errors, then rolls back, keeping those statuses. A missing update
selector is a status without an exception. A cancelled future alone proves no remote outcome.
[SQLite/source oracles](../../../framework-models/tests/sql.rs),
[app extensions](../../../framework-models/tests/extensions.rs) and the
[live replay](../../../framework-models/tests/live_search.rs) pin the observable branches.

Installation/upgrades use the accepted G48 setup selector: MySQL ordered non-atomic DDL
with native error progress; PostgreSQL/SQLite atomic plans. Ordinary Engine transactions
are unchanged. Purge retains PostgreSQL CASCADE, SQLite DELETE and MySQL child-first DELETE
for cascade requests, without changing FOREIGN_KEY_CHECKS. The native adapter's accepted
SQL type contract still governs physical DDL: Date/Time/Datetime use canonical DateTime text,
character declarations use Varchar and floating storage uses Real. Source metadata retains
the original distinctions; this is not a promise of byte-identical driver DDL.

## Ordinary source events

With `server` as well as `source`, `source::register::<Model>` queues the source event
inventory during Listen; each generated module also exposes `register`. This registrar
binds NativeActions to the emitted model schema. Arbitrary custom action implementations
use direct traits rather than being substituted by this native adapter.
`SourceEnvironment` is an explicit native service carrying seed/Date offset; Engine is
an explicit shared service. Registration/startup performs no SQL; absent Engine and
already-completed responses skip just as the source does. Ten event suffixes are available
where model capabilities permit: batch/create/detail/get/purge/remove/restore/search/update/upsert.
[Event tests](../../../framework-models/tests/events.rs) prove default selectors, errors,
request mutation, equal-priority order, duplicate composition and earlier Abort.

Constituent callbacks remain independently usable under G42. This registrar supplies no
global guard, OperationPolicy, default-deny or route arbitration. The existing Typed
`plugin::register` keeps its own contract. Concrete session/API plugin rewiring is M3.
