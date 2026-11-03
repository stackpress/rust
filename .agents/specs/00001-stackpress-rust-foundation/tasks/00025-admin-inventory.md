# Task 00025 admin/source-boundary inventory

Chronological lane checkpoints, 2026-11-01, current `trial/3` checkout. Parent:
[tiny template generation](00025-tiny-template-generation.md); load this note before
releasing the admin migration lane. The coordinator approved the inventory and froze
the engine design/API; its current message overrides that design file's older Draft
header. At the preparation checkpoint only new structural assets and this view-model
note were released; subsequent implementation and round releases are recorded below.

## Authority and write boundary

The initial lane brief authorized this inventory note only. Preparation now also owns
new `.rs.tpl` files in the admin codegen directory, without Rust emitter wiring.
Eventual scope
is `crates/framework/framework-admin/src/codegen/**`, new structural templates there,
the admin manifest if a codegen-only lib dependency is needed, and owned tests/docs.
Assess API Rust emitters if present; the inspected API files emit no Rust (below).
Do not edit schema, SQL, engine, transformer, examples, generated roots, task status,
shared manifests/lockfile, or other lanes. No external dependencies, subagents,
commits, pushes, listeners, builds, target cleanup, or regeneration during this audit.

Read the context index, project/source authority, Rust style/quality, the installed
coding skill under the explicit Rust adaptation, and the Rust programming,
coordinated-crate-development, fast-build-and-test, build-benchmarks, Agent File
creation and spec-task workflows. This is planning/evidence, not promoted context.
The directly authorized checkpoint does not change the program's Not Frozen status.

## Actual production Rust emission inventory

Paths below are relative to `crates/framework/framework-admin/src/codegen/`.
Line ranges identify the inspected pre-migration source, not permanent API anchors.

| Owner and source range | Actual assembly to migrate | Contract retained outside templates |
| --- | --- | --- |
| `emit.rs:117-177`, `model_source` | Conditional import blocks, escaped descriptor decoder, five independent role function structures, typed versus draft input, control sequences, optional table. | Bounded descriptor serialization, role admission/selection, public type derivation. |
| `emit.rs:181-205`, `control_source` | Field/draft lookup block, `RoleContext`, statically resolved component invocation, labeled list/view versus plain control wrapper. | Import validation, checked module/symbol tokens, role spelling, Rust literal encoding of wire names. |
| `table.rs:8-53`, `source` | Optional typed table function, row validation/iteration, selected cell adapters, header list and caption. | Selection/admission, label precedence, public type and Rust string literal encoding. |
| `pages.rs:17-168`, `emit_pages` | Pure/host imports, operation registrar, direct routes, plugin registrar inclusion, duplicated Route-phase mappings, import-existence match arms. | Selection order, duplicate/budget/projection/key checks, missing-model omission, soft-remove choice, reserved native module naming, output ownership/publication. |
| `pages.rs:172-285`, `model_source` | Per-model spec/input/import-validation/query/form adapters, locked-field locals, key-update guard, optional filters/table/detail and unavailable fallbacks. | Serialized approved `PageSpec`, typed DTO names, field admission, key membership and capability flags. |
| `pages.rs:288-359`, `dispatch` | `MODELS`, identity match, eight adapter function signatures/bodies/dispatch arms, empty-selection variants and unavailable helper. Includes the `format!("_:{}", ...)` parameter assembly at line 345. | Finite authored adapter signatures, ordered module/name inventory, checked identifiers and literal encoding. |
| `pages_host.rs.in:1-64` | Existing readable fixed source for `Mount`, singleton service and ordinary `register`; use as an actual template partial in the nonempty host structure. | Existing Config/Listen/Route lifecycle, SQL-before-admin requirements, service/contribution ownership and no implicit database/listener startup. |

`emit_roles` prepares all models before publishing files. `emit_pages` renders every
selected model before its final two writes. Preserve those boundaries and failure
ordering; a template error must not introduce partially contributed model output.
The transformer remains responsible for final aggregate modules, rustfmt, marked
root installation/replacement and managed manifests; this lane does not replace it.

## Excluded formatting and API assessment

