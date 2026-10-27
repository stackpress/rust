# Task 00013 — admin catalog part 2 and visual polish

Implementation receipt, 2026-10-27. Coordinator review pending. No commit, push, acceptance, publication or M6 claim.

## Scope and ownership

- The requested `git merge trial/3` fast-forwarded `e29500f..ff436c5`. The sandbox initially refused external Git ORIG_HEAD; the authorized merge succeeded with escalation.
- Changes stay within framework-admin, examples/generated-admin, the root facade feature and this receipt. No framework-ui/view/schema/sql, server, SQL, Idea, generated-workspace or profile-ui source changes. Frozen schema/UI revision 2 stays unchanged. Cargo.lock is unchanged; no new external dependency or version was introduced.
- The admin `format` feature forwards to framework-ui `format`. The root facade exposes `framework-admin-format`; each example crate forwards its own opt-in `format`. Default features stay empty.
- The same app-owned generator registry and single writer produce the alternative marked root `shared/src/generated-format`. `schema-format.idea` imports the ordinary fixture and extends Task with six nullable formatted fields. Host and browser select matching generated contracts/UI/stores through feature-gated includes. Default builds retain the original generated root. The formatted output is 983,705 bytes, below the unchanged 1 MiB writer ceiling.
- No listeners, devmetrics, installs, commits or pushes. Imports/exports and installed targets remain M6. Rich text, file/media and service-dependent relation adapters remain unavailable.

## Formatted catalog

- Registered `field.number`, `field.price`, `field.integer`, `field.date`, `field.time`, `field.datetime`, `field.phone`, and corresponding numeric/date/phone view definitions. The frozen registry supplies aliases and derived filter/span/list placements. Feature-off selections stay diagnosed as unavailable.
- Definitions contain typed props, trusted imports and availability; no prop supplies executable Rust/import/HTML identities. Locale is explicitly `en` or `fr`; phone input requires a declared region. Invalid policy parameters and incompatible scalar families are diagnosed at Idea attribute byte positions, with structural paths for normalized JSON.
- Wrappers delegate editing, admission, parsing, IME/native adoption, selection and canonical commit behavior to framework-ui policies. Partial or invalid input retains the accepted Draft and displays an error. The root `PendingEdits` context prevents writes while an intermediate edit remains unresolved; no automatic write/retry occurs.
- Number retains the frozen finite floating-point wire family, Integer retains its checked integer family, and exact decimal values use String because revision 2 has no Decimal wire type. Decimal scale/digit limits, explicit temporal offset admission and explicit phone region are checked before use. Successful dates/times/instants normalize through the actual schema constructors; phone commits valid E.164.
- Temporal presentation delegates the existing policy and changes only display: declared precision is shown; absent precision trims trailing fractional zeros while preserving nonzero digits and offset. Committed values keep their full admitted precision. Tests cover fractional padding/trimming, negative offsets, localized calendar punctuation and semantic wire preservation. The browser commits a nine-digit fractional instant through real SQLite and reads it back unchanged.
- Native and browser wrappers use only public descriptors. The existing public-projection tests remain active; private/sensitive fields, host providers, SQL and native diagnostic causes do not enter wrappers or browser graphs.

## Visual and form behavior

- Pagination/deep-link actions share a wrapping flex group with a consistent gap. Result links use an approved public name/title/label or readable identity values prefixed with View; they do not serialize key JSON as link text.
- Public rule messages take precedence. Generic issues receive specific descriptor-based wording. When the operation collapses a rule failure to `invalid`, the submitted public snapshot is evaluated only against client-checkable public rules to recover the matching message. Unavailable host rules retain a safe fallback. No rejected input or private cause is echoed.
- Required non-null create/copy fields hide presence selectors. Nullable fields retain clear/set choices; filters and patches retain omission. Labels use plain wording, including Leave unchanged, Clear (empty) and Set value. Patch controls do not use create-required validation attributes for ordinary inputs.
- A previous Saved acknowledgement is cleared after a subsequent edit. UnknownOutcome still preserves the draft, disables resubmission, makes no Saved claim and never retries.
- Example-owned styles size formatted controls to fit full values at 1280 px while retaining narrow max-width constraints. The help text is visible only in the focused formatted field. Every control retains its existing help/error `aria-describedby` association; no framework-ui change or manual DOM patch was needed.

