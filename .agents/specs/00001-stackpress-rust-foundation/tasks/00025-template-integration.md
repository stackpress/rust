# Task 00025: Coordinator integration receipt

Date: 2026-11-01. Branch `trial/3`, baseline HEAD
`62768feafeb40b13663751b288c62c83398dd843`. Status: **verified**.
Owner request and scope are preserved in the
[task](00025-tiny-template-generation.md); this receipt supplies final independent
verification rather than replacing milestone history.

## Result and source boundary

The reusable parser/renderer lives in `stackpress-lib::template`, with facade
access through `stackpress::template` and `stackpress::lib::template`. It supports
strict path substitution, nested boolean/inverse blocks, ordered loops and scope
metadata, registered helpers, compiled reuse, custom delimiters and finite work,
depth/data/output limits. Render values and helper output are inert. There is no
arbitrary expression execution, filesystem or runtime host dependency.

Schema, SQL, admin, transformer module indexes and the Profile generator now use
actual structural source assets. Rust identifiers/types/literal encoding, semantic
policy, callback ordering, managed output, ownership and rustfmt remain with their
existing owners. API generation delegates Rust source to those emitters; its own
Idea-schema emission is outside Rust source assembly. Runtime diagnostics, lexical
names/paths, SQL runtime rendering and test fixture formatting remain intentional
uses of ordinary Rust formatting. No source-payload-only template wrapper exists.

Reviewed implementation and three distinct verified refactoring rounds are recorded
in [engine/schema/transformer/Profile rounds](00025-template-rounds.md),
[SQL inventory and final receipt](00025-sql-inventory.md), and
[admin inventory and final receipt](00025-admin-inventory.md). Dedicated safety
reviews cover literal injection, private defaults/roles, lazy effects, ordered
contexts, bounded rendering and concrete error causes. The real generated-schema
consumer compiles hostile literal syntax and checks exact prepare/project values.
No material finding remains. No new dependency version or external package was added.

## Independent final workspace gates

Commands use absolute `target/template-review`, jobs 2, sccache with 20G cache,
incremental off and dev debug 0. Cargo resolution uses `--locked --offline`.
All commands are individually recorded under
`.agents/resources/build-benchmarks/runs/`; unique record suffixes below identify
their JSON/log pair. The JSON preserves exact commands and feature selections.

| Gate | Final result | Record suffix | Command seconds | Observed Cargo seconds |
| --- | --- | --- | --- | --- |
| Workspace all-target nextest | 971 passed, 85 preexisting skipped | dc766edcc330 | 550.086127 | 201.00 |
| Workspace rustfmt check | Passed | f3954f6d1f16 | 2.566243 | Not emitted |
| Workspace strict Clippy | Passed, warnings denied | b6460be6ed4e | 113.392093 | 113.00 |
| Workspace doctests | 965 passed | 04e71286cf1a | 88.182213 | 3.23 |
| Workspace Rustdoc | Passed, warnings denied | 23b59ae068d0 | 52.669131 | 52.60 |
| Fresh selected coverage suite | 285 passed, 12 preexisting skipped | 8e3ddbfa0f73 | 334.813553 | 223.00 |
| Production coverage accounting | Every changed crate above 70% | 7163cb14237d | 0.062859 | Not emitted |
| Standalone build-host compile | Passed | 42d2f4b261e2 | 87.073361 | 87.00 |
| Standalone API generator check | Fresh, no differences | e054ef0f2b41 | 87.778515 | 86.00 |

These are the final source state after all production rounds. The workspace test
gate is the workspace's default feature selection across all targets, not a claim
that every mutually exclusive platform/feature combination was run. The coverage
and lane gates separately exercise the selected codegen, server, SSR and format
features. Doctest counts are the sum of successful per-package test summaries.

## Production coverage

Fresh coordinator instrumentation is confined to
`target/template-review/coverage-final`, with supported normal instrumentation
cleanup and no `--no-clean` reuse of old source objects. It includes all six changed
crates plus existing generated-workspace native and generated-admin tool consumers.
Selected features: schema codegen/providers; SQL codegen/server/sqlite-maintenance;
admin codegen/ssr/format. Profile example paths are explicitly included using
`--no-default-ignore-filename-regex`.

Reports: `target/template-review/final-coverage.json` and
`target/template-review/final-production-coverage.json`. Exact source prefixes
scope each crate; separate test files and the wholly test-only migration fault
module are excluded. Inline test bodies are conservatively deducted from covered
lines only, retaining their entire denominator. SQL's six guarded test hooks are
similarly deducted. No production file is excluded to improve a percentage.

| Whole production crate scope | Conservative covered / full denominator | Lower bound |
| --- | --- | --- |
| Lib | 4,110 / 4,423 | 92.92% |
| Schema | 3,493 / 3,846 | 90.82% |
| SQL | 5,072 / 5,878 | 86.29% |
| Admin | 2,444 / 2,842 | 86.00% |
| Transformer | 2,022 / 2,162 | 93.52% |
| Profile generator | 150 / 194 | 77.32% |

The new template engine's separate final lane subtotal is 700/711 = 98.45%.
Coordinator totals differ from narrower lane totals because selected features and
executed native consumers differ. These are LLVM executable-line metrics and
conservative lower bounds, not physical-line dedup claims. SQL's separate clean
physical-line analysis is preserved in its receipt. Line coverage does not replace
the branch-risk matrices, actual generated consumers or browser proofs.

