# Generated admin example

This example imports the actual generated-workspace Profile, Project, Task, and Address schema. Its independent app-owned tool composes schema, SQL, and UI functions into one output root. The shared frontend includes public contracts, role wrappers, and generated page adapters. Its browser graph excludes stores. The native host includes the actual generated stores and selected operations and uses real SQLite. RoleProof is an additional normalized boundary fixture.

From the repository root:

```sh
cargo run -p generated-admin-tool --locked -- emit generate
cargo run -p generated-admin-tool --locked -- emit generate --check
cargo run -p generated-admin-web --locked > /private/tmp/generated-admin.html
```

`--input PATH` and `--output PATH` select disposable inputs/marked roots. `--plugin` selects the compiled app replacement. A check against another selection fails without changing output. The CLI accepts optional `emit generate` words and Boolean flags; it does not accept `--check true`.

All finite proofs use one parameterized driver and an already installed matching wasm-bindgen and Playwright. Set `ADMIN_PROOF_ROOT` to a directory owned by this run (default `/private/tmp/s8-admin`). Every suite shares its single `target` subdirectory; logs, copied artifacts, downloads and screenshots have separate suite subdirectories. Children have kill/reap watchdogs; browser cases have a 120-second parent and a 105-second inner deadline. The target is measured during commands and refused above 8 GiB. Delete only that owned target after collecting evidence.

```sh
export ADMIN_PROOF_ROOT="/private/tmp/s8-admin/my-admin-proof"
export ADMIN_PLAYWRIGHT="/absolute/path/to/installed/playwright"
python3 examples/generated-admin/proof.py build --suite roles
python3 examples/generated-admin/proof.py browser --suite roles
python3 examples/generated-admin/proof.py build --suite crud
python3 examples/generated-admin/proof.py browser --suite crud
python3 examples/generated-admin/proof.py browser --suite races
python3 examples/generated-admin/proof.py build --suite plugin
python3 examples/generated-admin/proof.py browser --suite format
python3 examples/generated-admin/proof.py build --suite transfer
python3 examples/generated-admin/proof.py browser --suite transfer
python3 examples/generated-admin/proof.py compile
python3 examples/generated-admin/proof.py checks
python3 examples/generated-admin/proof.py graphs
python3 examples/generated-admin/proof.py coverage
```

The `plugin` build prepares both formatted replacement and disabled artifacts for `format` browser cases. `races` reuses `crud` artifacts. Regeneration for plugin/disabled builds happens in a disposable workspace snapshot with current changes overlaid. These proofs never rewrite tracked generated roots. `compile` checks bootstrap, determinism, model rename/removal and the real stale-consumer compiler failure in disposable roots. `checks` explicitly skips the listener-binding `tcp_watchdog`; the coordinator verifies it separately. `probe --suite crud` runs the actual finite SQLite/HTTP smoke test. The former milestone Python entry points remain thin compatibility wrappers around these same implementations. `report` re-exports existing coverage counters without repeating tests (the compatibility `coverage_report.py` entry point uses it).

| Suite | Retained browser cases |
| --- | --- |
| `roles` (M4/catalog 1) | Expanded from 16 to 24: baseline/plugin/disabled × hydrate/CSR × both mounts × wide/narrow |
| `crud` (M5) | 8: both mounts × hydrate/CSR × 1280 px / 375 px at 200% |
| `format` (catalog 2) | 16: the same 8 base cases plus 8 compiled replacement cases |
| `transfer` (M6) | 8: upload/preview/confirm/download, all mounts/modes/layouts |
| `races` (closeout) | 32: create/update × success/validation/denial/uncertainty × both mounts/modes |

Each intercepted `.invalid` request executes buffered HTTP and actual generated operations against an owned SQLite fixture. The role gallery instead uses finite fixture rendering. No proof starts a listener or installs a package. `proofs/http_browser.cjs` owns the common HTTP/browser harness; separate scenario modules retain the milestone assertions. Python runtime tests exercise timeout kill/reap and failure/environment propagation (`python3 -m unittest discover -s examples/generated-admin -t examples/generated-admin -p test_runtime.py`).

The generated wrappers implement five independent placements. Missing roles stay absent; public descriptors omit sensitive/host-only fields. Native drafts distinguish omission/null/value and preserve defaults, false, zero, and empty strings. Browser assertions cover initial SSR/hydrated DOM identity, controlled edits, unique label/control IDs, plugin replacement/removal, and no captured browser errors.

Catalog breadth part 1 adds text/tag/metadata collections, a numeric slider, rating, explicit slug normalization, local enum combobox selection and a generated list-role table. Chromium checks edits/order/stable identities, duplicate metadata errors, omission/null/empty/zero, keyboard selection, both mounts and 375 px at 200% zoom. The [catalog reference](../../crates/framework/framework-admin/docs/api/catalog.md) describes exact value shapes and bounds.

Formatted number/date/phone edits are opt-in as described below. Rich text, file/media and relation services remain unavailable. The fixture principal and token below are explicit proof inputs; production authentication/session integration and installed targets are outside this example.


## Core admin CRUD (M5)

The normal generator now emits `ui/pages.rs` for Profile, Project and Task. The native host explicitly includes `ui/pages_host.rs`; its compiled registration selects actual generated Get, Detail, Search, Create and Update handlers, plus Remove/Restore for active Profile only. The generic batch, upsert and hard delete endpoints remain unavailable; M6 adds a guarded create-only import using the generated batch internally. Copy decodes a new create input and excludes the old identity. Updates are last writer wins, with no optimistic version check.

