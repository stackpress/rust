# Task 00025: SQL emitter inventory, migration and receipt

Audit, template preparation and implementation evidence, 2026-11-01, branch `trial/3`. This note
supports [the tiny template task](00025-tiny-template-generation.md). The coordinator
approved the inventory and froze [the engine design](00025-template-design.md),
then released asset preparation, independently verified M0 and released SQL M1.
M1 was independently approved before the three rounds, safety and docs release.
The initial inventory/preparation history remains below; final evidence follows it.

## Ownership and original checkpoints

- The initial audit changed only this note. The preparation follow-up additionally
  authorized new `.rs.tpl` assets under the owned SQL codegen directory. At that
  stage no emitter, manifest, generated root, task status, benchmark or target changed.
- Released ownership: `crates/framework/framework-sql/src/codegen.rs`, its
  `src/codegen/` modules and new templates, scoped SQL tests/docs, and only the
  `codegen` feature's activation of the existing optional `stackpress-lib` dependency.
- Engine/schema/admin/transformer code and other manifests belong to other lanes.
  Aggregate `mod.rs` emission belongs to the transformer lane.
- The preparation stop ended at the coordinator's M0 release. Cargo commands
  waited for the coordinator's shared-lock synchronization release. The later M1
  stop ended at independent M1 approval. No subagents, commits, pushes or listeners.
  Cargo.lock, generated roots, task status and unrelated dirty files remain unowned.

## Source inventory

All paths below are relative to `crates/framework/framework-sql/src/`; line numbers
refer to the audited pre-migration source. The eight emitter files total 1,471 lines.
Hand classification finds 73 actual `format!` calls: 66 structural source sites and
7 lexical/path sites. The initial audit classified five qualified Rust type paths
as lexical; the frozen design explicitly includes those compositions in templates.
There are 90 `push_str` calls and two source-initializing
`String::from` calls. The `format!` printed inside `operations.rs:236` is generated
runtime event-name formatting, not another emitter-side macro call.

The following inventory includes every structural `format!` site and every
`push_str` site, including nested expression construction and static closing text.

| File / functional group | Structural `format!` lines | `push_str` lines | Template responsibility |
| --- | --- | --- | --- |
| `codegen.rs`: per-model shell, columns, aggregate schema | 74, 117, 125, 230 | 86, 88, 117, 123, 125, 127 | Imports, revision/fingerprint, column array; ordered table/catalog lists and explicit MySQL definition function. `String::from` at 113 is aggregate source. |
| `codegen.rs`: checked DDL builder | 158, 171, 180, 182, 187, 190 | 158, 180, 182, 187, 190, 193, 195 | Store declaration; fields, nullability/length, primary key, indexes/unique flag, FK declarations and MySQL target requirement. |
| `codegen.rs`: expression/type support | 255, 303, 356 | none | Qualified enum type, complete `&Key { member: value }` convenience expression and conditional `.clone()` expression. |
| `codegen/bounds.rs`: private storage guards | 14, 17, 21, 23, 36 | 14, 21, 23, 30, 33, 36, 38 | Required/optional field guards and optional mutable-column patch match. Preserve key/generated exclusions and character counts. |
| `codegen/catalog.rs`: immutable metadata and finite views | 53, 93, 102, 113, 119, 123, 131, 133, 135, 139, 144, 158 | 52, 53, 93, 95, 102, 113, 123, 139, 142, 144, 158, 166 | `StoreSpec`, query/relationship metadata, strict decode/project, trusted/public depth 0-3 views, qualified type paths, cardinality wrappers and recursive projection. |
| `codegen/queries.rs`: typed native reads | 14, 21, 36, 38, 45 | 12, 14, 16, 21, 28, 36, 38, 43, 45 | Filter/sort enums, optional legacy scalar selector and active predicate, full-key/maintenance/search/count/public read functions. |
| `codegen/writes.rs`: prepared mutations and batches | 22, 63, 209 | 11, 22, 24, 26, 38, 40, 42, 63, 209 | Mutable columns, timestamp/no-timestamp branch, insert/update/delete/upsert, complete preparation/preflight batch flow, optional active-state functions. |
| `codegen/selection.rs`: opaque default projection/detail | 73, 99, 104, 107 | 73, 104, 110 | Selected wrapper, include-path `.into()` entries, key filter expressions, preflight and executing detail functions. |
| `codegen/conflicts.rs`: explicit native conflict targets | 12, 20, 22, 26, 29, 33 | 10, 12, 20, 22, 24, 26, 32, 33 | Primary/unique enum, validation match arms, copied/cloned full key members and executing native upsert. |
| `codegen/operations.rs`: public handler envelopes and nested patch preparation | 45, 49, 51, 53, 102 | 32, 35, 49, 51, 53, 55, 102, 177 | File header (`String::from` at 9), per-model module/DTOs, policy decoding, mutable patch assembly and singular/list fieldset preparation with qualified types. |
| `codegen/operations.rs`: named handlers and registration | 193, 195, 203, 205, 207, 208, 209, 210, 213, 214, 218, 222, 245 | 193, 195, 205, 207, 208, 209, 210, 212, 213, 214, 215, 218, 222, 225, 227, 231, 234, 236, 245, 250, 252 | Authorization/completed-response order, strict envelopes, qualified public-query type, operation-specific database flow, projection/publication, supported-operation guards and explicit listeners. |

