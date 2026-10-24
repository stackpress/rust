# Slice 3 lane B: M2 composition implementation gate

Spec file of [00001 Stackpress Rust foundation](index.md), recording Slice 3 lane B (composition, M2) of [task 00006](tasks/00006-slice-3-idea.md). Load for source loaders, `use` composition, the merge-table matrix including G19, reference-site movement, and the pending typed-reference switch. Moved losslessly from `crates/idea/idea-transformer/LANE-B-RECEIPT.md` (merged in `17dbcac`); relative links rebased.

Date: 2026-10-25. Branch: `trial/3-s3-lane-b-compose`.
Base and unchanged HEAD: `54b188e3a5811f8d05ec6017fcb7382466660135`.

**Stopped before refactoring rounds, as instructed.** Native/in-memory source acquisition,
owned non-final composition, merge policy, provenance and the reference-carrying seam are
implemented and verified. Actual typed-reference compilation/finalization awaits the
coordinator-authorized lane A integration described below. This is not whole-Slice-3 acceptance.

## Ownership and implementation

Only the assigned `src/compose/**`, `src/source/**`, new `tests/compose_*.rs` and
`tests/source_*.rs`, this receipt, and additive `mod`/`pub use` lines in `src/lib.rs` changed.
No dependency, manifest, lockfile, parser, writer, registry, wrapper, example or KB edits.
No commits, pushes, listeners, upstream modifications or upstream builds.

Source behavior was checked against the read-only upstream
`packages/idea-transformer/src/Transformer.ts` and `tests/Transformer.test.ts` in the sibling
Idea checkout. The upstream test suite was not run; Rust tests prove actual composed results,
including the JSON equivalence that the upstream uncalled-method comparison does not prove.

Public additions:

- `SourceLoader::load(importer, specifier, max_bytes) -> Result<LoadedSource, LoadError>`.
  The additional byte-budget argument deliberately strengthens the design sketch: remaining
  acquisition allowance reaches the adapter before native reads or memory clones.
- `SourceId`, `SourceKind`, `LoadedSource`, `LoadError`, `MemoryLoader`, `ProjectLoader`.
- `Composer::new(loader, limits).with_inputs(inputs).compose(entry)`.
- `CompositionLimits`, `ComposedSchema`, `ImportEdge`, and the temporary typed-reference
  seam types `ReferenceSite`, `ReferenceKind`, `ReferencePath`, `ReferenceSpan`.

`ProjectLoader` requires an explicit absolute root and canonicalizes it once. The root supplies
entry and `@/` resolution; it is not a filesystem sandbox. Absolute and parent-relative imports
are supported. Ordinary imports resolve against the importing canonical file's parent.
Candidate order is exact file, `.idea`, `.json`, `index.idea`, `index.json`; extension candidates
are omitted when the specifier already has an extension. Extensionless exact files are Idea.
Unsupported executable extensions error. Neither adapter changes or discovers the process cwd.

Native acquisition checks regular-file metadata and size before allocation, then reads through
`Read::take(max_bytes + 1)` with saturating arithmetic and checks the result again. Devices and
FIFOs are rejected before opening. Invalid UTF-8 preserves its conversion cause. Missing files,
directories without an index, non-directory parents and permission failures retain distinct IO
categories. Size/mtime changes during acquisition are rejected where detectable.

Each compose call owns fresh source/config snapshots, an active DFS stack and a completed cache.
Canonical aliases identify cycles; completed diamond nodes are reused without child mutation.
Unique inputs appear in first-acquisition DFS order, root first. Import edges preserve encounter
order, including each incoming diamond edge. Repeated acquisition of one identity must return
the same snapshot; changed bytes/kind/name fail instead of silently accepting stale cache data.
Repeated reads also spend aggregate byte allowance. Native/memory result clones own their data.

`ComposedSchema.config` consumes `use` but retains props and non-final references. Only root
plugins remain active. `sources` retains explicit developer-access source text; `imports`
records canonical graph edges. `provenance` records declaration origins and independently
winning attributes/whole columns using names rather than numeric column positions.

The internal normalized JSON adapter exists because frozen M0 config has no Deserialize/JSON
validation API. It checks byte/depth/node bounds before descent, duplicate keys at every depth,
exact integer range before serde can round overflow, normalized fields/flags/names, attribute
shape, column uniqueness and cross-kind names. It preserves object/member order. JSON strings
are data, including `${Name}` lookalikes. Lane A's validated JSON API can replace this internal
adapter at integration; no second public JSON schema model was introduced.

## Limits

