# Task 00013 — Slice 8 M6 admin import/export receipt

Status: implemented; stop for coordinator review. No commit or push. The native-target half of M6 is outside this receipt.

## Contract and ownership

Started by merging `trial/3` into `trial/3-s8-admin` (fast-forward `8879afa..e1c37d9`). This work implements the admin half of the [M6 verification row](../slice-8-design-verification.md), the [framework inventory](../slice-8-design-inventory-framework.md) import/export and related rows, and the accepted architecture/decisions. G20/G29/G30 registration/composition and G34 revision 2 stay intact.

Only framework-admin and examples/generated-admin code/docs/proofs plus this pre-approved receipt changed. No framework-ui/view/schema/sql, SQL/server/Idea, generated-workspace, profile-ui, dependency manifest or lockfile changes. No installs, listeners, background jobs, server-side upload files or handwritten operation substitutes.

## Amended G36 for the coordinator ledger

The coordinator approved these product rules, including the later explicit decoding refinement:

1. Import is **create-only**. An existing identity, including an inactive/soft-removed identity, or a repeated identity in the file blocks the whole batch. Preview reports those rows as conflicts. No upsert/update-by-import.
2. Export includes **all authorized filtered matches in selected order**, ignoring pagination. It refuses above its row/byte ceiling and suggests narrowing the filter. It never silently truncates.
3. CSV headers are public field wire names; unknown, duplicate, sensitive and nonwritable import headers are refused. Scalar String/enum/date/time/datetime cells are canonical strings, other values JSON, `\N` null, and import-only `\O` omission/use-default. Required/no-default omission fails. Export never emits omission; absent projected members fail. A literal leading backslash is doubled (`\\N`, `\\O`, `\\path`).
4. Export guards **string** cells starting `=`, `+`, `-`, `@`, tab or carriage return with an apostrophe; original leading apostrophes are doubled. Numeric JSON remains numeric, including negative numbers.
5. **“Decode Stackpress export guards” is explicit and off by default.** Ordinary import preserves leading apostrophes literally. When selected, only documented guard/escape forms decode: doubled original apostrophe, or apostrophe before a formula-leading character. Leading-backslash/presence decoding is independent. Preview displays raw cells beside decoded values. Export → confirmed import into fresh SQLite is exact with decoding selected; without it, guard/escape apostrophes intentionally remain literal. This replaces any earlier implied automatic apostrophe stripping.
6. App-configurable ceilings may only be lowered: import 1 MiB/file, 100 data rows, 100 columns and 64 KiB/cell; export **100 rows and 1 MiB**, without importing the import-only column/cell caps. No server-side upload storage beyond a request, no background jobs.

The implementation additionally bounds preview serialization to 1 MiB, with an explicit smaller-file refusal instead of dropping any row. This response limit is documented separately from CSV file limits.

## Implementation

The [transfer API](../../../../crates/framework/framework-admin/docs/api/transfer.md) owns strict bounded UTF-8 CSV parsing/escaping, presence and explicit guard codecs, public field selection and acknowledgement states. CSV supports LF/CRLF, quoted embedded line breaks, deterministic fully quoted CRLF output and doubled quotes. File/row/cell/column admission happens before the next allocation/append. Invalid rows retain ordered raw/decoded values and safe errors.

The ordinary app-owned generator now emits typed `validate_import` adapters using actual `PublicCreate<Model>::validate_create`, closed native existence adapters using actual generated maintenance reads (inactive identities included), actual Batch registration, and compiled import/preview/confirm/export routes. Both UI-owned partitions regenerate through the same writer. No generic batch/upsert HTTP endpoint is exposed. Generated stores and schema contracts remain untouched.