### Lexical values and excluded work

These seven `format!` calls construct values/paths rather than source structure:

- `codegen.rs:110`: output `stores/<module>.rs`; `321`: escaped aggregate-reserved
  module name; `332,334`: portable constraint IDs.
- `conflicts.rs:15`: wire tag `unique.<field>`.
- `operations.rs:92`: host provider ID.
- `selection.rs:54`: selection-path prefix comparison.

Keep lexical construction and existing schema validation in Rust. Rust literal
escaping, booleans/numbers, serialized policy/enum-choice JSON and ordered literal
lists remain generator-owned values. Existing `{:?}` string escaping must remain
Rust-compatible; substituting JSON escaping would change control-character escapes.
`Vec<T>`, `Option<T>`, `.clone()`, member access, `.into()` and `&Key { ... }` are
source structure even though small: put their punctuation/control flow in templates.

Outside `codegen*`, `stores/query.rs:18,38,39,196,199` constructs timestamps/SQL;
`operations.rs:129` names runtime events; `migrations.rs:728` hashes revisions;
`migrations/faults.rs:181` formats a fault observation;
`migrations/history.rs:288,320,324,334-336` creates paths/SQL exports;
`migrations/steps.rs:17,71,199` and `migrations/sqlite.rs:420,537` build SQL/names.
Tests build fixtures/expectations. None emit production Rust source. Do not migrate
them. Preserve the generated runtime `format!` in the
registration function; its containing Rust source moves into a template unchanged.
The coordinator also inspected API codegen: it emits Idea, not Rust, and is excluded
from the Rust-source migration. This SQL audit does not claim ownership of that code.

## Readable template boundaries

Prepared checked-in UTF-8 `.rs.tpl` assets live under `src/codegen/templates/`, for
later `include_str!` wiring. Their exact boundaries and view models are below.
Small repeated declarations use focused templates where that removes duplicated
syntax rather than adding ceremony.

Each template must show readable Rust declarations, attributes, comments, functions,
statements, braces and actual branch/loop bodies. Context contains checked names,
escaped literals, primitive flags and ordered per-field/model/relation records.
Resolve schema policy, relation mapping, type eligibility and operation selection in
Rust before rendering. Do not move semantic decisions into expression evaluation.

Examples of genuine template control: loop over column records inside the actual
array, conditionally emit `patch_bounds`, choose required/optional length checks,
loop depth-specific relation members, and choose the handler's actual operation body.
Precompute explicit branch booleans when the engine cannot compare strings; no
arbitrary expressions are required. Split handler bodies by named operation if that
is clearer than one large conditional. Retain the declared ten-operation order.

