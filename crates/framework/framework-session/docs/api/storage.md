# Auth/Profile schemas and SQL storage

`native` exposes columns, source codecs, `auth::AuthSchema` and `profile::ProfileSchema`. `sql` adds detached Store builders and Engine-backed Actions. Use [direct storage](../storage.md) for source coercion/query/return details and [account processes](../account.md) for server events.

## Account event registration

With `auth-server`, `account::events::register(&mut ServerBuilder) -> stackpress_lib::Result<()>` registers its seven SQL events immediately for direct composition. `account::events::register_listen(&mut ServerBuilder) -> stackpress_lib::Result<()>` queues the same inventory for Listen, after earlier Listen callbacks, for source-order aggregate composition. Both keep resources/policy explicit and perform no startup IO. [Registration proof](../../tests/main/account_registration.rs) pins both inventories, timing, no duplication and no-Engine pass-through; [implementation](../../src/account/events.rs) shares one handler list.

| Parameter | Meaning |
| --- | --- |
| `app: &mut ServerBuilder` | Application construction state; either call prepares that same host without opening a database. |

**Returns** `stackpress_lib::Result<()>`: immediate registration propagates handler-registration errors now; queued registration propagates queue-insertion errors now and handler-registration errors during `app.build().await`. The linked proof contains compiled usage of both forms.

## Schemas and values

The stable API composition path is `stackpress_framework_session::profile::ProfileSchema`. Auth/Profile schemas dereference to RecordSchema; `model()` returns an owned seed-free shared Model, including Profile's Auth/Application/Session metadata. It does not add reverse eager joins to ProfileStore. [Descriptor and column tests](../../tests/main/records.rs) anchor these contracts.

<!-- example: direct -->
```rust
use stackpress_framework_session::profile::ProfileSchema;

pub fn profile_defaults() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schema = ProfileSchema::new("");
    let defaults = schema.defaults_at(0)?;
    assert_eq!(defaults["roles"], serde_json::json!(["GUEST"]));
    assert!(schema.assert(&defaults, true).unwrap().contains_key("name"));
    assert_eq!(schema.model().name, "Profile");
    Ok(())
}
```

The example generates a fresh CUID/timestamps and literal defaults, then checks the missing name error. Construction alone generates no defaults and opens no database. `Profile`, `ProfileInput`, `ProfileExtended` and Auth's corresponding aliases are Record maps, including partial projections.

| Schema operation | Arguments / return |
| --- | --- |
| `new(seed)`, `default()` | Caller database seed, default empty; AuthSchema or ProfileSchema |
| `with_offset_minutes(i32)` | Owned schema with explicit local Date offset |
| `into_schema()` | Owned common RecordSchema |
| `column(&str)`, public `columns` | Option<&Column>, source-order descriptors |
| `defaults()` / `defaults_at(i64)` | SchemaResult<Record>, milliseconds clock |
| `filter(&Record, populate: bool)` / `populate(&Record)` | SchemaResult<Record>, optional defaults then supplied values |
| `assert(&Record, required: bool)` | Option<Record> field errors; None on success |
| `serialize(&Record, storage: bool)` / `unserialize(...)` | SchemaResult<Record>; storage enables source encryption/hash boundaries |

`Column` exposes name/sql/kind/nullability/required message/min/default and defaults/assert/serialize/unserialize operations; [`columns/mod.rs`](../../src/columns/mod.rs) and [`schema.rs`](../../src/columns/schema.rs) define each argument. `None` at a column boundary represents undefined; explicit Null is distinct. `Record` and Value::Object use insertion-ordered `stackpress_lib::DataMap`; use `shift_remove` to retain order when deleting entries. Native Value::Date stays distinct from JSON objects; date/as_date/to_json and explicit Date display helpers are described in [direct storage](../storage.md). Nested JSON and CSV order are pinned in [account export tests](../../tests/main/account_flows/sql.rs).

Profile stored columns are id/image/name/type/roles/tags/references/active/created/updated. Auth columns are id/profileId/type/token/secret/verified/consumed/active/created/updated. Source types, defaults, UTF-16 bounds and SQL names live in [schema definitions](../../src/columns/schema.rs), with [380 source column vectors](../../tests/records.json).

## Engine-backed Actions

`storage::{AuthActions,ProfileActions}` also re-export under auth/profile. Each constructor retains an application-owned Arc<Engine>; actions are not another pool/ORM. The [plugin consumer manifest](../../consumers/plugin/Cargo.toml) supplies the SQLite/Engine dependencies used below.

<!-- example: server -->
```rust
use stackpress_framework_session::{
    columns::Record,
    storage::{ProfileActions, StoreQuery},
};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::Arc;

pub async fn sqlite_profile() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(directory.path().join("profile.sqlite")).create_if_missing(true),
        )
        .await?,
    ));
    let profiles = ProfileActions::new(engine.clone(), "");
    profiles.install().await?;
    let input: Record =
        serde_json::from_value(serde_json::json!({"id": "reader", "name": "Reader"}))?;
    profiles.create(&input).await?;
    let row = profiles
        .find(&StoreQuery::eq("id", "reader"))
        .await?
        .unwrap();
    assert_eq!(row["name"], "Reader");
    engine.close().await?;
    Ok(())
}
```

This creates, reads and closes one real file-backed SQLite database. Install Profile before Auth when both are used. [Actions tests](../../tests/main/actions.rs) separately cover CRUD, uniqueness, batch rollback and partial effects; this small example does not claim live PostgreSQL/MySQL support.