The finite host checks policy before CSV parsing, then Origin/CSRF for imports. Import requires Batch/Create/Get grants. Every preview row passes actual generated creation validation, public error presentation, parent-scope enforcement and identity conflict checks. Confirmation repeats all checks, preserves input order and calls the actual generated create-only batch once. Generated whole-batch preflight/transaction ownership remains authoritative. A foreign-key failure after an earlier insert rolls the batch back. A trusted fault loses acknowledgement only after the real SQLite commit; public output reports UnknownOutcome, with no imported claim or retry.

Export reuses validated typed search state and generated Search, resets only pagination, checks total/complete rows, then projects/escapes public stored values. Row, CSV byte and generated aggregate-result limits refuse with narrowing guidance. Downloads have a compiled filename, CSV MIME, attachment and no-store headers. Related imports/exports authorize/load the parent, reimpose the FK and reject explicit mismatches.

The browser owns only its selected file draft. It checks File.size before arrayBuffer allocation and decodes UTF-8 strictly. A separate default-off checkbox invalidates prior preview/confirmation when changed. Confirmation requires a valid current preview and explicit checkbox. Errors preserve the selected file; Unknown locks this mounted form and never retries. Generations discard late file reads/replies; root disposal removes listeners/aborts fetch. File names are escaped display data, truncated by Unicode scalar and wrapped at narrow widths.

## Four test-verified rounds

| Round | Review and resulting changes | Evidence |
| --- | --- | --- |
| 1 — logic and branch risks | Corrected body-setting status reset so refused confirm is HTTP 400; distinguished structural preview failures from retained row errors; verified no writes, conflicts, rollback and durable-but-uncertain commit. | `round-1-final` native suite/doctests pass |
| 2 — responsibility/ownership | Extracted row contract/conflict validation from request orchestration; split preview rendering into a bounded AnyView helper, avoiding WASM type recursion overflow; added inert/escaped SSR import tests. | `round-2`, later complete native/format suites and both WASM builds pass |
| 3 — consistency/docs | Rustdoc/block explanations for budgets, state, transactions and codecs; documented wire rules and host obligations with the docing skill; module example compile-tested; corrected test Clippy finding. | Workspace/format Clippy `-D warnings`, warning-clean Rustdoc and doctests; API links/H1 checks |
| 4 — safety/soundness | Checked policy-before-parse, strict headers, output limits, public projection, scope, conflict checks despite rule errors, exact 100-row boundary, explicit guard round trips, disposal and unknown acknowledgement precedence. Added contradictory-status/impossible-count regressions, Unicode/long filename proofs, import-only versus export caps and large generated-result refusal/narrowing. | `round-4-final`, `round-4-export`, final default/format coverage suites and final 8-case Chromium matrix |

No unsafe code, new crates.io package, retry loop or shared-crate modification was introduced.

## Verification

All commands use CARGO_BUILD_JOBS=4, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0 and the sole owned target. Exact argv, deadlines, exit status and logs are under `/private/tmp/s8-admin/m6-admin-agent/`. Python parents kill/reap process groups; Chromium cases have a 120-second parent and native request children have bounded deadlines/output.