- `emit.rs:33` formats `ui/{module}.rs`, an output path; preserve it.
- `emit.rs:209-222` computes checked `Public<Model>` identifiers, including `Self`
  handling. Its string mutation/formatting is semantic name derivation, not Rust
  syntax assembly; preserve it.
- `pages.rs:90-94,152-156` computes SQL's `_schema`/`_operations` identifier escape;
  preserve it. The plural store type name at line 157 is also semantic spelling.
- `bindings.rs:117-138` formats diagnostic role-path prefixes. `catalog.rs:41` and
  `formatted.rs:48` format registry keys. These files validate/describe metadata;
  they do not construct Rust source. `codegen/mod.rs` registers ordinary transforms.
- `framework-api/src/codegen.rs:135-184` assembles a `model Profile` Idea declaration
  and passes it directly to `stackpress_idea_parser::parse`; it writes no `.rs`
  artifact. Its earlier `api.source.{name}` strings are component registry keys.
  `codegen/source_compat.rs:17-69` adapts normalized query/relation metadata; its
  `format!` calls build query paths. Exclude both API files from this Rust-emitter
  migration. A broader scan of API `src` found runtime SQL/HTML/URL/diagnostic
  formatting, without another Rust writer. Normalized API output may still be
  exercised by shared emitters. The coordinator independently confirmed exclusion.
- Rust `format!("{mount}...")` calls inside generated route functions, table row
  hex/instance formatting and field context IDs are runtime code to preserve as
  literal template text. Moving the enclosing source into templates does not
  authorize replacing these runtime formatters.
- Diagnostics, serialized JSON values, runtime UI formatting and test fixture
  construction retain their owners. Rust Debug-string literal formatting remains
  an emitter responsibility; it must not become JSON/HTML escaping in the engine.

## Proposed structural templates and data preparation

Prefer readable files in `codegen/templates/`, loaded with `include_str!`, with
normal Rust braces, comments, indentation and visibly complete source structures.
The coordinator-frozen design uses original-style grammar, strict lookups and inert
insertion, with custom `[% %]` delimiters suitable for Rust. The grammar below uses
boolean `#?:path`, inverse `#!:path`, ordered `#@:path alias` loops and matching end
tags, without helpers, engine partials or expression evaluation.

1. `roles.rs.tpl`: descriptor imports/decoder and a loop over selected role records.
   Each role exposes `name`, enum spelling, optional public type, typed-input flag,
   and prepared control data. Absent roles emit no function, in today's fixed order.
2. `control.rs.tpl`: a complete field-context block plus labeled/plain component
   branches. Data contains checked component module/symbol, Rust wire-name literal,
   role name/variant and display-label flag. Both role and page forms reuse it.
3. `table.rs.tpl`: the complete optional table function, cell-control loop, header
   literal loop and caption. Preserve row limits, key identity encoding and order.
4. `pages.rs.tpl` plus `dispatch.rs.tpl`: shared imports, model adapters,
   codec helper and closed dispatch. Model data includes checked type names, spec
   literal, controls with key-guard flags, and filter/list/view capability flags.
   Render key guards and locked-field locals as blocks; remove the `MODEL` string
   replacement. Dispatch parameters become records with name/type/unused state;
   do not keep a formatter that builds whole argument declarations.
5. `pages_host.rs.tpl` plus `routes.rs.tpl`: operation list, routes, optional plugin
   registrar and import-existence dispatch. Model data includes module/literal,
   native store module/type and soft-remove flag. A finite route inventory carries
   method, action/path data and authored handler. Render it twice with authored
   function name, receiver type and doc text; replace the current chained `.replace`
   of emitted Rust. Keep public `routes` and private `mount_routes` equivalent.
6. `pages_host_registration.rs.tpl`: preserve the existing `.rs.in` registration
   structure as a fixed partial. Emit only for nonempty selections. Empty selections
   retain warning-clean direct helpers and unavailable import/dispatch functions.