## Four test-verified review rounds

1. Complexity/state: separated committed values from intermediate presentation, checked canonical conversion and presence resets, and compared canonical acknowledgements rather than dirty presentation text after blur. `round-1-verified` passes.
2. Responsibility: shared public wording and readable-link helpers, root-owned editor/pending contexts and one shared field-error path. Cleared stale Saved claims after subsequent edits. `round-2` passes.
3. Consistency/Rustdoc/comments: documented feature/wire boundaries, strict configuration checks for direct construction and codegen, immutable trusted imports, finite reproduction and separate writer roots. Added type/policy/source-position fixtures. `round-3-verified` passes.
4. Safety: checked invalid locale/scale/digit/precision/region/type configurations, fixed generic-invalid error recovery against submitted public data, exercised hostile labels through the formatted import, stale/removal consumers and plugin replacement/removal. `round-4` and final suites pass.

Coordinator screenshot follow-up added compact time/instant presentation, wide control sizing and focused-only accessible hints. The final format suite, both WASM builds/Clippy graphs and entire repeated browser matrix pass after these changes. No ceremonial refactoring or shared-crate edit was used.

Early failures are retained: feature-on availability expectations were corrected; an Idea quote fixture was replaced with valid Idea plus hostile normalized-JSON compilation; the first browser exposed generic `invalid` wording and led to submitted-rule recovery. A browser driver stopped after a passing case because `du` raced a removed rustc temporary file; target measurement now retries up to three times and still fails closed if measurement is unavailable. The repeated final matrix passes.

## Verification and evidence

Evidence is exclusively under `/private/tmp/s8-admin/catalog-2-admin-agent/`. Drivers use one target, jobs 4, incremental off, dev debug off, bounded child process groups, and kill/reap on expiry. Each browser case has a 120-second parent/105-second inner watchdog; finite native requests have independent 10-second deadlines and 2 MiB output limits. Playwright comes only from ADMIN_PLAYWRIGHT. Every browser request is intercepted at `https://admin.invalid`; no listener or write simulator is used.

| Gate | Result / evidence |
| --- | --- |
| Four rounds, actual SQLite, public errors and state | `round-1-verified`, `round-2`, `round-3-verified`, `round-4`, `format-suite`, web/native-layer tests |
| Workspace fmt/Clippy, locked tests/doctests | `fmt-final`, `workspace-clippy`, `workspace-suite`: 693 native passed, 80 explicitly ignored, 918 doctests passed |
| Format-native lint/tests and warning-clean Rustdoc | `format-clippy`, `format-suite`, `rustdoc` pass |
| Explicit Rust 1.96.0 workspace check | `msrv` passes with facade/web/tool formatting enabled |
| SSR, hydrate and CSR artifacts | `plugin-format-build-*`, `plugin-disabled-build-*`; copies and hashes retained in `artifact-hashes.json` |
| WASM Clippy and normal dependency graphs, both feature states | `wasm-clippy-{hydrate,csr}-{default,format}`, `tree-*` pass; no server, SQL/SQLx, Idea or Tokio browser edge |
| Default and formatted generation checks | `generation-final-default`, `generation-final-format` pass |
| Full-root byte stability; rename/removal and obsolete consumer rejection | `c2-compile-proof.json`, `c2-compile-*`: obsolete consumer exits 101, renamed/omitted/empty consumers compile, empty consumer is Clippy-clean |
| Hostile labels/quotes/backslashes/newlines/closing tags and import-like text | `c2-compile-hostile-json-*`, `c2-compile-hostile-format-*`; native escaped formatted SSR and source-diagnostic fixtures |
| Compiled formatted plugin replacement and removal | `plugin-format-generate/check/disable`, byte-identical whole-root restoration, both-mode/mount/size Chromium plugin records |
| Final Chromium matrix | `browser-matrix.json`: 8 CRUD + 8 plugin cases, 272 intercepted HTTP calls, 48 final screenshots |

