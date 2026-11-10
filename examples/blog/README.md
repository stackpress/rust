# Stackpress blog example

A copyable Rust port of the original Stackpress `templates/blog`. It imports the framework
schema, generates models and admin pages, selects plugins through registration functions,
and stores data in a SQLite file. Leptos shares the views across server rendering (SSR),
browser hydration and client rendering (CSR).

The example includes:

- A public blog with articles and comments.
- Username sign-in and generated admin pages with a rich text editor.
- An API with application scopes and account CSV export.
- English (`en_US`) and Thai (`th_TH`) dictionaries.

## Project layout

The plugins are unpublished crates, selected before compilation by calling their registration
functions. Each folder maps to a part of the original template:

| Folder | Responsibility and original counterpart |
| --- | --- |
| `plugins/app/` (`blog-app`) | Original `plugins/app`: `pages/` and error hooks behind `server`; Leptos `views/`, helpers, wire data and generated admin selection behind `ui`. `ssr`, `hydrate` and `csr` select the renderer. |
| `plugins/assets/` (`blog-assets`) | Original `plugins/assets`: registration, authored CSS/brand and bootstrap JavaScript; serves separately compiled `.build/browser/` products. |
| `plugins/store/` (`blog-store`) | Original `plugins/store`: registration and `connect.rs` own file SQLite acquisition; startup opens an existing database. |
| `web/` (`blog-web`) | Original config and application entry: config, History, seed, HTTP, rendering and binaries; `client.rs` selects all three plugins by calling their registration functions. |
| `client/` (`blog-client`) | Thin browser startup/DOM/fetch glue; its app dependency enables only `ui` and hydration or CSR. |
| `tool/` (`blog-tool`) | Independent official schema/source generator, unchanged by plugin selection. |

`xtask/` builds browser bundles and runs the development loop; `tool/` stays focused on schema generation.

## Prerequisites

- Rust 1.96 or newer, with Cargo, and a Stackpress Rust checkout.
- The `wasm32-unknown-unknown` Rust target.
- `wasm-bindgen` CLI **0.2.129**, matching `wasm-bindgen` in `Cargo.lock`, on your PATH.

The commands use `--locked --offline`, so the locked Cargo dependencies must already be
cached. The normal setup and tests use only the tools above.

## Setup

Follow these steps from a fresh copy. All commands run from the copied blog directory.

### 1. Copy and connect the framework

Copy this directory without `.build/`, `target/` or `.env`. Keep its `Cargo.lock`.
The Stackpress crates are unpublished: **edit the `path = "../../crates/..."` and
`path = "../../tools/cli"` dependencies in `Cargo.toml` to point at your Stackpress Rust
checkout**. Keep paths between the example's own crates unchanged.

### 2. Generate the app

`schema.idea` imports `stackpress/stackpress.idea` and adds the blog models. Generate their
Rust code and admin UI, then check that the output matches the schema:

```sh
mkdir -p .build
cargo run -p blog-tool --locked --offline -- generate-app
cargo run -p blog-tool --locked --offline -- generate-app --check
```

The output is in `.build/app-generated/`. Edit the schema and regenerate instead of editing
these files; `--check` reports missing, edited or stale output without repairing it.

### 3. Create and populate SQLite

Use this sequence on the fresh copy. **Setup clears any existing tables.** The revision
records the starting schema so a later migration can compare it with your changes.

```sh
cargo run -p blog-web --bin setup --locked --offline -- .build/blog.sqlite
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit revision
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit populate
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit query 'SELECT COUNT(*) AS total FROM article'
```

The query should return `results.rows[0].total = 2`. The server opens this database;
it does not create tables or populate them on startup. `.env.sample` is a reference and
is not loaded automatically; the seed defaults to `abc123` unless you set `DATABASE_SEED`.

### 4. Build the browser bundles

Compile hydration and CSR and emit their JavaScript/WASM bundles:

```sh
cargo xtask build-browser
```

Both bundles are written under `.build/browser/` and served by the assets plugin.

### 5. Run the server

Replace `<port>` with a free port, such as `3000`:

```sh
cargo run -p blog-web --bin serve -- <port> config/develop.json
```

Open `http://localhost:<port>/`. The server binds to IPv4 loopback. Stop it with Ctrl-C.

## Develop with live reload

After setup, run:

```sh
cargo xtask dev --port 3000 --reload-port 3001
```

Open `http://localhost:3000/`. Both flags are optional: the app defaults to port 3000,
and the reload socket defaults to the app port plus one. Ctrl-C stops the loop and its server.

The loop watches the app's plugins, web and browser sources, assets, schema and config.
It debounces edits, regenerates after schema changes, builds the affected targets and
restarts the server when needed. Hydration and native builds run concurrently, with incremental
compilation enabled even if your shell disables it. A failed critical build keeps the last good
server and bundles running.

The page reloads as soon as the server is ready and hydration is published. CSR builds afterward
in the background; `?renderer=csr` tabs reload when that complete bundle is published. A failed
CSR build keeps its previous bundle. New edits cancel superseded background builds.
Schema changes still need the explicit database migration commands below.