Compose named sections rendered from these structural templates in the existing
emission order. Every section must expose its structure; do not keep long Rust
fragments in context or disguise the old implementation as `{{source}}`/`{{body}}`.
The static `encode_bounds`, `patch_bounds`, `bounds`, projection expressions,
`mutable` and delete-context fragments also become explicit template branches.
Replace the final `Loaded` import string-rewrite with the equivalent template flag.
Preserve generated comments/docstrings, including comments inside function bodies.

## Frozen engine usage and error contract

The lane needs variables, explicit booleans, nested conditions/inverse branches,
ordered loops with per-item fields and access to enclosing/root values. Standard
literal Rust braces must survive. Templates containing adjacent closing braces,
`use ...::{...}`, `serde_json::json!({...})` and a generated runtime `format!` must
render without interpreting Rust tokens as template instructions. The coordinator's
frozen design uses original-style default grammar and configurable `[% %]`
delimiters for Rust source. Use the Rust delimiters and prove their collision/
escaping rule rather than escaping every ordinary Rust brace.

Inserted values must be inert: policy JSON, field names and documentation literals
containing delimiters must never be parsed a second time. Preserve whitespace and
use explicit context errors for missing members/wrong types; no silent empty fallback.
No helper is essential if Rust prepares lexical literals. A generator-owned literal
helper is acceptable, but the generic engine must not acquire Rust language policy.

Parse/render return `Result`, with a typed error implementing standard error traits
so the lane can retain its cause through `TransformError::with_source`. Include a
stable template identity and position in the error, without dumping private policy
JSON or context values. The initial proposal mapped engine failures to contextual
callback errors; M1 boundary proof refined resource failures to `Limit` below.
Retain existing schema/mapping/serialization/ownership errors and their precedence.
Do not use `expect` to hide mistakes in embedded templates. `bounds::emit`, `storage`
and expression helpers currently returning plain values may need fallible internal
signatures; the existing public emitter signatures already return transform results.

Bounds require special review: `OutputPlan` currently checks one MiB of aggregate
**pre-rustfmt** source and 1,024 files. More readable whitespace increases this input,
and per-template work/output defaults must accommodate the accepted schema corpus.
Do not change shared limits here. Prove normal/boundary cases and report any newly
rejected accepted input to the coordinator before selecting different limits.

## Dependency implications

Change only SQL's existing `codegen` feature to activate `dep:stackpress-lib` in
addition to its current schema-codegen/transformer edges. `server` already activates
that same optional dependency. `codegen` alone must not activate `server`, an engine,
database adapter, listener, host IO, provider crypto, or any new external dependency.
The current lib manifest has no schema/transformer dependency, so this direct edge
does not introduce a static cycle. It does expose lib's existing transitive graph to
build-host codegen; verify the actual Cargo graph after release. Default/runtime-only
SQL retains its existing feature boundary. No root lockfile/manifests owned elsewhere
are changed by this lane.

## Prepared assets and explicit view models

Preparation added 19 assets in [the owned template directory](../../../../crates/framework/framework-sql/src/codegen/templates/).
They were initially unwired drafts; M1 wired and proved these structural assets. All flags
below are actual JSON booleans; collection records are ordered arrays, and values
ending in `_literal` are already Rust-encoded scalar strings, inserted once.
No policy JSON, schema object, or large rendered section is cloned into loop items.

