# Slice 3 lane B: typed-reference integration and review rounds

Implementation evidence for [task 00006](tasks/00006-slice-3-idea.md). Load for real parser
reference integration, merge/finalization regressions, rounds 1–4 and final verification.

Date: 2026-10-25. Branch: `trial/3-s3-lane-b-compose`.
Starting and unchanged HEAD: `8bdca2c`. **Implemented, verified locally, and stopped for
coordinator review.** No commit, push, publication, listener or dependency changes.

## Scope and resulting contract

Changes are confined to transformer `src/compose/**`, its tests, minimum `src/lib.rs`
re-exports, and this requested receipt. `src/source/**` was reviewed and exercised without
requiring edits. Parser, registry, plan, output, wrapper, example and shared manifests are
untouched. No other trial branch or lane worktree was read.

- Removed `ReferenceSite`, `ReferenceKind`, `ReferencePath` and `ReferenceSpan` placeholders.
  `stackpress_idea_transformer::Reference` directly re-exports the actual parser type;
  `ComposedSchema.references` is `Vec<stackpress_idea_parser::Reference>`. A wrapper adds no
  value: parser sites already own kind/declaration/column/attribute/path, target, source span
  and forward flag; existing composition provenance and import edges identify acquisition.
- `compile_source` now uses `SourceFile`, `parse_ast` and `compile_ast_with_references`.
  Sites move with the exact declaration, missing attribute or whole column that survives
  each merge. G19 replacement discards every site belonging to the replaced declaration.
  Column names remain stable when child-only columns prepend; numeric positions are not used.
- Finalization uses the parser's `finalize_with_references(&config, &references, &inputs)`.
  `Composer::compose` retains its frozen non-final result contract; no additional public
  method or result field was introduced. A compiling Rustdoc example demonstrates the full
  imported-reference/literal/finalization sequence.
- Plain normalized JSON uses `SchemaConfig::from_json_named` and never discovers references
  from strings. Explicit typed envelopes use `CompiledSchema::from_json`. The existing
  composition node/depth/byte gate remains in front of decoding. Duplicate semantic shape
  validation was removed in favor of the parser owner during the integration implementation.
- A typed JSON envelope can retain an original Idea filename in its reference spans.
  Composition provenance/import edges separately identify the JSON artifact. No referenced
  span filename causes implicit filesystem access.
- Real reference payloads spend expansion allowance before retaining/copying snapshots,
  alongside config bytes. Empty reference vectors preserve the previous config-only charge.
  This closes the integration's metadata-copy amplification gap without adding a public knob.

## End-to-end regression evidence

All tests below use real parser references and the real checked finalizer, not synthetic sites.
The old merge unit tests also now obtain their sites from actual source compilation.

| Test in `tests/compose_references.rs` | Observable contract |
| --- | --- |
| `imported_literal_lookalikes_remain_literal` | Imported `${Name}` literals remain verbatim next to real references, including nested array/object paths; real sites retain the importing child's coordinates. |
| `reference_target_uses_later_g19_replacement` | A reference from the first import resolves to the declaration that a later import installs through G19, not the replaced immutable target. |
| `unresolved_reference_reports_referencing_coordinates` | Missing imported target reports `nested/child.idea:3:18`, with the referencing byte offset and `UnknownReference`. |
| `only_winning_attributes_and_columns_are_finalized` | Local colliding attributes/whole columns suppress losing child sites; prepended columns resolve by name. |
| `g19_replacement_discards_losing_reference_sites` | Replacing a declaration discards its unresolved local sites; only the child's real site remains. |
| `imported_forward_reference_keeps_its_restriction` | Importing a declaration does not erase its source-local forward-reference restriction. |
| `typed_json_imports_preserve_actual_reference_metadata` | Typed JSON preserves explicit sites and original spans; plain normalized JSON preserves every placeholder-looking string as data. |
| `reference_metadata_is_charged_before_snapshot_copy` | Config-only allowance rejects a real reference payload; the exact combined allowance succeeds; cloned site strings/spans remain independent. |
| `cross_source_reference_cycles_are_not_import_cycles` | An acyclic file graph can compose successfully while the checked finalizer rejects a reference dependency cycle at its original source. |
| `finalization_expansion_limit_retains_imported_site` | Expansion can fail after a bounded config passes validation; the imported site owns the limit diagnostic and the composed input stays unchanged. |
| `invalid_typed_json_sites_fail_before_composition` | Missing column, empty path and mismatched target metadata are rejected through the import error chain. |

Existing merge-table, source adapter, exact-order, JSON equivalence, cycle/diamond, cloning,
concurrent-input, limits and deep-input watchdog tests remain green. G19 is still the deliberate
owner-approved exception to reference 00022: an imported child replaces an immutable local.

## Test-verified rounds

Integration baseline: `cargo test -p stackpress-idea-transformer --all-targets --locked`
passed **72 tests**, with 3 pre-existing ignored subprocess helpers.

1. **Complexity:** separated bounded edge acquisition/cycle checks from DFS composition;
   extracted absent-member selection from metadata mutation. Guard order, sequential merging
   and failure precedence were preserved. Added the exact reference-copy budget/clone test.
   A transient E0507 compile error during extraction identified an attempted move of borrowed
   `column.name`; restoring its owned clone fixed it. Final round result: **73 passed**.
