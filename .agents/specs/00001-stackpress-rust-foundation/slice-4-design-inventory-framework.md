# Slice 4 adapters, schema and framework inventory

Companion to [decisions](slice-4-design-decisions.md). Load for adapter and framework breadth, exact later ownership and schema-to-store generation. Source is Inquire `039a1b5` and Stackpress `a71d683051ba8350fdd12d6b5a33f268fdcc285f`, read-only; contracts [00025](../../references/00025-upstream-sql-api-contracts.md), [00026](../../references/00026-upstream-database-adapter-contracts.md), [00013](../../references/00013-upstream-modeling-and-generation.md), [00021](../../references/00021-upstream-schema-api-contracts.md), [00041](../../references/00041-framework-package-supplements.md) retain source-system authority only.

Dispositions follow the recommended S4-D02 scope; the owner may enlarge it. `Now` is proposed, not implemented. S4-F/M/N/H/X are named in the decision router.

## Adapter packages: every export/type and common behavior

| Source package / surface | Disposition and reason |
| --- | --- |
| `inquire-sqlite3`: default/named `connect`, default/named `Connection` / `BetterSqlite3Connection`, `Results`, `Resource`, `Connector` | Now native `stackpress-sql-sqlite` open/options/database adapter, statement-local result metadata. SQLx replaces Node addon distribution; native resource type is not a better-sqlite3 compatibility wrapper. |
| `inquire-pg`: default/named `connect`, `Connection` / `PGConnection`, `Results`, `Resource`, `Connector` | Later S4-N: real network PostgreSQL driver/pool, TLS and service fixture must be proved. The pure Pgsql renderer is now. |
| `inquire-mysql2`: default/named `connect`, `Connection` / `Mysql2Connection`, `Results`, `Resource`, `Connector` | Later S4-N: network MySQL adapter with acquired-connection transaction affinity and server tests. Pure Mysql renderer is now. |
| `inquire-pglite`: default/named `connect`, `Connection` / `PGLiteConnection`, `Results`, `Resource`, `Connector` | Later S4-H: embedded PostgreSQL/WASM host, persistence/extensions/concurrency need separate assessment. PostgreSQL syntax is insufficient evidence. |
| Each connection constructor, `dialect`, `query`, `format` | Now SQLite: eager validated open, owned binding without mutating query input; bool→integer, explicit JSON→text, date→explicit canonical text. Others later with their package above. |
| Each `resource` live object or lazy async zero-arg factory/cache | No JS runtime factory or implicit cached mutation. Now an owned database opened once explicitly; native-resource interop Later S4-X with exclusive leases and cleanup authority. |
| Each `raw` and native `Results` | SQLite native metadata can be exposed as adapter-specific owned outcome, never a driver lock/raw handle. Other native shapes Later S4-N/H. Generic normalized rows remain independent. |
| Each `lastId` getter, SQLite lastInsertRowid, MySQL insertId, PostgreSQL undefined | No shared mutable last-ID slot. Now optional ID on the exact statement result; prefer RETURNING. MySQL connection-local semantics Later S4-N. |
| Each `before` getter/setter and final-request mutation | Later S4-X, same hook decision as core. No reentrant hook runs while a pooled connection or application lock is retained. |
| Each `transaction(callback)` with begin/commit/rollback | Now SQLite via exclusive guard on one lease; cancellation/drop/error behavior explicit. Others later; never pool-query BEGIN and unrelated pool-query COMMIT. |
| `?` counting, PostgreSQL `$n` conversion and `??` escape | No global string replacement. Typed renderers emit native placeholders; raw SQL is dialect tagged, with prepared metadata arity checks. Question marks inside strings/comments/operators remain untouched. |
| Arrays/objects stringify; Date ISO; SQLite bool integer | Now explicit encoding without input mutation, no automatic row JSON/date decoding. Fallible typed read conversion; generated model decoding later S4-F. |
| SELECT/INSERT RETURNING textual branch in SQLite | No prefix-based result detection. Now native prepared row metadata handles WITH/leading comments and UPDATE/DELETE RETURNING correctly. |
| No shared shutdown in TS adapters | Now `close().await`, closed errors, bounded close wait and documented Drop fallback. No new global Server shutdown protocol. |

## Framework entry exports and generated interfaces

