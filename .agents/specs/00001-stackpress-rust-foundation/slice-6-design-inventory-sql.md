# Slice 6 upstream generated SQL inventory

Companion to [schema inventory](slice-6-design-inventory-schema.md) and [decisions](slice-6-design-decisions.md). Load for every relevant framework SQL export/type/member and source behavior disposition. Sources are `../stackpress/packages/stackpress-sql/src/{index,types,helpers,Migrations,plugin}.ts`, `interface/`, `transform/`, `scripts/`, `events/` and `tests/`, plus [SQL contracts 00025](../../references/00025-upstream-sql-api-contracts.md). No upstream test run or TypeScript ABI parity is claimed.

## Entry exports, store and action families

| Source export/member | Disposition and reason |
| --- | --- |
| Re-exported Alter/Create/Delete/Insert/Select/Update/Mysql/Pgsql/Sqlite/Engine/InquireException and all Inquire query/connection types | Already lower-level Slice 4 owners; consume directly, do not duplicate them in framework runtime. PostgreSQL/MySQL native adapters are Slice 7. PGlite/serverless remains S4-H. |
| Re-exported joinTypes/isIndex/backSlashes/doubleQuotes/escapeBackSlashes/escapeDoubleQuotes/safeJsonValue/jsonCompare | Not ported JS regex/escaping mechanisms. Typed builders/identifier/value/JSON paths already replace them. Structural equality and explicit query conversion remain owned by SQL/schema. |
| `StackpressSqlException` | Now DataError with operation/model/phase context, retained typed causes and no duplicated source Display; host maps disclosure deliberately. |
| `StoreInterface`: relations/table/create/alter/delete/insert/select/update/selectors/joins/paths/scalarize/unscalarize/where | Now generated pure Store/table/read/write plan/codec surfaces plus bounded explicit relation graph. DDL goes to migration planner; no attached pool or unrestricted selector string. |
| Store helpers `toSqlValue/whereEquals/whereLike/whereNotEquals/whereGreaterEquals/whereLessEquals/whereArrayContains/whereArrayNotContains` | Now strict conversion and typed eq/like/ne/ge/le/has/hasnt filters with NULL/list/path semantics. SQL fragments never come from wire strings. |
| `ActionsInterface`: store, constructor(engine,seed), batch/count/create/delete/find/findAll/update/upsert/remove; generated restore | Now generated callable Models functions over explicit DataSession. Restore only for valid active models; hard delete separate. Upsert needs reviewed SQL conflict support; no racy lookup fallback. |
| Action install/purge/uninstall/upgrade | Now explicit maintenance library/tool APIs, not automatic model operations or default app events. Separate ledger and ownership policy. |
| `toSqlString/toSqlBoolean/toSqlDate/toSqlInteger/toSqlFloat` | Now checked storage codecs; intentionally reject silent type defaults, invalid→epoch/zero and false/empty loss. JS coercion compatibility not ported. |
| Deep `flatten`; getAlias/storePathToAlias/storeSelectorToSqlSelector | Now internal bounded typed path/projection/alias mapping with collision checks. No arbitrary nested-object flattening that changes list/null semantics. |
| `transform` default, generateModel/generateModelTypes/generateTypes, package/model/store/actions/events/scripts/tests emitters | Now explicit registered function transforms with disjoint output and typed Rust emitters. Shared index/manifest mutation, TS template constants and module loading not ported. |
| Transform `typemap/getCharLength/getFieldSetup/getNumberInfo/makeCreateQuery/arrangeModelSequence`; stringable/floatable/dateable/boolable/intable/samples | Now normalized type/storage conversion and dependency graph with cycles detected/handled explicitly. No JS stringify/cast inference or source-generator helper ABI. |
| `plugin` idea/listen registration, generated client loading | Now independently registered tool transforms and app operation registration. No runtime import or client-nullability bootstrap; compile-time source/contract selection is G20. |
| Adapter facade mysql/pgsql/pglite/sqlite exports Store/Actions/builders/Engine/types | Existing independent SQL + explicit framework imports replace export duplication. Slice 7 supplies network driver capability; no PGlite/native-mobile support inferred. |

## Every framework SQL type family

