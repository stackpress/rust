# Task 00011 — Slice 6 M1 schema and generation

Implementation complete for coordinator re-verification; M2 has not started. This receipt supersedes the retained historical `00011-wip-m1.md`; the WIP's pending items are not current status.

## Authority and checkout

Work was confined to `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0`, branch `trial/3-s6-m0`, HEAD `57c67e6d328b543f9301265fe9fbaa6dc1b30feb`. The requested merge of current `trial/3` was a fast-forward from accepted M0. All M1 source remains uncommitted. No Slice 7/8 source, shared SQL/parser/transformer/server/CLI source, or other trials/targets were consulted or changed. No new shared request is needed. Cargo adds only existing locked regex/url/time dependency edges to framework-schema; no installs, listener or network DB execution.

## Implemented contracts

- Checked immutable normalization covers String/Text, i64 Integer, finite Number/Float, Boolean, Date/Time/Datetime, bounded Json/Object/Hash, homogeneous string/i64/Boolean enums, acyclic owned fieldsets, optional/required lists and explicit model relations. Compound scalar identities get typed Key DTOs. Model and field names retain exact wire spelling with deterministic Rust/SQL names and typed overrides; keyword/long-name compilation and generated DTO namespace collision refusal are proved.
- Defaults preserve false/zero/empty/null; missing and supplied null remain separate. Temporal literals are canonicalized before fingerprinting/emission. Date range is years 0001–9999; time requires seconds and at most nine fractional digits; datetime normalizes UTC RFC3339 to nanosecond precision. Leap seconds/excess precision fail. Explicit clock/providers supply now/cuid/nano/nanoid/random without ambient randomness/time or expression evaluation.
- Trusted Create inputs recursively use Create<Fieldset>, fill nested literal/provider defaults in one aggregate bounded preparation pass, then validate business rules once per instance. Generated members are unsuppliable. A counting fixture proves five provider calls and four host validations for four nested fieldsets plus the root. Stored validation skips relative creation constraints and host-only checks.
- Built-ins: type/presence, required/ne, eq/neq/option, numeric/date comparisons, Unicode character and whitespace word counts, case/slug/hex/URL/bounded regex, date/relative-time, typed min/max/step and aliases. Integer comparisons retain full i64 precision, including mixed real bounds; exact Number equality accepts safe integer and real spellings. Integer step requires a positive i64; Number step uses quotient tolerance 1e-10. DB uniqueness stays authoritative. Email/cc/color/price and custom named checks require explicit typed/versioned host registration. Unsupported crypto, missing providers, misspellings and incompatible args refuse generation.
- Public Create/Patch inputs reject denied/unknown/generated/relation fields, preserve tri-state values, validate client-safe and host rules separately, and prepare only approved literals. Host-private/generated assembly and DB execution remain M2. Patch omission/Set/Null and empty-update refusal are separate. Public project() is fallible, checks the stored value before copying, filters nested fieldsets and rejects browser-unsafe integers. Private literal defaults are resolved through `default:Model.field` and are absent from generated browser source.
- UiSchema contains app-approved models/fieldsets, keys, query fields/default selections, access/default flags, type/enum/rule metadata and all five role families. Aliases normalize once; duplicate roles refuse; prop types/order/fixed precedence/public selection and aggregate resolved-prop budgets are checked. Stable component keys/versions cross the boundary; executable imports remain in the tool-only catalog. Missing roles are omitted; unavailable components remain explicit metadata and selecting them for actual UI output refuses.
- Canonical SHA-256 fingerprint includes contract revision and selected provider/validator/component revisions, ignores inert raw attribute spelling and canonicalizes nested maps. Winning Idea model/field/attribute source byte spans survive imported composition; normalized JSON reports structural paths without invented coordinates. Errors retain typed causes without interpolating cause Display or rejected input.
- Typed query envelopes/predicates/sort/include paths are pure contracts, with allowlists and finite limits. There is no query execution, operation/route registration or inferred lookup URL. SQL codegen now selects scalar enum/date/number/object codecs/indexes, while executable stores deliberately retain the accepted M0 create/get subset.

## Review rounds

1. Logic/complexity: reviewed omission/null/default ordering, canonical values, list/rule applicability, disclosure and finite costs. Added aggregate direct-config/default budgets, private-default projection, bounded issue paths and exact integer step. `round1-tests.log` passed after the changes.
2. Responsibility/ownership: extracted assertion normalization, input preparation, direct-config accounting, query emission and typed nested-create emission. Runtime recursion owns one validation state; generated nested preparation shares one budget and validates once. `round2-tests.log` passed; `nested-scope-tests.log` and `nested-final-tests.log` passed the follow-up real generated corpus.
3. Consistency/Rustdoc/comments: reviewed public names/method roles/error contexts, emitted source, lint findings and README. Boxed diagnostic locations to keep error sizes bounded; removed generated clone-on-Copy/redundant enum guards; aligned creation/patch validation and public metadata checks. `round3-tests.log` and `clippy-round3.log` passed. The final branch audit then fixed default attribute provenance, mixed i64/f64 comparison/equality, and cross-model helper-name collisions; final checks below cover those edits.