2. **Responsibility and abstraction:** moved Idea/JSON source compilation into
   `compose/compile.rs` and reference-subtree membership into the merge-policy module.
   Reviewed the native/memory loader trait as a genuine adapter boundary; no new trait,
   wrapper or public API was needed. Result: **73 passed**.
3. **Consistency, Rustdoc and comments:** documented explicit finalization, original spans
   in typed JSON, partial-run discard and reference/config charging. Added local explanations
   for attribute versus whole-column selection and unchanged nested paths. Checked every
   owned function's Rustdoc and source line widths. Results: **73 passed**, **12 transformer
   doctests passed**, transformer Clippy warning-free.
4. **Targeted safety/branch follow-up:** found integration test gaps for cross-source reference
   cycles, expansion exhaustion after composition and malformed typed JSON destinations.
   Added the three regressions listed above, with **no production behavior edits**.
   Re-reviewed bounded recursion/copying, owned clones, UTF-8 coordinate retention, cycle versus
   completed-cache handling, source acquisition and IO causes. Result: **76 passed**.
   No further material in-scope finding requires another round.

Each round reran `cargo test -p stackpress-idea-transformer --all-targets --locked`.
Logs are `round-1-tests.log` through `round-4-tests.log` under the evidence directory below;
the corrected transient build diagnostic is retained as `round-1-build-error.log`.

## Final verification

Native `aarch64-apple-darwin`; rustc 1.96.0 (`ac68faa20`), cargo 1.96.0 (`30a34c682`),
LLVM 22.1.2, cargo-llvm-cov 0.8.6. Default valid workspace feature selection; no new target
installation or all-platform claim. `cargo clean` and `cargo llvm-cov clean --workspace`
ran before the first build; coverage was cleaned again immediately before measurement.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo test -p stackpress-idea-transformer --all-targets --locked` | PASS: 76 passed, 0 failed, 3 pre-existing ignored helpers |
| `cargo test -p stackpress-idea-transformer --doc --locked` | PASS: 12 doctests, 0 failed/ignored |
| `cargo test --workspace --all-targets --locked` | PASS: 247 passed, 0 failed, 21 pre-existing ignored helpers |
| `cargo test --workspace --doc --locked` | PASS: 424 doctests, 0 failed/ignored |
| `cargo clippy -p stackpress-idea-transformer --all-targets --locked -- -D warnings` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | PASS, including transformer docs |
| `git diff --check` | PASS |
| `python3 .agents/scripts/validate-agent-workspace.py` | Exit 1: 146 existing sibling-source link errors and 19 existing warnings; no diagnostic names this new receipt. |

The KB validator's broken links point into absent sibling checkout paths under
`/Users/cblanquera/.paseo/worktrees/342ftjlj/`. Every error-owning file is unchanged from
HEAD. The new receipt has one H1, a valid task link and fewer than 200 lines. Existing KB
link repair is outside the authorized lane scope and was not attempted.

Coverage commands:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea-transformer --all-targets --locked \
  --exclude-from-report stackpress-idea-parser \
  --ignore-filename-regex '/tests/|/tests\.rs$' --fail-under-lines 70 \
  --json --output-path /private/tmp/stackpress-lane-b-integration.iNzC9V/transformer-coverage.json
```

**PASS: transformer production coverage is 1,663 / 1,729 lines = 96.18%, above the 70%
development floor.** Only test files and the separate unchanged parser crate are excluded;
all transformer production modules, including the integrated writer/plan/registry, remain in
the denominator. The owned composition/source area is 688 / 716 lines = 96.09%.

| Composition/source module | Covered / executable lines |
| --- | ---: |
| `compose/compile.rs` | 22 / 22 |
| `compose/json.rs` | 133 / 137 |
| `compose/merge.rs` | 157 / 158 |
| `compose/mod.rs` | 164 / 177 |
| `source/memory.rs` | 54 / 54 |
| `source/mod.rs` | 36 / 36 |
| `source/native.rs` | 79 / 88 |
| `source/paths.rs` | 43 / 44 |

Disposable command logs, JSON coverage and missing-line summary are in
`/private/tmp/stackpress-lane-b-integration.iNzC9V/`. The summary uses the established
`cargo llvm-cov report` command with parser/test filename exclusions, without a second test run.

## Branch risks and handoff boundaries

Line coverage is not branch coverage. Semantic tests separately cover the full existing
merge table plus real site selection, G19 target replacement, unknown/forward/cyclic references,
plain/typed JSON, copy and finalization budgets, original coordinates and clone independence.
Remaining unforced native paths include canonicalization/read/metadata failures during a race
and size/mtime drift during one read; existing missing/permission/UTF-8 and between-edge drift
proofs still run. Native metadata checks are cooperative drift detection, not an atomic snapshot
against arbitrary writers. Internal missing-cache/unsupported-prefix and no-op-flush paths are
private invariants; no artificial public hooks were added just to cover them.

No unsafe code, process environment/cwd mutation, new locks or detached work was introduced.
Linux, Windows, WASM and network filesystems remain unverified. Wrapper/example orchestration
was not changed. The existing root crate introduction still describes M0-only progress;
updating that shared prose belongs to its integration owner because this lane's `lib.rs`
permission is limited to re-exports. The owned module docs describe the implemented contract.

The routine Paseo coordinator status-message call was rejected with
`user rejected MCP tool call`; no reason was supplied and it was not retried through another
channel. This receipt and the final response provide the handoff. No code/API blocker remains
within the assigned scope. Coordinator acceptance and merge remain pending.