| Asset (`.rs.tpl`) | Prepared render context |
| --- | --- |
| `store-header` | `model`, `module`, `revision`, `fingerprint_literal`, `has_relations`; ordered `columns` with `wire_literal`, `column_literal`, `codec`, `required`, `nonempty`. |
| `mutable-columns` | The same column record shape, filtered once to non-key/non-generated stored fields. No source-string trimming to reuse a column initializer. |
| `schema` | Ordered `models` with checked `store` module and `name` symbol. Both definition/catalog loops share this compact metadata. |
| `table` | `model`, `table_literal`, `mysql_target`; `fields` with name/kind/not-null/optional length; `primary` literal strings; indexes and FKs with ordered literal column arrays. |
| `bounds` | `bounded`, `patch_bounded`, `model`; length-bearing `fields` and separately filtered `patch_fields`. Empty/false branches need no optional payload. |
| `catalog` | Model/table, ordered key literals, `has_active`/conditional active literal, bounded flag, scalar query fields and checked relations. Query field records contain the column atoms plus existing codec/choices/search/public flags. |
| `views` | Root `model`; four ordered `depths`, each with depth value/previous value, ordered trusted/public `views`, and public `projection_fields`. View fields carry checked member/store, wire/relation literals, multiple/optional booleans. Type paths, `Loaded`, `Vec`/`Option` and projection expressions remain visible in the template. Depth-zero arrays are empty; no unsupported negative depth is rendered. |
| `queries` | Model/table, ordered filter/sort records; `legacy_key`, checked `key_type` atom, conditionally present key/active column literals, `has_active`, and a short engine-rendered `key_value` expression. Filter types come from `scalar-type`. |
| `timestamp`, `active` | Root presence flag (`timestamped`/`active`), one encoded policy literal and its wire/column atoms; active also has model/table. Large policy data is supplied only for the selected branch. |
| `writes` | Model/table, bounded/patch-bounded flags, key type and rendered convenience-key expression. The entire write/batch function bodies are static structural source here. |
| `selection` | Model, fallback boolean, ordered encoded selected/include paths, and key member/variant/clone records. Array punctuation, `.into()` and field-copy/clone expressions are visible. |
| `conflicts` | Model, ordered unique records with name/tag/column literals, variant and rendered scalar type; ordered key member/clone records. |
| `scalar-type` | `qualified` plus checked `module`/`symbol`, or a checked fixed scalar type symbol. It constructs the qualified enum path explicitly. |
| `key-argument` | `simple`, and conditional `member`/`string_key`; constructs `&Key { ... }` or `key`. No whole expression arrives from a formatter. The old `copy_key` expression helper can disappear: selection/conflict templates own member access and `.clone()` directly. |
| `operations-header` | Static file-level module documentation; no context values. |
| `operations-open` | Model/module/store, one policy literal, `has_fieldsets`, and ordered mutable fieldset records containing checked target module/model, name literal and multiple flag. Opens the model module and contains complete envelope/policy/input function bodies. |
| `operations-handler` | Small root model/model-literal/store atoms and one `operation` record, without policy JSON. `name`, `variant`, checked non-search input symbol; strict flags `host`, `get`, `detail`, `search`, `create`, `patch`, `upsert`, `batch`, `mutation`, `context`, `project_row`. Rust constructs only supported, consistent combinations in the original ten-operation order. Search type qualification is authored source, not a formatted string. |
| `operations-registration` | `active`, `mutable` and the ordered selected operation name/variant records. Contains the full registration body and closes the model module. The existing runtime event `format!` remains literal source text. |

Operation sections concatenate as header, then for each model: open, its selected
handlers, registration. Store sections concatenate in the existing order: header,
table, optional bounds, catalog, views, queries, mutable columns, timestamp, writes,
optional active, selection, conflicts. There is no generic source wrapper template
or engine include mechanism. Short rendered scalar/key fragments are explicitly
identified above; other context values are language atoms and metadata.

