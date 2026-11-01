# Direct Auth/Profile storage

Enable `stackpress-framework-session/sql`. Supply an application-owned `Arc<Engine>`;
constructing actions performs no I/O. Call `ProfileActions::install()` before
`AuthActions::install()` explicitly, or use the detached Store create definitions in an
application migration. No session plugin invokes installation or maintenance.

<!-- example: server -->
```rust
use stackpress_framework_session::{
    columns::Record,
    storage::{AuthActions, ProfileActions, Result, StoreQuery},
};
use stackpress_sql::Engine;
use std::sync::Arc;

pub async fn account(engine: Arc<Engine>) -> Result<()> {
    let profiles = ProfileActions::new(engine.clone(), "database-seed");
    let auth = AuthActions::new(engine, "database-seed");
    let input: Record = serde_json::from_value(serde_json::json!({
        "name": "Ada", "email": "ada@example.test", "secret": "hello"
    }))
    .unwrap();
    let profile = auth.signup(&input).await?;
    let stored = profiles
        .find(&StoreQuery::eq("id", profile["id"].clone()))
        .await?;
    assert!(stored.is_some());
    Ok(())
}
```

`ProfileSchema::new(seed)` / `AuthSchema::new(seed)` require `native`, independently of SQL.
`schema.columns` exposes each source name, SQL name, kind, optional/nullability, required
message, string minimum and literal default. Column `defaults[_at]`, `assert`, `serialize`
and `unserialize` are directly available. `None` represents JavaScript undefined. Records
are maps of `columns::Value`, preserving missing keys, null, arrays, objects and native Dates.
JSON input uses normal Serde conversion; arbitrary object keys are never Date markers.

Native JavaScript Date values use `columns::date(milliseconds)` values to distinguish a
Date from a numeric input. `columns::as_date` reads them; `columns::to_json` recursively
produces source JSON Date strings for an explicit wire boundary; normal Serde serialization does the same. Use
`date_with_display(milliseconds, display)` when an array's Date.toString needs the host's
local timezone label; the native default context is UTC. Schema `with_offset_minutes`
selects local Date column conversion without mutating process timezone state. The native
time library's representable years/offsets are explicit fallible mechanics limits. Date
strings support ISO/RFC3339, source SQL local text, date-only and V8 numeric month/year
spellings; arbitrary implementation-specific legacy Date.parse formats are not a portable
Rust input contract. Use a native Date value for those values.

| Surface | Behavior |
| --- | --- |
| `create[_in]` | Filter → defaults → serialize/unserialize → remove empty text → assert → Auth token preflight → insert. |
| `find[_in]`, `find_all[_in]`, `count[_in]` | Source query maps, default take=50, zero take unlimited; active=true unless explicitly supplied. `StoreQuery::all()` sets the source active=-1 sentinel. |
| `update[_in]` | Assert prepared patch, pre-read matching rows, merge prepared fields for returned rows, then write original input. Store mutates `input.updated`; returned rows retain the pre-read timestamp. |
| `delete[_in]` | Pre-read, physical delete only if rows were found, return pre-read rows. |
| `remove`, `restore` | Source update(active=false/true). Restoring an inactive row needs an explicit inactive/all selector. No JWT revocation is added. |
| `upsert[_in]` | Present ID takes precedence and returns null if not found. Auth then tries a nonempty token. Otherwise create. |
| `batch` | Explicit transaction on one executor. Caught item errors cause rollback after all items. Status rows describe attempts even if rollback/commit fails. Missing update keys add an error item without setting rollback. |
| `install`, `upgrade`, `purge`, `uninstall` | Explicit source maintenance through existing SQL plans; unsupported renderer/atomic-DDL operations fail under the SQL crate's existing mechanics. PostgreSQL cascade is retained; SQLite/MySQL ignore purge's cascade like source. |
| `signin`, `signup`, `assert` | Source password checks/errors, optional password comparison, consumed reload; signup creates profile/email/phone/username sequentially, preserving earlier writes after later failures. Missing phone succeeds under G44. |

`StoreQuery` supplies `eq`, `ne`, `ge`, `le`, `like`, `has`, `hasnt`, `q`, `columns`, `sort`,
`skip` and `take`. Arrays in scalar conditions form OR groups. Unknown ordinary scalar
selectors are skipped as source; JSON membership reaches the database. `Store::select`,
`insert[_for]`, `update`, `delete`, `create` and `alter` return detached SQL builders/plans.
Selectors, paths, joins, scalarize, unscalarize and to_sql_value are available directly.
Auth's requested `profile.*` is an inner join with no extra profile-active predicate.
ProfileStore has no reverse eager relations, matching the generated source store.

Every `_in` method exclusively borrows the caller's `DataSession`; it never reacquires
from the pool. Cancellation leaves the caller-owned transaction with its owner. `batch`
owns its transaction, so cancellation uses the database adapter's rollback-on-drop contract.

Source behaviors intentionally retained:

- Auth token encryption is deterministic AES-CBC only at storage/query encoding. Secret
  hashing is source SHAKE256, including ordinary equality comparison during signin.
- Token uniqueness spans all auth types. Active-only preflight can miss an inactive token;
  the SQL unique constraint still rejects it with a native exception, not a new field error.
- INSERT returned-row decoding uses schema.unserialize directly: Auth token remains
  ciphertext and raw profile_id is omitted in that branch. Ordinary find decrypts/maps it.
  No-row insertion uses the source last-ID lookup/input fallback. MySQL omits RETURNING.
- Omitted Profile lists become [] during preparation, including the merged update return;
  SQL writes only supplied fields. Empty text removed from a merged return can still be
  written by the original input. These returns are not a persisted-row acknowledgement.
- `hasnt` currently compiles as `has`. JSON membership arrays form source OR groups.
- G46 keeps ILIKE only on PostgreSQL. SQLite/MySQL use LIKE, relying on SQLite's default
  ASCII case-insensitive matching and MySQL's default _ci collations.

SQLite execution is proved in tests. [Live PostgreSQL searches](live-search.md) now execute
Profile q/like and Auth-to-Profile like with source installation/codecs. MySQL connects but
Profile install rejects transactional DDL before a search; the opted-in test remains failing.
Arbitrary collations and backend-specific JSON-query semantics remain unverified. Raw SQL
identifiers/JSON paths use the existing Rust SQL crate's checked/bound representation,
a mechanics boundary rather than a new application admission policy. No native resource,
DB seed, secret or full result becomes a browser snapshot automatically.

## Evidence and G46 decisions

[Schema/column vectors](../tests/main/records.rs), [real SQLite Actions](../tests/main/actions.rs), [codec challenges](../tests/main/storage.rs) and [exclusive-executor compile failure](../tests/ui-sql/exclusive_executor.rs) anchor the storage contract. [Storage API](api/storage.md) lists the public methods and shared providers.

**Resolved under G46:** PostgreSQL uses ILIKE; SQLite/MySQL use LIKE. Auth native create-return and positive hasnt follow the original source, unchanged. The [decision register](source-behavior.md#owner-decisions-g46) records the source anchors and rendering/execution evidence.