Source: `stackpress-sql/src/{index,types,helpers,plugin,Migrations}.ts`, `interface/*`, `transform/*`, `events/*`, `scripts/*`, four adapter facades and package export map. Lower-level re-exports in framework root/facades map exactly to the [core inventory](slice-4-design-inventory-core.md); Cargo uses direct imports/optional facade modules rather than duplicating every symbol.

| Every relevant framework export/type/member | Disposition and reason |
| --- | --- |
| `Alter/Create/Delete/Insert/Select/Update/Mysql/Pgsql/Sqlite/Engine`, `InquireException`, all re-exported Inquire helpers/types including `SelectColumn` | Now lower-level family only. Root convenience is `stackpress::sql` opt-in; no adapter transitively selected by default. `SelectColumn` becomes explicit SQL Column to avoid schema Column ambiguity. |
| `StackpressSqlException` | Later S4-F: framework contextual error over standalone SQL cause. Existing lib Error::from_source suffices for the native host proof. |
| `ActionsInterface`, `StoreInterface` | Later S4-F: no generic ORM promise before schema/value/assertion/storage semantics exist. Profile sketch illustrates the boundary only. |
| Store `relations/table/create/alter/delete/insert/select/update/selectors/joins/paths/scalarize/unscalarize/where` | Later S4-F. Relations/aliases must be schema-derived, cycle bounded and collision checked; create/alter consume the independent SQL definitions/diff. |
| Generated extra store `toSqlValue/whereEquals/whereLike/whereNotEquals/whereGreaterEquals/whereLessEquals/whereArrayContains/whereArrayNotContains` | Later S4-F: typed, allowlisted filters and conversion, not arbitrary client-controlled SQL selector strings. |
| Action `store`, constructor engine/seed, `batch/count/create/delete/find/findAll/install/purge/remove/restore/uninstall/upgrade/update/upsert` | Later S4-F; install/upgrade policy additionally S4-M. `restore` exists in generated source although missing from ActionsInterface. Preserve that mismatch as source evidence, not an omitted target operation. |
| `toSqlString/toSqlBoolean/toSqlDate/toSqlInteger/toSqlFloat` | No silent zero/epoch fallback in core. Later S4-F explicit model conversions; legacy compatibility may be deliberately added S4-X. |
| `flatten` (helpers subpath), `getAlias/storePathToAlias/storeSelectorToSqlSelector` | Later S4-F, bounded schema-owned flattening and stable collision-checked paths. No need to export JS-shaped helpers from core. |
| `ValueScalar`, `ValuePrimitive` | Now standalone typed SqlValue covers binding; framework model-specific compatibility Later S4-F. |
| `AliasPath`, `StoreJoin`, `StorePath`, `StoreRelation`, `StoreSelector`, `StoreSelectRelationMap`, `StoreWhere`, `StoreSelectOrWhere` | Later S4-F: typed relation traversal and path/alias rebuilding require normalized schema relations. Include deep types subpath, not only root exports. |
| `StoreSelectFilters` (`q/eq/ne/ge/le/has/like/hasnt`), `StoreSelectQuery` (`columns/sort/skip/take`), `StoreSearchQuery` (`total`) | Later S4-F: allowlisted searchable/sortable/filterable paths, nullable/list semantics and totals snapshot contract. Native example exposes only its explicit id/name search. |
| `SerializedEvent`, `DatabaseConfig` (`seed/migrations/schema.onDelete/onUpdate/populate`), `DatabasePlugin` | Later S4-F/M: don't add seed/crypto/population fields with no implementation. Now app-owned typed SQLite options in existing config section and Engine service registration. Source cascade config contains TODO, not proven application behavior. |
| `GenericEventHandler`, `GenericEvents`, `GenericListener`, `GenericAdminRouter` | Later S4-F; admin exposure further belongs to a named future framework-admin task. Existing named handlers and PluginSpec already supply registration, without JS props aliases. |
| `ClientModel`, `ClientScripts`, `ClientPlugin`, `Client` | Later S4-F/M: explicit generated Rust module registry, compile-time selection and independent codegen. No async JS client-module loader in Rust. |
| `Migration`, `MigrationRename`, `MigrationRenameAmbiguity`, `MigrationRenamePlan`, `MigrationRevision`, `InspectedSchemaChanges` | Later S4-M: schema revision comparison plus explicit rename intent/warnings, not inferred database state. |
| `DestructiveAlterBuild`, `DestructiveAlterChanges`, `DestructiveAlterChangeSet`, `DestructiveSchemaChanges` | Later S4-M for user/operator policy; pure core diff retains explicit removals now so later tooling can inspect without parsing emitted SQL. |
| `Migrations` constructor/database/revisions, `all/at/latest` | Later S4-M: revision history acquisition/plans are separate from executing applied migrations. |
| `Migrations.formatAmbiguousWarning/formatDestructiveWarning/formatWarning/getDestructiveAlterChanges/hasDestructiveAlterChanges/hasDestructiveChanges/inspectSchemaChanges/isNoAlterationsError/planRenames` | Later S4-M: preserve warning/refusal distinctions and unambiguous one-to-one same-semantics rename delegation. Use typed NoChanges, not message matching. |
| `events` exports `install/migrate/purge/push/query/populate/uninstall/upgrade` and corresponding `scripts` (all except query) | Later S4-M explicit operator workflows. No global raw query event now. Never authorize purge/migrate from ordinary HTTP merely because the operation exists. |
| `plugin` listen-phase generated model registration and idea-phase transform contribution | Later S4-F/M explicit selected functions with owned partitions. Now app-owned `database` plugin proves SQL service integration via existing ServerBuilder. |
| Adapter facades `mysql/pgsql/pglite/sqlite` re-export engine/builders/types/store/actions | Now independent core + SQLite adapter imports; optional root modules. Framework facades Later S4-F/N/H according to actual implementation, no fake empty packages. |
| Generated `ModelRelations/ModelStoreInterface/ModelActionsInterface`, model/input/extended/column/type exports | Later S4-F: compile-time typed contracts produced with the owning normalized schema; no driver edge in public DTOs. |
| `transform` default, package/model/store/actions/events/scripts generation, `generateModel/generateModelTypes/generateTypes` | Later S4-F/M using existing Registry/TransformSpec/OutputPlan APIs. ts-morph/ESM/CJS/package export rewrites not ported. |
| Transform helpers `typemap/getCharLength/getFieldSetup/getNumberInfo/makeCreateQuery/arrangeModelSequence`; `stringable/floatable/dateable/boolable/intable`, generated test `samples`, `TEMPLATE` constants | No JS template/compiler API. Their functional scalar mapping and emitted tests belong to S4-F; ordinary typed Rust emitters suffice, with no general template engine introduced. |

