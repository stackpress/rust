# From Idea to a queryable SQLite database

A project tracker needs people, projects and tasks to stay related while its schema changes. This walkthrough generates their Rust types and stores, installs synthetic data, queries a page with relations, and makes a reviewed schema change without losing those rows.

Run the commands from the repository root with the repository's Rust 1.96 toolchain, Python 3 and already-cached Cargo dependencies. The commands use `--offline`; SQLite is embedded, so this proof needs no database server, listener or installation. The existing app-owned `generated-workspace-tool` is reused with explicit paths; it has no dependency on this query app. Its compiled registry explicitly approves these synthetic example fields, including the new heading, summary and inverse links. That metadata grants no transport authorization.

## Generate and install

Start with [schema/001.idea](schema/001.idea): `Profile.projects`, `Project.owner`/`tasks`, and `Task.project` form the three-model graph. Only owning IDs become foreign-key columns. Keep generated output and immutable schema history in separate directories.

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
export TMPDIR="$CARGO_TARGET_DIR/tmp"
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
mkdir -p "$TMPDIR" examples/data-quickstart/src/generated
export QUICKSTART_DIR="$(mktemp -d "$CARGO_TARGET_DIR/data-quickstart.XXXXXX")"
tool() { cargo run --quiet --offline --locked -p generated-workspace-tool --features database-tools -- "$@"; }
field() { python3 examples/data-quickstart/inspect.py "$QUICKSTART_DIR/$1.json" "$2"; }
tool emit generate --input examples/data-quickstart/schema/001.idea --output examples/data-quickstart/src/generated/v1
tool emit generate --input examples/data-quickstart/schema/001.idea --output examples/data-quickstart/src/generated/v1 --check true
tool emit revision --input examples/data-quickstart/schema/001.idea --history "$QUICKSTART_DIR/history" --app quickstart --name 001
tool emit migration-plan --history "$QUICKSTART_DIR/history" --to 001 > "$QUICKSTART_DIR/install.json"
cat "$QUICKSTART_DIR/history/migrations/$(field install plan)/sqlite-up.sql"
tool emit db-install --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field install plan)"
```

Generation creates inspectable contracts, metadata and stores. It does not touch a database. Recording `001` publishes an immutable snapshot; planning writes a checked recipe and inspection-only SQL. `db-install` uses the exact reviewed hash and creates both managed tables and the namespaced migration ledger in one transaction.

<!-- sql: install -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
CREATE TABLE "profile" ("id" TEXT NOT NULL, "name" TEXT NOT NULL, PRIMARY KEY ("id"));
-- binds: []
CREATE INDEX "profile_name_index" ON "profile" ("name");
-- binds: []
CREATE TABLE "project" ("id" TEXT NOT NULL, "title" TEXT NOT NULL, "owner_id" TEXT NOT NULL, PRIMARY KEY ("id"), CONSTRAINT "project_owner_id_fk" FOREIGN KEY ("owner_id") REFERENCES "profile" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION);
-- binds: []
CREATE INDEX "project_title_index" ON "project" ("title");
-- binds: []
CREATE TABLE "task" ("id" TEXT NOT NULL, "project_id" TEXT NOT NULL, "title" TEXT NOT NULL, "done" BOOLEAN NOT NULL, PRIMARY KEY ("id"), CONSTRAINT "task_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "project" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION);
-- binds: []
CREATE INDEX "task_title_index" ON "task" ("title");
-- binds: []
```

The SQL shown here and below is the actual managed-schema plan, including bound values. The runner also checks the catalog and foreign keys and updates `_stackpress_migrations` / `_stackpress_schema_state`; those internal statements are deliberately not a public SQL execution interface. Never execute the SQL companions as migration authority.

## Seed and read a project page

[seed.json](seed.json) gives two profiles, three projects and three tasks explicit IDs. The named seed is one bounded, atomic DB plan; repeating the same approved seed is `AlreadyApplied`.

```sh
tool emit migration-plan --history "$QUICKSTART_DIR/history" --to 001 --operation populate --seed demo-001 --seed-file examples/data-quickstart/seed.json > "$QUICKSTART_DIR/seed.json"
cat "$QUICKSTART_DIR/history/migrations/$(field seed plan)/sqlite-up.sql"
tool emit db-populate --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field seed plan)" --seed demo-001
cargo run --quiet --offline --locked -p data-quickstart --features native -- "$QUICKSTART_DIR/data.sqlite" > "$QUICKSTART_DIR/read-001.json"
cat "$QUICKSTART_DIR/read-001.json"
```

