# Stackpress blog example

Start a Rust web project from the original Stackpress blog: import the framework schema,
generate eight model families and their admin, populate file SQLite, and share Leptos views
between SSR, hydration and CSR. Plugins are ordinary registration functions selected at build time.

## 1. Copy the boilerplate

Copy this directory without `.build/`, `target/` or `.env`. Keep the schema, manifests, standalone
lock, source and config together. Use the repository's cached Rust toolchain, dependencies and
matching `wasm-bindgen` CLI; this example installs nothing.

The Stackpress crates are unpublished. The `path = "../../crates/..."` and
`path = "../../tools/cli"` dependencies in `Cargo.toml` are repository-local. After copying the
folder out of the repository, point them at your Stackpress Rust checkout. Later, replace them
with published versions when available. This is a required setup step.

Run the following finite commands from the blog directory. Contributors run Cargo/proof commands
through the repository build-benchmark recorder with its repository store, as described in the
Rust contributor workflow. The commands below show the underlying application commands.

## 2. Read the schema, then generate

`schema.idea` imports `stackpress/stackpress.idea`, extends Profile, and adds PublishStatus,
Category, Catalog, Article and Comment. PackageLoader obtains imported sources from crate exports;
it never reads a hard-coded sibling schema path. The generator builds before output exists.

```sh
mkdir -p .build
cargo run -p blog-tool --locked --offline -- generate-app
cargo run -p blog-tool --locked --offline -- generate-app --check
```

The managed `.build/app-generated/` tree contains Profile, Auth, Application, Session, Category,
Catalog, Article and Comment. `web/src/generated/mod.rs` selects native contracts/stores/client;
`shared/src/lib.rs` selects only portable generated UI. Edit the schema and regenerate rather
than editing output. `--check` detects manual edits, deleted output and stale schema without
repairing files. Optional `generate` produces the separate native-only `.build/generated/`
bootstrap proof; it does not replace the full application output.

## 3. Set up and populate SQLite

```sh
cargo run -p blog-web --bin setup --locked --offline -- .build/blog.sqlite
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit revision
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit populate
cargo run -p blog-web --bin blog-web --locked --offline -- -b config/develop.json emit query 'SELECT COUNT(*) AS total FROM article'
```

Setup follows the source install behavior: it uninstalls existing tables before installing.
`revision` records the generated schema without changing SQL. Population dispatches all fifteen
original seed events in order; the query returns `results.rows[0].total = 2`. Startup only opens
an existing database; it never installs or populates. SQLite is this app's selected adapter.
MySQL/PostgreSQL have separate framework adapters; PGlite is unsupported.

Shared values in `web/src/config.rs` sit below the explicit environment JSON and later app layers.
The finite maintenance CLI uses the storage/command host; the web host adds the generated family,
session/auth/API, language, view, email, error hooks and assets. `.env.sample` retains the original
`abc123` seeds. The entries read `DATABASE_SEED` from the process; copying `.env.sample` does not
load environment variables automatically. Session configuration remains in the explicit layers.

## 4. Build the browser and run through devmetrics

```sh
python3 proofs/build.py
```

The finite pipeline compiles hydration and CSR separately and runs the already installed matching
`wasm-bindgen`. Its outputs are `.build/browser/{hydrate,csr}/blog.js` and `blog_bg.wasm`.
`assets/` contains authored source CSS and a small SVG brand; no upstream generated `public/`
output is copied. This replaces the Vite/UnoCSS pipeline; it is not an HMR watcher.
The shared native HTTP adapter explicitly allows32MiB responses for the unoptimized debug WASM;
request policies retain their framework defaults.

The explicit listener adapter is `web/src/bin/serve.rs`. It reads an existing SQLite file and a
JSON layer. Development launch must be managed:

```sh
devmetrics start --summary "Stackpress blog" -- 'cargo run -p blog-web --bin serve --locked --offline -- {port} config/develop.json'
```

Open **`http://localhost:<assigned-port>/`**, using the port printed by devmetrics. The app
binds only `127.0.0.1`; its public origin is `localhost`, and the existing native authority
allowlist also admits `127.0.0.1:<assigned-port>`. No framework host policy is changed.

Repository contributors also wrap that Cargo command in the benchmark recorder. Visit the
assigned URL for home, an article link for its raw HTML and
comments, and `/articles/missing` for the source 404 view. Add `?renderer=csr` to select CSR;
hydration is the default. Add `?json` to a public page to retain model results without its view.

## 5. Use the generated admin and API

