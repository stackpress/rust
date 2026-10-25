# Task 00006A parser documentation receipt

Implementation notes for the parser part of [task 00006A](00006A-idea-docs.md).
The implementer has completed author verification; coordinator review is still required.
No acceptance, commit, push, publication, or context promotion is claimed.

## Scope and source authority

- Checkout: `/Users/cblanquera/server/projects/stackpress/rust`, branch `trial/3`.
- Observed starting HEAD: `544aa4787ea429828cf131c131af4d7ad3b55d0c`; checkout HEAD at
  handoff: `d35066a5bded1b1eec74fc14472436c2455cd441` (concurrent coordinator work). The brief named
  `2546fbd`; this work used the current checkout without switching or resetting. Parser source,
  tests, manifests, and lockfile are unchanged between the observed starting and handoff HEADs.
- Authored only `crates/idea/idea-parser/docs/**` and this requested receipt.
  Concurrent transformer/wrapper documentation belongs to the other implementer.
- Ground truth: current parser public source, contract tests, M1/round tests, fixtures,
  `PROVENANCE.md`, the Slice 3 parser/round receipts and grammar, and decisions G18–G22.
  TypeScript contracts were historical/reference material, not current Rust API authority.
- Read the root/context instructions, coordinated development docs stage, installed
  `chrisai-docing`, and Lib/Server/CLI documentation precedents, including task 00005A notes.
- No source, tests, manifests, workspace lockfile, upstream checkout, other trial branch,
  listener, commit, push, publication, or KB MCP/indexing was changed or used.

## Page inventory

Created 18 pages / 1,767 lines. Paths below are relative to
[`crates/idea/idea-parser/docs/`](../../../../crates/idea/idea-parser/docs/README.md).

| Page | Lines |
| --- | --- |
| `README.md` | 44 |
| `api/README.md` | 34 |
| `api/error.md` | 73 |
| `api/finalization.md` | 159 |
| `api/inputs.md` | 133 |
| `api/references.md` | 160 |
| `api/schema.md` | 208 |
| `api/source.md` | 107 |
| `api/stages.md` | 141 |
| `api/syntax.md` | 83 |
| `api/tokens.md` | 68 |
| `concepts.md` | 133 |
| `getting-started.md` | 75 |
| `guides/diagnostics-and-limits.md` | 73 |
| `guides/environment-inputs.md` | 57 |
| `guides/json.md` | 75 |
| `guides/resolving-references.md` | 63 |
| `guides/writing-a-schema.md` | 81 |

The entry page routes first-time and returning readers. The quick start parses and compiles
in-memory text, inspects the Profile model and optional column, and prints a visible success.
Concepts covers the language, named source/AST/config stages, actual references versus literal
`${Name}` strings, strict versus compatibility finalization, limits, diagnostics, and JSON.
Five guides follow reader intent; nine public-surface API pages plus an index support lookup.

## Public-item coverage

A source-name scan and manual public-shape review cover:

- all 34 exported structs/enums/type aliases;
- all 8 free functions and 13 public inherent methods (21 total);
- all 98 public field declarations, with common syntax offsets documented together;
- all 31 enum variants and the `ParseLimits::HARD_DEPTH` associated constant;
- defaults, serialization traits/shapes, ordering, source/IO boundaries, parameter meanings,
  returns/errors, and cross-links to relevant guides.

The API groups related types by surface rather than mirroring private module layout.
`compile_schema`, private constructors/helpers, and private state are intentionally excluded.
Detached syntax inspection cannot construct a checked Ast; there is no public `compile` function.
Manual review complements the mechanical name scan; name presence alone is not semantic proof.

Coverage artifacts: `/private/tmp/stackpress-docs-00006a-parser/public-items.json`.

## Example evidence

All examples remain plain fenced Markdown; no tracked harness or doctest wiring was added.
The disposable consumer and evidence live under
`/private/tmp/stackpress-docs-00006a-parser/`.

- All 33 Rust blocks are complete independent programs. Each was extracted into a separate
  consumer binary, formatted with Rust 1.96 rustfmt, compiled, and executed successfully.
  Nine are learning/walkthrough blocks; the other 24 demonstrate API usage.