CSS edits swap the stylesheet without reloading. In hydration mode, static text and literal
HTML attributes in `plugins/app/src/views/` hot-patch in place. Entirely static `view!` blocks
can also change their markup structure. Logic, signals, component props, embedded Rust,
changed macro locations and uncertain edits take a full rebuild and reload; the terminal prints
why a view edit could not be patched. Static patches remain available while CSR builds.
CSS and accepted markup normally update after the 250ms debounce; compiled edits depend on
project size and the warm compiler cache. CSR pages, missing view markers and failed patches also fall back to a full rebuild and reload.
New pages replay accepted patches; the next full build compiles those changes into SSR and WASM.
The reload script is included only in the dev environment set by xtask.

## Try it

- Sign in as **developer / developer** at `/auth/signin/username`.
- Open `/admin/article/search`, `/admin/category/search`, `/admin/catalog/search`, or
  `/admin/profile/search`. Create or edit an Article with the rich editor. Catalog has two
  ID fields, so creating a Catalog returns to search.
- Create an Application at `/admin/application/search` with the **user** scope and a future
  expiry, for example `2099-01-01T00:00:00Z`; blur or press Enter before saving the expiry.
  Call `/api/profile/search` with `Authorization: Bearer <application-id>`. The seeded
  Application lacks `user` scope and is intentionally denied.
- Export account data at `/auth/account/security/export?download=1`.
- Add `?renderer=csr` to a public page to try CSR; hydration is the default.
  Add `?json` to return page data without its view. Visit `/articles/missing` for the 404 page.

## Command reference

Run these from the blog directory. The database events use `config/develop.json`;
[command behavior and migration details](docs/commands.md) explain their effects.

| Group | Command | Purpose |
| --- | --- | --- |
| Generate | `cargo run -p blog-tool --locked --offline -- generate-app` | Generate the app models, stores and admin UI. |
| Generate | `cargo run -p blog-tool --locked --offline -- generate-app --check` | Check generated output without changing it. |
| Generate | `cargo run -p blog-tool --locked --offline -- generate` | Emit the separate native bootstrap proof in `.build/generated/`. |
| Database | `cargo run -p blog-web --bin setup --locked --offline -- .build/blog.sqlite` | Create the SQLite file and reinstall its tables, clearing data. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit revision` | Record the compiled schema when it changes; no SQL. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit populate` | Insert the original seed data. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit query 'SELECT COUNT(*) AS total FROM article'` | Execute SQL and print its result as JSON. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit migrate` | Write baseline and adjacent-revision SQL for inspection. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit push` | Apply the latest pair; fewer than two revisions reinstalls and clears data. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit upgrade` | Apply only the latest adjacent plan, if present. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit install` | Reinstall the current tables and record the schema, clearing data. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit purge` | Clear rows while retaining tables and revision files. |
| Database | `cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit uninstall` | Drop tables while retaining revision files. |
| Build & run | `cargo xtask build-browser` | Build both browser bundles; exits when finished. |
| Build & run | `cargo xtask dev --port 3000 --reload-port 3001` | Watch, rebuild and restart with CSS swaps and safe markup hot patches. |
| Build & run | `cargo run -p blog-web --bin serve -- <port> config/develop.json` | Serve the existing database and built bundles on the chosen port. |
| Test | `cargo test --workspace --locked --offline` | Run the normal workspace tests after setup. |

## Add a column and migrate

After setup, append this nullable field at the **end of the Article block** in `schema.idea`,
just before its closing brace. Existing articles can leave it empty.

```idea
  subtitle Text? @label("Subtitle") @field.textarea @view.text
```

Regenerate, record the changed schema, and write the SQL:

```sh
cargo run -p blog-tool --locked --offline -- generate-app
cargo run -p blog-tool --locked --offline -- generate-app --check
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit revision
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit migrate
```

Inspect `.build/migrations/`. The baseline SQL reinstalls tables; the latest adjacent plan
should add `subtitle`. With the original and new revisions present, apply that pair:

```sh
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit push
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit query 'SELECT id, subtitle FROM article'
```

The two article IDs should remain, with null subtitles. Rebuild the browser bundles and
restart the server to use the new admin field. There is no applied-revision ledger: repeating
this push fails with a duplicate-column error. See [the commands guide](docs/commands.md)
for imported-model extensions and migration semantics.

## Test

After setup, run the normal test suite without starting a server:

```sh
cargo test --workspace --locked --offline
```

Repository contributors: see [the full Verify gate](../../tools/examples/blog/README.md)
for additional generated fixtures, browser flows, coverage and lock checks.
See [design notes](docs/design-notes.md) for source choices and boundaries.

## Limitations

- MCP / `stackpress-ai` is unsupported; the original config is kept as [reference data](docs/mcp-reference.json).
- Desktop/Tauri packaging is a later step; this example runs on the web.
- Email and webhook delivery are configured but not exercised by the proofs.
- Hot patches cover static markup in shared app views; other changes rebuild and reload.
- Debug hydration WASM is about 24 MB (4.7 MB gzipped); no release or size profile exists yet.
