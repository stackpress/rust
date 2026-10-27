# M2 shared SQL-core upsert proposal

Status: **accepted exactly as proposed by the coordinator and implemented**. The sections below preserve the original ownership request and rendering contract. Both shared grants remain exclusive through the M2 receipt, which records final verification. G34 schema revision 2 is unchanged.

## Requested shared write grant

Allow the implementer to edit only these SQL-core paths, in addition to the existing Slice 6 write set, after the coordinator sequences ownership:

- `crates/sql/sql/src/dml.rs`: the new type, two Insert methods, private optional clause and rendering/validation.
- `crates/sql/sql/src/lib.rs`: re-export the new type.
- `crates/sql/sql/tests/dml.rs`: renderer, inspection, validation and regression tests.
- `crates/sql/sql/docs/api/insert.md`: document the accepted surface and dialect gates.

No dependency, manifest, adapter, Database/TransactionSession trait, ordinary raw-query gate, existing WriteValue/Expr variant or frozen Slice 6 DTO change is requested. Native SQLite execution/concurrency tests belong in framework-sql's allowed integration-test scope. PostgreSQL/MySQL native execution remains coordinated later with the SQL lane.

## Exact proposed public API

Accepted public API, now compiled and exercised through SQLite:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum InsertConflict {
    DoNothing {
        target: Vec<String>,
    },
    DoUpdate {
        target: Vec<String>,
        assignments: Vec<(String, WriteValue)>,
    },
}

impl Insert {
    pub fn on_conflict(self, conflict: InsertConflict) -> Self;
    pub fn conflict(&self) -> Option<&InsertConflict>;
}
```

Insert owns a private `Option<InsertConflict>`, initially None. `on_conflict` replaces that optional clause, like the existing returning setter; no ordered list of multiple conflict clauses is introduced. Cloning retains both the original insert data and the clause. `InsertData` and `build() -> &InsertData` stay unchanged, preserving existing inspection/struct-literal callers; the new getter inspects the complete added clause separately. Existing inserts render byte-for-byte as before.

Reuse existing WriteValue for explicit bound update values, structured JSON, DEFAULT and expressions. No excluded-row-reference syntax is needed for the initial `upsert(key, input, patch)`: the create values and patch values are prepared separately and bound in one atomic statement. This avoids adding a variant to existing public enums. Framework generation owns immutable key/generated-field policy and validates that the explicit key/target is primary or unique; the generic SQL renderer does not discover schema constraints.

Use:

```rust
let insert = Insert::new("project")
    .columns(["id", "title"])
    .values(["p1", "Initial"])
    .on_conflict(InsertConflict::DoUpdate {
        target: vec!["id".into()],
        assignments: vec![("title".into(), WriteValue::from("Updated"))],
    })
    .returning(["id", "title"]);