These eight files are now drafted. The engine has no includes/partials; the future
Rust emitter composes rendered templates at named structural boundaries. No template may
just substitute a whole legacy formatter's `source`; loops/conditions and Rust
declarations must live in templates. Preparation produces checked data and flags,
not import snippets, operation suffix snippets, or whole preformatted functions.
Use one small private renderer/error adapter if needed: retain a concrete engine
cause under existing schema/transform error contexts and avoid exposing label data.
The released API must support ordered record/array iteration, nested boolean blocks,
literal Rust/Leptos braces and inert inserted strings. Verify template whitespace
and runtime format strings against final rustfmt bytes. Choose finite render limits
that accommodate existing descriptor/model/field budgets and Rust literal expansion;
do not silently reduce accepted generator inputs with a smaller engine default.

Admin currently has no direct lib dependency. Prefer optional `stackpress-lib`
activated by admin's existing `codegen` feature. The module is already feature-gated
in `src/lib.rs:17-18`; no gate edit appears necessary. Preserve default/hydrate/CSR
dependency graphs and independent library usage. The coordinator owns any required
lockfile synchronization; this lane must not edit it implicitly.

## Compatibility and risk proof plan after API release

- Baseline both complete marked roots, `examples/generated-admin/shared/src/generated`
  and `shared/src/generated-format`, before coordinated regeneration. Each currently
  has eight `ui/*.rs` files; five model/fieldset role files, `pages`, `pages_host` and
  the transformer-owned `mod`. Compare full file sets, exact final rustfmt bytes and
  writer manifests/hashes, alongside the coordinator's expanded 190-file baseline.
  The JSON map was read and its 190 entries confirmed at
  `/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/stackpress-template-baseline-19tcw996/generated-hashes-expanded.json`. Do not
  hand-edit consumers. Use disposable output roots for lane proofs; only the
  coordinator installs tracked generated output.
- Preserve role source preflight, import/prop diagnostics, descriptor revision,
  role absence, sensitive-field omission, fixed prop precedence, unsupported native
  controls, typed/public names and reserved store module mappings. Existing coverage:
  `framework-admin/tests/{roles,catalog,pages,presentation}.rs`. Add migration-specific
  cases for delimiter-looking labels/props, quotes, slashes, newlines, Unicode and
  closing tags; prove inserted values stay inert and compile as Rust literals.
- Preserve empty/no-role/removed models and soft versus hard delete, every role,
  key-update guards, related locked controls, table labels/captions and fieldsets.
  Compare complete prepared output for representative fixtures, not only substrings.
- `generated-admin-tool/tests/generation.rs` already proves absent-root generation,
  repeat/check, plugin replacement, disable restoring complete bytes, rename/removal,
  invalid selections and failed regeneration preserving prior output. Reuse it.
  `proof.py compile --suite format` uses disposable real consumers to prove obsolete
  imports fail after rename, hostile inputs compile, no-role/empty output is warning
  clean, and formatted generation repeats deterministically. Preserve the separate
  consumer targets and final external lockfile tuple comparison.
- Generated shared tests exercise SSR, public DTOs, role components, page adapters,
  table keys, exact draft values and transfer surfaces. Native web tests exercise
  actual SQLite CRUD/transfer; `web/tests/main/registration.rs` proves plugin/direct
  operation/route parity, startup failures, duplicates and omitted integration.
- Feature gates: admin `codegen,ssr`, then `codegen,ssr,format`; independent default
  admin/facade consumers; affected generated tool/shared/web tests; SSR separately
  from WASM hydrate and CSR, each with and without format. Review normal-edge trees
  for browser exclusion of parser/transformer/server/SQL and new codegen dependencies.
- Rebuild matching native/WASM artifacts before browser proof. Existing `proof.py`
  roles/CRUD/format/transfer/plugin suites retain both mounts, wide/narrow layouts,
  hydrate/CSR, plugin/disabled states, controlled inputs, labels/IDs and no browser
  errors. Use their finite interception/in-process harnesses; this lane starts no
  listener. The separate TCP/listener watchdog is coordinator-owned.

## Build, review and checkpoint receipt

After explicit implementation release, use only `target/template-admin`, with
consumer/WASM/coverage subdirectories inside its aggregate 12 GiB budget. Record
initial `cargo clean` and `cargo llvm-cov clean --workspace` there before the first
build, never against the owner's other targets. Keep it warm afterward, use the
workflow's sccache/nextest environments and bounded jobs, and record all clean,
build/test/check/lint/doc/coverage/proof commands through `build-benchmark.py`.
Wrap an outer proof once; nested Cargo calls are not separately benchmarked.