Sign in at `/auth/signin/username` with the source seed `developer` / `developer`, then visit
`/admin/article/search`. Create/edit Article through generated controls, including the real rich
editor; `Article.contents` retains its original metadata and raw HTML display. Categories are
unowned. Catalog has two ID fields, so its create redirect returns to search. The native HTTP
host defaults the session cookie to `Path=/`, retaining explicit cookie options.

The source app seed has scopes `profile-write` / `auth-read`; the configured profile endpoints
require `user`. Its denial is intentional. A separately created Application with `user` scope
and a valid expiry can call `/api/profile/search` using `Authorization: Bearer <application-id>`.
The Expires editor accepts an instant with an explicit offset, such as `2099-01-01T00:00:00Z`;
blur or press Enter to apply it before saving.
Menu visibility does not grant permission; real session and endpoint checks still run.

The original language dictionaries are `en_US` and `th_TH`, including their demonstration copy.
The source prefix/data operation stages the locale cookie; a following request uses it. Public
page data keeps underscore names, while the checked document's `lang` uses `en-US` / `th-TH`.
The shared initial theme bootstrap is reused, with the Light no-JS fallback.

## 6. Add a column and migrate

Follow [Add a nullable column and inspect its SQL](docs/commands.md) for revision, `migrate`,
`push`, `purge` and query behavior. A regenerated nullable `Profile.nickname` fixture proves a
real SQLite upgrade preserves existing articles and reaches CRUD, sign-in, API joins, generated
admin and account CSV export. Stock signup keeps its original accepted fields.

Read the generated SQL before pushing it. With fewer than two revisions, source `push` reinstalls
and clears tables. There is no applied-revision ledger; repeating adjacent ADD COLUMN fails
natively. The commands guide explains those retained source semantics.

## Source choices and boundaries

- G52 removes only the dangling `Category.profile` edge; Article/Comment relations and framework
  validation remain unchanged.
- G52 intentionally changes the home handler to a real `eq.status = PUBLISHED` filter. It overrides
  a caller's status filter, preserves a caller's sort, and leaves detail-route behavior unchanged.
- MCP / `stackpress-ai` is unsupported. The original config, including its `content`/`contents`
  mismatch, is retained as [reference data](docs/mcp-reference.json).
- Desktop/Tauri packaging is a later step, outside this first web release.
- The full host selects existing email/webhook mechanisms with original config. No external mail
  or webhook delivery is proved here. Owned test/smoke fixtures explicitly disable webhook URLs
  to keep proofs offline. Configure a real transport separately when your app needs delivery.
- Both renderer modes pass the cached in-process public browser matrix at1280px,375px and
  200% mobile zoom: no-JS first paint, head/assets, keyboard links, article/comments,404, live
  theme interaction and idempotent root disposal. Remote seed images are fulfilled locally.
- The copied-out proof uses a byte-identical lock and a real SQLite upgrade. Form serialization,
  seed selection and relation response/error decisions now live in target-independent
  `blog-shared` functions with native tests and coverage. The existing framework RelationField
  builds search URLs in plain Rust; the browser adapter forwards them unchanged.
- G56 exempts `blog-client` from the numeric coverage floor: it contains startup/mount/disposal
  and thin DOM/navigation/fetch/AbortController glue only. Real hydration/CSR browser tests must
  exercise the entry; the gate requires both public and full browser flows. This exemption is
  not a claim of 100% coverage. The missing WASM profiler runtime is no longer a gate blocker.
- Authenticated browser proofs sign in through the real source form, update rich content,
  create Category/Catalog and a scoped Application, verify API denial/success, and export CSV.
  The copied-out app adds Nickname through regeneration/migration and edits it in both renderers.

## Finite contributor proofs

```sh
python3 proof.py storage
python3 proof.py generation
python3 proofs/extension.py
python3 proofs/graphs.py
python3 proofs/browser.py --public-only --matrix
python3 proofs/gate.py
python3 proofs/locks.py check --release blog-post-g57
```

Generate the native bootstrap, extension fixture and browser artifacts before the complete
integration tests. The repository gate also requires the prepared copied-out nickname fixture
(`proofs/nickname.py prepare`); its root-path manifest patch is reviewed before that proof runs.
`storage` includes the Catalog redirect, native HTTP Date and cookie-scope regressions.
The gate checks all authored Rust formatting while generation freshness checks the managed
output. It runs native and renderer-specific Clippy/docs/tests/graphs/browser proofs and exports
native LLVM coverage with fresh per-run profiles and the explicit G56 browser-entry exemption.
`web/src/bin/probe.rs` is a finite stdin/stdout real-HTTP/SQLite fixture for later cached Playwright
routing. It opens no socket. The lock audit uses an ignored repository baseline and is not a
copied-out app runtime requirement. No command installs packages or updates registry versions.