- Final fence contents match the compiled consumer source files exactly.
- All three standalone Idea text examples were separately parsed/compiled successfully.
- The quick start also ran through `cargo run --offline --locked`, printing exactly
  `Profile: id String` with no program stderr. No input file or ambient environment was read.
- The consumer lock was seeded from the workspace lock, adapted once with offline resolution
  for the disposable package, then used with `--offline --locked` for checks/builds/runs.
  Every dependency version and checksum was compared to the workspace lock and matches.
  The repository lockfile stayed unchanged.
- `cargo clean` and `cargo llvm-cov clean --workspace` ran against fresh external target
  directories before the first build. Workspace and consumer builds use separate targets,
  leaving concurrent agents' default build output alone.

Commands (target directories set as described):

```sh
cargo check --manifest-path /private/tmp/stackpress-docs-00006a-parser/consumer/Cargo.toml --offline --locked --bins
cargo build --manifest-path /private/tmp/stackpress-docs-00006a-parser/consumer/Cargo.toml --offline --locked --bins
cargo run --manifest-path /private/tmp/stackpress-docs-00006a-parser/consumer/Cargo.toml --offline --locked
cargo fmt --all -- --check
cargo test --workspace --doc --locked --offline
```

All passed on Rust 1.96.0, native `aarch64-apple-darwin`. Workspace doctests: **430 passed,
0 failed**. Ordinary workspace source/contract tests and coverage were not rerun for this
docs-only task; the existing source/receipt claims are not substituted for fresh tests.

Evidence files include `examples.json`, `run-results.json`, `schema-examples.json`,
`schema-results.json`, `workspace-doctests.log`, and `workspace-doctests.json`.

## Author review gates

Followed the skill sequence: guided-learning ownership first, public API reference next,
then a standard editorial pass, then formatting/links and rustfmt for snippets.

- Learning review: explicit audience and first success; explained examples; small initial case;
  concepts after success; separate lookup material; task-based follow-ups and learning checkpoints.
- Reference review: public-only scope, signatures/fields/defaults, realistic examples,
  parameter tables where applicable, Returns prose including errors, and integration links.
- Accuracy review: current source/tests for raw Idea strings versus JSON escaping, scalar env
  values, all modifiers/casing, duplicate-related spans, prior/forward/import reference behavior,
  typed envelope validation, caller-built config limits, and compatibility import discarding.
- Mechanical scan of all 18 pages: one H1 each, tagged/closed fences, no empty parent headings,
  no double blank lines, no trailing whitespace, and **114 relative links/anchors resolving**.
  The temporary missing wrapper-doc link resolved once the parallel docs appeared.
- `git diff --check` passed; tracked source/tests/manifests/lockfile remain unchanged and the
  index is untouched. Concurrent docs and spec work are preserved.

Mechanical evidence: `scan_docs.py` and `scan.json` in the disposable evidence directory.
Agent Workspace validation after writing this receipt is reported in the handoff.

## DX observations (no fixes made)

- `finalize` silently discards `use` directives as explicit compatibility behavior, whereas
  `finalize_ast` and typed finalization reject them. The entry-selection table makes this
  difference prominent; a successful compatibility result does not establish composition.
- Plain `SchemaConfig` JSON loses reference intent. Callers must deliberately retain the
  `CompiledSchema` envelope and keep sites synchronized when editing/composing config.
- `SchemaConfig` supports named JSON import; `CompiledSchema` has no named import counterpart.
  Envelope errors may use `<json>` or stored site coordinates, depending on the failing check.
- `SourceFile::new` constructs coordinate indexes before a parse policy is supplied.
  Public caller-built config/raw Clone/Serde also remain outside checked resource boundaries;
  callers retain responsibility for constructing and destroying their own recursive values.
- `FinalSchema` has no checked JSON import/export/validate method family. Its public Serialize
  implementation is distinct from the checked SchemaConfig/CompiledSchema methods.
- `@flag` and `@flag()` normalize to the same true boolean, so compiled output does not retain
  that syntax choice. Syntax inspection remains available when the difference matters.

No unresolved product decision or in-scope implementation defect was found. Coordinator
review, independent re-verification, and folding these notes into task 00006A are next.
