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

Formatted number/date/phone edits are opt-in as described below. Rich text, file/media and relation services remain unavailable. The fixture principal and token below are explicit proof inputs; production authentication/session integration and installed targets are outside this example.


## Core admin CRUD (M5)

The normal generator now emits `ui/pages.rs` for Profile, Project and Task. The native host explicitly includes `ui/pages_host.rs`; its compiled registration selects actual generated Get, Detail, Search, Create and Update handlers, plus Remove/Restore for active Profile only. The generic batch, upsert and hard delete endpoints remain unavailable; M6 adds a guarded create-only import using the generated batch internally. Copy decodes a new create input and excludes the old identity. Updates are last writer wins, with no optimistic version check.

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

## Formatted catalog and polish

Each example crate has an opt-in `format` feature. `schema-format.idea` imports the ordinary fixture and extends Task with numeric, exact decimal String, date, time, explicit-offset instant and phone fields. The same app-owned generator registry writes the alternative marked root `shared/src/generated-format`; feature selection includes this root on both the client and host. Default builds continue to select `shared/src/generated` and gain no formatting dependency packages.

```sh
cargo run -p generated-admin-tool --features format --locked -- --input examples/generated-admin/schema-format.idea --output examples/generated-admin/shared/src/generated-format --check
python3 examples/generated-admin/c2_build.py
python3 examples/generated-admin/c2_plugin_build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/c2_browser.py
python3 examples/generated-admin/c2_compile_proof.py
python3 examples/generated-admin/c2_checks.py
python3 examples/generated-admin/c2_graph.py
python3 examples/generated-admin/c2_coverage.py
```

Catalog-2 scripts own `/private/tmp/s8-admin/catalog-2-admin-agent/` and one target directory, bounded to 8 GiB and deleted after evidence collection. They preserve other proof directories. Eight CRUD cases and eight compiled-plugin cases use the same finite intercepted HTTP/SQLite path as M5. They exercise formatted entry, rejected partial commits, preserved canonical values, actual writes, replacement and removal. They also check readable result links, pagination spacing, specific public error wording, presence-selector visibility, compact fractional time display, wide input sizing and focused-only accessible hints. See the [catalog reference](../../crates/framework/framework-admin/docs/api/catalog.md) for wire shapes and policies.

## Bounded CSV import/export (M6, G36)

The generated pages register `GET <model>/import`, `GET <model>/export`, and fixed `POST api/<model>/preview` / `confirm` routes. Import additionally selects the actual generated Batch handler, reachable only through the guarded create-only confirmation boundary. Arbitrary batch, upsert and hard delete remain unavailable as HTTP endpoints. The host rechecks authorization, Origin/CSRF, parent scope, contracts and identity conflicts on confirmation; visible links grant no authority.

Export includes **all filtered matches in the selected order**, ignoring pagination. It exports only the actual public stored projection in descriptor order. The import-only column/cell caps do not restrict export. Above the row/byte ceiling it refuses and suggests narrowing the filter; it never returns a truncated file. Empty search results produce a header-only download. Related export reimposes its verified parent FK and rejects a conflicting supplied filter.

Import reads a selected UTF-8 CSV file, displays every bounded row's raw cells, decoded public values and errors, and writes nothing during preview. Unknown, duplicate, private and nonwritable headers are refused. The header uses public field wire names. Missing identity/default/rule checks use actual generated create contracts; existing identities, including soft-removed records, and duplicates in the file are conflicts. No row is applied if any row is invalid. Confirmation explicitly resends the selected file and invokes one actual generated create-only batch transaction in input order. Related imports reimpose the verified parent FK; supplied mismatches block confirmation. Foreign-key or later write failures roll the transaction back. A lost commit acknowledgement shows an unknown outcome, locks this mounted form and never retries automatically. Check the records before starting another import.

The format uses comma separators, doubled embedded double quotes, and quoted export cells with CRLF record endings; import also accepts LF. Quoted cells may contain line breaks. Cell values are:

| Value | CSV cell after quote decoding |
| --- | --- |
| Scalar String, enum, date, time or datetime | Verbatim canonical string; generated contracts validate it |
| Number, Boolean, array or object | JSON (`0`, `false`, `["a"]`, `{"city":"Paris"}`) |
| Null | `\N` |
| Omitted/use default (import only) | `\O`; refused for required fields without defaults |
| Literal leading backslash | Double the first backslash (`\\N`, `\\O`, `\\path`) |

Export never emits omission. String cells beginning with `=`, `+`, `-`, `@`, tab or carriage return receive a leading apostrophe to neutralize spreadsheet formula injection. Original leading apostrophes are doubled. Numeric JSON, including negative numbers, is unchanged.

**Decode Stackpress export guards** is off by default: ordinary imports preserve every leading apostrophe literally. When explicitly enabled, `''` decodes to one original apostrophe and an apostrophe before `=+-@`, tab or carriage return decodes to the original string. Other apostrophes remain literal. This option is intended for Stackpress exports; the preview always shows raw and decoded values before confirmation. Thus a guarded export round-trips exactly with decoding enabled, while an ordinary import intentionally retains its guard/escape apostrophes. Leading-backslash and presence-token decoding apply independently of this option.

`host::Config::transfer_limits` accepts `stackpress_framework_admin::transfer::Limits`. Each limit may only be lowered: 1 MiB/file or exported CSV, 100 data rows, plus import-only 100 columns and 64 KiB/cell. The host checks raw size before UTF-8 conversion/CSV allocation and limits each append; the browser checks `File.size` before allocating its bytes. Preview serialization has a separate 1 MiB response ceiling and refuses excess rather than omitting rows. No file is stored on the server beyond the request; no background job or automatic retry exists. The browser retains only the selected draft until cancellation, replacement or root disposal.

```sh
python3 examples/generated-admin/m6_build.py
ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright" python3 examples/generated-admin/m6_browser.py
python3 examples/generated-admin/m6_compile_proof.py
python3 examples/generated-admin/m6_checks.py
python3 examples/generated-admin/m6_coverage.py
```

M6 scripts own only `/private/tmp/s8-admin/m6-admin-agent/`, use one target bounded to 8 GiB, and retain screenshots/downloads and exact command logs. Chromium runs both mounts in hydrate/CSR at 1280 px and 375 px/200%, intercepting `.invalid` HTTP and attachment downloads with no listener. Each case has a 120-second kill/reap watchdog. The trusted finite request wrapper converts its `body.csv`, `body.decode_export_guards` and `body.confirm` fields into an opaque `text/csv` request and explicit option/confirmation headers; these wrapper fields are not a public JSON upload endpoint.
