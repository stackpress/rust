# Task 00006A: transformer and wrapper documentation receipt

Implementation evidence for [task 00006A](00006A-idea-docs.md). Load for this lane's page
inventory, public API coverage, executable examples, author review gates, and DX observations.
Coordinator verification and task status remain coordinator-owned.

## Scope and sources

Date: 2026-10-25. Checkout: `trial/3`, live starting HEAD `544aa47`, clean before writing.
The brief named older HEAD `2546fbd`; current source/tests were used. No other trial branch
was read. This lane owns only `crates/idea/idea-transformer/docs/`,
`crates/idea/idea/docs/`, and this specifically authorized receipt. The concurrently authored
parser docs were left untouched and linked after their targets landed.

Ground truth: current transformer/wrapper source and tests, canonical
[generated-profile](../../../../examples/generated-profile/README.md), decisions G18–G22,
Slice 3 generation/grammar/API designs, and receipts
[lane B compose](../slice-3-receipt-lane-b-compose.md),
[lane B integration](../slice-3-receipt-lane-b-integration.md),
[lane C writer](../slice-3-receipt-lane-c-writer.md),
[lane C rounds](../slice-3-receipt-lane-c-rounds.md),
[lane D integration](../slice-3-receipt-lane-d-integration.md), and
[lane D M4](../slice-3-receipt-lane-d-m4.md). Live declarations supersede proposal sketches.
Lib/server/CLI docs and task 00005A supplied structure and evidence conventions.

## Result and public coverage

Created **25 pages / 2,403 lines**: transformer **21 / 2,076**; wrapper **4 / 327**.
Transformer has entry, first-success walkthrough, concepts, five intent guides, API index,
and twelve public-surface references. Wrapper is deliberately smaller: entry, native-loader
walkthrough, one app-owned-tool guide, and one API page.

The transformer first success composes two in-memory sources with an import, registers one
function transform, and checks a temporary output root without creating it. Wrapper first
success uses ProjectLoader, GenerateRequest, Install, then Check. The app-tool guide links
to the canonical example rather than copying its README or broadening its Profile mapping.

Public coverage from source-name and heading audit (`api-audit.json`):

| Surface | Covered items |
| --- | --- |
| Source adapters | SourceLoader::load; SourceId new/as_path; SourceKind variants; LoadedSource fields; ProjectLoader new/root; MemoryLoader new/insert/alias. |
| Composition | Composer new/with_inputs/compose; all seven CompositionLimits fields and HARD_DEPTH; ComposedSchema/ImportEdge fields; real parser Reference re-export; merge_type. |
| Function plugins | Registry/TransformRegistry new/register/add/plan/plan_with; Transform/ContextTransform aliases; all five TransformSpec builders; all five TransformContext accessors. |
| Explicit selection | TransformInputs new/insert; TransformSelection new/contract/with_inputs; defaults, metadata/input budgets and consumed-name behavior. |
| Plans | GeneratedFile new/fields; OutputPlan executed/inputs/files/contributions/write/contribute_module; ModuleContribution fields; default plan authority restriction. |
| Native output | OutputWorkspace new/check/install/install_report/recover; CheckReport is_current/field; all GenerationReport fields and writer version. |
| Errors | TransformError new, Result alias, LoadError new/with_cause, Error/Display/cause conversions, read_source and its low-level limitations. |
| Wrapper | GenerateRequest::new and all six public fields; GenerationMode variants/traits; generate signature, phase order, returns/errors; parser/transformer re-exports. |

All **49 transformer free/inherent/trait operations** have lookup entries, with purpose,
examples, parameter tables where applicable, and Returns prose. Both wrapper operations
are covered. All **31 transformer public fields**, **6 request fields**, enum variants,
aliases, constant, defaults, and relevant derived traits are documented. There are 29 local
named transformer declarations plus the parser Reference re-export; no public export is omitted.
Context accessors share one real callback example exercising all five methods.

Concepts and reference explicitly cover the G19 matrix and sequential replacement/order;
root-only plugins and the complete G20 selection table; real reference sites moving with
winning attributes/whole columns; source graph versus effective provenance; finite work;
claims and module/aggregate conflicts; check versus pre-install reports; markers and legacy
length readability; cooperative locks, recovery preference and partial cleanup; case-directory
aliases; special-file rejection; and no crash-atomic/power-loss or outside-writer guarantee.
Language, parser config/finalization, and diagnostics link to the parser's owning pages.

## Example evidence

Disposable consumer and scripts/logs: `/private/tmp/stackpress-docs-00006a-transformer/`.
Nothing from the harness, lock, or build output was added to the repository. Its Cargo.lock
was seeded from the workspace lock and projected with `cargo generate-lockfile --offline`;
a shared-package comparison confirms **no external package/version drift**. Every compilation
used `--offline --locked`; the repository lockfile is untouched.

The extractor checks **all 68 Rust fences**. API blocks use each page's stated setup and
independent function scope; complete programs retain their named functions and main entry.
All 68 compile and their runnable assertions pass. This includes **six complete walkthroughs**:
transformer first check, imported reference/literal finalization, contracts/auxiliary inputs,
module aggregation, install/check/removal, and wrapper native install/check. Both displayed
quick-start outputs were observed. Recovery/app-tool helper definitions are compile-checked;
no interrupted-state recovery or arbitrary app tool is executed merely by defining them.

