# Slice 3 lane A: M1 parser completeness

Spec file of [00001 Stackpress Rust foundation](index.md), recording Slice 3 lane A (parser, M1) of [task 00006](tasks/00006-slice-3-idea.md). Load for the corrected-grammar parser, fixture parity, JSON import/export, the typed reference API (`compile_ast_with_references`/`finalize_with_references`), and documented limitations. Moved losslessly from `crates/idea/idea-parser/LANE-A-RECEIPT.md` (merged in `b149a0e`); relative links rebased.

Status: implemented and verified locally; **stopped before refactor rounds** for coordinator
review. No merge, commit, push, publication or KB promotion is claimed.

## Scope and provenance

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s3-lane-a-parser`.
- Branch: `trial/3-s3-lane-a-parser`; unchanged HEAD/base:
  `54b188e3a5811f8d05ec6017fcb7382466660135` (accepted M0).
- Every tracked/untracked change is inside `crates/idea/idea-parser/**`.
  No manifest, lockfile, facade, transformer, example or `.agents` file was changed.
- Existing Apache-2.0 LICENSE is retained. `PROVENANCE.md` records M1 additions and fixtures.
- Upstream was read only, at `cf8e922ba6e57746e15a05c26df0546dda2cc5c9`; final
  `git -C /Users/cblanquera/server/projects/stackpress/idea status --short` was empty.
  No upstream build/install/modification, other trial-branch inspection, listener or new
  dependency was used.
- Before the first build, `cargo clean` and `cargo llvm-cov clean --workspace` both exited 0.
  This isolated worktree had no existing build files to remove.

## Implemented contracts

- All six declaration forms; approved casing, exactly `T`, `T?`, `T[]`, `T[]?`, immutable
  marker preservation, open lowercase dotted metadata, ordered members/plugins/columns,
  raw double-quoted Idea strings, nested objects/arrays, comments and explicit separators.
- Exact signed/unsigned 64-bit integers, finite decimal floats, integer negative zero and
  rejection of overflow/nonfinite/malformed literals. JSON additionally supports its standard
  quoting/escaping and exponent syntax with the same finite/range checks.
- Duplicate declarations across kinds and duplicate columns/enum/object/attribute members
  error with a primary range plus the earlier related range.
- EOF failures use zero-width EOF spans; Unicode scalar columns, CRLF/CR, UTF-8 byte ranges
  and synthetic enclosing/zero-width syntax spans are covered by tests.
- Pure syntax preserves environment names as typed nodes. Compilation takes an immutable
  explicit environment snapshot, with missing-input error or explicit empty compatibility.
  Repeated expansions spend bytes before copying. No ambient environment reads/mutations.
- Immutable parser-owned AST; compilation rechecks caller-lowered limits by reparsing its
  retained source. Public inspection cannot turn a manually assembled syntax tree into Ast.
- Source depth has a non-raiseable 64-container ceiling. Byte limits also lower derived node
  and collection budgets (hard maxima 100,000 nodes and 10,000 entries per collection).
  Tokenization, JSON preflight, checked public-config export and typed reference resolution
  enforce finite work/depth/width/copy budgets.
- Tokenizer reuses the parser's cursor, trivia, raw-string, numeric, env and namespace scanners.
  It returns exact raw spelling and source coordinates without evaluating environment inputs.
- Normalized JSON imports reject duplicate keys at every depth, unknown structural fields,
  wrong shapes/types, invalid names/flags/modifiers, cross-kind collisions and duplicate
  columns. Ordered metadata remains open data. Checked export validates caller-built Values
  before recursive Serde/clone operations and bounds the serializer's allocation.
- Typed reference sites support nested argument arrays/objects, declaration and column
  metadata, prior-declaration rules, imported targets, missing targets/cycles, stale/duplicate
  sites and bounded materialization. Only listed sites are resolved; literal `${Name}` strings
  remain unchanged. Per-run owned snapshots and clone independence are tested.

## Additive API handoff for lanes B and D

The M0 `parse(file, source, inputs)` and `finalize(file, source, inputs)` signatures remain.
`SchemaConfig`, `FinalSchema`, `CompileInputs` and `ParseLimits` fields are unchanged.
Legacy `finalize` still explicitly discards use directives without loading them.

New public entry points:

```rust
SourceFile::new(name, text)
parse_ast(&SourceFile, ParseLimits) -> Result<Ast, IdeaError>
tokenize(&SourceFile, ParseLimits) -> Result<Vec<Token>, IdeaError>
compile_ast(&Ast, &CompileInputs) -> Result<SchemaConfig, IdeaError>
compile_ast_with_references(&Ast, &CompileInputs) -> Result<CompiledSchema, IdeaError>
finalize_ast(&Ast, &CompileInputs) -> Result<FinalSchema, IdeaError>
finalize_with_references(&SchemaConfig, &[Reference], &CompileInputs)
    -> Result<FinalSchema, IdeaError>
SchemaConfig::{from_json, from_json_named, validate, to_json}
CompiledSchema::{from_json, to_json}
```

- `Ast::{source, syntax}` borrows checked source/syntax. Syntax node types are exported for
  inspection, but their construction never bypasses Ast's checked compilation boundary.
- `CompiledSchema { config: SchemaConfig, references: Vec<Reference> }` is the lossless
  composition/JSON handoff. Both JSON methods take `ParseLimits`; `from_json` takes `&str`.
- `Reference` has `kind: DeclarationKind::{Type, Model}`, `declaration: String`,
  `column: Option<String>`, `attribute: String`, `path: Vec<ValuePath>`, `target: String`,
  `span: SourceSpan`, `forward: bool`. Paths start with an argument index; nested steps are
  `ValuePath::{Index(usize), Key(String)}`. All are owned Clone data; reference/path/kind/span
  implement Serialize and Deserialize. `SourceSpan` has `file`, `start`, `end`, `line`, `column`.
- `forward` records a target explicitly declared at/after its owner in the same original
  source. Imported targets absent from that source are not marked forward.
- **Lane B must retain only sites belonging to surviving declarations/attributes/columns.**
  Paths name columns, not column indices, so prepending imported columns does not shift sites.
  The composer removes resolved use directives before typed finalization.
- `finalize_ast` and `finalize_with_references` reject outstanding use directives. They never
  imply that a loader ran. Plain `SchemaConfig::from_json` treats all strings literally;
  typed envelope imports carry explicit sites instead of guessing from placeholder spelling.
- `IdeaError.related: Box<[SourceSpan]>` adds duplicate-related locations. Boxing keeps the
  existing by-value Result error small enough for warning-clean Clippy without allowances.
- `ParseLimits::{max_nodes, max_collection}` exposes the derived resource policy.

## Fixture pass table

All 21 original fixture files were compared byte-for-byte against both the pinned Rust
`6c289d1` tree and current TypeScript fixtures: **21/21 identical**. There are ten unique Idea
inputs, not twenty independent cases. A disposable current-parser copy produced complete
non-final and final goldens in `tests/fixtures/compiled/*.json`, using `DATABASE_URL=test`.
Both complete original `references.json` and `final.json` goldens also remain tested.

| Fixture | Parse / tokenize | Complete compiled golden | JSON round trip | Legacy / typed final golden |
| --- | --- | --- | --- | --- |
| address.idea | PASS | PASS | PASS | PASS |
| big.idea | PASS | PASS | PASS | PASS |
| enum.idea | PASS | PASS | PASS | PASS |
| model.idea | PASS | PASS | PASS | Expected UnknownReference Text |
| mutable.idea | PASS | PASS | PASS | Expected UnknownReference Text |
| plugin.idea | PASS | PASS | PASS | PASS |
| prop.idea | PASS | PASS | PASS | PASS |
| schema.idea | PASS | PASS | PASS | PASS |
| type.idea | PASS | PASS | PASS | Expected UnknownReference Text |
| use.idea | PASS | PASS | PASS | PASS |

Typed fixture finalization explicitly clears uses to compare the legacy oracle's single-source
view; this is not a composition test. Separate tests prove strict staged finalization refuses
uses. No ESTree/TypeScript AST JSON parity is promised.

Oracle reproduction used the disposable copy at `/private/tmp/slice3-parser-oracle-vv6kr920`:

```sh
node --import /Users/cblanquera/server/projects/stackpress/idea/node_modules/tsx/dist/loader.mjs /private/tmp/slice3-parser-oracle-vv6kr920/oracle.mjs <each fixture path>
```

The script imports the copied `src/index.ts`, calls `parse` and `final` for each fixture and
records the complete JSON outputs/error category. It sets the fixture env in its child process,
without modifying the upstream checkout or process-global environment in Rust tests.

## Final checks and exact commands

Every command below exited **0**. Logs are retained in this worktree's ignored `target/` as
`lane-a-{fmt,parser,workspace,doctests,clippy,rustdoc,tree}.log`; command/count metadata is
`target/lane-a-checks.json`.

```sh
cargo fmt --all -- --check
cargo test -p stackpress-idea-parser --all-targets --locked
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo tree -p stackpress-idea-parser --edges normal --locked
git diff --check
```

- Parser: **19 passed, 0 failed**, with three enumerated ignored child helpers, each invoked
  by an active watchdog parent. These exercise 20,000-deep source/JSON/manual Values, exact
  accepted/rejected depth boundaries, source/env/output/collection/reference limits and
  amplification. Parent deadlines kill and reap; hostile manually built Values are dismantled
  iteratively in the child. No critical hazardous test is silently skipped.
- Workspace: **177 passed, 0 failed**, 19 enumerated helper entries ignored by direct harness
  discovery. Doctests: **412 passed, 0 failed**. Fmt, Clippy and Rustdoc: warning-clean.
- Mechanical scan: every source/test function has Rustdoc; no Rust source line exceeds 100
  columns. This is a style signal, not a substitute for the coordinator's requested rounds.
- Normal dependency tree remains only indexmap, serde, serde_json and their ordinary
  transitive dependencies; no lib/server/CLI/IO/async/Node/N-API dependency was added.

## Clean parser coverage

After final source changes and with no concurrent build, these exact commands passed:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-idea-parser --all-targets --locked --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/target/' --json --output-path target/lane-a-coverage.json
```

Production parser source: **2,251 / 2,404 executable lines, 93.64%**. The report sums each
`crates/idea/idea-parser/src/` file once; `target/lane-a-coverage-summary.json` records the total.
The per-crate 70% development floor passes. No production parser file/branch is excluded.
Exclusions are toolchain/dependency/build output and integration-test harnesses. Coverage
includes guarded child-process execution. Ordinary doctests ran separately, not in coverage.

Toolchain: rustc 1.96.0 (`ac68faa20`), Cargo 1.96.0 (`30a34c682`), cargo-llvm-cov 0.8.6,
LLVM 22.1.2, native `aarch64-apple-darwin`, parser default features. No other target was run.
Remaining uncovered lines chiefly include defensive extracted-compiler malformed-private-AST
paths and some checked JSON/reference validation failures; coverage does not prove all paths.

## Rough edges, limits and next gate

- The typed path is additive to the frozen compatibility config. Direct derived Clone or raw
  Serde on arbitrary caller-built recursive Values remains unchecked; use `validate`/`to_json`
  and typed finalization at external boundaries. No breaking field-privacy migration was made.
- JSON reference metadata is structurally validated, including sites, names, ranges and
  nonzero coordinates. Its historical Unicode coordinates cannot be authenticated without
  original source text. Live parser/tokenizer coordinates are checked against the actual source.
- Callers give distinct logical SourceFile names; transformer canonical source identity and
  source loading remain lane B's responsibility. No filesystem/global source registry is hidden.
- Syntax depth counts recursive Idea data; JSON depth counts normalized JSON containers.
  Structural JSON wrappers can therefore require a larger caller budget than the inner Idea
  value, and the hard ceiling is never raised. Export may return Limit for that boundary.
- Staged compilation reparses retained source to honor lowered limits. Checked config export
  uses validation/serialization passes. These are bounded linear passes, with review/polish
  available in the requested rounds rather than an unmeasured performance claim.
- Refactor rounds 1–4, coordinator re-verification/merge, cross-lane integration and full crate
  documentation are **not performed or claimed**. The coordinator explicitly requested this
  stop before those rounds. No outstanding dependency or product decision blocks review.