Context limits must account for JSON encoding, not only the emitted bytes. In
particular, a Rust-encoded policy string stored inside JSON escapes quotes and
backslashes again. Keep one policy literal at the render root; render timestamp,
active policy and operations independently, and never repeat a policy per handler.
Proposed SQL engine options retain the one-MiB template/output limits and existing
depth/iteration/step bounds, with an explicit **four-MiB render-data limit** for
encoded metadata overhead. This is a bounded SQL renderer input setting, not a
change to lib defaults or either global OutputPlan/artifact cap. Root contexts are
shallow arrays/records; policy JSON remains one inert string, not a nested object.
After M0 release, use the existing high-byte/escaping corpus and limit-boundary
cases to prove every previously accepted generation still works. Record encoded
data versus raw/formatted output sizes; if the proposed bound or any work budget
rejects accepted data, return to the coordinator rather than shrinking the corpus.

Whitespace belongs to the assets. `bounds`, `views`, `scalar-type` and
`key-argument` have no newline after their outer closing tag; disabled bounds
stays empty. Final `active` ends with a newline and Rust invokes it only when an
active field exists. Preserve source comments and expression-versus-block
closure shapes so readable layout does not introduce token changes before rustfmt.
The preparation stage ran no Cargo, rustfmt, rendering, regeneration or coverage.

## Compatibility proof and test matrix after release

1. Before edits, use the coordinator's expanded 190-file tracked-artifact baseline and capture
   the current SQL emitter's normalized output for the expanded cases below. Generate
   only into disposable roots; never hand-edit or independently regenerate shared
   example roots. Record sorted relative paths and full bytes/hash, not token counts.
2. After migration, pass the same corpus through the actual registry, `OutputPlan`,
   managed `OutputWorkspace` and unchanged rustfmt path. Compare final Rust/JSON and
   managed manifest bytes with the baseline; check filenames, owners, contributions,
   ordering, fingerprints and the public APIs. Readable raw source whitespace may
   differ; the required compatibility comparison is the normalized installed artifact.
3. Cover no models; String/Integer/simple and compound keys; required/optional scalars;
   enums of each codec; lists, objects and nested fieldsets; no/active/timestamp models;
   unique and nonunique fields; storage lengths on key-only and mutable fields;
   plain versus MySQL-target DDL; owning/inverse and singular/optional/many relations;
   depth-zero through depth-three trusted/public views; reserved module/member names;
   empty/default/nested selection; and operations disabled/enabled.
4. Test quotes, slashes, newlines, control characters, Unicode and template delimiters
   in allowed wire data/defaults; generated identifiers still come from normalizers.
   Assert private policy/default handling and readable/public relation filtering stay
   unchanged, and rendered values cannot inject source/template instructions.
5. Retain failures for missing keys, unsupported scalar/storage type/length, broken
   relation/default selection and depth/path limits. Add focused render/error mapping
   tests and prove a failed generation leaves the installed root unchanged. Generic
   malformed-template, nesting and engine-limit cases remain the engine lane's tests.
6. New SQL emission tests belong in a module of `tests/main.rs` gated by `codegen`,
   with existing `tests/storage_codegen.rs` coverage retained for `codegen,migrations`.
   Exercise `codegen` without `server`, then `codegen,server` and
   `codegen,migrations`; retain default/no-feature checks. Compile generated consumers
   in isolated targets and run meaningful read/write/operation behavior, not only
   snapshots. No live PostgreSQL/MySQL connection is needed to prove this formatting
   migration; do not describe SQL snapshots as live database evidence.
7. Coordinator integration reuses generated-workspace/admin generation tests for
   absent roots, repeat/check, full-type corpus, model rename/removal, disabled/plugin
   selections and source diagnostics; include api-checkpoint/native build-host
   consumers. Keep SSR/hydrate/CSR and browser evidence separately scoped. Changes to
   those shared tests/roots require coordinator ownership, not this lane's writes.
8. On release, first record `cargo clean` and `cargo llvm-cov clean --workspace` with
   `CARGO_TARGET_DIR=target/template-sql`. Keep it warm and at most 12 GiB thereafter;
   never clean owner targets. Use sccache, bounded jobs, nextest and the documented
   edit/gate environments. Record every Cargo/proof/clean invocation exactly once,
   including failures, and report IDs/counts/command versus Cargo-stage times.
