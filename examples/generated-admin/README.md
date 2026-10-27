# Generated admin example

This M4 example imports the actual generated-workspace Profile, Project, Task, and Address schema. Its independent app-owned tool composes schema, SQL, and UI functions into one output root. The shared frontend includes public contracts, role wrappers, and generated page adapters. Its browser graph excludes stores. The native host includes the actual generated stores and selected operations and uses real SQLite. RoleProof is an additional normalized boundary fixture.

From the repository root:

```sh
cargo run -p generated-admin-tool --locked -- emit generate
cargo run -p generated-admin-tool --locked -- emit generate --check
cargo run -p generated-admin-web --locked > /private/tmp/generated-admin.html
```

`--input PATH` and `--output PATH` select disposable inputs/marked roots. `--plugin` selects the compiled app replacement. A check against another selection fails without changing output. The CLI accepts optional `emit generate` words and Boolean flags; it does not accept `--check true`.

For finite browser evidence with the already installed matching wasm-bindgen and Playwright:

```sh
python3 examples/generated-admin/build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/browser.py
python3 examples/generated-admin/compile_proof.py
```

The browser proof intercepts `https://admin.invalid` at `/` and `/admin/`, uses file-backed assets and a finite stdout SSR process, and starts no listener. Its parent kills/reaps after 120 seconds. Logs, exact command results, assets, compile failures, and screenshots live in `/private/tmp/s8-admin/`. The build driver preserves baseline/plugin/disabled binaries for hydrate and CSR and restores default generated source in its final step.

The generated wrappers implement five independent placements. Missing roles stay absent; public descriptors omit sensitive/host-only fields. Native drafts distinguish omission/null/value and preserve defaults, false, zero, and empty strings. Browser assertions cover initial SSR/hydrated DOM identity, controlled edits, unique label/control IDs, plugin replacement/removal, and no captured browser errors.

Catalog breadth part 1 adds text/tag/metadata collections, a numeric slider, rating, explicit slug normalization, local enum combobox selection and a generated list-role table. Chromium checks edits/order/stable identities, duplicate metadata errors, omission/null/empty/zero, keyboard selection, both mounts and 375 px at 200% zoom. The [catalog reference](../../crates/framework/framework-admin/docs/api/catalog.md) describes exact value shapes and bounds.

Formatted number/date/phone edits, rich text, file/media and relation services remain unavailable. The fixture principal and token below are explicit proof inputs; production authentication/session integration and installed targets are outside this example.


## Core admin CRUD (M5)

The normal generator now emits `ui/pages.rs` for Profile, Project and Task. The native host explicitly includes `ui/pages_host.rs`; its compiled registration selects actual generated Get, Detail, Search, Create and Update handlers, plus Remove/Restore for active Profile only. Batch, upsert and hard delete remain unavailable over the admin HTTP surface. Copy decodes a new create input and excludes the old identity. Updates are last writer wins, with no optimistic version check.

Search links encode one strict `state` JSON value. Null removes an equality filter; false and zero remain values. Order cycles absent → descending → ascending → absent. Skip and page size are validated by both the UI state and actual generated query. Related links include a checked compiled parent mapping, authorize and load the parent through generated Get, and reimpose its foreign key on search/create. There is no unbounded relation lookup service.

The page/API routes use buffered HTTP with no listening socket. A trusted finite harness supplies `actor` (`editor`, `reader`, or denied); `FixturePolicy` is a labeled proof policy, not a login implementation. Mutation admission also requires the exact proof Origin and explicit fixture CSRF token. Production deployments must provide their own authentication/session policy and anti-forgery token lifecycle.

Forms use one live draft. Public field validation errors preserve the draft and focusable controls. Acknowledged writes show Saved; UnknownOutcome says the write may have completed, disables resubmission, and never retries automatically. The trusted fault fixture loses an acknowledgement **after a real SQLite commit**; it is inaccessible through browser operation JSON. Public output excludes host diagnostics, SQL, sensitive and host-only fields.

Run finite M5 evidence using the one owned target (matching installed wasm-bindgen required):

```sh
python3 examples/generated-admin/m5_build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/m5_browser.py
python3 examples/generated-admin/m5_compile_proof.py
```

These scripts keep M5 logs, copied finite binaries, assets and screenshots in `/private/tmp/s8-admin/m5-admin-agent/`, separate from other proofs. Each Chromium case has a 120-second parent watchdog. Both `/` and `/admin/` mounts run hydrate and CSR at 1280 px and 375 px/200%. Every intercepted API call executes the real buffered HTTP adapter and generated operations against the case's owned SQLite file. No listener or database write simulator is used.

For a single process request, build `generated-admin-web` and pass `--request` with one JSON stdin object containing `database`, `mount`, `actor`, `method`, `path`, optional `body` and headers. `initialize: true` installs generated tables only into a previously absent owned file and exits. Browser JSON cannot choose the file or fault mode. Each request exits after explicit pool close. The old no-argument finite catalog render remains available for the earlier role proofs.
