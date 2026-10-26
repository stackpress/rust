# Slice 6 stores, queries and generated operations

Proposed runtime contract behind [the walkthrough](slice-6-design-dx.md). Load before implementing `framework-sql`. Standalone builders/Engine remain the current `stackpress-sql` implementation; the following data-layer names are sketches to prove at M0.

## Executor and transaction ownership

Generated `ModelStore` constructs inspectable typed read/write plans and decodes owned rows; generated plural `Models` functions execute those plans. Neither stores an Engine, a current transaction, a mutable identity map or a global connection. `DataSession<'a>` is an exclusive enum wrapping `&'a Engine` or `&'a mut Transaction`. Its methods borrow `&mut self`, call the exact underlying executor, and return Send futures bounded by that borrow. It is non-Clone and not a host service. An additive `Transaction::dialect()` getter is requested through the coordinator for dialect-aware plan preparation.

For a caller-owned transaction all store calls use the same session across await. No shared reference, mutex guard, task-local transaction, unsafe lifetime conversion or nested pool acquisition is introduced. `DataSession::pooled` permits independent single-statement calls; multi-statement mutation/search-with-relations/count promises require one explicit transaction. Provide a convenience `Models::..._atomic(&Engine, ...)` only for real multi-statement workflows; it opens one guard, calls the same session functions, commits once or awaits rollback with retained cleanup failure.

The library never commits because Flow::Continue was returned. A later listener or response serialization can fail after a DB commit; operation success and lifecycle success are separate facts. Atomic domain work belongs inside an app-owned transaction before producing an owned response. Nested emit/resolve does not inherit a transaction, identity or authorization. To compose atomic operations, call generated domain/store functions with the same session, then optionally dispatch outside the transaction. Existing finite pool/busy budgets still apply when an app deliberately reacquires while holding a guard.

## Generated callable surface

| Method family | Input/result and behavior |
| --- | --- |
| `get(key)`, `find(query)`, `find_all(query)` | Owned Option<Model>/Vec<Model>; find returns at most one in deterministic order. Missing is distinct from SQL/decode error. Full records and selected projections have separate types. |
| `search(query)` | `Page<T> { rows, total: Option<u64>, skip, take }`; total is matched base rows before paging, not number returned. Zero remains zero. A typed request chooses whether count is needed. |
| `count(filters)` | Counts base identities without include fanout; typed filters only. |
| `create(input, values)` | Validate, fill declared defaults/generated values once, encode, insert, decode/reload authoritative row. Use RETURNING where supported; otherwise known supplied/generated key or same-statement insert ID followed by same-session read. |
| `update(key, patch, values)` | Key-scoped, nonempty typed patch. Immutable IDs/generated fields/relation objects rejected. Zero matches returns None; timestamps updated only as documented. |
| `delete(key)` | Explicit hard delete. FK policy initially RESTRICT; cascading requires an explicit typed schema/tool policy and a destructive plan review. |
| `remove(key)`, `restore(key)` | Generated only for valid Boolean-active models. Idempotently set active=false/true; ordinary reads select active=true, maintenance reads must explicitly choose inactive/all. |
| `batch(inputs, mode)` | Create/update/upsert modes; finite count/bytes, whole-batch validation before effects. Atomic variant owns one transaction; transaction-session variant leaves commit to caller. Per-item outputs retain input order. |
| `upsert(key, input, patch)` | Conflict target is the explicit primary/unique key, never arbitrary filter. Prefer dialect-native atomic upsert after renderer proof; initial fallback is not a racy find-then-insert. Return Unsupported until an atomic implementation is available. |
| `table`, `select`, `insert`, `update_plan`, `delete_plan`, `decode`, `encode` | Pure inspectable plans over SQL AST. DDL/migration/population methods are separate operator APIs, not normal model mutations. |

Upsert requires a small reviewed SQL builder extension because current Insert has no conflict clause; this is an explicit shared-core request, not permission to splice caller input into raw SQL. A withheld extension keeps upsert unavailable with a clear inventory limitation. Compound identity works throughout get/update/delete and total counting; implicit many-to-many storage is not added.

Initial updates are key-scoped last writer wins. There is no expected-version field or optimistic-lock conflict result unless the owner separately adds that behavior. Slice 8 must present the actual write/timeout outcome and can refetch for display; refetch does not recover a lost write or authorize automatic retry. A future compare-and-swap operation needs a version-field/type/SQL predicate contract and concurrency tests.

## Filter, sort and page semantics

Generate enum selectors and typed values for direct Rust calls. Wire search is strict JSON decoded to that same allowlisted representation; field/operator/direction strings never become SQL fragments. Defaults: take=20, maximum=100, skip bounded by configured maximum (initial 100,000), relation depth at most 3 and include paths at most 16; budgets are lowerable. Empty search means a bounded page, while mutations always require a key or a separately named bulk method with explicit predicate.

Supported source filter families have explicit mappings: `q` searches declared searchable text fields; `eq/ne/ge/le` perform typed scalar comparisons; `like` is a documented pattern operator on eligible text fields with bound patterns and escaped literal-search helpers; `has/hasnt` means direct JSON-array scalar membership. Missing JSON path, SQL NULL and JSON null are distinct. Null equality uses IS NULL/IS NOT NULL; ordinary comparisons with null are rejected. Empty groups are rejected rather than widened. Regex matching is input validation, not a database query operator.