| Operation | Arguments / returns |
| --- | --- |
| `new(Arc<Engine>, seed)` | Actions with direct literal seed |
| `create(&Record)` | async Result<Record> prepared/returned row |
| `find(&StoreQuery)` / `find_all(...)` / `count(...)` | async Result<Option<Record>> / Result<Vec<Record>> / Result<u64> |
| `update(&StoreQuery, &mut Record)` | async Result<Vec<Record>>; mutates updated input; source returned rows differ from persisted state |
| `delete(&StoreQuery)` | async Result<Vec<Record>>, physical delete/pre-read rows |
| `remove(...)` / `restore(...)` | async Result<Vec<Record>>, active=false/true; select inactive explicitly to restore |
| `upsert(&mut Record)` | async Result<Option<Record>>, ID-first then Auth token, else create |
| `batch(&mut [Record], BatchMode)` | async Vec<BatchStatus>; Create/Update/Upsert attempts, not committed-row acknowledgements |
| `install()` / `upgrade(&Create)` / `purge(cascade: bool)` / `uninstall()` | async Result<()>, explicit maintenance |

Create/find/find_all/count/update/delete/upsert have `_in` counterparts with `&mut DataSession<'_>` first, exclusively borrowing the caller's executor. Batch owns its transaction and catches outer commit/rollback errors as source. [Batch source](../../src/storage/batch.rs) and [exclusive-executor compile-fail case](../../tests/ui-sql/exclusive_executor.rs) pin this boundary.

AuthActions adds `with_options(engine, ActionOptions)`, `make(engine, Option<String>, Option<PasswordConfig>)`, `profile()`, `assert(&Record) -> Option<Record>`, `signin(kind, &Record, password: bool).await -> Result<Option<Record>>`, and `signup(&Record).await -> Result<Record>`. Options hold seed/password; make defaults a missing database seed to abc123. Signup writes sequentially; missing phone succeeds under G44. [Account action source](../../src/storage/account.rs) and [signup/signin tests](../../tests/main/actions.rs) preserve the flags/errors.

## Queries and detached Store

`StoreQuery` has q/columns/eq/ne/ge/le/like/has/hasnt/sort/skip/take; default take=50, take=0 unlimited, active=true implicit. `StoreQuery::eq(name,value)` builds one equality; `.all()` adds only the equality active=-1 escape. Arrays form source OR groups. Unknown ordinary scalar selectors are skipped; Auth profile.* is an inner join with no extra Profile-active check. [Query source](../../src/storage/query.rs) and [SQL predicates test](../../tests/main/actions.rs) are the anchors.

`AuthStore` / `ProfileStore` alias `Store<AuthModel>` / `Store<ProfileModel>`. `new(seed)` constructs a detached schema/store; `table`, `create`, `alter`, `scalarize`, `unscalarize`, `to_sql_value`, `insert[_for]`, `update`, `delete`, `select`, `where_clause`, `paths`, `selectors` and `joins` expose the existing SQL builders/plans without executing. Use [Store signatures](../../src/storage/store.rs), [query signatures](../../src/storage/query.rs) and `SourceStatement`'s [renderer](../../src/storage/statement.rs) for parameter lookup. `ActionError` retains public code/error/errors, status() and native Error::source cause; [storage module](../../src/storage/mod.rs) defines the error contract.

**Resolved under G46:** PostgreSQL alone renders ILIKE; SQLite/MySQL render LIKE. SQLite and [live PostgreSQL](../live-search.md) prove ASCII case-insensitive searches. MySQL's opted-in case stops at the existing transactional-DDL install refusal before search. Auth's native INSERT-return branch and positive `hasnt` follow the original source, unchanged: raw returned token remains ciphertext/profile_id is omitted, and `hasnt` uses the same membership operation as `has`. [Source behaviour](../source-behavior.md#owner-decisions-g46) routes the source/test evidence.

## Shared provider paths

The stable native helpers are `stackpress_framework_schema::providers::{cuid,CuidGenerator,BuiltinProviders,encrypt,decrypt,hash}` under schema/providers. With codegen too, `providers::cuid_definition()` is the explicit DefinitionRegistry provider entry. These shared helpers are also used by API; session's source codecs re-export encrypt/decrypt/hash. [Provider source](../../../framework-schema/src/providers/mod.rs) and [provider tests](../../../framework-schema/src/providers/tests.rs) contain CUID2 and original Node codec vectors.

CUID2 uses SHA3-512, ring randomness, a native fingerprint, clock and atomic counter. Source secret hashing uses SHAKE256(32-byte hex); source token storage uses AES-256-CBC/PKCS7 with key=first32 characters of base64 SHA256(seed), IV=first16 of that key, ciphertext=hex. There is no integrity envelope or Argon2 migration. Deterministic equality/uniqueness and consumed-Date challenges are pinned by [codec/SQL tests](../../tests/main/storage.rs). Native date/calendar/parser range limits are documented in [direct storage](../storage.md).

`cuid() -> SchemaResult<String>` uses the shared generator. `CuidGenerator::new()` and `with_length(usize)` return SchemaResult<CuidGenerator>; `generate(&self)` returns SchemaResult<String>. `BuiltinProviders` implements the existing ValueProviders trait for no-argument cuid only; other providers/arguments and host validators return explicit unsupported errors. Codegen registration remains explicit, with revision cuid2-3.3.0. These signatures live in the linked provider modules and their tests.