```

## Exact rendering and bind order

For the example above, binds are `[Text("p1"), Text("Initial"), Text("Updated")]` in VALUES order followed by assignment order. All values remain binds; identifier validation/quoting and existing query/bind/byte limits remain in force. The clause is appended before RETURNING.

| Dialect | Proposed result |
| --- | --- |
| SQLite | `INSERT INTO "project" ("id", "title") VALUES (?, ?) ON CONFLICT ("id") DO UPDATE SET "title" = ? RETURNING "id", "title"` |
| PostgreSQL | `INSERT INTO "project" ("id", "title") VALUES ($1, $2) ON CONFLICT ("id") DO UPDATE SET "title" = $3 RETURNING "id", "title"` |
| MySQL | `ErrorKind::Unsupported` before any Query is returned/executed for a valid targeted conflict clause. No translation to ON DUPLICATE KEY UPDATE or INSERT IGNORE. |

For `DoNothing { target: vec!["id".into()] }`, replace the DO UPDATE segment with `DO NOTHING`; the third bind disappears. SQLite/PostgreSQL retain optional RETURNING; a conflicting no-op returns no row. An acknowledged DO UPDATE returns the authoritative row through RETURNING. No inserted-versus-updated Boolean is inferred from affected rows or insert IDs.

SQLite's targeted UPSERT acts on the named uniqueness constraint and leaves unrelated NOT NULL/CHECK/FK failures as failures. [SQLite UPSERT](https://www.sqlite.org/lang_upsert.html). PostgreSQL supports the same target/action form and guarantees atomic insert-or-update absent an independent error. [PostgreSQL INSERT](https://www.postgresql.org/docs/18/sql-insert.html).

MySQL's native action reacts to any primary/unique-key conflict. Counterexample: an existing row has `id=p2, email=taken`; inserting `id=p3, email=taken` with target `id` should fail the unrelated email uniqueness check. Translating to ON DUPLICATE KEY UPDATE would instead update p2. This is an inference from the documented any-key behavior, not a network database execution result. [MySQL 8.4 conflict semantics](https://dev.mysql.com/doc/refman/8.4/en/insert-on-duplicate.html). Supporting a different explicit any-unique-key API would require its own later review.

## Deliberate first-extension limits

- Always require a nonempty target, including DO NOTHING. No catch-all target, named constraint, expression/partial-index target, collation/opclass or extra conflict WHERE predicate is exposed.
- Validate target and assignment column lists with the existing bounds/identifier rules and ASCII case-insensitive duplicate rejection. DO UPDATE requires at least one assignment. Malformed shape is InvalidQuery, without echoing input.
- Admit exactly one explicit VALUES row when a conflict clause is present. Multi-row conflicted inserts and effective DEFAULT VALUES return Unsupported; ordinary multi-row/default inserts stay unchanged. SQLite disallows UPSERT after DEFAULT VALUES, including the existing renderer's all-DEFAULT-column omission case. [SQLite INSERT](https://www.sqlite.org/lang_insert.html).
- The single-row restriction also avoids promising cross-dialect repeated-key behavior: PostgreSQL rejects a statement affecting the same existing row twice. [PostgreSQL cardinality rule](https://www.postgresql.org/docs/18/sql-insert.html). Framework batch upsert can validate/prepare the entire bounded batch first and run one statement per item in one supplied transaction, preserving order and atomicity.
- Existing WriteValue dialect limits still apply: SQLite assignment DEFAULT remains Unsupported; PostgreSQL can use it. RETURNING support remains unchanged. No read-then-insert fallback, replacement/delete-and-reinsert behavior or retry is introduced.
- This extension renders one statement only. Caller-owned transaction completion, last-writer-wins updates and unknown-outcome/no-automatic-retry behavior remain framework responsibilities; no adapter contract change is hidden in it.

## Verification before merge

Once authorized: prove inspection/clone/setter replacement, unchanged ordinary inserts, SQLite/PostgreSQL exact SQL/bind order for both actions and compound targets, structured JSON/expression/NULL assignments, invalid/duplicate/empty/over-budget cases, DEFAULT/multi-row gates, MySQL Unsupported and hostile identifiers/binds. Then execute through the real typed Insert + SQLite adapter: insert/update/no-op RETURNING, untargeted uniqueness/FK failure, two concurrent writers on one key with one resulting row, stable key/authoritative values, and explicit transaction rollback. Concurrency/hang cases retain parent watchdogs with kill/reap. Run core and scoped fmt/Clippy/tests/doctests/MSRV, current generated regressions and workspace checks after implementation.

A bounded **SQL syntax-only** probe has already passed using Python stdlib SQLite 3.51.0 and an owned temporary file: insert RETURNING, update RETURNING, targeted DO NOTHING, unrelated unique violation without row changes, hostile bound text, and compound target. Parent deadline was 10 seconds with process-group kill/reap; the connection closed and its file was removed. Evidence: `/private/tmp/s6-m2/upsert-syntax.py` and `upsert-syntax-result.json`. This does not validate the unimplemented Rust API, SQLx's linked SQLite version, framework concurrency or network dialect execution. That initial probe created no Cargo target; the accepted implementation now has separate Rust test evidence in `/private/tmp/s6-m2/`.

Accepted choice: the small targeted API for SQLite/PostgreSQL, with MySQL targeted upsert explicitly unavailable. Generated `create_if_absent` uses `InsertOutcome::{Inserted, AlreadyExists}` and never fabricates a row on the no-change path. Full M2 CRUD/relations/operations and the final receipt remain pending.
