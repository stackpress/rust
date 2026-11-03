# 00025: engine and schema/transformer/Profile review rounds

Task summary: complete the three distinct test-verified rounds and dedicated
safety review released after [M1](00025-template-m1.md). Current `trial/3`, base
`62768feafeb40b13663751b288c62c83398dd843`, 2026-11-01. Acceptance criteria:
none. This receipt covers lib engine, schema emitters, transformer aggregates and
Profile generator; SQL/admin receipts and integrated gates are coordinator-owned.

## Implementation steps and notes

Intended behavior: the frozen [tiny engine contract](00025-template-design.md),
unchanged public generation APIs/policy/error layers and exact final rustfmt
artifact bytes. All edits remained private and inside the agreed ownership lane.
No third-party dependency, lockfile, generated root, context, listener, installation,
commit, push or unrelated benchmark record was changed by these rounds.

### Round 1: logic, decisions and test boundaries

Reviewed parser block matching/terminators, strict path/helper argument handling,
borrowed data accounting, lazy branches/empty loops, helper registration snapshots,
render counters/scopes and error locations. Reviewed schema branch/policy order,
field/type wrappers, defaults/host providers, projection/patch/query selection,
module ownership/insertion order and Profile's finite mapping/compiled selection.

No further control-flow defect was found. The exhaustive parser/renderer matches
remain clearer than a new evaluation abstraction; no ceremonial behavior edit was
made. M1 had already corrected the boundary fixture to distinguish raw aggregate,
formatted output and callback-template failures. A fresh focused baseline passed
**23 tests** (`05e357594a36`): all 19 engine tests, schema formatted parity and
real boundary, exact module bytes/order and Profile repeat/removal.

### Round 2: responsibility and abstraction

- `schema/codegen/emit.rs::emit_public` formerly mixed field type/projection,
  public input and model-only key/patch/query preparation. Extracted named
  `public_fields`, `public_inputs`, and `model_operations` phases, with a private
  `ModelOperations` value for the model-only fragments. All fallible calls retain
  their original order, including the separate type-then-projection passes;
  template keys, output bytes and insertion sequence remain unchanged.
- `template/render.rs` now groups a visited loop's borrowed body, parent and
  aliases in private `LoopBody`. Per-entry values/keys/positions remain local.
  This removes nine positional arguments and the Clippy suppression while
  preserving borrowed scopes, counters, object order and immutable compiled state.
  No parser representation or public API was changed.
- Transformer/Profile ownership and language boundaries were already cohesive;
  no additional abstraction or trait was justified.

Tests: all four affected crates, all targets, schema `codegen,providers`:
**201 passed, 12 preexisting skipped**, including actual independent Profile
consumer generation/compilation/removal (`a6fd8a1b4f26`). The coordinator approved
the exact proposal before edits; admin had captured its immutable 1,033-input
snapshot identity `accc65169a98cb2d25dd68907720feb2bfa7c0909425b712f7d2b9aef5141f60`.

### Round 3: consistency, Rustdoc, comments and source readability

Added local explanations for scope lifetime/alias ownership, per-render counters,
valid loop positions, virtual members, type-before-projection error precedence,
Input/Patch inner required types, shared patch policy selection and ordered
fallible phases. New private helpers/state have purpose/ownership documentation.
The projection expression asset is now multiline Rust rather than one compressed
conditional line; it retains actual structural branches and inert fragments.

The fresh **23-test** focused proof passed (`650f4066e4cd`), including final
formatted fingerprints and the real raw/formatted boundary. Strict Clippy passed
(`3ea77e74e9ed`), **287 doctests passed** (`23eed12cbeee`, including 271 lib,
3 schema, 13 transformer; Profile has none), and warning-clean Rustdoc passed
(`5996b7ac8902`). The final owned formatting check passed (`8b38e58b02f9`).

### Safety and soundness round

No further production change was required. Rechecked UTF-8 slice boundaries,
bounded recursive descent/drop depth, checked counters/append arithmetic,
borrowed root validation before cloning pair strings, aliases without root
mutation, exact nearest-block matching and independent concurrent render state.
Only selected branches resolve values/call helpers; every branch still compiles.
Inserted strings, pre-rendered fragments and helper results are never reparsed.

An independent disposable consumer normalized an actual schema with string and
Object defaults containing quotes, a source-injection-looking `compile_error!`,
backslashes/newline, NUL/ESC, Unicode, and both template delimiter pairs. The real
registry/emitter/writer generated Rust, which was compiled and executed. Its
`prepare()` and public `project()` values matched the exact expected JSON. This
proves Rust literal encoding and inert insertion through actual generated code,
independently of expected-source string construction.

Safety records: offline standalone lock `696ccddbd009`, actual generation
`1bda3066e87a`, compiled value roundtrip **1 passed** `5debdae3563e`. All 75
external `(name,version,source,checksum)` tuples in that lock are a subset of the
root lock's 699 external tuples; no fetch/update occurred. Root lock stayed at
SHA-256 `95628fcee231a400a99165c92350261829b79ca12c02f0f49484128a79c46a51`.
The independent source and generated tree remain in
`/private/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/stackpress-template-safety-b6d4vj28`.

| Risk | Observable proof / boundary |
| --- | --- |
| Wrong nested same-name/else association | Repeated nested conditions/loops and malformed terminator matrix |
| Unselected branch lookup/effect | Missing paths/custom helper counter and empty array/object loops |
| Scope or shared render-state leakage | Parent/root/alias metadata plus concurrent and failed render reuse |
| Unbounded work/bytes/depth | Exact UTF-8/data/output/iteration/step thresholds; external 10-second nesting watchdog |
| Rust syntax injection / recursive expansion | External literal-brace fixture plus actual compiled schema default/projection consumer |
| Source cap or error-layer regression | Real >95%-cap managed install, raw/formatted failures and concrete asset Limit cause |
| Ownership/order/removal regression | Interleaved partition byte test, empty manual aggregate, managed-writer tests and Profile consumers |
| Privacy/policy drift | Existing private defaults/roles projection tests and frozen host-default artifacts |

