# Evolve the SQLite schema

Once the README query shows two articles, you can add a column while keeping those rows.
The generated stores describe the current schema; revision files retain earlier descriptions
so the existing SQL diff engine can plan the change. All commands here run from the blog
directory and exit without starting a listener.

## Add a nullable column

Open `schema.idea` and append this field at the **end of the Article block**, just before its
closing brace. Leave Article.contents and its editor/HTML metadata unchanged.

```idea
  subtitle Text? @label("Subtitle") @field.textarea @view.text
```

The `?` allows existing articles to have no subtitle. Appending the field also matches
SQLite's physical column order for a simple ADD COLUMN migration.

Regenerate the native stores, then record their new description:

```sh
cargo run -p blog-tool --locked --offline -- generate-app
cargo run -p blog-tool --locked --offline -- generate-app --check
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit revision
```

The explicit `revision` command captures the newly compiled generated stores and complete
source metadata. It writes a new `.build/revisions/<milliseconds>.json` only when the
serialized snapshot changes; repeating it without a change leaves the file bytes and
modification time alone. It does not alter the database. The original TypeScript transform
recorded revisions during generation; this native scaffold makes that step explicit.

Now ask for SQL, before applying anything:

```sh
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit migrate
```

Inspect `.build/migrations/`. The first revision produces baseline SQL that drops and
recreates the catalog; the next file describes the adjacent change, including the new
subtitle column. `migrate` writes files only. Its baseline is a reinstall script, not a
script to run over the populated database.

With the original setup checkpoint and the new revision both present, apply the latest pair:

```sh
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit push
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit query 'SELECT id, subtitle FROM article'
```

The two original article IDs remain, with null subtitles. The SQLite integration proof
checks the same native nullable-column operation, all fifteen seed-record counts, and native
failure/rollback behavior.

## Extend an imported model

To give profiles a nickname, add this to the app's existing `model Profile` block instead of
editing the imported framework schema:

```idea
  nickname String? @label("Nickname") @searchable @field.string @list.string @view.string
```

Run the same generate/check/revision/migrate/push sequence above, then rebuild both browser
modes with `python3 proofs/build.py` and restart the app through devmetrics. Open
`/admin/profile/update/developer` and save a Nickname. The generated detail page, an authorized
`/api/profile/search` response and `/auth/account/security/export?download=1` include that value.
The copied-out browser proof follows this path on a real SQLite upgrade and preserves both
seeded articles. Stock signup retains its original accepted fields.

## Command behavior

The finite host selects ordinary app registration functions in `web/src/lib.rs`. Its command
plugin passes actual generated table definitions and an explicit History binding to existing
framework SQL functions. It does not need full UI generation or a placeholder compiled client.

| Event after `emit` | Effect |
| --- | --- |
| `revision` | Record changed compiled source metadata/native definitions; no SQL. |
| `migrate` | Write the baseline and each adjacent revision's SQL under `database.migrations`. Warned plans are still written. |
| `push` | Fewer than two revisions: uninstall/install current tables, then record the snapshot. At least two: execute only the latest adjacent plan. |
| `upgrade` | Execute only the latest adjacent plan, if one exists. |
| `install` | Uninstall/install the current generated catalog, then record the snapshot. |
| `populate` | Dispatch the fifteen original seed events sequentially. |
| `purge` | Clear the generated tables through the existing SQLite DELETE/cascade mapping, retaining schema and revision files. |
| `uninstall` | Drop the generated tables, retaining revision files. |
| `query 'SQL'` | Execute the final argv token as trusted native SQL and expose its real `QueryResult` as JSON. |

The query result is `results = {rows, affected, insertId}`; each row maps column names to
native JSON values, with blobs represented as byte arrays. The source backslash-to-single-quote
conversion of the final query argument is retained. SQL failures exit unsuccessfully with the
native error chain preserved.

Destructive changes and ambiguous rename groups produce the source warning text. `migrate`
still writes that SQL; `push` and `upgrade` refuse to execute warned plans unless the operator
uses the CLI option **before** `emit`:

```sh
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json --force emit push
```

Use that option only after reviewing the actual intended schema change. A unique one-to-one
same-shape field rename preserves its values; ambiguous candidates are not guessed.

## History and execution boundaries

`client.revisions` defaults to `.build/revisions` and `database.migrations` to
`.build/migrations` in `web/src/config.rs` and the compiled source reference. Relative paths
resolve from the process working directory; `-b` environment layers can override them.
Startup opens the configured existing SQLite file but never reads/writes revisions or applies
SQL. Use the explicit `setup` binary to create a missing database file first.

The app's revision JSON contains `{source, tables}`: full source metadata plus mechanically
serialized generated SQL definitions. It is a native Rust format, not an importer for old
TypeScript revision files. The codec retains native CURRENT_TIMESTAMP defaults and defers
validation/diff/rendering to the SQL owner. It supports this SQLite catalog; custom SQL types,
custom defaults, MySQL storage metadata, and cyclic catalog ordering need separate support.

The original execution policy is retained. There is no applied-revision ledger: repeating
an adjacent ADD COLUMN push attempts it again and reports SQLite's duplicate-column error.
A push with zero or one revision reinstalls and clears existing data. Metadata-only changes
also create revisions, and changes recorded in the same millisecond overwrite that filename.

History writes and SQL are separate operations. A revision write failure after successful
installation is reported even though installation has already completed. Native SQL transaction
failures retain their rollback context; revision files use ordinary filesystem writes without
crash-atomic or automatic-repair guarantees. These behaviors are covered by owned disposable
SQLite fixtures, without changing framework/store policy.