Search links encode one strict `state` JSON value. Null removes an equality filter; false and zero remain values. Order cycles absent → descending → ascending → absent. Skip and page size are validated by both the UI state and actual generated query. Related links include a checked compiled parent mapping, authorize and load the parent through generated Get, and reimpose its foreign key on search/create. There is no unbounded relation lookup service.

The page/API routes use buffered HTTP with no listening socket. A trusted finite harness supplies `actor` (`editor`, `reader`, or denied); `FixturePolicy` is a labeled proof policy, not a login implementation. Mutation admission also requires the exact proof Origin and explicit fixture CSRF token. Production deployments must provide their own authentication/session policy and anti-forgery token lifecycle.

Forms use one live draft. Public field validation errors preserve the draft and focusable controls. Acknowledgements belong to a structural snapshot of the submitted draft. An unchanged (or exactly reverted) draft shows Saved. Later edits remain editable and unsaved: “Saved the submitted version. You have newer unsaved changes.” Server echoes never overwrite controls. Pending submissions block duplicate requests before local field validation runs, so later partial edits cannot release Pending or UnknownOutcome. Errors remain marked “Submitted version” until the next admitted request, so later edits are not described as invalid by an older result. UnknownOutcome explicitly refers to the submitted version; UnknownOutcome says the write may have completed, disables resubmission, and never retries automatically. The trusted fault fixture loses an acknowledgement **after a real SQLite commit**; it is inaccessible through browser operation JSON. Public output excludes host diagnostics, SQL, sensitive and host-only fields.

For a single process request, build `generated-admin-web` and pass `--request` with one JSON stdin object containing `database`, `mount`, `actor`, `method`, `path`, optional `body` and headers. `initialize: true` installs generated tables only into a previously absent owned file and exits. Browser JSON cannot choose the file or fault mode. Each request exits after explicit pool close. The old no-argument finite catalog render remains available for the earlier role proofs.

## Formatted catalog and polish

Each example crate has an opt-in `format` feature. `schema-format.idea` imports the ordinary fixture and extends Task with numeric, exact decimal String, date, time, explicit-offset instant and phone fields. The same app-owned generator registry writes the alternative marked root `shared/src/generated-format`; feature selection includes this root on both the client and host. Default builds continue to select `shared/src/generated` and gain no formatting dependency packages.

The `format` suite retains formatted entry, rejected partial commits, preserved canonical values, actual writes, compiled replacement and removal. Its eight base cases additionally exercise delayed Saved/Unknown acknowledgements with a newer formatted draft and repeated submit events. It also asserts readable result links, pagination spacing, specific public error wording, presence-selector visibility, compact fractional time display, wide input sizing and focused-only accessible hints. The graph comparison preserves the existing URL/IDNA ICU baseline and permits no new default normal-edge packages. See the [catalog reference](../../crates/framework/framework-admin/docs/api/catalog.md) for wire shapes and policies.

## Bounded CSV import/export (M6, G36)

The generated pages register `GET <model>/import`, `GET <model>/export`, and fixed `POST api/<model>/preview` / `confirm` routes. Import additionally selects the actual generated Batch handler, reachable only through the guarded create-only confirmation boundary. Arbitrary batch, upsert and hard delete remain unavailable as HTTP endpoints. The host rechecks authorization, Origin/CSRF, parent scope, contracts and identity conflicts on confirmation; visible links grant no authority.

Export includes **all filtered matches in the selected order**, ignoring pagination. It exports only the actual public stored projection in descriptor order. The import-only column/cell caps do not restrict export. Above the row/byte ceiling it refuses and suggests narrowing the filter; it never returns a truncated file. Empty search results produce a header-only download. Related export reimposes its verified parent FK and rejects a conflicting supplied filter.

Import reads a selected UTF-8 CSV file, displays every bounded row's raw cells, decoded public values and errors, and writes nothing during preview. Unknown, duplicate, private and nonwritable headers are refused. The header uses public field wire names. Missing identity/default/rule checks use actual generated create contracts; existing identities, including soft-removed records, and duplicates in the file are conflicts. No row is applied if any row is invalid. A successful preview grants confirmation only for that exact selection and decode option. Replacing the file (even with identical bytes), cancelling, or changing the decode option requires a new preview. Pending requests retain their lock while file controls stay editable; their acknowledgements refer to the submitted file and preserve any newer selection. Confirmation explicitly resends the previewed file and invokes one actual generated create-only batch transaction in input order. Related imports reimpose the verified parent FK; supplied mismatches block confirmation. Foreign-key or later write failures roll the transaction back. A lost commit acknowledgement shows an unknown outcome, locks this mounted form and never retries automatically. Check the records before starting another import.

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

The `transfer` proof retains all previous M6 cases and adds singular row wording, readable “1 MB” / “64 KB” limits, labelled stacked preview cells at 375 px/200%, no page overflow, stale-preview refusal and delayed-confirm acknowledgement with a newer file. Byte admission still uses the exact binary quantities stated above. The trusted finite request wrapper converts `body.csv`, `body.decode_export_guards` and `body.confirm` into an opaque `text/csv` request and explicit option/confirmation headers; those wrapper fields are not a public JSON upload endpoint.