| Budget | Default | Meaning |
| --- | ---: | --- |
| `max_source_bytes` | 1 MiB | Acquisition bound per source |
| `max_total_bytes` | 16 MiB | All acquired bytes, including repeated alias/diamond reads |
| `max_files` | 256 | Distinct canonical snapshots |
| `max_depth` | 32 | Active import path, including root; unraiseable ceiling 64 |
| `max_imports` | 4,096 | Encountered import edges |
| `max_expanded_bytes` | 16 MiB | Cumulative serialized config size charged before retention/copy |
| `max_json_nodes` | 100,000 | Nodes per normalized JSON input |

Explicit `CompileInputs` adds the parser's environment, byte and data-depth policy. JSON depth
counts normalized structural containers too; both grammars honor the hard data-depth ceiling.
Expansion charging uses a nonallocating serialization sink. It conservatively charges complete
child config even when local collisions win, bounding repeated-import work and cached closures.
Budget failures do not truncate output or report a partial composition as successful.

## Merge-table test matrix

`full_type_and_model_merge_table` covers **both type and model** for every local state
(absent/mutable/immutable) crossed with child mutability (mutable/immutable), with collisions
in attributes and whole columns. Other named tests cover the sequential cases below.

| Situation | Result | Proving test |
| --- | --- | --- |
| Child section/declaration absent | Keep local, including immutable local | `absent_sections_and_inert_child_plugins` |
| Local prop/enum present | Keep entire local value; never merge its members | `props_enums_and_multiple_import_order` |
| Local prop/enum absent | Adopt child; child-first spread determines map order | `props_enums_and_multiple_import_order` |
| Local type/model absent | Adopt complete child, including its mutable flag | `full_type_and_model_merge_table` |
| **G19: local type/model immutable** | **Replace completely with child; this deliberately follows upstream code over reference 00022** | `full_type_and_model_merge_table` |
| Local mutable, either child flag | Keep local mutable flag and colliding attribute values | `full_type_and_model_merge_table` |
| Local mutable, missing attributes | Adopt child attributes with child-first map order | `full_type_and_model_merge_table` |
| Local mutable, colliding columns | Keep complete local column, including all its attributes | `full_type_and_model_merge_table` |
| Local mutable, child-only columns | Prepend child group in child order; never reverse child storage | `full_type_and_model_merge_table`, `diamond_snapshots_and_result_clones_are_independent` |
| Multiple imports | Earlier conflicting values win in a mutable accumulation; later child-only column groups prepend | `props_enums_and_multiple_import_order` |
| Accumulated imported declaration immutable | Next import replaces it; mutable accumulation instead extends | `multiple_imports_recheck_accumulated_mutability` |
| Type/model declaration order | Local names first, then first-encounter imported-only names | `declaration_order_is_local_then_imported` |
| Imported plugin blocks | Remain inert, including transitive blocks | `absent_sections_and_inert_child_plugins`, `json_root_transitive_uses_and_plugin_order` |
| Winning origins/reference sites | Move with adopted/replaced declaration, absent attribute or whole column; discard losing sites | `reference_sites_follow_mutable_winners`, `reference_sites_follow_replacement_and_adoption` |

Additional tests cover importer-relative/root-alias/absolute/index resolution, canonical and
symlink cycles, diamonds, native/memory parity, sparse large files, IO/encoding causes,
independent clones, concurrent explicit environments, fresh native runs after edits,
changed-canonical-snapshot injection, all lowerable limits and hard import depth.
The 20,000-level JSON case runs in a child with a ten-second watchdog, kill and reap on timeout.
The same child proves the maximum accepted JSON depth and one above. No new ignored tests.

## Typed-reference integration seam and remaining work

The coordinator explicitly authorized a local shim while lane A's additive parser API is absent.
`ReferenceSite` mirrors A's `Reference` fields: kind, declaration, optional named column,
attribute, nested index/key path, target, source span and forward flag. Origins and sites move
together in `replace_metadata`, covering adoption, mutable extension and G19 replacement.
Synthetic typed-site unit tests verify this behavior independently of placeholder strings.

**Ready now:** effective config, props, ordered source snapshots/import graph, winner provenance
and merge logic carrying real-shaped reference payloads by declaration/attribute/column.
No declaration or column is renamed; nested paths and source spans remain attached unchanged.

**Await integration:** `compile_source` still calls frozen M0 `parse`, so production
`ComposedSchema.references` is empty. No finalization is attempted and no actual source
reference is inferred from a `${Name}` string. Imported props are composed but reference
expansion against them is not yet an end-to-end claim.

Integration steps for the coordinator's next lane:

1. At `compile_source`, use A's `SourceFile`/`parse_ast` and
   `compile_ast_with_references`, preserving the config plus owned reference vector.
