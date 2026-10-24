# Slice 3 parser assessment evidence

Design checkpoint for [task 00006](tasks/00006-slice-3-idea.md). Load for the pinned core's build, fixture results, reproducible assessment method, dependencies, and reuse recommendation; [grammar findings](slice-3-design-grammar.md) separates source compatibility from proposed corrections.

## Authority and isolation

Assessed 2026-10-24 on `trial/3`, initial HEAD `bbbcd3d5443954a5ebaea1fc492d59f826a5c105` (clean). No other trial branch was read. The Idea checkout remained clean at `cf8e922ba6e57746e15a05c26df0546dda2cc5c9`. The read-only parser source is `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`, path `packages/idea-rust/crates/idea_parser_core`.

All assessment compilation ran in `/tmp/stackpress-idea-assessment.RrS9HO`; `/private/tmp` is its canonical macOS spelling. A fresh extraction had no existing `target/`. No Cargo production code, manifests, or dependencies were changed in the Rust repository. No upstream checkout, modification, build, commit, push, listener, or N-API compilation occurred.

The [earlier assessment](../../references/00008-discussion-diagnostics-existing-parser.md) established separation from N-API, not compilation or parity. This receipt supplies that missing evidence. These are assessment results, not Slice 3 implementation verification.

## Exact commands and baseline

Commands ran from this repository unless a working directory is specified:

```sh
rustc --version
cargo --version
git -C /Users/cblanquera/server/projects/stackpress/idea status --short
git -C /Users/cblanquera/server/projects/stackpress/idea rev-parse HEAD
mktemp -d /tmp/stackpress-idea-assessment.XXXXXX
git -C /Users/cblanquera/server/projects/stackpress/idea archive 6c289d1f3fba17c758baafb2d4dcefd87fd448ea packages/idea-rust | tar -x -C /tmp/stackpress-idea-assessment.RrS9HO
```

Inside `/tmp/stackpress-idea-assessment.RrS9HO/packages/idea-rust`:

```sh
cargo test -p idea_parser_core --offline
cargo build -p idea_parser_core --example assess --offline --locked
cargo tree -p idea_parser_core --edges normal --offline --locked
cargo clippy -p idea_parser_core --lib --offline --locked -- -D warnings
RUSTDOCFLAGS='-D warnings -D missing-docs' cargo doc -p idea_parser_core --no-deps --offline --locked
```

| Check | Actual result |
| --- | --- |
| Toolchain | rustc 1.96.0 (`ac68faa20`), cargo 1.96.0 (`30a34c682`), native `aarch64-apple-darwin`. |
| Core tests | PASS: 3 integration tests; 0 unit tests; 0 doctests. No ignored tests. |
| Assessment runner build | PASS. Added only to the disposable extraction. |
| Clippy, library only | PASS with `-D warnings`; this does not enable missing-documentation checks. |
| Rustdoc under repository-equivalent missing-docs policy | FAIL (exit 101): undocumented modules, declarations, fields, variants and functions. |
| Formatting | FAIL: unmodified core `finalize.rs`, `lib.rs`, `parser.rs`, `token.rs` differ from rustfmt. |
| Coverage | Not measured for the upstream assessment. Three smoke tests do not establish the implementation coverage gate. |

The first test command generated a disposable Cargo.lock offline. It resolved workspace N-API packages into that lock but compiled only the core and its Serde dependencies. The original workspace uses resolver 2 and edition 2024; extraction must adopt the accepted resolver 3/MSRV 1.96/workspace lints, rather than copy its workspace manifest.

For the uncontaminated source-format check, Python invoked `subprocess.run(['rustfmt', '--edition', '2024', '--check', *sorted(core_src.glob('*.rs'))], capture_output=True, text=True)` and saved output in `rustfmt.log`. An earlier `cargo fmt --all -- --check` also failed, but included assessment code and the excluded Node crate; it is not the basis for the four-file core finding.

## Runner and comparison method

The temporary `crates/idea_parser_core/examples/assess.rs` contained this complete runner:

