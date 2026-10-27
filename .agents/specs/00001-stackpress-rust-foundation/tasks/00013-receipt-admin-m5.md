# Task 00013 — Slice 8 M5 core admin CRUD

Implementation receipt, 2026-10-27. Coordinator review pending; no commit, push, acceptance, publication or M6 claim.

## Scope and ownership

- Requested `git merge trial/3` fast-forwarded `787eaa3..44aa10f`, including Slice 6 M2. The sandbox initially refused external Git ORIG_HEAD; the authorized merge succeeded with escalation.
- Changes stay within framework-admin, examples/generated-admin, Cargo.lock's existing dependency edges and this receipt. No framework-schema/framework-sql/framework-ui/framework-view, server, SQL, Idea, generated-workspace or profile-ui source edits. Frozen schema/UI revision 2 is unchanged.
- The same app-owned registry/single writer owns contracts, metadata, stores and UI partitions. SQL emits actual M2 stores/operations; UI adds pure typed pages and a separate explicit native registration include. No second generator or handwritten operation substitute.
- No listener, devmetrics, install, new crates.io package/version, commit or push. Import/export and installed targets remain M6; formatted number/date/phone, rich text, file/media and service-dependent relation inputs stay deferred/unavailable.

## Actual generated CRUD

- `emit_pages` selects present Profile/Project/Task models with complete public identities. Pure `ui/pages.rs` composes selected field/filter/list/view roles, actual strict public create/patch DTOs, validated generated query types and typed public rows. Native `pages_host.rs` explicitly registers actual named handlers and fixed compiled routes (G20), outside the browser aggregate.
- Search/detail/create/update/copy use actual generated operations. Profile alone has explicit remove/restore because its data layer supports active-only removal. Batch/upsert/hard-delete admin APIs remain unavailable. Copy seeds exclude old identity and require a new create identity; update excludes identity and retains data-layer last-writer-wins semantics.
- `SearchQuery` has strict bounded one-decode URL state. Null removes a filter; false/zero remain values. Order cycles absent → desc → asc → absent. Public names/capabilities, values, skip and limit are checked before execution, then actual generated queries validate again. URL deep links enforce server policy independently of visibility.
- Related search/create checks compiled public forward mappings, authorizes/loads the parent through generated Get, reimposes its FK on search/create and rejects mismatches. Related form/filter FK controls are disabled. Scoped detail cannot load another parent's child. Related update/copy/remove/restore are unavailable; actual data-layer query/include budgets remain authoritative.
- A live draft survives validation, authorization and acknowledgement errors. Public field issues reach controls and a summary. Pending requests cannot be submitted twice. Only acknowledged writes show Saved. UnknownOutcome explains the possible effect, preserves the draft, disables resubmission and never automatically retries. Read failures do not claim mutation uncertainty.
- The native host executes generated definitions/handlers against real SQLite through buffered HttpAdapter requests, with no socket. A trusted finite fault adapter loses acknowledgement only after an actual SQLite autocommit effect; generated Get proves the write. Browser JSON cannot select database, actor or fault mode.
- Authentication is explicitly a finite fixture policy, not login/session support: trusted process configuration supplies editor/reader/unknown. Mutation admission also requires exact proof Origin and an explicit fixture CSRF token. Production authentication/session and token lifecycle remain app-owned. Copy/Update admission and visible actions require initial Get too. Policy precedes DTO/URL decoding; actual generated handlers authorize independently again.
- Only public projections enter adapters, SSR, browser seeds and results. Inline JSON escapes closing tags/HTML delimiters. SQL, host diagnostics, sensitive/host-only fields remain private. Native/browser compile the same writer-owned contract source in isolated graphs.

## Four test-verified review rounds

1. Logic/complexity: reviewed omission/null/default/false/zero and effect/ack ordering; added app-owned nullable Task.note and default-zero Task.priority, copy identity stripping, related field/filter locks and honest read-failure state. `round-1` passes.
2. Responsibility/abstraction: moved model dispatch, actual DTO/query codecs, form/filter composition and public table/detail rendering into generated adapters; centralized model/registration selection and removed handwritten model switches. Reserved native Schema/Operations modules and repeated Public prefixes retain exact identities. `round-2` passes.
3. Consistency/Rustdoc/comments: documented pure/native output, URL/mutation/error semantics, fixture authority and finite reproduction, added the page/search API reference and corrected prior milestone documentation; removed duplicate normalization and hid unavailable text search. Actual typed adapters execute all selected models. `round-3-verified`, `pages-native` pass.
4. Safety: required Get for Copy/Update admission/visibility, rejected unused API parameters, prevented reserved fixture-header principal replacement, retained both primary/cleanup causes, checked scoped-detail isolation and refused unselected APIs. `round-4` passes. Browser follow-up found 5 px textarea overflow at 375 px/200%; an example-owned width constraint fixes it and the entire repeated matrix passes.

Generator follow-up found that omitted field roles needed an explicitly typed empty form, and fully removed selections needed direct unavailable functions rather than empty match statements. Those fixtures now compile, including warning-clean Clippy for the empty selection. Choosing imports before embedding descriptors also fixes literal-label corruption from old whole-source import removal (including `{Role, SchemaError`); label-like import text is now preserved exactly. `m5-compile-omitted-*`, `m5-compile-empty-*` and the entire repeated final gate pass.