- fmt and diff whitespace checks pass; workspace and opt-in format Clippy `-D warnings` pass.
- Listener-free workspace suite: **706 passing test results and 930 doctests**, before the final additional large-export/import-cap regressions; all affected default/format tests rerun afterwards under coverage. Rustdoc uses `-D warnings`; `cargo +1.96.0 check` passes. Final scoped native Clippy/MSRV/Rustdoc and all four WASM Clippy variants pass again after the last changes (`final-*` logs).
- **TCP exception:** the first workspace run failed because the existing `server-http::tcp_watchdog` child tried to bind and the sandbox refused. The coordinator explicitly chose “Skip TCP test; record for coordinator.” The successful workspace command is `cargo test --workspace --locked -- --skip tcp_watchdog`. No listener was started; real admin HTTP is buffered/in-process. This receipt does not claim that TCP test passed.
- Generation checks, whole-root byte stability, model rename/removal, obsolete consumer failure (expected exit 101), updated/omitted/empty consumers, hostile JSON/format literal compilation and native generated registration compilation pass. Empty native/pure consumer is Clippy-clean. The proof freezes its format-enabled tool binary so workspace builds cannot change its feature selection mid-proof.
- All four normal-edge WASM graphs (hydrate/CSR, default/format) exclude server/DB/SQL/Tokio. Default graphs retain URL/IDNA ICU but no formatting ICU or phonenumber. No dependency manifest/lockfile changed.
- Final Chromium matrix: both `/` and `/admin/`, hydrate/CSR, 1280 px and 375 px/200%; **8 cases, 152 intercepted HTTP requests, 40 screenshots and 8 CSV downloads**. Assertions cover SSR node adoption/no mount fetch, preview without writes, per-row errors/draft retention, explicit guard modes/raw-vs-decoded effect, explicit atomic confirmation, denied actions/deep links, related scope/mismatch, strict UTF-8/oversize files, Unicode/long filename safety, intercepted export filtering/pagination and durable UnknownOutcome without automatic retry. No page/hydration errors, unexpected network or page overflow. Representative wide/narrow screenshots inspected. No manual screen-reader or installed-target claim.
- Native SQLite tests also cover soft-removed conflicts, duplicates plus other validation errors, malformed/unknown/duplicate headers, required omission, downward app limits, 101 rows, >1 MiB file, CSRF/missing confirmation, no generic batch endpoint, atomic FK rollback, formula guards for all six prefixes, literal backslashes/apostrophes, deterministic public export and confirmed round trips in both modes.

## Coverage

`m6_coverage.py` measures default and formatted native codegen/SSR variants with cargo-llvm-cov. It removes the automatic examples exclusion and includes both shipped generated roots and native generated stores/handlers. Dedicated `_tests.rs` is test code; every physical inline-test-module line is conservatively subtracted from the covered numerator while retaining the full executable denominator. Raw files/counts/ranges are retained in `coverage-raw.json` and `coverage-production.json`.

| Crate | Conservative covered / executable lines | Production coverage at least |
| --- | --- | --- |
| stackpress-framework-admin | 2388 / 2730 | 87.47% |
| generated-admin-tool | 219 / 238 | 92.02% |
| generated-admin-shared | 7140 / 10099 | 70.70% |
| generated-admin-web | 9895 / 11969 | 82.67% |

All exceed the 70% development floor. The changed client is WASM-gated, with no native executable implementation lines; native coverage makes no WASM callback line-coverage claim. The browser matrix exercises file admission, transport, mount/disposal and live flows. No generated production code is excluded to raise these results.

## Reproduction and cleanup

The [example README](../../../../examples/generated-admin/README.md#bounded-csv-importexport-m6-g36) documents M6 build/browser/compile/check/coverage commands. Use ADMIN_PLAYWRIGHT for the installed module; do not install or hard-code it. Reuse/clean the sole target before instrumentation to stay under 8 GiB, preserving copied non-instrumented artifacts outside it. The only skipped workspace test is the coordinator-approved TCP proof above.

- The sole target remained below 8 GiB: 6.91 GiB before the instrumentation reset, 3.72 GiB before final deletion. It is deleted. `coverage-target-reset.json` and `cleanup.json` retain exact sizes. Copied finite binaries, assets, downloads, screenshots and logs remain outside it.
- All 264 profiler files present before instrumentation were preserved size/mtime-identically. New owned profiler strays were archived under the evidence directory, and only M6 Python caches were removed. No new profiler strays remain in the checkout. Other proof directories were untouched.
- Final scope audit is confined to authorized paths with an empty staged set. No commit/push was made.
- KB validation baseline is 146 inherited missing-upstream errors and 24 warnings; final comparison is recorded after receipt creation. This is a baseline failure, not a green KB claim.


Stop here for coordinator review. Do not start another milestone or publish/commit/push.