The coordinator independently checked the default/codegen normal dependency graphs
(`c0d97e84ccc3`): schema/admin/SQL select the direct lib dependency only with
codegen; transformer uses lib directly; lib has no other Stackpress dependency.

## Generated compatibility and standalone locks

The expanded pre-change baseline contains 190 tracked generated Rust/JSON files.
Independent hash integrity passed (`ca5fe84ed9a2`). These installed files were
never hand-edited to accept a migration difference. Read-only generator checks
also inspect managed manifests and all currently generated output:

| Generation selection | No-difference record suffix |
| --- | --- |
| Generated workspace default | c221d0e2bc2f |
| Contract schema matrix | fda070517815 |
| SQL native matrix | cd8e7271be31 |
| SQL reserved names | 9f232142940a |
| Data quickstart v1 | d40885b68333 |
| Data quickstart v2 | f9f478f864c3 |
| Data quickstart v3 | f0e4d316c93c |
| Admin default | 5072fbafc7db |
| Admin formatted | bb307abe54ec |
| Profile default | b8171bb62154 |
| Standalone API checkpoint | e054ef0f2b41 |

The SQL lane additionally compared original/current emitters across 12 corpus/mode
pairs and the accepted 98.7%-of-formatted-cap case (`e24bb3005cad`). Oversized
inputs preserve the registry Callback/direct Limit error layers and installed bytes.
The temporary autodiscovered differential driver was held stable during workspace
enumeration, then removed after release; no nonportable temporary test remains.

Root Cargo.lock adds three local dependency edges. Thirteen relevant standalone
native locked metadata checks were audited: eleven passed unchanged; build-host
and API-tool locks needed synchronization (`08cfbe0c7ae6`, corrected `bb5206bdbfeb`).
Build-host adds four local lib edges (admin/schema/SQL/transformer); API-tool adds
three (admin/schema/transformer). Package identities and all registry
name/version/source/checksum tuples remain identical. Actual locked/offline
build-host compilation and API generation then passed. No fetch/update occurred.

## Native and browser evidence boundary

Admin finite proofs use captured source identity
`accc65169a98cb2d25dd68907720feb2bfa7c0909425b712f7d2b9aef5141f60`, with 1,033
hashed inputs. Native generation/consumers and SSR/hydrate/CSR artifact builds pass
(`528f7ae4b754`), including two expected compile failures, removal/rename, hostile
literals, absent roles and formatted controls. Default and formatted UI files
match all 16 baseline files exactly; disabling a plugin restores the managed root.

Browser roles **24/24** (`7442b2b33415`) and formatted controls **16/16**
(`3bf892fdc58b`) pass, with zero console errors or unexpected requests. Both modes,
mounts and desktop/mobile layouts are covered; coordinator inspected selected
desktop/mobile captures. The existing Playwright proof uses installed Chrome
through an ephemeral executable adapter, without installation or a listener.
Four browser graphs and tool isolation pass (`f5f638d9156c`), with regex positive
controls independently checked. Evidence remains under
`/private/tmp/stackpress-template-admin-proof`.

The captured heavy browser build precedes the engine's private LoopBody refactor
and schema helper/readability rounds. Final engine tests, formatted fingerprints,
current-source generation checks and independent native coverage verify that
behavior-preserving bridge. This is not a second browser artifact capture from
the final source state. No installed Tauri, live PostgreSQL/MySQL, exhaustive
cross-target or publication-readiness claim is made.

## Retained evidence, validation and closeout

The 20 independent final gates comprising the table above, the baseline integrity
check and ten root-workspace generation checks all passed: summed command time
**1,365.364339 s**, observed Cargo-stage subtotal **801.29 s**. These overlapping
commands are accumulated measurements, not elapsed task duration. Lane timings,
retries and previous milestones remain separately counted in their receipts.

Latest owned target samples: coordinator 9,469,708 KiB, engine root 5,499,416 KiB,
admin 2,850,540 KiB, SQL 1,801,512 KiB. Earlier engine-owned rebased targets remain
accounted in its receipt; coordinator's earlier rebased 107,668 KiB adds to its
sample. Every owned aggregate remains below 12 GiB. Admin's finite proof had an
active 8 GiB guard. Warm targets and disposable proof evidence are retained;
no unrelated target/process or the owner's old dirty benchmark files were touched.

Changed API/guide pages and examples were source-reviewed and mechanically checked;
the five engine API examples execute against final source, and SQL/admin external
documentation consumers compile. Source and docs remain in their authoritative
crate directories. No accepted context promotion, commit, push, publication,
dependency installation or development listener occurred.

Final Agent Workspace validation (`bcfdff3e2274`) returns exit 1 for the same
eleven historical task-00013 missing temporary screenshot links, with 33 advisory
warnings. No new broken link exists. Those findings are unrelated to task 00025
and are not clean validation. All task-00025 files remain within the 500-line hard
cap; this integration receipt keeps final verification together. The
coherent main task preserves intermediate checkpoint history rather than deleting
evidence to clear its preferred-size warning.

Shared recorder snapshot after final integration: 2,433 attempts, 1,081 with
observed compilation; successful declared cold workspace builds remain 88.
These repository-wide counters include all lanes and unrelated earlier work.

Acceptance criteria: none for this library/source-generation migration. Verification
is complete; owner product acceptance and publication remain separate actions.