## Schema-to-store responsibility contract

Idea parser/transformer currently produce ordered typed syntax/metadata and function-registered owned output. They do not interpret IDs, defaults, assertions, table/column names or relations. Current generated-profile accepts exactly `Profile { id String name String }`, rejects unsupported metadata and uses an example-owned in-memory template. Its README/output are evidence for this narrow shape, not an implementation of framework-schema.

The future S4-F pipeline is: composed/finalized Idea → normalized schema lenses → persistence model → SQL definition/store/action/event source → generated module registry → explicit host registration. Syntax remains Idea-owned; validation/defaults/name/store facets are schema-owned; SQL owns persistence/render/execution; public/UI/auth exposure remains the relevant surface owner. Unknown metadata cannot silently imply persistence or access control.

S4-F must settle every type (String/Text/Integer/Float/Number/Boolean/Date/Time/Datetime/Json/Object/Hash, enums, nested fieldsets, required/optional/list, relations), @id/unique/index/generated/default/active/timestamp/name/storage fields, precision/range, serialization and missing/NULL behavior. UUID/cuid/nanoid/random/now are explicit generation/default strategies. No deterministic-IV encryption is copied as a recommendation.

S4-M then adds schema signatures/revisions, explicit rename reconciliation, ambiguous/destructive warning/refusal/force, reviewable raw migration artifacts and an independently designed applied-state ledger. Source migration artifact writing intentionally lacks the same refusal policy as upgrade; source populate emits sequential events without an outer transaction. Preserve these distinctions while obtaining owner decisions for the Rust operator contract.

Generation must reuse G20's independent codegen, owned partitions/single aggregate writer, repeat stability, `--check`, selection/revision compatibility and promised stale-file cleanup. Neither Cargo compilation nor existence of a snapshot applies database schema. Generated types/validation usable in UI must not acquire drivers, pools, host services or generator tooling.
