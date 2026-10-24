# Task 00006: Slice 3, Idea parser, transformer, and generation

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-24 ("do both in parallel", alongside [task 00005A](00005A-server-and-cli-docs.md)). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Bring the Idea language into the Rust monorepo under the accepted structure, as three crates:

- `crates/idea/idea-parser` (`stackpress-idea-parser`): grammar, AST, compiled schema, diagnostics;
- `crates/idea/idea-transformer` (`stackpress-idea-transformer`): `use` composition and ordered transform plugins;
- `crates/idea/idea` (`stackpress-idea`): the standalone convenience wrapper.

Then prove generation before compilation with one model. Per the KB, first **assess the existing Rust parser** on Idea branch `feature/rust-parser-2` (pinned `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`, `packages/idea-rust/crates/idea_parser_core`) against the current fixtures before writing anything new. Keep its Node/N-API distribution concerns out of the Cargo crates.

This slice resolves the carried Gaps Q08 (parser parity and extraction strategy) and Q09 (generator packaging, order, output cleanup and ownership, build/runtime consistency) in [decisions](../decisions.md). It covers the Slice 1 deferrals assigned to it: host filesystem (`FileLoader`/`NodeFS` capability) and the template engine. Its target proofs from the `00006` reference are item 1 (one Idea model produces compilable public types and an operation registration) and item 10 (repeat generation is stable and the generated code compiles).

## Implementation steps

1. Design checkpoint, with no production code. Write it as spec files `slice-3-design-*.md`. It covers:
   - parser assessment evidence, built and tested from a read-only extraction outside the repository;
   - the upstream inventory (Idea parser, transformer, language catalog, Stackpress schema/generation boundaries, and the Slice 1 deferrals);
   - the API sketch;
   - the risk inventory;
   - open decisions with recommendations;
   - the milestone plan with lane ownership.
2. Coordinator review and owner decisions, then M0, the riskiest proof.
3. Milestones, a stop before rounds, rounds 1–4, and closeout.
4. Docs as a follow-up task.

## Verification process

As defined in the coordinated workflow: coordinator re-verification at every stop, including generated-code compilation, clean and repeat generation, and per-crate coverage of at least 70%.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Design checkpoint prepared on 2026-10-24; start with [Slice 3 decisions and recommendations](../slice-3-design-decisions.md), which routes assessment, inventory, API/generation contracts and risks/milestones. The pinned core was assessed only in `/tmp`; no production code or manifests changed. Unchanged reuse is vetoed on bounded-resource evidence. Q08/Q09 recommendations and disclosed compatibility/template scope choices await coordinator/owner review. Workspace validation passed with existing warnings only; all seven new files are below 250 lines. Verification notes remain coordinator-owned.


### M0 implementation receipt (2026-10-24 to 2026-10-25)

**Stopped after M0; coordinator re-verification is pending.** Started at `99055e7`.
The separate docs lane committed `ad3575f` during this effort. This implementer made no
commits/pushes and did not edit its reserved docs or task 00005A. No other trial branch,
upstream checkout/build/modification, listener, publication or KB promotion was used.

#### Implemented proof and ownership

- Extracted the pinned core's AST/parser/compiler/config into `stackpress-idea-parser`.
  [Parser provenance](../../../../crates/idea/idea-parser/PROVENANCE.md) records the exact pin,
  original author and modifications; the applicable Apache-2.0 LICENSE is retained byte-for-byte.
  Pinned tree inspection found no NOTICE. N-API and the separate debug tokenizer are excluded.
- The private AST originates in bounded parsing. Source/env bytes, recursive data and
  materialized reference expansion are bounded; env values come from an explicit map. Compiled
  maps retain declaration/member/plugin order. Corrected grammar, finite numbers, duplicate
  errors and valid named-file UTF-8/EOF diagnostics are tested. Full compiled/final fixture
  goldens match. The old 20,000-array abort now returns Limit under a child-process watchdog.
- Added the real `stackpress-idea-transformer` registry/planner/writer and `stackpress-idea`
  orchestration, with optional facade re-exports/features. The writer owns a marked root,
  disjoint plugin partitions and an aggregate owner. It checks prior content digests and exact
  owned paths, rejects manual files/edits/symlinks, formats through rustfmt stdin/stdout, stages
  output, retains backup/intent and supports explicit recovery. Check changes no generated
  files/manifest, while still acquiring a transient sibling writer lock.
- G19 is implemented in `merge_type` and tested: imported child **replaces an immutable local**;
  mutable locals retain attribute/column precedence and prepend missing child columns in order.
  The child is not mutated. Updated the design merge table to the accepted exception.