```rust
use idea_parser_core::{parse_ast, compile_schema, finalize};
use serde_json::json;
fn main() {
    for file in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&file).expect("read fixture");
        let ast = parse_ast(&source);
        let compiled = ast.as_ref().map_err(Clone::clone)
            .and_then(|a| compile_schema(a, false));
        println!("{}", json!({
            "file": file, "ast": ast,
            "compiled": compiled, "final": finalize(&source)
        }));
    }
}
```

The runner is assessment infrastructure, not proposed production API/style. `compare.py` enumerated both directories below, passed every sorted `*.idea` to this executable in a subprocess with `DATABASE_URL=test`, and compared parsed JSON structurally. Arrays retained order. Object key order was ignored for equality, so the separate plugin-order probe is essential.

- Pinned: `/tmp/stackpress-idea-assessment.RrS9HO/packages/idea-rust/tests/fixtures`.
- Current: `/Users/cblanquera/server/projects/stackpress/idea/packages/idea-parser/tests/fixtures`.

For declaration/import goldens the comparison selected `ast.body[0]`; for `schema.json` it selected the entire AST. The second AST comparison recursively removed **only** `start` and `end`, matching the intent of upstream tests. It did not discard missing fields or normalize method/flag nodes to obtain a pass. `schema.idea` additionally compared non-final compilation to `references.json`, and finalization to `final.json`.

All 21 files in the two fixture directories are byte-identical (10 Idea, 11 JSON). This is one unique corpus executed twice, not 20 independent grammar scenarios.

## Exact per-fixture results

Each row states the pinned result and current result explicitly. `P/P` means pinned PASS/current PASS; `F/F` means pinned FAIL/current FAIL. AST columns measure equality, not parse success.

| Fixture | Parse | Compile non-final | Exact AST | AST excluding spans | Finalization |
| --- | --- | --- | --- | --- | --- |
| `address.idea` → `address.json` | P/P | P/P | F/F | F/F | P/P |
| `big.idea` | P/P | P/P | No golden in either set | No golden in either set | P/P |
| `enum.idea` → `enum.json` | P/P | P/P | P/P | P/P | P/P |
| `model.idea` → `model.json` | P/P | P/P | F/F | F/F | Unknown `Text` in both |
| `mutable.idea` → `mutable.json` | P/P | P/P | F/F | P/P | Unknown `Text` in both |
| `plugin.idea` → `plugin.json` | P/P | P/P | F/F | F/F | P/P |
| `prop.idea` → `prop.json` | P/P | P/P | F/F | F/F | P/P |
| `schema.idea` → `schema.json` | P/P | P/P | F/F | F/F | P/P |
| `type.idea` → `type.json` | P/P | P/P | F/F | P/P | Unknown `Text` in both |
| `use.idea` → `use.json` | P/P | P/P | P/P | P/P | P/P |
| `schema.idea` → `references.json` | N/A | P/P, exact config | N/A | N/A | N/A |
| `schema.idea` → `final.json` | N/A | N/A | N/A | N/A | P/P, exact final config |

Per set: 10/10 parses, 10/10 non-final compilations, 2/9 exact AST matches, 4/9 AST matches excluding spans, 7/10 finalizations, and both compiled-schema goldens match. The three standalone `Text` failures are also failures in the current TypeScript oracle; those fixtures omit their referenced prop. They are not additional Rust parity failures.

AST differences: ordinary object properties gain `kind`, `method`, `shorthand`, `computed`; empty attribute methods become empty arrays instead of literal `true`; constructed synthetic node/declarator spans differ. `address`, `plugin`, and `prop` have only the extra property metadata once spans are removed; `model` has two empty-method node differences; `schema` combines both classes. Do not label these as lost compiled metadata: both full schema config goldens match.

## TypeScript oracle and additional probes

Current parser `src/` was copied read-only into temporary `ts-parser/`; no upstream build ran. A temporary node_modules symlink supplied already installed dependencies. `ts-oracle.mjs` loaded `index.ts` and `trees/SchemaTree.ts`, called `SchemaTree.parse`, `parse`, and `final` for each source, and captured success/error independently. No installed packages were changed.

```sh
ln -s /Users/cblanquera/server/projects/stackpress/idea/node_modules /tmp/stackpress-idea-assessment.RrS9HO/node_modules
python3 /tmp/stackpress-idea-assessment.RrS9HO/compare.py
python3 /tmp/stackpress-idea-assessment.RrS9HO/probes.py
```