Every walkthrough output root is beneath a canonical temporary parent and cleaned by TempDir.
No run writes the repository's `src/generated`. Rust fences were formatted with the installed
Rust 1.96 rustfmt before the final extraction and execution. The native writer's recovery,
platform and crash limitations are source/test-backed documentation, not newly claimed proofs.

## Author gates

Followed installed chrisai-docing sequencing: guided-learning ownership, lookup API ownership,
then copy-editing, then formatting. Reviewed reader prerequisites, visible success, explained
examples, order/transitions, consistent terms, nested-heading orientation, separate reference
material, and complete public methods/returns. The final source pass checked G19/G20, typed
sites versus strings, automatic indexes versus app mod.rs, and writer error/recovery limits.
No source refactoring was appropriate for this documentation-only scope.

Toolchain: rustc/Cargo 1.96.0, rustfmt 1.9.0-stable; native aarch64-apple-darwin only.
Before builds, `cargo clean` and `cargo llvm-cov clean --workspace` ran against fresh external
target directories, preserving concurrent agents' repository build output. No coverage claim
is made for documentation-only work.

| Gate / command | Result |
| --- | --- |
| `python3 /private/tmp/stackpress-docs-00006a-transformer/verify_docs.py` | PASS: 25 pages, 161 relative links/anchors, one H1, tagged/closed fences, no empty parent headings, no double blank lines. Parser links resolve. |
| `CARGO_TARGET_DIR=/private/tmp/stackpress-docs-00006a-transformer/target cargo test --manifest-path /private/tmp/stackpress-docs-00006a-transformer/consumer/Cargo.toml --offline --locked -- --nocapture` | PASS: 68 passed, 0 failed/ignored; six walkthrough main functions executed. `consumer-test-final.log`. |
| `CARGO_TARGET_DIR=/private/tmp/stackpress-docs-00006a-transformer/workspace-target CARGO_NET_OFFLINE=true cargo test --workspace --doc --locked` | PASS: 430 passed, 0 failed/ignored. `workspace-doc.log`. |
| `cargo fmt --all -- --check` | PASS. |
| `git diff --check` | PASS. |
| Source-name/public-field audit | PASS: 49 transformer operations, 31 fields; wrapper 2 operations and 6 fields. `api-audit.json`. |
| Agent Workspace validator before receipt | PASS, warnings only. `kb-before.log`; final receipt validation recorded below. |

## DX observations, no code changes

1. `OutputWorkspace::install` and wrapper `generate(... Install ...)` return pre-install
   CheckReport differences. A first successful installation reports `is_current() == false`.
   Docs distinguish this from a subsequent freshness check and explain install_report.
2. LoadError has no direct From conversion into TransformError. App helpers returning
   transformer::Result must preserve it through another owned error cause; the guide shows
   the existing example's `std::io::Error::other` mapping rather than changing the API.
3. The wrapper aggregate renderer runs before Registry preflight. Transform preflight still
   precedes every registered transform callback, but the app aggregate must stay pure even
   when a later selection check will fail. This phase distinction is explicit in the API page.
4. Automatic module contributions claim root mod.rs. They cannot be combined with an app's
   custom mod.rs in the same plan. The guides explain choosing one root renderer and link to
   the canonical example's richer app-owned registry.
5. TransformSpec::owns Rustdoc calls an empty list contribution-only work, but actual module
   contributions require an owned partition and matching file. Docs state that an empty owner
   may inspect prior contributions/emit nothing; it cannot contribute a module without ownership.

No source, tests, manifests, repository lockfile, parser docs, task status, or other spec files
were changed by this lane. No commit/push, listener, deployment, publication, KB promotion,
MCP indexing, or agent messages were performed. Stop for coordinator review, not acceptance.

## Page inventory

Paths are relative to each crate's docs directory; counts include all lines. Use this table
with the mechanical JSON when reviewing completeness or checking later edits.

| idea-transformer page | Lines |
| --- | ---: |
| `README.md` | 46 |
| `api/README.md` | 39 |
| `api/composition.md` | 152 |
| `api/errors.md` | 121 |
| `api/memory-loader.md` | 74 |
| `api/output-plan.md` | 184 |
| `api/output-workspace.md` | 159 |
| `api/project-loader.md` | 64 |
| `api/registry.md` | 156 |
| `api/reports.md` | 73 |
| `api/source-loader.md` | 91 |
| `api/transform-context.md` | 93 |
| `api/transform-selection.md` | 106 |
| `api/transform-spec.md` | 103 |
| `concepts.md` | 209 |
| `getting-started.md` | 100 |
| `guides/checking-and-installing.md` | 63 |
| `guides/composing-sources.md` | 72 |
| `guides/generating-modules.md` | 53 |
| `guides/recovering-output.md` | 56 |
| `guides/registering-transforms.md` | 62 |

| idea page | Lines |
| --- | ---: |
| `README.md` | 33 |
| `api/README.md` | 125 |
| `getting-started.md` | 90 |
| `guides/app-owned-codegen.md` | 79 |

## Receipt validation

`python3 .agents/scripts/validate-agent-workspace.py` after the approved receipt write: exit 0; 19 warnings, 0 errors. Full output: `/private/tmp/stackpress-docs-00006a-transformer/kb-final.log`. Warnings concern existing KB line lengths/link descriptions; none names this receipt.