Set `CARGO_TARGET_DIR` and `CARGO_LLVM_COV_TARGET_DIR` explicitly; admin proof output
also needs a unique `ADMIN_PROOF_ROOT`. Existing `proofs/runtime.py` has a stricter
8 GiB guard; respect it when reusing the script, and report it separately from the
lane cap rather than editing out-of-scope proof scripts. Reused proofs perform
package-scoped cleans for generated feature variants; all must resolve inside the
lane target. Report aggregate peak, command wall time and observed Cargo-stage
time separately. No install or dependency fetch is authorized by this audit.

Stop after working migration tests/coverage for coordinator review. Then complete
three distinct test-verified rounds (complexity; responsibility/abstraction;
consistency/Rustdoc/comments), a dedicated safety review, docs and released gate
checks. Target at least 70% line coverage in each changed crate's declared scope;
generated compilation, browser evidence and native coverage stay separate.

Audit verification: source/manifests/tests/proof drivers read; no Cargo, codegen or
browser command run. Before this note, Agent Workspace validation exited 1 with
11 missing historical screenshot links and 29 existing warnings. Preserve those
unrelated findings. Validation after creation reported the same 11 errors and 29
warnings, with no added/removed diagnostics. The coordinator should link this
inventory from the parent task; later freeze/asset release is recorded below.

## Frozen-grammar preparation and concrete view models

Current release: new assets only; no production Rust, manifest, build, cleanup or
generated-root changes. `TemplateEngine::new(TemplateOptions { delimiters:
["[%".into(), "%]".into()], ..Default::default() })?` supplies the future local
engine; use `compile(...)?`/`render(&serde_json::Value)` after independent M0 proof.
The eight assets contain real functions/declarations; none wraps legacy source.

| Template | Minimal render object and composition |
| --- | --- |
| `control.rs.tpl` | `wire_literal`, `role_name`, `role_variant`, `component_module`, `component_symbol`, Boolean `labeled`. Import/name validation and literal escaping precede rendering. This is the only field-context source body. |
| `roles.rs.tpl` | Boolean `rendering`, `fields_literal`, ordered `roles`, Boolean `has_table`, optional rendered `table`. Role records contain `name`, `variant`, Boolean `typed_input`, checked `public_type` only when typed, and ordered `controls` strings rendered with `control.rs.tpl`. |
| `table.rs.tpl` | Checked `public_type`, ordered rendered `controls`, ordered `headers: [{literal}]`, `caption_literal`. Labels use current prop-over-field precedence. Invoke only when list roles exist. |
| `pages.rs.tpl` | Boolean `has_controls`, ordered `models`, rendered `dispatch`. Each model has checked `module`, `type_name`, `public_type`, `spec_literal`, Boolean `has_controls/has_filter/has_list/has_view`, ordered `controls: [{key_guard: bool, body}]`. `body` comes only from the shared control template. |
| `dispatch.rs.tpl` | Boolean `has_models`, ordered `models: [{module,module_literal,name_literal}]`, eight ordered `adapters: [{name,parameters:[{name,ty}],result,arguments}]`. Parameters/result are finite compiler-authored type atoms; argument names are authored tokens, not generated call strings. No split/rejoin signature formatter remains. |
| `routes.rs.tpl` | Booleans `private` and `has_models`, ordered `models: [{module,page_actions,import_actions,api_actions}]`. Action arrays are the existing finite selections in existing order; semantic filtering removes soft-only actions for hard-delete models. Both direct and private functions render the same records with only `private` changed. |
| `pages_host_registration.rs.tpl` | Empty object. Its complete fixed registrar is byte-identical to the prior `pages_host.rs.in`; include its rendered text only for nonempty selections. No source rewriting. |
| `pages_host.rs.tpl` | Boolean `has_models`, ordered `models: [{module_literal,native_module,store_type,soft_remove:bool}]`, rendered `direct_routes`, and selected-only `registration`/`mounted_routes`. Operations and import-existence branches are authored in this template. |