The first complete workspace attempt failed in the unchanged Slice 7 PostgreSQL M2 harness's SQLite reference, scalar case 9 (`PoolTimedOut`). Its owned SQLite file/sidecars were removed. The isolated `cargo test -p stackpress-sql-postgres --test m2 --locked` rerun passed (12.41s), and the complete workspace rerun passed (75.55s); no Slice 7 code or watchdog threshold was changed. The original failure remains in `workspace-first.log`/`workspace-first-result.json`, rather than being concealed by the green rerun.

Transient M1 failures were fixed rather than excluded: an output-ownership sentinel was moved outside the exclusively managed root; the existing Idea grammar rejected escaped quotes, so quote/UTF-8 Rust emission is compiled from normalized JSON; an overly broad default diagnostic was narrowed to its exact attribute. An attempted Clippy `--fix` could not bind its locking listener under the sandbox; it stopped immediately. It was not escalated or rerun; all lint fixes were applied manually and normal Clippy is used. No development listener was started.

A coverage-driven branch follow-up added consumer tests for supplied values across every scalar/list codec, explicit null on every required defaulted field, standalone fieldset preparation, and client-versus-host validator behavior. These are public contract outcomes absent from the initial minimal-default fixture. The first raw export measured pure contracts at 58.19%, below the floor; no generated files or lines were removed from measurement. The follow-up tests pass; final fresh coverage is recorded below.

## Slice 8 frozen interface for coordinator review

Short freeze note: **revision 2**, descriptors `UiSchema`/`UiModel`/`UiField`/`RoleBinding`, generated stored/Create/public/Patch/Key contracts and `<Model>Filter`/`<Model>Sort`/`<Model>Query`, and partitions **contracts/metadata/stores** are the proposed frozen M1 surface. Concrete UI generation and query execution await later milestones. The exact supporting names and wire fields follow for review.

Freeze request: **`stackpress_framework_schema::CONTRACT_REVISION = 2`**. Framework SQL's independent runtime revision remains 1. A mismatch is checked in generated metadata with a compile-time assertion. This is the M1 handoff contract; coordinator acceptance/Slice 8 integration remain separate.

- Runtime descriptors: `UiSchema { contract_revision, schema_fingerprint, models, fieldsets }`; both collections hold `UiModel { id, label, fields, key, query_fields, default_selection, operations }`. `UiField` carries `id, wire_name, label, value_type, required, input_required, readable, writable, sensitive, nullable, default, validation, filterable, searchable, sortable, roles, relation`. Supporting names: `PublicType`, `PublicDefault::{None, HostGenerated, Literal}`, `UiRelation`, `FieldAccess`, `ValidationRule`/`PublicRule`, `FieldId`, `ComponentId`, `BoundedProps`. `Role::{Field, Filter, List, Span, View}` serializes snake_case; `RoleBinding { role, component, props, definition_revision, available }` uses stable keys. `FieldId`/`ComponentId` are String aliases, props are BTreeMap<String, serde_json::Value>. Operations are empty and relation lookup is None until a later explicitly registered capability exists.
- Generated stored modules: `contracts::<snake_model>::<Model>` and `Create<Model>`; equivalent stored/Create types for fieldsets. Public module `contracts::public` exports `Public<Model/Fieldset>`, `PublicCreate<Model/Fieldset>`, `PublicPatch<Model>`, `Patch<Model>`, `<Model>Key`, `<Model>Filter`, `<Model>Sort`, `<Model>Query`. Type names follow checked UpperCamelCase source identities. `Input::{Missing, Null, Value}` and `Patch::{Unchanged, Set, Null}` are schema runtime types. Trusted create methods are prepare/prepare_with/prepare_create/validate_create; public input methods validate_client plus validate_create or validate_patch; public create preparation returns approved JSON. Stored validate/validate_with/project are distinct. No whole-row public serialization should bypass project().
- Query names: `Predicate<T>::{Eq, Ne, Ge, Le, Like, Has, Hasnt, IsNull}`, `Sort<T>`, `SortDirection::{Asc, Desc}`, `NullOrder::{First, Last}`. Generated `<Model>Query` has q/filters/sort/includes/skip/take/total; defaults are None/empty/empty/empty/0/20/false. Limits: q4096 bytes; filters100; sorts/includes16; include path1024 bytes/depth3; skip100000; take1..100. Field selectors are Serde wire-name variants. This freezes data/validation, not future executor implementation.
- Partitions remain **contracts**, **metadata**, **stores**. `metadata::schema::{FINGERPRINT, CONTRACT_REVISION}` and `metadata::ui::schema() -> SchemaResult<UiSchema>` are entry points. Schema owns contracts+metadata, SQL owns stores. `DefinitionRegistry`, `ComponentDefinition`, `PropDefinition`/`PropType`, `ProviderDefinition`, `ValidatorDefinition`, `ComponentImport`, `DiagnosticMap`/`DiagnosticLocation` are codegen-only. `DefinitionRegistry::imports()` supplies actual Rust module/symbol mappings to the future UI emitter; imports/source provenance never enter UiSchema. The default role catalog is metadata-only with no prop signatures; Slice 8 supplies concrete component definitions, typed props/imports and availability through this app-owned catalog before normalization.