Python subprocesses invoked the TypeScript oracle as follows, with the exact enumerated fixture or probe paths appended and environment `DATABASE_URL=test`:

```sh
node --import /Users/cblanquera/server/projects/stackpress/idea/node_modules/tsx/dist/loader.mjs /tmp/stackpress-idea-assessment.RrS9HO/ts-oracle.mjs
```

Node 26.2.0 oracle: current fixtures 10/10 parse and compile, 7/10 finalize, with the same three `Unknown reference Text` results. This was not a run of the Mocha suite. The oracle also ran the 29 explicit small probes listed/interpreted in [grammar findings](slice-3-design-grammar.md); deep nesting ran only against Rust in a separate process with a 10-second deadline.

The deep input was constructed as `'prop X { x ' + '[' * 20000 + '1' + ']' * 20000 + ' }'`. The Rust executable exited **-6 (SIGABRT)**, reporting stack overflow. It failed before a recoverable parser diagnostic. An unchanged extraction is vetoed on resource-safety grounds; this is not evidence of memory unsafety/undefined behavior.

Disposable evidence files: `fixture-results.json`, `probe-results.jsonl`, `ts-fixture-results.jsonl`, `ts-probe-results.jsonl`, `deep-result.txt`, `rustfmt.log`. These may be deleted with `/tmp`; authoritative findings and the runner contract remain here. No source-code archive was placed in the KB.

Assessment script SHA-256 values for coordinator inspection of the surviving temporary directory:

- `compare.py`: `168ca2f59f0a4f4092461dcf3623fd5a2e8a89b2641043519a6ec2f7e43ef155`.
- `probes.py`: `a1810b559ebeb6ca9670c10d3b7ba2e753df4012be67b12e696c8773ba9c2fd0`.
- `ts-oracle.mjs`: `3933fcbc571a63f795dad969143f518b02801bab05f6501eebacc837ce7fc1d9`.

## Dependencies and code quality

Core manifest: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, both with defaults. Offline normal tree resolved serde/serde_core/serde_derive 1.0.229, serde_json 1.0.154, proc-macro2 1.0.107, quote 1.0.47, syn 3.0.9, unicode-ident 1.0.27, itoa 1.0.18, memchr 2.8.3, zmij 1.0.23. N-API is absent from normal core edges. No parser generator, async runtime, filesystem crate, or unsafe production code exists in the core.

There are 1,567 source lines: `parser.rs` 646, `finalize.rs` 349, `token.rs` 292, `ast.rs` 174, `error.rs` 49, `types.rs` 35, `lib.rs` 22. Only 19 Rustdoc lines exist, and no `// ` explanatory block comments. Cursor logic is duplicated between parser and debug tokenizer; stringly typed public AST fields admit invalid states; compilation silently falls back to null on serialization failure; BTreeMap loses declaration order. The layer separation is useful, but documentation/comment density and defensive validation do not meet [Rust style](../../context/rust-code-style.md).

The upstream test mutates process environment in an `unsafe` block, without a safety justification, in a multithreaded test harness. Do not copy that test pattern; use explicit environment input or child-process environment setup. No global runtime registry, locks, or shared mutable parser state were found; ordinary AST/config clones own their data.

## Q08 recommendation

**Extract and repair the core source, with Apache-2.0 provenance and applicable license/notice retention, after approval.** Use `crates/idea/idea-parser`, the accepted package name, and independently reviewed fixture tests. Keep Node bindings/npm packages out. Preserve useful scanner/parser/compiler separation, then replace unsafe test environment mutation, add limits/order/source diagnostics, and refactor to typed Rust contracts and required docs/comments.

A pinned git dependency would retain defects, pull an upstream workspace policy into development, and make necessary fixes awkward. An entirely new parser discards a core already matching both compiled-schema goldens, without demonstrated benefit. Source extraction allows focused repair; the [M0 gate](slice-3-design-verification.md) must prove the changes before broad implementation. This recommendation does not claim full grammar parity or approve any public-behavior deviation.