All collections are arrays, preserving existing selection/field/role order. Every
condition receives a JSON Boolean, including `@first` metadata; no array/string
truthiness is used. Nested aliases stay explicit, with `@root.has_models` used in
adapter loops. `public_type`, table/registration/private-route fragments and other
branch-only keys need exist only in the branch that reads them. Empty loops perform
no body lookups. Rendered child text is inserted once, so labels containing `[%`
or closing tags cannot acquire template meaning. No whole formatter output is used.

Whitespace: preserve generated comments and runtime `format!` syntax, including
macro token spacing that rustfmt leaves intact. Retain the page-model leading blank
line, the blank after form locals and the conditional leading blank before a list
adapter. Full byte parity remains unverified until engine/consumer proof release;
asset preparation does not claim equivalent generated artifacts.

### Encoded data and unchanged output caps

`to_bounded_value` already admits up to 1 MiB of encoded descriptor data.
`plan/mod.rs:159-163` caps aggregate planned source at 1 MiB;
`output/format.rs:31-50` separately caps aggregate formatted artifacts and each
formatter stdout at 1 MiB. These existing limits and owners must remain unchanged.

The engine's default 1 MiB *data* limit cannot be reused blindly: a Rust string
literal containing descriptor JSON is then JSON-encoded again as a view-model
string. Count quotes, backslashes, control escapes, keys, commas, brackets, record
flags and rendered child fragments. Do not compare only raw descriptor/literal
lengths or route source through schema's one-MiB `to_bounded_value` a second time.

Proposed local render options: retain the 1 MiB output cap, use a finite 8 MiB
compact encoded-data cap for these prepared contexts, and keep default depth/work
limits unless a released proof demonstrates a needed task-local adjustment.
The data headroom covers at most six JSON bytes per string byte plus bounded record
overhead. Minimize each call to the exact keys above; never retain the full UiSchema
beside its encoded literal or repeat all role/model contexts in each child. Check
the actual compact encoded size with bounded counting, without an uncapped
serialization buffer. The 128-model/fieldset, 128-field and five-role bounds remain.

After M0, prove the bound against both complete roots and large quote/backslash/
control-heavy fixtures near the existing accepted output limit, checking descriptor
encoding, rendered child/context overhead and final writer bytes separately. Any
unexpected refusal or whitespace-driven plan growth must be solved within this
lane's data/asset preparation, without increasing global caps or dropping cases.
API freeze is complete; Rust emitter wiring/manifests/builds await the next release.

Preparation receipt: eight assets, 175 statically balanced grammar tags; the fixed
registrar asset matches the old `.rs.in` bytes exactly. This read-only tag inspection
does not prove the engine parser or generated Rust. No engine rendering/compilation,
Cargo, regeneration, listener, cleanup or repeated KB validator run occurred during
asset preparation. Keep this cohesive inventory intact for lane retrieval.

## M1 implementation and stop receipt

2026-11-01: coordinator released admin M1 after independent engine proof. The first
edit added optional local `stackpress-lib` to the admin manifest, activated only by
`codegen`; the coordinator was notified immediately. No Cargo/proof command ran
until the separate shared-lock release. The coordinator alone updated Cargo.lock.
The checkpoint descriptions above retain the earlier audit/preparation boundaries;
this section records the current implemented state and evidence.

`emit.rs`, `table.rs`, `pages.rs` now prepare checked data and render the eight
structural assets. Both roles and pages reuse `control.rs.tpl`; direct/plugin
mounting reuse `routes.rs.tpl`. The old `pages_host.rs.in` was replaced by its
byte-identical registrar template. No legacy formatter source is substituted.
`templates.rs` owns Rust Debug literal encoding and engine options: `[% %]`, 8 MiB
compact encoded data, unchanged 1 MiB rendered-output cap and default work/depth
limits. Every error names its authored static asset and retains concrete engine
kind/span; rejected data never enters the error context.

The admin-local `source_error` adapter preserves the direct emitter's Limit cause
when a template reaches the source limit before OutputPlan. Registry keeps its
established outer Callback wrapper. Only concrete template Limit causes use this
mapping; ordinary schema Limit errors retain their original Callback policy. No
shared schema/registry code or writer/artifact cap changed.

Remaining production string operations are intentional: output paths; checked
public/native/plural identifiers; finite Role enum spelling; Rust string literals;
catalog keys and diagnostic prefixes. Runtime `format!` inside templates remains
unchanged. API files were audited and excluded; no edits there.