## Verification

All commands ran in this worktree under rustc **1.96.0 (ac68faa20, 2026-05-25)**, host **aarch64-apple-darwin**, LLVM **22.1.2**. `cargo-llvm-cov` is **0.8.6**. One owned `target/` root held ordinary, WASM and instrumented outputs, including existing workspace tests' nested disposable consumers. `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, no network DB URLs. The runner used file logs, separate process groups and a 600-second compile/suite watchdog with kill/reap; hang-class tests kept their existing 5–10-second parent controls unchanged.

| Gate / log under `/private/tmp/s6-m1/` | Observed result |
| --- | --- |
| `fmt-final.log`, final `fmt-closeout.log` | Workspace fmt check passed |
| `scoped-closeout.log` | 37 passed, 0 failed, 0 ignored, including the SQL doctest; 9.72s |
| `msrv-final.log` | Explicit `cargo +1.96.0 check` passed for all five packages with codegen/server/maintenance |
| `clippy-closeout.log` | All targets for all five packages, `-D warnings`, passed; 31.05s |
| `rustdoc-final.log` | All five packages, `RUSTDOCFLAGS=-D warnings`, no-deps, passed |
| `workspace-final.log` | 585 passed, 0 failed, 75 ignored; 75.55s on the successful complete rerun |
| `workspace-doc-final.log` | 885 doctests passed, 0 failed, 0 ignored; 33.12s |
| `wasm-final.log` | Actual `wasm32-unknown-unknown` pure consumer check passed |
| `feature-*.log` | Schema default/codegen; SQL default/server/codegen/migrations/sqlite-maintenance; tool database-tools; opt-in facade, all passed |
| `tree-*.log` | Pure consumer has no SQL/server/Idea/codegen/Tokio/native-driver edge; tool has no generated app/consumer edge |
| `bootstrap-final-results.json`, `bootstrap-final-hashes.json` | Both real output roots absent → tool alone checks → real generation → native/pure consumers compile; check/repeat preserve every SHA-256 byte. Actual remove+rename causes stale consumer exit 101, updated consumer compiles. Keyword/long-name fixture compiles; normalized-JSON public UTF-8/quote literal test passes. |
| `coverage-run.log`, `coverage-summary.json` | Fresh full instrumented run: 51 passed, 0 failed, 9 ignored, including SQLite maintenance; every changed crate clears 70% |
| `kb-before-receipt.log`, `kb-final.log` | 146 inherited errors and 22 warnings before and after the receipt; identical findings, no M1 WIP/receipt finding. Validator exit 1 is the inherited baseline, not a passed KB gate. |

The workspace run used the final production source; the later coverage follow-up added only two pure tests and extended the existing shared consumer test. Final scoped tests/Clippy and fresh coverage include those final test edits. Six unchanged trybuild alias/Clone/escape failures still pass, alongside actual borrowed store/Send service and SQLite maintenance/cancellation/negative-control proofs. Network adapter native cases remain visibly ignored with both opt-in URLs unset; ignored cases are not passes.

Reproduction commands (arrays avoid an ambiguous abbreviated package set; full argv/results are in each named `*-result.json`):