2. Replace/convert the local shim types with A's `Reference`, `DeclarationKind`, `ValuePath`
   and `SourceSpan`. If replacing them with re-export aliases, move the small `is_within`
   predicate to a local function/extension trait: an inherent impl on a foreign type is invalid.
3. Substitute A's validated JSON API where appropriate, including typed JSON reference metadata.
4. Add actual imported-prop/reference-versus-literal, forward-reference and expansion-budget
   end-to-end tests; call `finalize_with_references` deliberately at the selected integration seam.
5. Reconcile shared exports and update the root crate's M0-only introduction. Its existing
   prose was left untouched because lane B may only add `mod`/`pub use` lines there.

## Verification commands and results

Toolchain: rustc 1.96.0 (`ac68faa20`), cargo 1.96.0 (`30a34c682`), LLVM 22.1.2,
cargo-llvm-cov 0.8.6, installed native `aarch64-apple-darwin`. Default transformer features;
no all-features or additional-target claim. Commands ran from this worktree with locked Cargo
commands. `cargo clean` and `cargo llvm-cov clean --workspace` ran before the first build.

| Exact command | Result |
| --- | --- |
| `cargo clean` | PASS, 0 stale files initially present |
| `cargo llvm-cov clean --workspace` | PASS before first build, and again immediately before coverage |
| `cargo fmt --all -- --check` | PASS |
| `cargo test -p stackpress-idea-transformer --all-targets --locked` | PASS: 41 tests, 0 failures, 1 pre-existing ignored writer child helper |
| `cargo test --workspace --all-targets --locked` | PASS: 197 tests, 0 failures, 17 pre-existing ignored helpers |
| `cargo test --workspace --doc --locked` | PASS: 398 doctests, 0 failures, 0 ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | PASS |
| `cargo test -p stackpress-idea-transformer --test source_loaders native_symlinks_permissions_and_special_files --locked -- --nocapture` | PASS; native mode-000 permission denial reproduced, no bypass/skip message |
| `cargo tree -p stackpress-idea-transformer --edges normal --locked` | PASS; dependencies unchanged |
| `git diff --check` | PASS; tracked diff is additive exports only, new files are within lane ownership |

Test helpers were kept in a separate test-only module so production coverage excludes their
lines. After that test-only relocation, transformer and workspace tests, fmt and Clippy passed
again. No production behavior changed after the reported checks. All added source lines are
at most 100 columns; no unsafe code, process env mutation, cwd changes or locks were added.

Coverage command, after the clean listed above:

```sh
cargo llvm-cov -p stackpress-idea-transformer --all-targets --locked \
  --exclude-from-report stackpress-idea-parser \
  --ignore-filename-regex '/tests/|/tests\.rs$' --fail-under-lines 70 \
  --json --output-path /private/tmp/stackpress-lane-b.XTpOBc/transformer-coverage.json
```

**PASS: 1,258 / 1,318 production lines, 95.45%.** Only test files and the separately owned
parser crate are excluded; no transformer production file is excluded.
The seven new production modules alone cover 790 / 819 lines (96.46%).

| Production file | Covered / executable lines | Coverage |
| --- | ---: | ---: |
| `compose/json.rs` | 270 / 275 | 98.18% |
| `compose/merge.rs` | 132 / 132 | 100% |
| `compose/mod.rs` | 176 / 190 | 92.63% |
| `source/memory.rs` | 54 / 54 | 100% |
| `source/mod.rs` | 36 / 36 | 100% |
| `source/native.rs` | 79 / 88 | 89.77% |
| `source/paths.rs` | 43 / 44 | 97.73% |

Logs and the JSON report are in `/private/tmp/stackpress-lane-b.XTpOBc/`; they are disposable
review evidence, not repository inputs. A secondary coverage-summary invocation initially used
unsupported `report --exclude-from-report`; correcting it to a parser/test filename filter
produced the identical 95.45% summary without rerunning tests.

Remaining unforced paths include native mid-read metadata/size races and canonicalize/read/
metadata IO failures, plus internal invariant/error-rendering/no-op-flush paths. Actual missing,
permission, invalid-UTF-8 and between-edge source-change failures are proven. Size/mtime checks
are cooperative-workspace drift detection, not an atomic filesystem snapshot guarantee.
Branch coverage was not measured; the semantic decision matrix supplies separate branch evidence.
Linux, Windows, WASM and network filesystem behavior remain unverified.

No refactoring rounds were performed. The next gate is coordinator review/integration and
explicit release into the requested rounds, with the typed-reference integration gap retained.