| M1 evidence | Result | Recorder ID suffix |
| --- | --- | --- |
| Required initial absolute-target clean / llvm-cov clean | Both passed, no owner target touched | `7652562c6d2c`, `b66f4645fa1d` |
| Native `codegen,ssr,format` admin nextest, all targets | 34 passed, 0 skipped | `1ce2a3aae48f` |
| Coordinator independent same scoped suite | 34 passed, 0 skipped | `7c4846cad34e` (coordinator) |
| Actual tool lifecycle suite with format | 5 passed: absent/repeat/check, plugin/disable restore, rename/remove, failed regeneration and registration | `8c8b996fab5a` |
| Formatted disposable root generation | Passed at `/private/tmp/stackpress-template-admin-m1/formatted` | `f9788a936719` |
| Expanded-baseline exact UI SHA256 comparison | Default 8/8 and formatted 8/8 Rust files equal, including pages/host | `d8aad87977bd` |
| Instrumented native admin suite | 34 passed, 0 skipped | `d85e330ac23b` |
| Production-file coverage scope and 70% gate | Conservative 84.8256%, 2359/2781 | `6882eb46c08f` |
| Final owned-file rustfmt | Passed; subsequent tracked diff whitespace check passed | `0011eca8d821` |

The default comparison reads the coordinator's disposable
`/private/tmp/stackpress-template-m1-compat-foe9emja/admin-generated` output; the
formatted comparison reads this lane's disposable output. Both use the expanded
190-entry baseline recorded above. No checked-in generated root was written.

`template_codegen.rs` parses actual normalized JSON, normalizes the schema and
executes the real transform/writer. A 94,000-repeat quote/backslash/newline label
retains more than 1,000,000 planned bytes and passes real rustfmt under the unchanged
1 MiB artifact cap. At 100,000 repeats the concrete emitter cause is Limit under
the unchanged registry wrapper. These real schema tests complement the unit data
encoding/output-boundary, inert insertion and selected-only lookup tests.

Coverage files: absolute owned target plus `coverage/admin-m1.json` and
`coverage/admin-m1-production.json`. Declared scope is native `codegen,ssr,format`
admin implementation, 19 measured `src` files. The regex
`/tests/|/tests[.]rs$|_tests[.]rs$` excludes test files, and the scope calculation
verifies no test path survives. Raw coverage is 2444/2781. Deducting all 85 physical
lines in the two existing inline test modules from the covered numerator only
produces the conservative production lower bound 2359/2781 = 84.8256%.
Generated-source byte checks and generic engine coverage are separate evidence.

All 14 lane records are under `.agents/resources/build-benchmarks/runs/`, labeled
`template-admin M1`: 11 passed and 3 failed investigation runs. Accumulated command
time is 266.891830 seconds; observed Cargo-stage subtotal is 226.21 seconds. The
lane has eight compilation-emitting attempts and zero cold workspace rebuilds.
The failures were the initial Idea-string fixture (changed to supported normalized JSON)
and two assertions targeting the outer registry category (corrected to its concrete
emitter cause). Historical failures remain recorded; final suites pass. A sandbox
write refusal before the first recorder start created no attempt; the authorized
cleanup was rerun with escalation for the required benchmark store.