9. Establish tests, then stop for milestone review before three distinct test-verified
   rounds (logic, responsibility, consistency/Rustdoc), plus safety review. Final
   changed-crate coverage must meet 70% with its actual feature scope and branch gaps
   stated; run scoped docs/doctests/Clippy/format checks and coordinator workspace gates.

## Original audit evidence and handoff

During the audit, source and manifests were read; no code was built, tested or regenerated.
The subsequent preparation release added only the 19 assets and refined this note.
The coordinator reports workspace/admin/Profile baseline generation checks passed.
The expanded hash baseline is
`/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/stackpress-template-baseline-19tcw996/generated-hashes-expanded.json`;
its 190-file count and baseline check results are coordinator evidence, not runs by
this SQL lane. Preserve that external baseline for the eventual differential proof.
The Agent Workspace validator before/after this note reports the same 11 inherited
missing screenshot-link errors in task 00013 receipts, with no new hard errors.
This note adds a preferred-200-line advisory; other warnings include existing and
concurrently authored files. It remains within the 500-line hard cap and cohesive
as a single lane design checkpoint; no split is warranted. This is not code verification.
The coverage inventory above preserves all eight emitter files, structural/lexical
sites, template boundaries, errors, limits, features and proof obligations in this
single coherent note. The original next step was independent M0 proof and review
of the drafts/encoded-data allowance before implementation release; those releases
are now complete. Cargo.lock remains solely coordinator-owned.

## Final SQL implementation receipt

Scope: `stackpress-framework-sql`, native aarch64-apple-darwin, Rust 1.96.0;
codegen-only, codegen/migrations and all SQL features. M1 and its coverage were
independently approved before rounds. The implementation now wires all 66
inventoried structural formatter sites, all 90 concatenation sites and both source
initializers through the 19 real structural assets. Tiny type/key syntax is also
authored in assets. No source-payload wrapper, external dependency, runtime
formatter/query-builder migration or public template configuration was introduced.

Seven original lexical `format!` sites remain exactly as inventoried. The shared
Debug-based Rust literal encoder is the eighth formatter; one `push_str` appends
only a successfully rendered asset. SQL's sole manifest edit activates its existing
optional lib dependency under `codegen`. Default normal edges omit lib, transformer
and server; codegen normal edges include lib without server or database adapters.
Shared lock synchronization and the expanded 190-file baseline remain coordinator-owned.

### Three rounds and safety

- Round 1 reviewed operation selection/order, complete versus legacy keys,
  Copy/clone choices, required/optional/list branches, public/private relations,
  selected-branch lookup, validation/error order and resource failures. Existing
  clear decisions needed no logical rewrite. Codegen-only tests passed **8/8**,
  record `3e41869028ab`; command 11.472529 s, Cargo stages 2.98 s.
- Round 2 separated query-field metadata, relation metadata and finite-view
  preparation from catalog rendering. This makes distinct schema responsibilities
  visible while preserving checks before render and depth/audience order.
  Codegen/migrations tests passed **11/11**, `8a4c2efcf42c`;
  command 12.554360 s, Cargo stages 2.66 s.
- Round 3 replaced parent wildcard imports with explicit dependencies, expanded
  the existing private-default predicate for readability, documented the fixed
  operation-name invariant and public error contracts. Focused tests passed
  **11/11**, `57136a67ed95`; command 11.980096 s, Cargo stages 2.89 s.
  Strict Clippy found one redundant reference to an already borrowed relation name;
  removing it was the only follow-up source edit. Strict all-feature/all-target
  Clippy then passed `f7a9fa1fdbd4` (3.089934 s / 3.00 s).
- Dedicated safety review checked checked identifiers versus encoded literals,
  inert insertion of hostile delimiters/control characters, private provider policy,
  finite ordered contexts, no global cache/shared mutable state, concrete errors,
  and append/install boundaries. No unsafe code, locks or async execution were added.
  No further safety change was justified. Final **44/44** SQL/native consumer tests
  followed the final source edit and used clean selected instrumentation,
  `efafe199a0ce` (43.463926 s command / 14.46 s Cargo stages).