You should see Ada, project `p2` (`Ship UI`), its loaded owner and two tasks, an exact matching-project total of 2, and a task count of 3. The search is literal and case-sensitive. `skip: 1, take: 1` selects the second matching base identity; relation includes are loaded afterward and do not change page size.

<!-- sql: seed -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
INSERT INTO "profile" ("id", "name") VALUES (?, ?) ON CONFLICT ("id") DO UPDATE SET "name" = ?;
-- binds: [{"type":"text","value":"u1"},{"type":"text","value":"Ada"},{"type":"text","value":"Ada"}]
INSERT INTO "profile" ("id", "name") VALUES (?, ?) ON CONFLICT ("id") DO UPDATE SET "name" = ?;
-- binds: [{"type":"text","value":"u2"},{"type":"text","value":"Grace"},{"type":"text","value":"Grace"}]
INSERT INTO "project" ("id", "owner_id", "title") VALUES (?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "owner_id" = ?, "title" = ?;
-- binds: [{"type":"text","value":"p1"},{"type":"text","value":"u1"},{"type":"text","value":"Ship API"},{"type":"text","value":"u1"},{"type":"text","value":"Ship API"}]
INSERT INTO "project" ("id", "owner_id", "title") VALUES (?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "owner_id" = ?, "title" = ?;
-- binds: [{"type":"text","value":"p2"},{"type":"text","value":"u1"},{"type":"text","value":"Ship UI"},{"type":"text","value":"u1"},{"type":"text","value":"Ship UI"}]
INSERT INTO "project" ("id", "owner_id", "title") VALUES (?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "owner_id" = ?, "title" = ?;
-- binds: [{"type":"text","value":"p3"},{"type":"text","value":"u2"},{"type":"text","value":"Write guide"},{"type":"text","value":"u2"},{"type":"text","value":"Write guide"}]
INSERT INTO "task" ("done", "id", "project_id", "title") VALUES (?, ?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "done" = ?, "project_id" = ?, "title" = ?;
-- binds: [{"type":"bool","value":true},{"type":"text","value":"t1"},{"type":"text","value":"p1"},{"type":"text","value":"Design endpoint"},{"type":"bool","value":true},{"type":"text","value":"p1"},{"type":"text","value":"Design endpoint"}]
INSERT INTO "task" ("done", "id", "project_id", "title") VALUES (?, ?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "done" = ?, "project_id" = ?, "title" = ?;
-- binds: [{"type":"bool","value":false},{"type":"text","value":"t2"},{"type":"text","value":"p2"},{"type":"text","value":"Build page"},{"type":"bool","value":false},{"type":"text","value":"p2"},{"type":"text","value":"Build page"}]
INSERT INTO "task" ("done", "id", "project_id", "title") VALUES (?, ?, ?, ?) ON CONFLICT ("id") DO UPDATE SET "done" = ?, "project_id" = ?, "title" = ?;
-- binds: [{"type":"bool","value":false},{"type":"text","value":"t3"},{"type":"text","value":"p2"},{"type":"text","value":"Review page"},{"type":"bool","value":false},{"type":"text","value":"p2"},{"type":"text","value":"Review page"}]
```

The developer writes this function in [src/queries.rs](src/queries.rs). The caller gives it `DataSession::transaction(&mut transaction)`; `read` also queries `Profiles::get` and `Tasks::count`, then awaits completion or rollback. Direct stores are trusted domain APIs; an exposed operation needs its own authorization and public projection.

<!-- rust: project_page -->
```rust
use data_quickstart::generated::stores::project::{Projects, Query as ProjectQuery, View3};
use stackpress_framework_sql::{DataResult, DataSession, stores::Page};

pub async fn project_page(db: &mut DataSession<'_>) -> DataResult<Page<View3>> {
    Projects::search(
        db,
        ProjectQuery {
            q: Some("Ship".into()),
            skip: 1,
            take: 1,
            total: true,
            includes: vec!["owner".into(), "tasks".into()],
            ..Default::default()
        },
    )
    .await
}
```

The executable records the actual SELECTs and synthetic binds in its `statements` result. One transaction keeps the base page, total and relation reads consistent on SQLite; it never inherits a transaction through event dispatch.

<!-- sql: read -->
```sql
SELECT "id", "name" FROM "profile" WHERE ("id" = ?) LIMIT 1;
-- binds: [Text("u1")]
SELECT (COUNT(*)) AS "total" FROM "project" WHERE ((instr("title", ?) > 0));
-- binds: [Text("Ship")]
SELECT "id", "title", "owner_id" FROM "project" WHERE ((instr("title", ?) > 0)) ORDER BY "id" ASC LIMIT 1 OFFSET 1;
-- binds: [Text("Ship")]
SELECT "id", "name" FROM "profile" WHERE "id" IN (?) ORDER BY "id" ASC LIMIT 10001;
-- binds: [Text("u1")]
SELECT "id", "project_id", "title", "done" FROM "task" WHERE "project_id" IN (?) ORDER BY "id" ASC LIMIT 10000;
-- binds: [Text("p2")]
SELECT (COUNT(*)) AS "total" FROM "task";
-- binds: []
```

## Add a nullable field and explicitly rename a field

In [schema/002.idea](schema/002.idea), `Project` gains `summary String?` and `title` becomes `heading`. Generate the matching Rust before compiling the evolved app. A plain plan shows likely matches; it cannot approve the rename for you.

```sh
tool emit generate --input examples/data-quickstart/schema/002.idea --output examples/data-quickstart/src/generated/v2
tool emit revision --input examples/data-quickstart/schema/002.idea --history "$QUICKSTART_DIR/history" --app quickstart --name 002 --parent 001
tool emit migration-plan --history "$QUICKSTART_DIR/history" --from 001 --to 002 > "$QUICKSTART_DIR/unmapped.json"
cat "$QUICKSTART_DIR/unmapped.json"
tool emit migration-plan --history "$QUICKSTART_DIR/history" --from 001 --to 002 --rename project.title=heading > "$QUICKSTART_DIR/up.json"
cat "$QUICKSTART_DIR/history/migrations/$(field up plan)/sqlite-up.sql"
tool emit db-upgrade --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field up plan)"
cargo run --quiet --offline --locked -p data-quickstart --features native,evolved -- "$QUICKSTART_DIR/data.sqlite" > "$QUICKSTART_DIR/read-002.json"
```

The map uses checked SQL table/column names: `project.title=heading`. Existing titles survive as headings, `summary` is NULL, and task foreign keys still refer to the same project IDs. This edit uses native SQLite ALTER/index operations under the dedicated maintenance transaction.

<!-- sql: up -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
DROP INDEX "project_title_index";
-- binds: []
ALTER TABLE "project" ADD COLUMN "summary" TEXT;
-- binds: []
ALTER TABLE "project" RENAME COLUMN "title" TO "heading";
-- binds: []
CREATE INDEX "project_heading_index" ON "project" ("heading");
-- binds: []
```

## Reverse the safe change

The planner publishes a separate reverse recipe and hash. Here every `summary` is still NULL, so the reverse's data-preservation precondition holds. Writing a summary would cause refusal until that value was preserved elsewhere or cleared deliberately.

```sh
cat "$QUICKSTART_DIR/history/migrations/$(field up plan)/sqlite-down.sql"
tool emit db-downgrade --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field up reverse_plan)"
cargo run --quiet --offline --locked -p data-quickstart --features native -- "$QUICKSTART_DIR/data.sqlite" > "$QUICKSTART_DIR/read-reversed.json"
```

The old Rust app reads the original rows again. Source history still contains `002`; the database ledger accurately says `001`. A downgrade is an additional applied ledger entry, not deletion or rewriting of source history.

<!-- sql: down -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
DROP INDEX "project_heading_index";
-- binds: []
ALTER TABLE "project" DROP COLUMN "summary";
-- binds: []
ALTER TABLE "project" RENAME COLUMN "heading" TO "title";
-- binds: []
CREATE INDEX "project_title_index" ON "project" ("title");
-- binds: []
```

## Add a model, then inspect a refused loss

Return to `002` and add `Link` in [schema/003.idea](schema/003.idea). It has an owning Profile foreign key and a virtual inverse collection. Adding a model is supported, but the current planner refuses an automatic reverse that could discard that model's future rows. The safe round trip above deliberately happens before this separate step.

```sh
tool emit db-upgrade --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field up plan)"
tool emit generate --input examples/data-quickstart/schema/003.idea --output examples/data-quickstart/src/generated/v3
tool emit revision --input examples/data-quickstart/schema/003.idea --history "$QUICKSTART_DIR/history" --app quickstart --name 003 --parent 002
tool emit migration-plan --history "$QUICKSTART_DIR/history" --from 002 --to 003 > "$QUICKSTART_DIR/model.json"
cat "$QUICKSTART_DIR/model.json"
cat "$QUICKSTART_DIR/history/migrations/$(field model plan)/sqlite-up.sql"
tool emit db-upgrade --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field model plan)"
cargo run --quiet --offline --locked -p data-quickstart --features native,expanded -- "$QUICKSTART_DIR/data.sqlite" > "$QUICKSTART_DIR/read-003.json"
```

The new store reports `link_count: 0`; the original relations and counts remain intact. The plan says `reverse_available: false`. A reviewed custom reverse or backup restoration is a separate recovery operation.

<!-- sql: model -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
CREATE TABLE "link" ("id" TEXT NOT NULL, "owner_id" TEXT NOT NULL, PRIMARY KEY ("id"), CONSTRAINT "link_owner_id_fk" FOREIGN KEY ("owner_id") REFERENCES "profile" ("id") ON DELETE RESTRICT ON UPDATE NO ACTION);
-- binds: []
```

Now [schema/004-lossy.idea](schema/004-lossy.idea) removes `Task.done`. Planning is allowed for review; default execution refuses its `drop_column` finding. No generic force flag authorizes loss.

```sh
tool emit revision --input examples/data-quickstart/schema/004-lossy.idea --history "$QUICKSTART_DIR/history" --app quickstart --name 004 --parent 003
tool emit migration-plan --history "$QUICKSTART_DIR/history" --from 003 --to 004 > "$QUICKSTART_DIR/lossy.json"
cat "$QUICKSTART_DIR/lossy.json"
cat "$QUICKSTART_DIR/history/migrations/$(field lossy plan)/sqlite-up.sql"
if tool emit db-upgrade --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --plan "$(field lossy plan)" > "$QUICKSTART_DIR/refused.json" 2> "$QUICKSTART_DIR/refused.stderr"; then
  echo 'Expected the unapproved lossy migration to refuse' >&2
  exit 1
fi
tool emit db-status --history "$QUICKSTART_DIR/history" --database "$QUICKSTART_DIR/data.sqlite" --to 003 > "$QUICKSTART_DIR/status.json"
cargo run --quiet --offline --locked -p data-quickstart --features native,expanded -- "$QUICKSTART_DIR/data.sqlite" > "$QUICKSTART_DIR/read-after-refusal.json"
printf 'QUICKSTART_DIR=%s\n' "$QUICKSTART_DIR"
```

The database stays at `003`, and reading it gives the same rows and counts as before the refusal. Destructive approval, when appropriate, requires an exact plan hash plus an explicit target label and loss option; this tutorial does not approve it.

<!-- sql: lossy -->
```sql
-- Inspection only. Execute the checked plan.json recipe.
ALTER TABLE "task" DROP COLUMN "done";
-- binds: []
```

## Repeat the proof or continue

Run `python3 examples/data-quickstart/prove.py` to execute every shell block above under a parent watchdog, compare the shown SQL with actual artifacts, compile the exact Rust snippet, and verify the result and refused-step ledger. It also compares the complete closed database file before and after the refusal. It prints its unique disposable directory; the proof never opens an existing owner database. Generated files are included for ordinary consumers, but generation is proved with all three roots absent.

The default `data-quickstart` package contains only exact generated contracts/metadata; `native`, `evolved` and `expanded` choose the runtime and schema revision explicitly. No frozen contract-revision-2 shape is changed. See the [migration reference](../../crates/framework/framework-sql/docs/api/migrations.md) for bounds and operator commands and [task 00012's native store evidence](../../.agents/specs/00001-stackpress-rust-foundation/tasks/00012-receipt-sql-store-integration.md) for PostgreSQL reads/writes and MySQL reads/capability refusals. Their migrations remain render-only; MySQL string-key width is still an open product decision.