| Source types | Disposition |
| --- | --- |
| `ValueScalar/ValuePrimitive`, `StoreWhere/StoreSelectOrWhere` | Now checked wire/model values and typed predicates; no Date/any/raw clause in browser input. |
| `AliasPath/StorePath/StoreSelector/StoreJoin`, `StoreRelation/StoreSelectRelationMap` | Now normalized stable paths/typed relation includes/projection plans; batched reconstruction replaces arbitrary joins/`__` inference from row strings. |
| `StoreSelectFilters` q/eq/ne/ge/le/has/like/hasnt | Now defined allowlisted filters, strict operand/null semantics; no empty-predicate widening. |
| `StoreSelectQuery` columns/sort/skip/take; `StoreSearchQuery` total | Now typed projections/sort/page/bounded count contract. To-many sort/implicit recursive wildcard expansion rejected. |
| `SerializedEvent` event/data | Default population replaced by typed seed functions in one session. Arbitrary sequential event seeding is an owner alternative with partial-progress/idempotency limits. |
| `DatabaseConfig` seed/migrations/schema.onDelete/onUpdate/populate; `DatabasePlugin` | Now explicit tool/runtime settings and Engine service. Crypto seed removed pending S6-Security; FK actions typed/reviewed, defaults RESTRICT rather than source's unimplemented CASCADE TODO. No request-controlled DB selection. |
| `GenericEventHandler/GenericEvents/GenericListener` | Now named Send handlers with current Request/Response/Invocation and function registration. JS alias props/void promises not ported. |
| `GenericAdminRouter` | Slice 8 admin generation; external access policy still app/security owner. No automatic routes from a model. |
| `ClientModel/ClientScripts/ClientPlugin/Client` | Now static generated contracts/stores/operations and explicit operator library functions. Dynamic ESM client factory/nullable loader and runtime seed constructor not ported. |
| Generated Model/ModelInput/ModelExtended/ModelRelations/ModelStoreInterface/ModelActionsInterface/column types | Now Rust full-row/create/patch/key/query/finite include DTOs; body types and public projection separate. Contracts independent of drivers/server/tool. |
| `Migration/MigrationRevision`, `MigrationRename/MigrationRenameAmbiguity/MigrationRenamePlan`, `InspectedSchemaChanges` | Now structured revision/plan/rename/risk data plus authoritative applied ledger. No automatic inferred history→applied-state equivalence. |
| Deep `DestructiveAlterBuild/DestructiveAlterChanges/DestructiveAlterChangeSet/DestructiveSchemaChanges` | Now typed aggregated loss/constraint findings and policy-independent inspection. Allowing execution remains operator policy, not renderer permission. |

## Migrations, operations and scripts

| Source surface | Disposition and behavior |
| --- | --- |
| Migrations constructor/database/revisions/all/at/latest | Now pure MigrationPlanner + explicit RevisionStore, can plan baseline/adjacent chain. Runner starts from DB applied head, not merely last two source snapshots. |
| formatAmbiguousWarning/formatDestructiveWarning/formatWarning | Now structured RiskFinding plus bounded human formatter, no source text matching. Artifact generation can report warnings without executing. |
| getDestructiveAlterChanges/hasDestructiveAlterChanges/hasDestructiveChanges/inspectSchemaChanges | Now compare complete typed tables/constraints, account for rename-equivalent constraints and aggregate findings. Metadata-only changes are no-op storage plans. |
| isNoAlterationsError | Not ported message matching; existing SchemaDiff::NoChanges and framework NoChanges handle it. |
| planRenames | Now suggestions + explicit one-to-one RenameMap, subject to owner choice on automatic clear matches. Ambiguous shape guesses cannot be authorized by generic force. |
| Generated events batch/create/detail/get/purge/remove/restore/search/update/upsert | Now normal selected data operations; purge is tool-only by default. Explicit public projection/policy required for exposed handlers. No automatic HTTP/admin route. |
| Package events install/migrate/purge/push/query/populate/uninstall/upgrade | Now explicit tool events db-install/migration-plan/db-purge/db-populate/db-uninstall/db-upgrade; status/downgrade added for ledger. Generic query and auto install-or-upgrade push not ported by default because of exposure/destructive ambiguity. |
| Matching scripts install/migrate/purge/push/populate/uninstall/upgrade | Now pure planner or explicit maintenance runner called by app-owned tool; no shared CLI edit. Migration artifact writing remains policy-distinct from application. |
| Install uninstalls then reinstalls | Deliberately not ported. Initial install refuses conflicting managed schema; destructive reset is separate uninstall/purge with exact plan approval. |
| Upgrade force permits ambiguous raw add/drop and removals | Replaced by plan-hash-specific data-loss option and mandatory explicit rename/backfill resolution. No waiver of drift/integrity. |
| Populate resolves arbitrary events sequentially, no outer transaction | Recommended typed atomic DB seed steps now; alternative partial-progress event mode needs explicit owner choice. Never pretend an outer Engine transaction is inherited. |
| Revisions alone determine latest migration | Replaced by exact parent chain and actual applied-state ledger; stale/incomplete history refuses. |
| SQLite unsupported type/key/FK alteration | Now framework reconstruction through reviewed adapter maintenance API. Independent ordinary ALTER remains strict; unknown objects block rather than disappear. |
| MySQL nontransactional DDL execution/recovery | Later S6-N migration protocol; render inspection now, network CRUD via Slice 7. Never ignore existing execute_plan refusal. |