- Added independent `examples/generated-profile/codegen` and `app` packages. Codegen uses
  `stackpress_cli::main_with`, a generation event and an awaited blocking worker. It has no
  dependency on the final app. Profile generates public contracts, the real in-memory operation,
  host registration and buffered route, with compiled selection/revision assertions.
- Parity tests compare complete CLI/direct/HTTP envelopes and Slice 2's Ada/id/total behavior.
  A disposable consumer proves clean bootstrap without output, repeat-byte stability,
  selection/revision compile failures, serde-only contracts compilation, and model removal
  to a compiling empty registry and actual CLI status 404 (exit 1).
- A separate native process proves writer-lock exclusion. The actual install state machine is
  interrupted by an injected error **after backup publication**, then recovered for both an
  existing root and a first install. This is not a killed-process/power-loss durability proof.

#### Exact commands and results

Initial clean, before the first build (both exited 0):

```sh
cargo clean
cargo llvm-cov clean --workspace
```

Initial `cargo check -p stackpress-example-profile-codegen --offline` updated Cargo.lock but
could not unpack an approved crate into the sandboxed Cargo cache. The following passed with
approved cache/network access; subsequent build/test commands used `--locked`:

```sh
cargo check -p stackpress-example-profile-codegen --locked
```

Final checks, all exit 0 (logs `target/slice-3-m0-final-{0..5}.log`):

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo check -p stackpress --features idea,idea-parser,idea-transformer --locked
```

- Workspace all-targets: **165 passed, 0 failed**; 17 enumerated helper entries ignored.
  The two new ignored entries are child-only helpers exercised by their watchdog parents.
- Workspace doctests: **395 passed, 0 failed**. New parser example is a compiling doctest.
- All five new packages' scoped all-targets suite: **22 passed, 0 failed**, plus the two child
  helpers invoked by parents. Final workspace tests include this same scope.
- Warning-clean Clippy/Rustdoc passed. Initially ambiguous copied module links were fixed in
  generator source and regenerated before the passing documentation run.
- Rust source review found no lines above 100 columns. No unsafe code was introduced.

Generation and disposable consumer commands:

```sh
cargo run -p stackpress-example-profile-codegen --locked -- emit generate
cargo run -p stackpress-example-profile-codegen --locked -- emit generate --check true
cargo run -p stackpress-example-generated-profile --locked -- emit profile-search --id abc123
python3 examples/generated-profile/verify-m0.py
```

Results: inspectable source; unchanged output passes check; native search returns Ada Lovelace,
id abc123, total 1; repeat output is byte-identical. The
disposable verification script `examples/generated-profile/verify-m0.py` (removed by lane D in `20bb649` and replaced by Rust consumer tests; recoverable at `54b188e`) contains
the exact Cargo commands for independent consumers and compile-fail cases, enforces per-process
180-second deadlines and only mutates copies under `/private/tmp/stackpress-slice3-m0-*`.
The last proof output and its temporary path are retained in `target/slice-3-m0-proof.log`.

#### Clean coverage and exclusions

After `cargo llvm-cov clean --workspace`, the final measurement and per-crate gate were:

```sh
cargo llvm-cov -p stackpress-idea-parser -p stackpress-idea-transformer -p stackpress-idea -p stackpress-example-profile-codegen -p stackpress-example-generated-profile --all-targets --locked --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/tests/|/src/output/tests.rs|/src/registry/tests.rs|/target/' --json --output-path target/slice-3-m0-coverage.json
python3 examples/generated-profile/check-coverage.py target/slice-3-m0-coverage.json
```

| New package production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| stackpress-idea-parser | 1075 / 1172 | 91.72% |
| stackpress-idea-transformer | 470 / 499 | 94.19% |
| stackpress-idea | 15 / 15 | 100.00% |
| stackpress-example-generated-profile | 85 / 90 | 94.44% |
| stackpress-example-profile-codegen | 127 / 129 | 98.45% |

Every new package passes the **70% per-crate** floor; the script fails on empty measurement or
an individual package below the floor. Scope includes generated executable server/registry
source, handwritten app/codegen and both main files. App main has 0/5 covered lines in this
instrumented run; its native entry is separately exercised by the disposable proof.
The generated data-only type has no separately measurable executable lines; serialization
is exercised through all adapters. Shared selection contains constants only.
Exclusions are toolchain/dependencies/build output and test harnesses, not production branches
or generated code. The default examples exclusion is disabled. The wrapper's orchestration is
measured, not an empty-crate 100% claim. Native rustc/cargo 1.96.0, cargo-llvm-cov 0.8.6,
aarch64-apple-darwin; no other target was run.

#### Dependency and feature evidence

```sh
cargo tree -p stackpress --edges normal --locked
cargo tree -p stackpress-lib --edges normal --locked
cargo tree -p stackpress-server --edges normal --locked
cargo tree -p stackpress-example-generated-profile --edges normal --locked
cargo tree -p stackpress-example-profile-codegen --edges normal --locked
```

Saved as `target/slice-3-m0-tree-<package>.txt`. Default facade/lib/server and generated app
contain **no idea-parser, idea-transformer or tempfile**. Only the independent codegen tool
acquires generator/writer dependencies. The contracts consumer's normal tree is serde-only
(and derive dependencies). Opt-in facade features compile together. No existing dependency
version changed. New direct external dependencies beyond already used Serde/JSON/indexmap/
tokio/CLI/server are the approved tempfile 3.27.0 and sha2 0.10.9; their ordinary transitive
dependencies account for the 13 new registry packages in Cargo.lock. No Node/N-API/template
engine, database, UI or new toolchain dependency was introduced.

#### Evidence, limitations and handoff

- M0 exposes source-to-config parsing over a private extracted AST. The richer public
  source-map/typed-reference AST API, tokenizer unification and full composition are M1/M2,
  not claimed complete. Env is injected during this M0 parse rather than retained as an
  unresolved AST env node. Finalization documents single-source compatibility behavior of
  discarding use directives without loading them; it is not composition.
- The G19 merge primitive is proved; recursive use resolution/JSON imports are not implemented.
  Dependency-rich transform preflight and full install fault/platform matrices remain M2/M3.
- The writer requires cooperative ownership. SHA-256 checks detect drift, not outside-write
  races or marker authenticity. Process death can leave a stale lock requiring explicit operator
  inspection/removal. A stage retained before intent publication needs operator inspection;
  the proved recovery path begins after recognized intent. No crash-atomicity, power-loss
  durability, uncooperative-writer or all-platform claim is made.
- Rustfmt is an explicit installed toolchain dependency. Profile-only mapping rejects
  unsupported models/fields; this is not a generic framework-schema implementation.
- G14 requires **`--check true`**, not bare `--check`; no CLI grammar was changed. Typed input
  deserialization errors retain Slice 2's 500; missing/empty id guards return 400. These are DX
  observations for review. No M1 or rounds were released; rounds 1–4 remain after milestones.
  Coverage above 90% does not authorize publication or imply whole-slice completion.


## Verification notes

**Design checkpoint review (coordinator, 2026-10-24): approved; M0 released.**

- **Reviewed:** `slice-3-design-decisions.md`, `slice-3-design-api.md`, and the M0 plan in `slice-3-design-verification.md`.
- **Parser evidence:** own tests 3/3; all fixtures parse; compiled goldens match; AST parity 4/9 excluding spans. The unchanged-reuse veto stands on the stack-overflow abort, lost plugin order, env-in-enum rejection, and span defects.
- **DX:**
  - the parser is usable standalone with no IO;
  - plugins are function registrations;
  - output is inspectable;
  - diagnostics carry file, line, and column;
  - the separate codegen binary is justified by the bootstrap cycle and reuses the G14 CLI grammar.
- **Decisions:** owner decisions recorded as G18–G21 and the coordinator dependency approval as G22 in [decisions](../decisions.md). G19 deliberately follows upstream code over the reference 00022 catalog wording.


**M0 re-verification (coordinator, 2026-10-25): passed; M0 accepted.**

- **Checks rerun:** fmt passed; 165 workspace tests passed, 0 failed; 395 doctests; Clippy and Rustdoc warning-free.
- **End to end, run by the coordinator:**
  - `emit generate --check true` reported `differences: []`;
  - regenerating left `src/generated/` byte-identical (directory SHA-256 unchanged);
  - the generated app's `emit profile-search --id abc123` printed the same envelope as the Slice 2 `capability-host` binary: `{"code":200,...,"results":[{"id":"abc123","name":"Ada Lovelace"}],"total":1}`.
- **Generator isolation:** `cargo tree` normal edges of `stackpress` (facade default), `stackpress-lib`, `stackpress-server`, and `stackpress-example-generated-profile` contain no `stackpress-idea*`, `tempfile`, or `sha2`.
- **Coverage**, each measurement after `cargo llvm-cov clean --workspace`: `stackpress-idea-parser` 91.72% (1,172 lines); `stackpress-idea-transformer` 92.99% (499); `stackpress-idea` 100% (15). Generated-app 94.44% and codegen 98.45% are implementer figures, not re-measured.
- **Provenance:** `crates/idea/idea-parser/LICENSE` (Apache-2.0) and `PROVENANCE.md` are retained. The `../idea` checkout is unmodified.
- **Coordinator notes carried into M1–M4:**
  - The demo generator deliberately accepts only `Profile { id String name String }` and emits a fixed server module (`profile.rs.txt`) that uses the generated contract type, consistent with G20. General type mapping belongs to the later schema layer.
  - `--check true` follows the G14/Slice 1 argv grammar. Upstream `ArgString` likewise has no bare boolean long flag (it drops a dangling `--key`), so no grammar change is made. A short `-c` alias in the codegen tool remains an option.

**Slice 3 closeout (coordinator, 2026-10-25): verified.**

- **Integration:** all four lanes merged on `trial/3`.
  - Lane A rounds: `c90ef7e` ([receipt](../slice-3-receipt-lane-a-rounds.md)). Parser public declarations are unchanged against `8bdca2c`.
  - Lane B real-reference composition: `a5c9a13` ([receipt](../slice-3-receipt-lane-b-integration.md)). The `ReferenceSite*` placeholders are removed, and `stackpress_idea_transformer::Reference` re-exports the parser type.
  - Lane C writer rounds plus three ownership fixes: `90b9dc4` ([receipt](../slice-3-receipt-lane-c-rounds.md)).
  - Lane D wrapper composition and M4: `b4f3750` ([receipt](../slice-3-receipt-lane-d-m4.md)).
- **Behavior changes, all defect fixes approved by the coordinator:**
  - Plans reject ancestor directories that differ only by case.
  - Special files such as FIFOs are rejected before opening, so a read can't block.
  - Cleanup keeps the ownership marker until the retained tree is otherwise empty.
  - An import-depth-32 parser diagnostic now returns the accepted 400 instead of 500.
- **Wrapper API (planned reshape):** `generate(GenerateRequest, impl SourceLoader, &Registry, &OutputWorkspace) -> Result<CheckReport>` with `GenerationMode::{Check, Install}`. The function composes, runs `finalize_with_references`, then `plan_with`, all before any workspace IO.
- **Coordinator caught a native-only failure:** lane C's non-UTF-8 filename test failed outside the sandbox, because APFS returns EILSEQ while the sandbox returns EPERM. The fix was test-only.
- **Checks rerun by the coordinator on the integrated tree:**
  - fmt passed;
  - 262 workspace tests passed, 0 failed, and 430 doctests passed;
  - Clippy `-D warnings` and Rustdoc `-D warnings` are clean;
  - `cargo +1.96.0 check --workspace --all-targets` passed;
  - codegen `--no-default-features` 2/2 and `stackpress-server --no-default-features` 32/32.
- **End to end:**
  - `cargo run -p stackpress-example-profile-codegen -- emit generate -c` printed `differences: []`, exit 0. The committed manifest only gained the writer's existing optional `bytes` field; generated Rust SHA-256 values are unchanged.
  - A malformed `--input` returned public 400 `broken.idea:3:15: …`, exit 1.
- **Generator isolation:** `cargo tree --edges normal` for `stackpress-example-generated-profile` and the default `stackpress` facade shows no `stackpress-idea*`, `tempfile` or `sha2`.
- **Coverage**, each measurement after `cargo llvm-cov clean --workspace`:

  | Crate | Line coverage |
  | --- | ---: |
  | `stackpress-lib` | 92.96% |
  | `stackpress-server` | 92.62% |
  | `stackpress-cli` | 96.32% |
  | `stackpress-idea-parser` | 94.44% |
  | `stackpress-idea-transformer` | 96.38% |
  | `stackpress-idea` | 100% |
  | `stackpress-example-profile-codegen` | 95.27% |
  | `stackpress-example-generated-profile` | 81.31% |
  | `stackpress-example-capability-host` | 100% |

  The generated-app figure counts its generated module under the coordinator's example regex. The app's Rust is byte-unchanged since the implementer's 94.44%.
- **Limits carried forward:**
  - cooperative single-writer ownership, with no crash-atomic or power-loss claim;
  - non-UTF-8 native filenames are unprovable on this APFS host;
  - Linux, Windows and WASM are unverified;
  - the Profile-only demo generator is not a generic schema mapper;
  - template engine deferred (G21).

## Acceptance notes

Library and generator work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