`q` initially uses case-sensitive literal text containment with per-dialect rendering/tests, not locale folding, FTS ranking or raw SQL LIKE defaults. Empty q adds no predicate. Sort is allowed only for declared eligible scalar paths, appends the complete primary key as a deterministic tie-breaker, and specifies null order explicitly. To-many relation sort is rejected without a named aggregate. No silent sort direction fallback. Offset pagination may change across separate calls when data changes; keyset pagination is a named future extension. The dialect compiler may use narrowly audited constant expressions containing only validated identifiers and bound values for containment/count features not yet expressed by a builder; that does not admit client SQL or relax ordinary-command policy.

Field `filter.*` UI roles describe control presentation; the query allowlist is a separate generated policy. The initial schema derives eligible scalar filter candidates from explicit roles/keys, then the app's exposure narrows them. Direct stores can deliberately use additional typed paths; external handlers cannot accept undeclared paths. `@query` is a default selected projection, not an exposure grant.

Exact total and included relations are read in the same explicit transaction for the SQLite proof. Other adapters must document actual isolation: ordinary PostgreSQL READ COMMITTED does not give a fixed multi-statement snapshot. Until Slice 7 provides a validated repeatable-read/read-snapshot contract, network search reports rows and count as observations without promising one snapshot, or refuses the requested consistent mode. Do not silently claim snapshot consistency from one transaction alone.

## Relations without hidden N+1 work

Owning singular relation maps local FK→referenced unique key; optionality must agree with nullable local FK. Reverse one/many cardinality is validated from that mapping and uniqueness. Missing a required relation is a typed integrity error, not silently None. Multiple eligible inverse mappings require an explicit relation name; no first-match guessing. Reciprocal descriptors are finite IDs. Many-to-many is an explicit join model, with separate generated stores and FK constraints.

Read the bounded base page first. Collect/deduplicate relation keys, batch-fetch each requested path level, then attach by typed key while preserving base and child order. To-many collections never determine base LIMIT/OFFSET or inflate count. Include plan compilation checks unknown paths, cycles/depth, aliases, field projection and required unique targets before IO. An explicitly repeated self-relation is bounded; wildcards never recurse forever.

Each relation path is loaded in chunks derived from the executor's bind budget. Also cap total related rows, keys, bytes, statements and nested elements (initial aggregate 10,000 rows, 16 MiB, 128 statements). Limits return an error rather than partial/truncated collections. A typed result marks unloaded data separately; it never represents unloaded as an empty collection. For the fixed walkthrough, `with_owner` always loads the owner and returns `ProjectWithOwner`; more general projections use generated finite include result enums/structs rather than recursive full models.

Instrument the actual Database seam to assert query counts: increasing base page from 1 to 20 does not add one query per parent; growth comes only from documented bind chunks and requested paths. Queries and outputs are reviewable; no lazy property fetch or implicit cascade writes. Related writes are explicit calls in one transaction, not nested input object magic.

## Generated event and server integration

Emit named handlers for `batch/create/detail/get/remove/restore/search/update/upsert` when the operation is supported. `detail` accepts the fixed default include/projection contract; `get` returns a base projection. Hard delete and maintenance purge require explicit operation selection. Generated registration functions use existing `ServerBuilder::on/listen`, typed services, plugin prerequisites and named startup steps; no server dependency points back to framework crates.

Registration returns ListenerIds/registration handles where replacement/removal is useful; it preserves all-matching priority/FIFO semantics. Duplicate operation registration is rejected by the generated registration wrapper's app-owned marker/config, not by changing server's valid duplicate-listener contract. Missing/wrong-order database/policy services fail startup before publication. A plugin can customize a named listener through existing handles without editing generated files.

Handlers skip work if a prior listener has deliberately completed/stopped/errored the response according to an explicit tested guard. They decode only operation-specific DTO fields, validate app policy, run generated functions, project approved output, then call existing response APIs. They neither invoke `Server::call` recursively nor swallow Flow::Abort from delegation. Request→operation→response/error behavior remains the existing host's responsibility. Direct store calls are trusted domain APIs and do not magically run transport authorization; applications that need universal policy call an app-owned service wrapping stores.

Validation errors produce bounded field messages (400); absent key returns the operation's explicit 404; known uniqueness conflicts map to a safe 409 only when a typed adapter error proves that category. Unknown driver/decode/internal failures remain sanitized 500. Never classify conflicts by matching database error text. Slice 7 must retain backend code/constraint identity privately for portable conflict mapping; an adapter lacking it gets the safe generic failure.

At new error boundaries use distinct context such as `project creation failed`, retain the concrete SchemaError/DataError/SqlError as `source()`, and never concatenate its Display into the wrapper. Propagate an existing framework error with `?` if no new context is needed. This follows [task 00010 #11](tasks/00010-receipt-lib.md). Native diagnostics retain path/origin/frames; public output omits SQL, bind values, DB paths, credentials and rejected field values.

HTTP/CLI/Tauri adapters remain app-owned and explicit. The proof compares the same selected generated handlers through direct operation calls, raw nested delegation, CLI and buffered HTTP; it requires no live listener. This new multi-model search contract does not redefine profile-search's earlier ID-prefix/returned-count contract.
