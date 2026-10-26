# Slice 4 upstream core inventory

Companion to [decisions](slice-4-design-decisions.md). Load for the relevant exports, types, methods and behavior from Inquire `packages/inquire/src`, inspected read-only at `039a1b5f41500aa63286a9f9b34f4c7f71b990a0`. Source entrypoints are `index.ts`, `types.ts`, `helpers.ts`, `Engine.ts`, `Exception.ts`, `builder/*.ts`, `dialect/*.ts` and package.json export paths. No upstream build/test was run. Tests are individually mapped in [core tests](slice-4-design-tests-core.md).

`Now` means recommended implementation after approval, not implemented. `Later S4-*` names the follow-ups in the decision router; `No` means a deliberate non-port. Types grouped in one row all have the same disposition and reason. Rust names may differ, but functional omissions must remain explicit.

## Entry exports and builders

| Export / public members | Disposition and reason |
| --- | --- |
| `Engine` constructor, `connection`, `dialect` | Now: owned `Engine` over a Send+Sync database adapter; inspect dialect, explicit adapter access only through backend-specific APIs. No raw mutable connection on the shared engine. |
| `Engine.alter/create/delete/insert/select/update` | Now: convenient detached builders; standalone constructors also work. No engine ownership stored in query data. |
| `Engine.query(QueryObject)`, `query(sql,values)` | Now: explicit `query`, `fetch`, `execute` with owned bound query data, limits and statement-local results. Raw SQL is dialect tagged and trusted-author only. |
| `Engine.drop/rename/truncate` | Now: checked identifiers and capability-specific plans. SQLite truncate uses DELETE, does not reset sequence, rejects cascade. No empty-name success. |
| `Engine.diff(from,to)` | Now: pure structural `SchemaDiff`, same-table check, no live introspection and no inferred rename. Compare ordered compound keys and every field property by value; never JS array identity. |
| `Engine.transaction(callback)` | Now underlying atomic behavior through explicit begin/commit/rollback guard; callback syntax Later S4-X because higher-ranked borrowing ergonomics need their own proof. |
| `Engine.before` getter/setter, mutable hook and results short-circuit | Later S4-X: approval required for rewrite, ordering, synthetic results and transaction semantics. Read-only explicit observation can be implemented now; never expose bind values by default. |
| `Engine.sql` template segments/backtick replacement | No JS tagged-template API/global backtick substitution. Now explicit raw literal with bind plus typed expression composition; a future Rust macro is optional S4-X. Quotes inside literal values are never rewritten. |
| `Create` constructor, `addField/addForeignKey/addKey/addPrimaryKey/addUniqueKey`, `build/query` | Now: typed fields, deterministic constraint/index order, inspectable immutable definition and fallible dialect plan. Standalone render requires a dialect argument. |
| `Alter` constructor including source fields, all Create add methods; `changeField/renameField/removeField/removeForeignKey/removeKey/removePrimaryKey/removeUniqueKey`, `build/query` | Now: typed change plan with checked conflicts and rename reconciliation. SQLite rejects operations it cannot execute natively before executing any part. Full reconstruction Later S4-M. |
| `Insert` constructor, `values(record/records)`, `returning`, `build/query` | Now: one/bulk rows, stable column order, equal key sets or explicit defaults, checked bind limits and explicit returning columns/all. Empty input errors. |
| `Select` constructor default `*`; `select/from/join/order/limit/offset/where/whereJson/whereJsonContains/build/query` | Now: typed identifiers/aliases, explicit raw expressions, ordered joins/filter groups/JSON paths/sort/page. Default ascending; Some(0) means zero rows, absent means unbounded until fetch budget. |
| `Update` constructor, `set/where/whereJson/whereJsonContains/build/query` | Now: checked nonempty assignments, bound values and JSON filters; unfiltered mutation requires explicit `all_rows()`. |
| `Delete` constructor, `where/whereJson/whereJsonContains/build/query` | Now: bound filters and explicit `all_rows()` for intended whole-table mutation. An empty raw condition is rejected, not permission to delete everything. |
| All builders' `engine` setter/getter and optional engine constructor | No: builders are pure owned query descriptions and can be reused against deliberate executors. Reattaching a shared engine would conceal transaction affinity. |
| All builders' `then(resolve,reject)` and awaitable semantics | No: explicit Rust futures arise only at execution. Awaiting a detached builder or rendering without a dialect is prevented by API shape, not late thrown exceptions. |
| Create/Alter multiple statements in transaction, last rows result | Now atomic batch execution on one guard; return per-statement outcomes with an optional last-result convenience. MySQL DDL atomicity is not promised; gate transactional execution by dialect capability. |
| Unique/index same-name precedence | Now as explicit normalization: a unique definition wins over a regular index with identical fields; incompatible same-name definitions error. No accidental insertion-order weakening. |
| Rename remove/add reconciliation | Now: reject identical source/target, duplicate targets, missing halves and missing source definition. Same-semantics remove/add becomes rename; changed semantics retain an additional change operation, which SQLite may reject. |

## Every core type export