### Compatibility, limits and errors

The final disposable original-source versus current-source proof passed
`e24bb3005cad` (15.769349 s command / 6.60 s Cargo stages): **12** corpus/mode pairs,
plus a real accepted near-limit schema. It compared all installed files and the
managed manifest after the unchanged rustfmt path. Cases include default app,
native/type matrix, reserved names, compound keys, active/timestamp and storage
lengths, each with/without handlers. The temporary autodiscovered driver was removed
after the coordinator confirmed its independent workspace enumeration/build had
completed. Original source copies remain disposable `/private/tmp` proof inputs,
not repository resources or production code.

The real 56-field repeated-enum boundary gives original raw **1,004,844 B** versus
template raw **1,034,170 B**, with identical formatted **1,035,095 B**. All remain
below the unchanged aggregate 1 MiB budgets. The 127-field oversized schema preserves
outer registry `Callback` then direct emitter `Limit`; the new per-asset refusal
also retains a concrete `TemplateError` with kind `Limit`. A failed generation leaves installed
bytes unchanged. Adapter unit tests separately prove static asset identity, private
cause retention, lazy false branches and JSON-encoded context overhead above 1 MiB.

The SQL adapter permits **4 MiB encoded context**, independently of engine defaults
of **1 MiB template/per-render output**. Existing finite depth/iteration/step limits
remain. `OutputPlan` still caps **aggregate raw source** at 1 MiB/1,024 files; the
managed writer separately caps **aggregate formatted artifacts** at 1 MiB. No global
limit, common schema error policy or registry wrapping changed. Legacy-sensitive
blank lines, long policy declarations and JSON macro spacing are documented in
the [source guide](../../../../crates/framework/framework-sql/docs/guides/change-generated-source.md).
The coordinator independently confirmed the default generated-workspace golden
bytes, `4fcfc594e4b5`; full integrated 190-file verification belongs to that lane.

### Coverage accounting

Final command: `cargo llvm-cov nextest -p stackpress-framework-sql -p generated-workspace-app
--all-targets --features stackpress-framework-sql/codegen,stackpress-framework-sql/sqlite-maintenance
--locked --offline --ignore-filename-regex '/tests/|/tests[.]rs$|/migrations/faults[.]rs$'
--json --output-path <owned-target>/sql-final-clean-native-consumer-coverage.json
--no-fail-fast`. This final command deliberately omitted `--no-clean`: supported
selected-package instrumentation cleanup removed stale old executables while
retaining the normal target and dependency cache.

The report contains exactly **24 SQL production source files**. LLVM line-instance
totals are **5,030/5,838 = 86.1596%**; conservative subtraction of all six covered
test-only hook statements from the numerator alone is **5,024/5,838 = 86.0569%**.
Explicit same-package report-only LCOV export is `ae7ec3fe2e02`; physical-line
measurement is `87a5766db31b`: **4,686/5,357 = 87.4743%**. The measured sourcegen subset
is **873/942 = 92.6752%**. This passes the 70% development floor for the whole declared
crate scope, without hiding runtime SQL files behind sourcegen-only coverage.

Exclusions are test directories/`tests.rs`, the entirely `cfg(test)` fault module,
and exact guarded injection calls at `sqlite.rs:91,178,312,323,604,791`. Physical
lines are unioned across LLVM instantiations; no production lines are excluded.
Reports and exporter remain in absolute `target/template-sql/`: final JSON, LCOV,
`sql-final-production.json`, `production-coverage.py`. Final JSON source ranges
match current files (EOF+1 marker allowed); obsolete pre-round monolithic catalog
function mappings are absent. No LLVM hash/profile warnings occurred.