## Individual source test mapping

The two framework SQL test files contain 23 tests, all mapped below by exact declaration order/title. “Now” means a required proof after approval. All generated store/action/event test templates are mapped separately; template comments are not executed tests.

| File/order and exact source title | Proposed proof/disposition |
| --- | --- |
| helpers 1: should coerce values into SQL-safe primitives | Replace coercion with checked supported codecs; prove null/missing/false/zero/empty/list/JSON distinction and rejection. |
| helpers 2: should coerce dates and fall back invalid values to epoch time | Not ported epoch fallback; invalid date/time rejected and canonical valid roundtrip proved. |
| helpers 3: should flatten nested records and optionally expand arrays | Now bounded schema-directed JSON/relation paths without lossy arbitrary flattening. |
| helpers 4: should derive aliases and selector strings for nested store paths | Now deterministic typed paths/aliases with keyword/case/length/collision tests. |
| helpers 5: should collect removed alter members and detect destructive changes | Now typed complete findings from real SchemaDiff. |
| helpers 6: should format an aggregate destructive warning message | Now all affected tables/fields/constraints included; no bind values in warning. |
| helpers 7: should treat empty removals as safe | Now NoChanges plus no ledger mutation for no-op. |
| helpers 8: should detect destructive removals from real builder diffs | Now actual table/drop/constraint loss classification and refusal. |
| helpers 9: should return destructive SQL and warning metadata together | Now plan inspection includes SQL/binds/risks, without application. |
| helpers 10: should treat real no-op diffs as safe across builder dialects | Now all three renderers; no fabricated native execution claim. |
| helpers 11: should plan a clear one-to-one same-semantics field rename | Now suggest candidate; explicit map reconciles via core rename and preserves real rows. Automatic acceptance is owner alternative. |
| helpers 12: should refuse to guess across multiple same-shape candidates | Now refusal before effects even with data-loss flag. |
| helpers 13: should delegate rename SQL to every supported Inquire dialect | Now core delegation/render tests; execute SQLite, network tests only when gated adapters available. |
| helpers 14: should allow equivalent renamed keys and constraints | Now classify equivalent replacements without spurious loss; reject genuinely weakened constraints. |
| upgrade 1: should preserve a clear field rename through Inquire | Now explicit-mapped rename via real core/native SQLite and reverse proof. |
| upgrade 2: should refuse ambiguous field rename candidates | Now unresolved map refused; no partial schema/ledger changes. |
| upgrade 3: should refuse destructive field removals when force is false | Now default refusal and unchanged rows/schema/ledger. |
| upgrade 4: should execute destructive alter SQL when force is enabled | Replace broad force with exact hash + data-loss option; mismatch still refuses. |
| upgrade 5: should refuse dropped tables when force is false | Now dropped models produce destructive plan, not automatic prune/drop. |
| upgrade 6: should report all destructive changes in one warning | Now aggregated findings before acquisition/effects. |
| upgrade 7: should write a clear field rename through Inquire | Now inspectable checked artifact, deterministic bytes and binds. |
| upgrade 8: should write ambiguous rename candidates as raw SQL | Now unresolved structured artifact may be written for review, marked non-executable until resolution; no silently approved raw drop/add. |
| upgrade 9: should write destructive migration SQL without requiring force | Now inspection/artifact creation allowed with warnings; execution separate and refused by default. |

Generated `transform/tests/store.ts` seven behaviors (count/alter/delete/insert/select/update/where), `actions.ts` seven (create/batch/search/get/update/remove/restore), and `events.ts` seven matching operations are all Now with actual generated compilation, SQLite execution and typed error/denial cases. Source `samples` scalar/date/JSON/list/optional/relations expand the type matrix, not just the two-field Profile proof. Conditional remove/restore uses valid active models; unsupported output has a compile/generation failure proof.

The older [Slice 4 integration inventory](slice-4-design-tests-integration.md) maps the same framework tests as S4-F/M; this checkpoint supersedes only their proposed destination, not its historical evidence. Already implemented core builders and adapter tests remain Slice 4 ownership. New Slice 6 tests add ledger/hash/drift/concurrent migrator, reconstruction/cancellation, N+1/pagination, transaction composition, disclosure, generated byte/feature drift and strict external query policy.
