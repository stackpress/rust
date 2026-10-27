# Task 00013 — admin catalog breadth, part 1

Implementation receipt, 2026-10-27. Coordinator review pending; no commit, push, acceptance or M5 claim.

## Scope and ownership

- Started with the requested `git merge trial/3`; fast-forwarded this worktree to `fdcd2f9`. The sandbox initially refused the worktree's external `ORIG_HEAD` metadata write; the authorized merge succeeded with escalation.
- Changed only framework-admin, examples/generated-admin, the lockfile's internal web-to-framework-ui edge, and this receipt. No framework-schema, framework-sql, generated-workspace, framework-ui, framework-view, SQL, server, profile-ui or Idea source changes.
- The frozen schema/UI contract remains revision 2. The existing app-owned schema/SQL/UI registry and single writer still own their separate generated partitions. No new generator, dependency download/install, service or listener.

## Implemented catalog

- `field.stringlist`, `field.textlist` and their frozen aliases use UI TextList; `field.tags`/taglist use UI TagList. Exact String arrays retain empty/zero strings, duplicates and order; 100 rows / 4096 bytes per value. Fixed tag mode overrides a supplied false prop.
- `field.metadata` uses UI MetadataInput for string-valued JSON Objects: 100 entries, 1024-byte keys, 4096-byte values. Duplicate editable keys produce a visible error and preserve the last accepted object; no overwrite or JSON/string coercion.
- `field.slider` uses UI Slider for a scalar Integer or Number. Numeric `min/max/step` props default to 0/100/1. Shared generation/runtime validation rejects nonfinite, reversed, fractional Integer, browser-inexact or incomplete/excessive grids. Initial values must already be on the grid; no silent clamping.
- `field.rating` uses UI RatingInput for Integer, including zero. Numeric max is integral 1..=100, default 5. Clear selects omission; explicit nullable Null remains selectable.
- `field.slug` uses UI SlugInput. Typing stays verbatim; Apply slug explicitly selects the library's ASCII/lowercase/dash transformation and byte budget.
- `field.select({ searchable true })` uses UI Combobox over the finite string enum with no acquisition callback. Search edits invalidate the selected machine value; only declared choices publish values. Native Select remains the default. Searchable-plus-placeholder is diagnosed because the current Combobox API has no placeholder; no prop is silently ignored.
- Field/filter/span aliases retain schema precedence and public capability checks. Formatted number/date/phone edits, rich text, file/media, `field.suggest`, and relation/service controls remain unavailable and diagnosed. No UI API change was needed.
- Explicit list selections additionally generate `table(&[(String, Public<Model>)], instance) -> SchemaResult<AnyView>`, using native UI table slots and the selected display adapters as cells. It checks at most 100 unique nonempty caller keys of at most 128 bytes before rendering; injective hex key encoding separates control identities. Captions, headers, row attributes and cells are escaped. Public DTOs remain the only row input; no data fetching, sorting, paging or writes.

## State and example

- A shared controller bridge distinguishes programmatic presence resets from user edits. Missing/Null can clear presentation without becoming an empty value. Repeated native edits at the same displayed value still publish intent, including omitted slider zero and Clear on an already visually unrated Null rating.
- Collection keys belong to the UI controller and never enter domain JSON. Ordinary editing/reordering preserves native node identity. Explicit draft replacement rebuilds the collection; metadata objects have no domain ordering.
- Disabled presence selectors now guard admission too. Composite controls retain authoritative required/domain validation in generated public input contracts; editing/snapshotting is not validation or submission.
- The actual generated-workspace models remain the example's foundation. Project exposes tag and searchable-enum fields; RoleProof exposes text rows, metadata, slider, rating and slug; Task renders a generated list-role table. Existing list/view/form placements remain.
- The app plugin now replaces both the original input adapter and the new tag adapter, fixing the latter label to `Plugin tags label`. The taglist span alias resolves to the replacement too. Disabling/regenerating/rebuilding removes both replacements.
- Relative asset URLs support both `/` and `/admin/`. Native component styles and bounded overflow keep the 375 px / 200% page within the viewport.

## Baseline and four verified review rounds

The baseline native suite passed after extending the expected public RoleProof JSON with its new nullable fields. New fixtures use the real Idea composer, revision-2 projection, app access policies and ordinary transform registry.

1. **Logic/complexity:** consolidated the slug state path, prevented disabled presence edits, and initialized numeric Value presence from its configured lower bound. Native gate `round-1` passed; Chromium later proved Null resets, exact zero and stable row mutations.
2. **Responsibility/abstraction:** extracted one numeric scale/rating policy shared by generation and runtime, retaining separate collection, scalar, synchronization and table-emission modules. Added pure admission boundaries for fractional/off-grid/oversized/unknown values. `round-2` passed.
3. **Consistency/Rustdoc/comments:** documented every new adapter, typed prop, bound, error, state transition, public table signature and target limitation. Updated API navigation, concepts, plugin guide and example instructions. Added synchronization ownership comments and kept build-only policy re-exports feature-gated. `round-3` passed.
4. **Safety:** replaced temporary reactive allocations during slider validation with a pure grid check. Distinguished same-value native edits from programmatic echo; added the zero/Null-Clear browser regressions. Extended plugin replacement/removal to tags, checked every model's real public table DTO, and diagnosed the unsupported searchable-placeholder combination. `round-4` and `round-4-repeat` passed; final coverage/scoped/workspace tests exercise the follow-up boundaries.