Earlier raw M1 **71.01%** included the test-only fault module and was rejected;
native consumer suites raised actual production coverage before M1 approval.
Early final `--no-clean` reports mixed source versions. Raw-profile-only cleanup
proved old functions had zero hits, but retained stale objects in the denominator.
The clean final report above supersedes those measurements; physical dedup was not
used to hide stale source mappings. One report-only export omitted package selection
and excluded all SQL files; the zero-denominator exporter failed and the explicit
same-package export corrected it without rerunning tests.

Remaining coverage gaps include defensive missing-key/type/relation paths after
normalization, registry ownership/diagnostic wrappers and pre-existing runtime/DDL
error branches. Line coverage is not branch proof: permanent tests explicitly
exercise near/over limits, inert/private data, active absent/present, key-only/empty
models, complete identities, public/trusted views and operation ordering. Native
consumer tests execute actual SQLite reads, writes, batches, operations/projection
and session borrow contracts. No live PostgreSQL/MySQL or browser/WASM claim is made.

### Documentation and scoped gates

The `chrisai-docing` API-reference/formatting workflow produced a linked
[codegen reference](../../../../crates/framework/framework-sql/docs/api/codegen.md)
and source guide, with entry/concept/index updates. It documents local asset/context
coupling, lexical exceptions, private/error boundaries and raw/formatted limits.
Mechanical checks passed for all five touched Markdown pages. The exact public
callback fence compiled as an external disposable Rust crate under `/private/tmp`,
record `ed2523fd5b88`, SHA256 `0e8311a6e96c664508209a47ab24cedb75b55dc845397c198dce31eeb2529186`.
No public API change was needed; callers still supply normalization and output authority.

- Doctests: **5/5** including compile-fail, `06766001b243` (11.374402 s / 5.46 s).
- Warning-clean all-feature Rustdoc: `f37b8d934257` (3.515639 s / 3.47 s).
- Default no-feature library check: `740f66a8b5dc` (11.967626 s / 11.92 s).
- Codegen-only normal-edge graph: `e8dc09b26056`; default graph `5ab7e6d13fa6`.
- Final owned-source/test rustfmt check: `844475ae3f28`.

Benchmark JSON/log receipts are under `.agents/resources/build-benchmarks/runs/`,
identified above by unique final ID suffix. Every Cargo/proof/clean invocation used
the recorder, gate environment, sccache and jobs=2; dependency-resolving commands
used `--locked --offline`. Report-only commands have no resolver flags. Initial
absolute owned-target clean `bcc00e7d32ef` and llvm-cov clean `43dcc822936d` preceded
builds. Later cleanup was confined to supported coverage instrumentation/profile
operations. Observed target peak **1,901,648 KiB (1.814 GiB)**, below 12 GiB; it stays
warm. No owner/shared targets were cleaned. Command wall times and parsed Cargo-stage
subtotals are separate measures, not interchangeable timings.

Benchmark snapshot through receipt validation `3efe9b1d245d`: **51 completed records**,
41 passed and 10 failed, **513.431237 s** accumulated command time and **210.26 s**
observed Cargo-stage subtotals. These sums include retries and are not elapsed
task duration. That validator reports the same **11 inherited missing screenshot
links**, plus 33 advisory warnings including this coherent 427-line note. No new
hard link errors were introduced. The combined inventory/receipt stays below the
500-line cap and keeps the lane's context and evidence together; no accepted KB
context or task status is promoted by this implementation receipt.

Failed receipts remain intact: fixture type and Idea/temp-path mistakes; three
disposable original-source harness setup errors; the other lane's then-unused schema
helper; unsupported report `--features`; the owned Clippy borrow; and the empty
report's measurement refusal. No dependency fetch/install, lock/generated-root/task
status edit, commit, push, listener or subagent occurred in this lane. Full workspace,
integrated consumers and platform gates remain coordinator-owned. The lane stops
for coordinator review; this receipt is verification evidence, not owner acceptance.
