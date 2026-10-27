# Runtime and generated APIs

## Runtime ownership

`DataSession::{pooled,transaction,dialect,limits,execute,fetch,write,require_transaction,require_returning}` selects one exact executor. `write` preflights and submits once; a native failure is uncertain. `finish` consumes one transaction, preserving primary and cleanup errors. `DataError` provides `kind`, `source` and `cleanup_error`; `InsertOutcome<T>` distinguishes Inserted from AlreadyExists.

`encode`, `decode`, `encode_patch`, `StorageColumn` and `StorageCodec` implement checked ordered wire/storage mapping. Dates/times/datetimes are canonical text, numbers are exact i64/finite f64, JSON is bounded. Omitted patch fields produce no assignment; null obeys declared nullability. The runtime `stores` module owns Page, Loaded, Query, StoreSpec, query catalogs, Exposure, Visibility and lowerable RelationLimits. `projection::select` accepts an already projected finite eager view and compiler-owned selection paths.

## Generated native stores

Each `stores::<model>` contains `<Model>Store` detached builders: table, encode/decode, insert, select_key, search_plan, update_plan, upsert_plan, upsert_on_plan, delete_plan, plus active_plan where supported. Legacy simple String/Integer select/get/update/upsert conveniences remain. Complete `<Model>Key` identities support all accepted scalar and compound key families.

Plural `<Model>s` functions are create/create_with/create_if_absent, get/get_key/get_maintenance, find/find_all/count, search/search_with_limits/search_atomic/search_public, detail, update/update_key, upsert/upsert_key/upsert_on, batch/batch_atomic, delete, and active-only remove/restore. Native ConflictKey selects Primary or a declared nonnull Unique<Field>; it validates target agreement before IO. BatchInput has Create/Update/Upsert/UpsertOn; ordered BatchOutcome retains absent updates. Public operation envelopes keep primary-key Upsert and do not expose arbitrary native unique-target selection.

Views are View0–View3 and PublicView0–PublicView3, with a distinct opaque Selected default projection. Their `row` member holds the base record. A source relation named `row` uses Rust member `_relation_row` and serialized eager member `__row`, which cannot collide with portable source names; includes and Selected output retain logical `row` spelling. `COLUMNS`, `SPEC` and `FINGERPRINT` are immutable generated catalogs. `stores::schema::{definition,catalog}` returns explicit definitions/catalog entries. Models named Schema or Operations use `_schema`/`_operations` store and operation modules to avoid aggregate collisions; their contract modules retain the accepted normalized names.

## Generated operation boundary

Generated public Search and Detail handlers authorize before decoding, then preflight
their exact public query before BEGIN. Preflight checks public field/path eligibility,
the selected dialect's search/include/snapshot capabilities, and rendered base/count
plans against the selected Engine's input limits. COUNT's fixed one-row result also
has a known minimum row/byte cost. Actual page/related row sizes and relation lookup
keys depend on database contents and retain their execution-time guards. This does not
enable network search, includes or snapshot support that was previously unavailable.

`operations::{Operation,OperationPolicy,HostInputs,Policy,Registration}` supplies app policy/host inputs and registration ownership. Emitted `stores::operations::<model>` contains KeyRequest, CreateRequest, UpdateRequest, UpsertRequest, BatchItem, BatchRequest, named async handlers and register. DTOs are strict and use the schema-owned public fields. Importing these APIs registers nothing.