| Upstream type | Disposition and reason |
| --- | --- |
| `Field` | Now: typed SQL type/default/nullability/length/auto-increment metadata. Unknown SQL type/attribute must be explicit trusted SQL, not unvalidated string interpolation. SQLite-only unsupported annotations error unless documented as storage affinity metadata. |
| `ForeignKey` | Now: typed local/foreign table/column and action enums; support compound form deliberately in Rust. Invalid SET NULL vs NOT NULL combinations error. |
| `AlterFields`, `AlterKeys`, `AlterUnqiues`, `AlterPrimaries`, `AlterForeignKeys` | Now: correctly spelled Rust change types, value equality, ordered add/change/remove/rename data. Preserve behavioral contract, not the `Unqiues` typo. |
| `Column`, `Selector`, `Table` | Now: separately checked identifier segments, optional qualification/alias and explicit wildcard/expression variants. A dotted string is not a SQL expression parser. |
| `JoinType`, `Join`, `OrderType`, `Sort` | Now: enums with validated operands/direction. CROSS has no ON; capability checks reject unsupported joins rather than claiming all engines accept them. |
| `Where`, `WhereJson`, `WhereBuilder` | Now: structured `Expr`, bounded filter groups and typed JSON key/index paths; Select/Update/Delete reuse one mechanism. Native raw fragment has an explicit trust boundary. |
| `StrictValue`, `StrictOptValue`, `FlatValue`, `JSONScalarValue`, `Value` | Now: `SqlValue` and explicit JSON values. Exact i64, finite f64, bool/text/blob/NULL; absence/default separate from NULL. No JS Date or undefined union. |
| `Resolve`, `Reject` | No: standard `Future<Output=Result<T,SqlError>>` replaces callbacks and rejection swallowing. |
| `JsonDialect` (`selector/extract/contains/where`) | Now: typed JSON rendering outputs with bound paths/values or correctly escaped structural literals; `contains` means direct scalar membership at the chosen array path, not recursive object containment. |
| `Dialect` (`name/q/separator/splitter`, all render methods, `json` overloads) | Now: pure Sqlite/Postgres/MySql renderer plus explicit capability metadata. No mutable global singleton. Strings `:`/`.` may be parsed by a checked convenience constructor; typed paths are authoritative. |
| `OrQueryObject` | Now: typed OR expression group, preserving parentheses and bind order, no empty-group widening. |
| `QueryObject` | Now: immutable rendered query, dialect identity and parameters, inspected via accessors; private fields preserve binding invariants. Ordinary query clones are independent. |
| `Transaction` | Now: exclusive non-Clone guard, Send query futures borrowing `&mut`, consuming commit/rollback, rollback-on-drop; no process/task-global current transaction. |
| `Connection` including `format/query/resource/transaction`, `lastId/before` | Now split shared `Database` from exclusive transaction/lease; prepared binding and normalized results. Generic raw resource/getter and lazy resource factory Later S4-X. Shared lastId not ported; native results are adapter-specific and statement local. |

## Every dialect/helper/exception export

| Export / public surface | Disposition and reason |
| --- | --- |
| `Sqlite`, `Pgsql`, `Mysql`; `alter/create/delete/drop/insert/json/rename/select/truncate/update` | Now renderers and snapshots; only SQLite actual execution. PostgreSQL/MySQL execution Later S4-N. Unsupported clauses fail as typed `Unsupported` before effects. No one-dialect SQL masquerading as parity. |
| `Json` base class, public `separator/splitter` setters/getters | No inheritance/global mutability; Now per-query `JsonPath`/parse options and shared rendering helpers. |
| Dialect static `parse` helpers | Now checked selector parsing, explicit alias and JSON key/index path; reject ambiguity/invalid input. No comma-splitting raw function expressions or regex replacement across SQL string literals. |
| JSON dialect `selector/extract/contains/where` | Now extraction and direct scalar array membership; tests cover missing/NULL/type distinctions and nested arrays. General object containment Later S4-X. |
| `joinTypes` | Now `JoinKind` enum and private renderer mapping; no public mutable dictionary. |
| `isIndex` | No regex export; Now numeric `JsonSegment::Index` with checked range and distinct key `"01"`. |
| `backSlashes`, `doubleQuotes` | No JS regex exports; SQL safety comes from binding/identifier quoting, not blanket escaping. |
| `escapeBackSlashes`, `escapeDoubleQuotes`, `safeJsonValue` | No standalone escaping API. Now serde_json/path encoders at their owning boundaries; bind data rather than splice escaped values. |
| `fieldCompare` | Now Rust structural equality for every field property, including composite length values; pure diff tests. |
| `jsonCompare` | No stringify/order-sensitive universal equality; Now explicit typed structural equality and ordered-key semantics where order matters. |
| `Exception` / default `InquireException` | Now standalone `SqlError` implementing Error/source with stable kinds and no core dependency on lib. Framework conversion preserves causes and sanitizes public output. |

## Dialect capability boundary

`Now` does not mean reproducing every generated upstream string. Preserve functional intent while fixing unsafe/invalid output: SQLite cannot alter column type or add primary/foreign constraints in place; table-copy reconstruction is S4-M. Renderers reject unsupported returning (notably MySQL), cascade semantics, incompatible auto-increment, invalid type/length/default and empty statements. A plan is wholly validated before its first statement executes.

Explicit empty/zero/false defaults survive. SQL default expressions use an enum such as CurrentTimestamp or an explicitly trusted expression; no arbitrary function-name uppercasing. Typed identifiers quote embedded dialect quote characters, reject NUL/control/empty segments and preserve literal dots via explicit segment APIs. JSON-path data never becomes executable SQL.

No automatic schema introspection, ORM identity map, hidden population or migration effects are added. See [execution](slice-4-design-execution.md) for the result, transaction and memory/resource contracts.