Bounds concern engine-owned source/data/output length and defined work counters,
not process allocation or elapsed time. Trusted compiled helpers can allocate,
panic or perform effects; no sandbox claim or arbitrary source evaluation is
introduced. Existing framework normalization/validation and writer caps remain
the relevant generator/language boundaries.

## Final coverage and verification process

All Cargo/proof commands are benchmark-recorded, locked, jobs 2, with absolute
warm `target/template-refactor`, sccache/20G cache, incremental off, dev debug 0.
Disposable consumers use their own absolute child target. No additional cold
workspace build or repeated cleanup was introduced.

Final all-four-crate instrumented tests passed **201, with 12 preexisting ignored**
(`51c37969f939`). [Main report](../../../resources/template-refactor/final-coverage.json)
excludes only test filenames with `/tests/|/tests[.]rs$`. Each package's entire
production file scope is counted, with inline test bodies audited separately.
Profile needs [report-only inclusion](../../../resources/template-refactor/final-profile-coverage.json)
using `--no-default-ignore-filename-regex` (`8910cfe601f2`), because cargo-llvm-cov's
default regex excludes all workspace `examples/` paths. Its subtotal uses all
three Profile source files; dependency rows are not attributed to that package.

| Production scope | Final covered / executable lines | Result |
| --- | --- | --- |
| Lib raw, including old inline tests | 4,157 / 4,423 | 93.99% raw |
| Lib conservative production lower bound | 4,104 / 4,423 | **92.79%** |
| New template engine, exact subtotal | 700 / 711 | **98.45%** |
| Schema, codegen + providers, exact | 3,206 / 3,846 | **83.36%** |
| Migrated schema emitter files, exact subtotal | 489 / 568 | 86.09% |
| Transformer, exact | 2,022 / 2,162 | **93.52%** |
| Profile raw, including old inline tests | 174 / 194 | 89.69% raw |
| Profile conservative production lower bound | 150 / 194 | **77.32%** |

Lib lower bound subtracts all 53 physical lines from old inline `error.rs` and
`queue.rs` test modules from covered numerator, retaining the entire denominator.
Profile similarly subtracts all 24 physical `lib.rs:122-145` lines. Actual
production ratios can only be higher. Schema/transformer test modules are separate
excluded files (including the transformer's `#[path]` watchdog). Neither report
has `/tests/` or `/tests.rs` source rows. No production source was excluded or
moved to manipulate coverage. All changed crates exceed the 70% development floor.

Source assets have no executable Rust coverage denominator; their behavior is
proved through compiled rendering, full fingerprints, branch matrices and real
consumers. Remaining engine misses include defensive compiled-state and error
format/span branches. These native results do not claim browser/WASM/Tauri or
live database acceptance; coordinator lanes own their distinct integrated proofs.

## Documentation and source-emitter audit

The chrisai-docing pass covers the lib API/guide, schema registration/template
maintenance section, module guide and Profile guide. Source claims were reread;
mechanical checks found no heading/fence/blank-spacing/relative-link defects in
the changed pages. All five documented API examples execute against final engine
source in the existing disposable consumer (`8ec1d76fc73c`). The combined
`git diff --check` for owned paths passes.

The assigned emitter scopes contain no remaining `push_str` source assembly.
Remaining `format!` sites have explicit non-template responsibilities:
five output/lookup paths, one provider registry key, two authored asset error
contexts, and one primitive Rust string-literal encoder. Runtime logs/errors,
SQL parameter formatting, lexical naming and unrelated lanes were not expanded
into this scope. No generic `{{source}}` wrapper or post-render compactor exists.

## Benchmark and retained target receipt

This implementer's rounds/safety/final gates comprise **15 attempts, all passed**,
**290.003654 s** command time and **76.36 s** observed Cargo stages. Record labels
also include concurrent coordinator `independent` gates, workspace formatting and
190-artifact integrity; those records are not attributed to this implementer.
Command time and Cargo stage time are separate measurements.

Shared checkpoint counters: M1 **2355 -> 2410** attempts, **1053 -> 1070**
compilation-emitting attempts, cold successful workspace builds **88 -> 88**.
The shared deltas include other lanes and still-running coordinator gates.
Own earlier milestones remain separately counted in M0/M1 receipts.

Latest/peak retained owned targets: **6,568,452 KiB = 6.2642 GiB**, below 12 GiB.
Root task target is 5,500,412 KiB, including separate `doc-consumer` and
`schema-safety-consumer` children. Earlier rebased session/server/lib/sqlite/app
paths remain 373,724 / 112,376 / 107,668 / 220,596 / 253,676 KiB. Warm targets and
owned disposable sources remain available for review; no unrelated target or
owner's two dirty benchmark records were removed or rewritten.

## Verification notes and handoff

Production final-stable notice was delivered after R3 parity, enabling coordinator
workspace tests/coverage/Clippy and generated-190 comparison in parallel. No
further production edit is planned. Agent Workspace validation retains the 11
historical missing screenshot-link errors and 33 warnings; no new task 00025
broken link exists. The 436-line approved design accounts for one coherence warning.
Inherited errors are never a clean validation claim. No unresolved
owner behavior decision remains; final integrated acceptance/status is the
coordinator's responsibility.
