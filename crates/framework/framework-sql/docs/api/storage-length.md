# Explicit string storage length

G38 selects schema contract **3**. Use one positive integer on a stored scalar
String/Text model field:

```idea
model Profile {
  id String @id @store.length(191)
  name String @store.length(255) @max(120)
}
```

Lists, embedded fields, relations, enums and other scalar types refuse this attribute.
The argument is an integer literal in `1..=i32::MAX`; each dialect has smaller physical
limits. Normalized `Field.storage_length: Option<u32>` is omitted when absent and checked
again on descriptor decoding. Validation rules never infer a storage declaration.
Unannotated fields keep TEXT and the existing MySQL TEXT-key/index refusal.

All dialects render VARCHAR(n). SQLite treats the length as declaration metadata only:
direct SQL can store a longer string. Generated create/provider/default, patch, batch,
trusted encode and decode paths check the bound. Private fields have host guards without
publishing private validation arguments or SQL storage intent into browser snapshots.
Admin shares this validation; an HTML UTF-16 maxlength is not a replacement.
Existing record, query and include byte limits still apply; a larger declaration does not
raise those limits.

Length counts Unicode scalar values, including NUL and trailing spaces, without trimming
or normalization. One emoji is one scalar and up to four UTF-8 bytes; a base character and
combining mark are two. The declaration implies `cle(n)`. An equal explicit rule retains
its message; a stricter upper bound is allowed, while a looser upper bound or impossible
static minimum/exact count refuses normalization. Raw PostgreSQL/MySQL coercion of trailing
spaces is not an exact round-trip guarantee: generated values are checked before binding.

## Explicit MySQL target

Generated `stores::schema::definition()` retains an unresolved target requirement when a
field is annotated. Select facts explicitly for MySQL rendering:

```rust,ignore
let tables = generated::stores::schema::mysql_definition("utf8mb4_bin", 16_384)?;
```

The build-host equivalent is `codegen::mysql_storage_definitions(schema, collation, page_bytes)`;
`ddl_policy::mysql_definitions` checks an existing bounded complete catalog. The app-owned
revision command accepts the pair `--mysql-collation utf8mb4_bin --mysql-page-bytes 16384`.
Both are required together and checked before source/history IO. No connection-derived
width, default 191 or implicit target selection exists.

The policy pins InnoDB, DYNAMIC, utf8mb4 and a checked collation in definitions, artifacts
and hashes. Supported collations are utf8mb4_bin, utf8mb4_unicode_ci, utf8mb4_general_ci,
utf8mb4_0900_ai_ci, utf8mb4_0900_as_cs and utf8mb4_0900_bin. The selected local server must
actually support the supplied facts; these are operator expectations, not discovery.
Page sizes are 4096, 8192, 16384, 32768 or 65536 bytes. Full primary, unique, secondary and
FK-support keys reserve four bytes per declared character over the entire compound key:
768 bytes at 4 KiB, 1536 at 8 KiB and 3072 at 16 KiB or larger. Four parts of 191 fit a
3072-byte key; five refuse. No prefix indexes or truncation are introduced.

Planning also checks part/index counts, SQL row size, variable prefixes/null overhead,
physical inline record layout and secondary-record primary-key overhead. The admitted
layout is conservative: Integer, Real, Boolean, DateTime and bounded VARCHAR. Unknown or
off-page TEXT/JSON/Blob/Decimal/trusted layouts refuse under this optional policy rather
than guess. Plain SQL-builder VARCHAR behavior remains unchanged when no policy is selected.
PostgreSQL generated VARCHAR above 10,485,760 refuses; MySQL VARCHAR above 16,383 refuses
before applying its tighter whole-table/key budgets.

Every managed FK string pair must agree on storage family and declared width, including
absence. MySQL pairs must also agree on charset/collation/target facts. Missing width is
never copied from the referenced field; external string targets refuse pure validation.

## Migration and compatibility

New immutable revision/plan artifacts use format **3**, with typed length preconditions
and target facts. An explicit frozen format-2 reader/serializer preserves old artifact
bytes and hashes. The namespaced ledger remains format **2**. Schema contract 3 is a
separate version: regenerate every consumer; admin refuses revision 2, and generated
compatibility assertions reject mixed revisions.

Widening or removing a bound produces a reviewed plan. Target row/index/FK limits are
rechecked; removing a MySQL key/index width refuses. Narrowing (including unbounded TEXT
to VARCHAR) requires approval of the exact plan hash and complete existing-value checks.
Approval cannot waive a value violation. Reverse is a fresh plan/check, so later longer
values can refuse reverse. No data is truncated, cast or trimmed.

Introducing or changing configured MySQL target facts between snapshots refuses, including
when rendering another dialect. Select them at a history's baseline and retain them across
revisions; the planner does not adopt unknown prior charset/collation settings.

SQLite checks values after acquiring the maintenance writer lock and rechecking drift
and ledger state, before reconstruction. It scans eight-row stable batches using a
complete nonnull key. Aggregate limits are 10,000 scanned values, 16 MiB text and five
seconds; an incomplete scan refuses. Rust character counting handles embedded NUL.
Reconstruction preserves exact rows, FKs and AUTOINCREMENT high-water history. Only
AUTOINCREMENT tables maintain a sequence; ordinary INTEGER PRIMARY KEY retains SQLite's
max(rowid)+1 reuse behavior. PostgreSQL/MySQL migrations remain render-only; MySQL apply
and generated RETURNING writes stay refused.

The [permanent fixture](../../tests/proofs/lengths/README.md) generates and compiles a
three-model SQLite consumer and checks all-dialect DDL snapshots. The opt-in MySQL M2
`--g37-ddl` case executes that snapshot unchanged in an owned throwaway database and
checks metadata, exact bound text and FKs; it makes no generated MySQL CRUD claim.