Coverage-driven tests also execute independently compiled native generated DTOs, all four store families, explicit transaction rollback, mixed atomic batches, authoritative no-change/upsert/delete results, finite include-depth public projections and actual handlers. Extra capabilities are selected in a private regression host only; production admin routes retain the narrow selection. Generated code stays measured.

## Verification and evidence

All M5 commands use the sole owned target `/private/tmp/s8-admin/m5-admin-agent/target`, jobs 4, incremental off, dev debug off. `m5_verify.py` preserves exact argv/result JSON, bounds every child and kills/reaps its process group on expiry. Hang-class tests have independent parent watchdogs. Chromium uses ADMIN_PLAYWRIGHT only, .invalid interception, a 120-second parent/105-second inner watchdog and separately bounded native requests. No listener is used.

Evidence under `/private/tmp/s8-admin/m5-admin-agent/`:

| Gate | Evidence |
| --- | --- |
| Scoped native tests/doctests; four rounds | `round-1`, `round-2`, `round-3-verified`, `round-4`, `pages-native` logs/results |
| Real SQLite CRUD/allow/deny/validation/uncertainty/remove/restore/relations | `web/tests/crud.rs`, `native_layer.rs`, `first-real-flow` and final suite logs |
| Actual generation/check, whole-root byte stability, plugin replacement/removal, failed-generation preservation | Tool integration tests; `generation-check`; `m5-compile-proof.json` |
| Absent-root consumer, rename/removal, stale consumer exit 101, updated consumer compile | `m5-compile-*` logs/results, including omitted-role and fully removed selections |
| Quote/backslash/newline/closing-tag labels compile as inert Rust data; Idea positions/JSON structural diagnostics | `m5-compile-hostile-json-*`; framework-admin role/catalog tests |
| Eight Chromium cases: both mounts, hydrate/CSR, 1280 px and 375 px/200% | `browser-{root,admin}-{hydrate,csr}-{1280-1,375-2}.json`, 32 requests/case, 32 final screenshots |
| fmt; workspace Clippy; locked native tests/doctests | `fmt`, `workspace-clippy`, `workspace-suite`: 684 native passed, 64 explicitly ignored, 895 doctests passed |
| Explicit Rust 1.96 workspace check; warning-clean Rustdoc | `msrv`, `rustdoc` pass |
| Both WASM builds, Clippy and normal dependency graphs | `{hydrate,csr}-build`, `wasm-clippy-*`, `tree-*` pass; no server, SQL/SQLx/SQLite, Idea or Tokio browser edge |

The eight browser records pass with no page/hydration errors, unexpected requests or overflow. They prove parsed SSR/hydrated DOM parity/input identity, no duplicate initial fetch, create/detail/update/copy, validation and denied mutation drafts, policy-hidden links/denied deep links, explicit remove/restore, related create/locked FK, exact nullable/false/zero, filter/order/paging links, committed UnknownOutcome without Saved/retry, and disposal. Expected HTTP 400/403/404/500 resource diagnostics are recorded separately. Representative wide/narrow screenshots were inspected. No manual screen-reader or installed-target claim.

## Coverage

`m5_coverage.py` runs cargo-llvm-cov in native codegen + ssr scope. It removes the automatic examples exclusion, includes every compiled src file for each crate (including shipped generated contracts/UI and web's native generated stores/handlers), and excludes dependency/test/support/target files from implementation scope. To ensure inline tests cannot raise coverage, it subtracts every physical inline-test-module line from the covered numerator (even comments/braces), retaining the full executable denominator. These are conservative production lower bounds; raw counts and exact ranges are retained in `coverage-production.json`/`coverage-raw.json`.

| Crate | Conservative covered / full executable denominator | Production coverage at least |
| --- | --- | --- |
| framework-admin | 1494 / 1746 | 85.57% |
| generated-admin-tool | 198 / 217 | 91.24% |
| generated-admin-shared | 3423 / 4723 | 72.48% |
| generated-admin-web | 4958 / 5944 | 83.41% |

The client contains only WASM-gated startup/transport code, with no native executable implementation lines. Native coverage does not claim WASM callback coverage; browser cases exercise those paths.

## Reproduction and closeout

Run sequentially from the root: `python3 examples/generated-admin/m5_checks.py`, `python3 examples/generated-admin/m5_coverage.py`, `python3 examples/generated-admin/m5_build.py`, `ADMIN_PLAYWRIGHT=<installed module> python3 examples/generated-admin/m5_browser.py`, `python3 examples/generated-admin/m5_compile_proof.py`. Scripts share only the sole owned target/log root. Build normal artifacts before directly executing binaries after instrumentation.

- Final workspace gates pass without an unrelated load-sensitive failure or serial rerun. Native stable and named 1.96.0 are Rust 1.96.0; WASM uses the preinstalled stable target. No toolchain/target was installed.
- Final repeated eight-case browser matrix passes (256 intercepted HTTP requests, 32 final screenshots). The sole target was 4.96 GiB immediately before deletion and is deleted (`cleanup.json`). Only M5 Python caches and profiler strays created during this run were cleaned/archived; other proofs' output is preserved.
- KB validation retains the baseline of 146 inherited errors (absent upstream sibling sources) and 22 warnings; no new receipt finding. Validator exit 1 remains a baseline failure, not a green KB gate.
- Keep logs, copied finite SSR binary/assets and screenshots for review; preserve other proofs' output.
- Stop for coordinator review; do not start M6.