```sh
cd /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0
export CARGO_TARGET_DIR="$PWD/target" CARGO_LLVM_COV_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 TMPDIR="$PWD/target/tmp"
export LLVM_PROFILE_FILE="$PWD/target/verify-%p-%m.profraw"
mkdir -p "$TMPDIR" /private/tmp/s6-m1
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
s6_packages=(-p stackpress-framework-schema -p stackpress-framework-sql -p generated-workspace-tool -p generated-workspace-app -p generated-workspace-contracts)
s6_features=stackpress-framework-schema/codegen,stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance
cargo fmt --all -- --check
cargo test "${s6_packages[@]}" --features "$s6_features" --locked
cargo +1.96.0 check "${s6_packages[@]}" --features "$s6_features" --locked
cargo clippy "${s6_packages[@]}" --all-targets --features "$s6_features" --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc "${s6_packages[@]}" --features "$s6_features" --no-deps --locked
cargo test --workspace --all-targets --locked --features stackpress-framework-sql/sqlite-maintenance
cargo test --workspace --doc --locked --features stackpress-framework-sql/sqlite-maintenance
cargo check -p generated-workspace-contracts --target wasm32-unknown-unknown --locked
cargo run -p generated-workspace-tool --locked -- emit generate --check true
cargo run -p generated-workspace-tool --locked -- emit generate --check true \
  --input "$PWD/examples/generated-workspace/contracts/matrix.idea" \
  --output "$PWD/examples/generated-workspace/contracts/src/matrix"
cargo check -p stackpress-framework-schema --no-default-features --locked
cargo check -p stackpress-framework-schema --no-default-features --locked --features codegen
cargo check -p stackpress-framework-sql --no-default-features --locked
cargo check -p stackpress-framework-sql --no-default-features --locked --features server
cargo check -p stackpress-framework-sql --no-default-features --locked --features codegen
cargo check -p stackpress-framework-sql --no-default-features --locked --features migrations
cargo check -p stackpress-framework-sql --no-default-features --locked --features sqlite-maintenance
cargo check -p generated-workspace-tool --no-default-features --locked --features database-tools
cargo check -p stackpress --no-default-features --locked --features framework-schema,framework-sql
cargo tree -p generated-workspace-contracts --edges normal --locked
cargo tree -p generated-workspace-tool --edges normal --locked
cargo tree -p stackpress-framework-schema --edges normal --locked
cargo tree -p stackpress-framework-sql --edges normal --locked
cargo clean
mkdir -p "$TMPDIR"
unset LLVM_PROFILE_FILE
cargo llvm-cov --no-report --locked "${s6_packages[@]}" -p stackpress-sql-sqlite \
  --features "$s6_features",stackpress-sql-sqlite/maintenance
cargo llvm-cov report --json --output-path /private/tmp/s6-m1/coverage-default.json -vv
python3 /private/tmp/s6-m1/export-coverage.py
python3 .agents/scripts/validate-agent-workspace.py
```

The raw coverage export repeats the exact `llvm-cov export` argv printed by the verbose report after removing **only** `-ignore-filename-regex` and its argument. It retains all current instrumented objects and the fresh profdata file. `/private/tmp/s6-m1/export-coverage.py`, `coverage-export-command.json`, `coverage-all-sources.json` and `coverage-files.json` record the extraction and file-level evidence. Aggregation selects each package's production `src/` paths and excludes actual test files/modules; these five crates have no inline `#[test]` blocks in measured source. The pure consumer's relative-path original contracts and its matrix are both included; no shipped generated code or examples were excluded.

| Changed crate / complete selected production scope | Covered / executable lines | Percent |
| --- | ---: | ---: |
| stackpress-framework-schema, codegen enabled | 3210 / 3546 | 90.52% |
| stackpress-framework-sql, codegen/server/migrations/SQLite maintenance | 750 / 836 | 89.71% |
| generated-workspace-app, including emitted contracts/stores | 967 / 1204 | 80.32% |
| generated-workspace-contracts, original contracts + full pure matrix | 1463 / 2083 | 70.24% |
| generated-workspace-tool, library + real CLI binary | 151 / 202 | 74.75% |

Critical-branch review covers public/private default and rule disclosure, browser integer loss, supplied/null/default semantics, aggregate allocation bounds, source provenance, borrowed transaction aliasing, cancellation, uncertain commits and maintenance watchdogs. Uncovered lines include impossible-to-inject generated constant JSON codec errors and additional failure/selector combinations; the 70% development floor is met, not the separate 90% publication target. No later CRUD/query executor, operation or UI renderer is claimed by this evidence.

## Limits and stop point

M1 proves schema, generation and pure contracts on the recorded native/WASM scopes. It does not prove M2 CRUD/list/count/search/include execution, operations, auth, generated admin rendering/hydration/accessibility, general migrations/history/DB commands, network DB execution, or desktop/mobile integration. Ordinary stores remain single supplied String/Integer-key create/get; broader typed keys are schema/DTO support only and SQL emission refuses unsupported executable identities. Concrete UI role implementations are Slice 8-owned. Private rule arguments use explicit host checks rather than exposing them in browser policy snapshots. Callers must supply the named host-private literal default values and algorithms they registered.

Final cleanup/audit: all owned commands finished; process audit found no owned build/test process; the single worktree target was deleted; git diff --check passed; staged set is empty and the source audit contains only the granted paths. The WIP note is retained as historical restart evidence; this receipt supersedes it. Stop here for coordinator re-verification and Slice 8 freeze review; do not start M2.