All sixteen final browser records pass without page/hydration errors, unexpected requests or page overflow. Both `/` and `/admin/` run hydrate/CSR at 1280 px and 375 px/200%. Assertions cover SSR/hydrated DOM parity, no duplicate initial fetch, actual CRUD/relations/remove/restore/denials/UnknownOutcome, draft retention, specific errors, no raw JSON link labels, pagination gap, presence visibility/plain wording, formatted zero/null/omission, rounding/localized decimal, E.164, missing-offset/partial rejection, exact fractional SQLite commits, compact time display, full wide input text and focused-only nonduplicated hints with description links. Representative wide/narrow related-create, validation and search screenshots were inspected. No manual screen-reader or installed-target claim.

## Default graph clarification

The owner/coordinator explicitly clarified that existing URL/IDNA ICU is preserved. The gate excludes added formatting ICU and phonenumber, rather than requiring zero ICU.

`c2_graph.py` extracts a temporary read-only `git archive` snapshot of trial/3, uses the sole target and offline locked normal-edge `cargo tree`, and compares exact package/version sets for hydrate and CSR. The recorded baseline commit is `72176840fb23fb9fcd88d41abcbebf4b93c3be89`. `default-graph-comparison.json` reports 183 packages in each default graph, with zero added and zero removed. Reverse evidence identifies the existing ICU normalizer path through url/idna. No formatting ICU decimal/datetime/locale-data package or phonenumber enters the default graph.

## Coverage

`c2_coverage.py` collects default and formatted native feature variants with cargo-llvm-cov. It removes the automatic examples exclusion and measures all compiled implementation sources in each declared crate, including both generated roots and native generated stores/handlers. Dedicated `_tests.rs` source is excluded as test code. Every physical inline-test-module line is conservatively subtracted from the covered numerator while retaining the full executable denominator. Exact ranges, files and raw counts are retained in `coverage-production.json` and `coverage-raw.json`.

| Crate | Conservative covered / full executable denominator | Production coverage at least |
| --- | --- | --- |
| framework-admin | 2034 / 2370 | 85.82% |
| generated-admin-tool | 219 / 238 | 92.02% |
| generated-admin-shared | 7019 / 9908 | 70.84% |
| generated-admin-web | 9465 / 11482 | 82.43% |

All exceed the 70% development floor. The client source is unchanged and WASM-gated; native coverage makes no WASM callback line-coverage claim. The browser matrix exercises those paths.

## Reproduction and closeout

From the root, run sequentially: `c2_checks.py`, `c2_graph.py`, `c2_build.py`, `c2_plugin_build.py`, `ADMIN_PLAYWRIGHT=<installed module> c2_browser.py`, `c2_compile_proof.py`, and `c2_coverage.py`, all under examples/generated-admin. The README includes full commands. Clean/reuse the sole owned target before instrumentation if needed to remain under 8 GiB; keep copied non-instrumented browser artifacts. Delete the sole target after evidence collection.

- The sole target stayed below 8 GiB. It was 6.93 GiB before reuse for instrumentation and 2.29 GiB before final deletion; it is deleted. `coverage-target-reset.json` and `cleanup.json` retain exact sizes.
- Only catalog-2 Python caches were removed. All 252 pre-existing profiler files (older than this task) were preserved byte-size/mtime-identically. No new profiler strays were left in the checkout. Other proof directories remain untouched.
- Final scope audit contains only authorized paths and an empty staged set. `scope-audit.json` retains the file list. No commit or push was made.
- KB validation retains the inherited baseline of 146 errors (missing upstream sibling sources) and 24 warnings. Exit 1 remains a baseline failure, not a green KB claim; see `kb-final.log` and its result record.
- Stop for coordinator review; do not start M6.