Environment: `RUSTC_WRAPPER=sccache`, shared 20G cache setting, jobs 2,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`; all Cargo/proof commands locked
and offline. Absolute target is
`/Users/cblanquera/server/projects/stackpress/rust/target/template-admin`, with
coverage inside its `coverage` child. Observed aggregate peak: 2,193,552 KiB
(2.092 GiB), below 12 GiB; no cleanup after initial setup. Global recorder attempts
changed 2279 to 2315 while other lanes ran, so that delta is not attributed to this
lane; successful cold workspace rebuild counter stayed 88 (no lane workspace run).

M1 stops here before rounds. Coordinator owns integrated workspace, consumer,
feature/browser gates and generated-root publication after stable lanes. Three
rounds, dedicated safety review and docs await release; compiler/template reuse is
a deferred responsibility-round DX observation, not an unresolved M1 defect.
Temporary schema-lane unused-helper warnings observed during active edits were
outside this lane; no installs, commits, pushes, listeners or subagents were used.

## Released rounds and safety review

After coordinator M1 approval, the lane completed three separate rounds. The
approved rounds release supersedes the historical preparation/M1 stops above.

| Round | Reviewed risk and justified result | Scoped test receipt |
| --- | --- | --- |
| Complexity and branches | Read preflight/publication ordering, role admission/order, key guards, native store selection, empty/omitted arms and selected-only lookups. No behavior-preserving simplification justified an edit. | 34 passed, 0 skipped; `406b917f080d` |
| Responsibility and abstraction | Introduced an emitter-local lazy `Renderer` cache and finite `Asset` identities. A used asset compiles once; eight is the maximum cache size. Shared controls/routes remain structural templates. Added render/failure/render reuse proof with fresh contexts. No global cache or state. | 35 passed, 0 skipped; `0b101bd2d5b8` |
| Consistency, Rustdoc and comments | Unified the error adapter name, documented concrete template errors, limits and renderer ownership, and wrote the generator guide/API integration page. Owned rustfmt passed `5dc0730db73c`. | 35 passed, 0 skipped; `51836874d3a1` |
| Dedicated safety/source boundary | Reviewed Rust Debug literals, checked name/import tokens, inert insertion, selected branches, static asset/error cause/span, bounded local cache and unchanged output budgets. No additional edit was justified. | 35 passed, 0 skipped; `293c6d053ef4` |

The reuse regression renders one control, rejects incomplete data, then renders a
different hostile-looking wire name through the same compiled asset. Neither data
nor failed lookups carry into the next render. Generic engine compilation/render
coverage remains the engine lane's responsibility. Strict owned Clippy with
`codegen,ssr,format`, all targets and `-D warnings` passed `3b8623a60f91`.

The documentation work uses the installed `chrisai-docing` skill: the new
`docs/guides/change-generated-source.md` explains asset ownership, literal policy,
ordered records, local compilation reuse and independent context/output caps;
`docs/api/codegen.md` contains the complete integration function. Its exact Rust
fence compiles in `target/template-admin/consumers/doc-api` under locked/offline
resolution (`d47b1968b86b`). An earlier locked attempt correctly required a local
lock resolution (`b165bf2a2b9b`); the coordinator authorized one offline metadata
resolution in that disposable consumer only (`8fd73aac4b7c`), which verified 197
packages and external package tuples contained in the approved root lock. No root
lock edit or fetch occurred. All three changed Markdown pages pass the H1,
tagged-fence, heading spacing and relative-link checks.

## Final scoped gates and finite consumer receipt

All released admin gates passed after the last production edit. The coordinator
retains integrated workspace gates and final review; no additional implementation
or generated-root publication is implied by these lane proofs.

| Gate | Result | Benchmark record suffix |
| --- | --- | --- |
| Native selected doctests, `codegen,ssr,format` | 2 passed, 0 ignored | `25807b060b93` |
| Selected Rustdoc, `RUSTDOCFLAGS=-D warnings` | Passed | `ee325c84af34` |
| Default library, no default features | Passed | `d5fe69c370db` |
| Codegen-only library, no SSR/render mode | Passed | `f158093e4544` |
| Final instrumented scoped tests | 35 passed, 0 skipped | `530105ff06a5` |
| Final production coverage scope/gate | Conservative 85.1513%, 2420/2842 | `48e9c3d64f95` |
| Captured native consumers and all finite artifacts | Passed; 26 native subprocesses, including two expected exit-101 compiler rejections | `528f7ae4b754` |
| Final regenerated default UI bytes | 8/8 exact expanded-baseline matches | `6ec734ad7343` |
| Final regenerated formatted UI bytes after plugin disable | 8/8 exact expanded-baseline matches | `703af514d2ed` |
| Default hydrate/CSR package equivalence, four browser graphs and tool boundary | Passed | `f5f638d9156c` |
| Roles browser: base/plugin/disabled, hydrate/CSR, both mounts/layouts | 24/24 passed | `7442b2b33415` |
| Formatted browser: plugin/disabled, hydrate/CSR, both mounts/layouts | 16/16 passed | `3bf892fdc58b` |

Final coverage reports are `coverage/admin-final.json` and
`coverage/admin-final-production.json` under the absolute owned target. Scope is
the same 19 native production `src` files and features as M1: raw 2505/2842, minus
the same 85 physical inline-test lines from the covered numerator only. No test
file survives the exclusion/scope check. The report lists every measured file and
gap; the largest gaps are existing component/native render paths, complemented by
the separate browser matrix. Structural assets and shipped generated Rust have
exact-byte, real consumer/compiler and finite browser verification, rather than
being silently treated as covered Rust implementation lines.

Finite evidence lives at `/private/tmp/stackpress-template-admin-proof`. The
existing `proof.py` helpers captured current source once, then reused that isolated
workspace for compilation and default/formatted variant builds. Its
`snapshot/source-snapshot.json` hashes 1033 compiler/proof inputs at HEAD
`62768feafeb40b13663751b288c62c83398dd843`; snapshot identity is
`accc65169a98cb2d25dd68907720feb2bfa7c0909425b712f7d2b9aef5141f60`.
The coordinator was notified immediately after capture. Owned admin compiler
inputs were subsequently checked against that identity; no later admin edit
changed them. The disposable source and hashed consumer targets were removed by
the existing helper after children were reaped; hashes, logs and artifacts remain.

`compile/compile-proof.json` records actual bootstrap/repeat checks, mixed-revision
and stale-consumer compiler failures, rename, hostile JSON/formatted literal
compilation, omitted roles, complete removal plus strict Clippy, and the full
formatted consumer. Its independent locks contain only approved root package
tuples. Browser artifacts include native SSR and separate hydrate/CSR WASM for
default/plugin/disabled selections, plus formatted plugin/disabled builds. The
formatted builder also proves disabling restores its entire managed root exactly.

Graph reports retain exact default package-set equivalence to the current trial/3
commit and four default/formatted browser normal-edge exclusions. The actual
absence regexes were inspected and positively matched server/SQL and formatter
package samples; their leading codepoints are 92,98, a real word boundary, with
controls retained in `graphs/absence-gate-positive-controls.json`. The tool graph
contains neither generated shared frontend nor application consumer packages.

All 40 browser cases report zero console errors and zero unexpected requests;
roles retain 72 screenshots and formatted controls retain 64. Desktop roles,
mobile rating and mobile formatted-error captures were visually inspected. The
existing installed Playwright module used installed Chrome 156.0.8078.76 through
an ephemeral executable-path adapter because its expected bundled Chromium was
absent. These are finite `.invalid` interceptions/native request subprocesses,
without listeners, installs or shared generated-root writes.

Post-M1: 21 recorded attempts, 20 passed and the one explained consumer-lock
resolution retry failed; accumulated wall time 1092.374208 seconds, observed Cargo
stage subtotal 235.94 seconds. All lane stages: 35 attempts, 31 passed/4 historical
failures, 1359.266038 seconds wall and 462.15 seconds observed Cargo stages. The
recorder observed compilation in 7 post-M1/15 total commands; the captured finite
proof additionally compiled according to its nested logs. Its 582.351566-second
outer record has no Cargo subtotal because helper output is captured, and browser
records likewise have no Cargo stage. No full workspace rebuild was run here.

Global attempt count at closeout measurement is 2409 (other lanes also ran); the
cold workspace rebuild counter remains 88. Observed aggregate target sample peak
was 5,108,836 KiB (4.872 GiB), with the proof's active 8 GiB guard throughout and
the lane's 12 GiB ceiling unchanged. Final retained size is 2,850,540 KiB (2.718
GiB). Benchmark summary is retained with the finite evidence. No extra cleanup,
lockfile edits, commits, pushes, listeners or subagents were introduced. Stop at
the released round/docs/proof review gate.

Owned diff whitespace and changed documentation checks pass. Agent Workspace
validation still exits 1 for the same 11 historical missing temporary screenshot
links in task 00013, with 33 current advisory warnings across shared work. This
note intentionally retains one coherent, sub-500-line inventory-to-proof history
for lane review instead of splitting the sole authorized receipt across files;
its preferred-200-line warning is acknowledged. The full validator output is
retained as `workspace-validation.txt` with the finite evidence.