## Verification

All commands ran from this worktree with `CARGO_BUILD_JOBS=4`, `CARGO_INCREMENTAL=0`, and `CARGO_PROFILE_DEV_DEBUG=0`. `verify.py` gives each child an independent deadline, retains the exact command/result, and kills/reaps its process group on expiry. Generator, executable and existing hang-class fixtures retain their parent watchdogs.

| Gate | Evidence under `/private/tmp/s8-admin/` |
| --- | --- |
| fmt; scoped tests/doctest; Clippy `-D warnings`; Rustdoc `-D warnings`; facade check | `fmt-final`, `scoped-final`, `clippy-final`, `rustdoc`, `facade` logs/results |
| Explicit `cargo +1.96.0 check` | `msrv.log` / result |
| Hydrate and CSR WASM build/check, dependency trees | `base-*-build`, `wasm-*-check`, `tree-*` logs/results; no server, SQL/SQLx, Idea, Tokio or native SQLite edge in browser graphs |
| Plugin and disabled SSR / hydrate / CSR rebuilds | `plugin-*`, `disabled-*`, `restore-default-generation` logs/results |
| Absent-root generation, full-byte stability, model removal/rename and preserved output on errors | Tool integration tests and `compile-proof.json` |
| Actual stale consumer refuses after rename; updated consumer compiles | `compile-stale-consumer.log` (expected 101), `compile-updated-consumer.log` |
| Quotes, backslashes, newlines, closing tags and hostile labels remain data in generated Rust | `compile-hostile-json-*` logs; native catalog and table tests; Chromium row edits |
| Unknown/unavailable props/components, invalid numeric configuration and incompatible value types retain Idea positions | `catalog.rs` / `roles.rs` integration tests; normalized JSON retains structural paths |
| Chromium interception proof | `browser.json`, `browser-result.json`, 16 screenshots in `screenshots/` |
| Full workspace locked tests/doctests and Clippy | `workspace`, `workspace-doc`, `workspace-clippy` logs/results |

The Chromium run used only `ADMIN_PLAYWRIGHT`, no module fallback/install, `.invalid` interception and a 120-second parent watchdog. Its 16 cases cover both mounts, hydrate/CSR, base/plugin/disabled, 1280 px and 375 px at 200%. It passed in about 30 seconds: parsed SSR/hydrated DOM parity, preserved input node, 136 unique IDs, exact collection mutations, duplicate metadata recovery, enum keyboard selection, slider bounds/zero, rating Null/Clear/zero, explicit slug action, plugin removal, no page overflow, no console/page errors or unexpected requests, and root disposal. Screenshots were inspected. This is not a manual screen-reader or installed-target claim.

Final workspace evidence: **663 native tests passed, 78 explicitly ignored; 892 doctests passed**. Scoped evidence: **21 native tests and one doctest passed**. No unrelated load-sensitive test failure required a rerun.

## Coverage

`cargo llvm-cov` measured the `codegen + ssr` native matrix. The raw export removes the tool's default example-path filter and includes every implementation source in the four crates, including compiled generated contracts/wrappers. No source exclusions were added to reach the floor. Native-only line counters do not claim WASM callback coverage; the browser matrix exercises those interactions. The client crate is a pure WASM-gated startup adapter with no native executable implementation lines.

| Crate | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-admin | 1018 / 1180 | 86.27% |
| generated-admin-tool | 179 / 197 | 90.86% |
| generated-admin-shared | 2453 / 3335 | 73.55% |
| generated-admin-web | 11 / 11 | 100.00% |

Reproduction: `checks.py`; `build.py`; `ADMIN_PLAYWRIGHT=<installed module> browser.py`; `compile_proof.py` in `examples/generated-admin/`. Coverage uses `CARGO_LLVM_COV_TARGET_DIR=/private/tmp/s8-admin/target/llvm-cov-target`, `verify.py coverage-closeout cargo llvm-cov` selecting the four packages with `stackpress-framework-admin/codegen,generated-admin-shared/ssr`, then `verify.py coverage-command cargo llvm-cov report` for the same packages with `--json --output-path /private/tmp/s8-admin/coverage.json -vv`, followed by `coverage_report.py`. Exact invocations and raw JSON are retained beside the logs.

## Closeout and limits

- Default generation is restored and checked; generated files contain no app replacement imports. No sensitive/host-only field/default reaches wrapper metadata or rendered output.
- Native named toolchain 1.96.0 and installed stable both report Rust 1.96.0. WASM uses the already installed stable target; no target/toolchain was installed.
- The KB validator has the unchanged baseline of 146 errors (absent upstream sibling source paths) and 22 warnings. This receipt introduces no additional validator finding.
- Owned build roots and transient Python caches were removed after verification. Finite logs, assets, SSR binaries, screenshots and workspace fixture artifacts remain under `/private/tmp/s8-admin/` for review.
- No listeners, installs, new crates.io dependencies, commits, pushes, CRUD/auth/service work, schema revision changes or edits outside the assigned ownership set. Stop here for coordinator review.
