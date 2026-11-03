# G49 packaged Idea sources — prerequisite receipt

Status: **Prerequisites and the authorized inherited-drift fix verified locally; ready for coordinator review. Stop before G50 M0.** Authority: G49 at **fc7b921**, replaced in part by G50 at **2abc9f1**; G50 checkpoint accepted and answers read from **5c3df92**. Worktree HEAD remains 2abc9f1 with the authorized uncommitted changes. No commit/staging/push, listener, install, network, license or context edit.

## Delivered and superseded scope

- Ship the original session `schema.idea` and aggregate **`stackpress.idea`** byte-for-byte. The existing API asset is unchanged. Canonical imports are `stackpress-session/schema.idea`, `stackpress-api/schema.idea` and `stackpress/stackpress.idea`; **no `stackpress/schema.idea` alias**. Exact source commit/bytes/hashes remain in the historical provenance table below and are rechecked at closeout.
- Add opt-in `stackpress_idea_transformer::PackageLoader<L: SourceLoader>` with `new` and `with_source`; no alias API, replacement parser or change to undecorated loaders. Exact registered bare names win; explicit relative, `@/`, absolute and unregistered application imports retain fallback rules. Embedded relative imports stay in the catalog namespace, with existing exact/extension/index candidates. Missing embedded files never probe synthetic host paths. Limits/cycles/errors stay owned by the existing Composer/loader contracts; static text is bounded before cloning. Each loader owns its catalog, with no global state or registration IO.
- Expose additive `SCHEMA_SOURCE` constants from session, API and aggregate; retain `framework_api::codegen::SOURCE` as an alias. Add `stackpress_framework::commands::schema::rust::package_loader` under existing native `codegen`, and use it in the real generate provider. Generic acquisition stays in Idea; package knowledge stays in the owner/build host.
- **G50 supersedes G49's API three-model projection, six descriptor changes and maintained 78-omission inventory.** Their implementation-only fixtures/module were removed; no descriptor/runtime corrections were applied. Existing API config/import stripping and handwritten models are unchanged, pending generation. The 528-property diagnostic / all 84 verbatim differences remain in [historical parity findings](00030-g49-schema-parity-findings.md); load it for the original evidence, not accepted omissions. Former normalized-baseline SHA and failure are preserved below as historical diagnostics.
- [Accepted G50 checkpoint](00031-g50-generated-framework-models.md) and [full construct ledger](00031-g50-generator-inventory.md) describe the separately gated generation work. Session-only links stay external with only Profile/Auth output; stock signup keeps name/type/roles. The shared default package and CompiledClient binding are approved in principle only. **No G50 M0 implementation or broad migration is started.**

## Concrete risk evidence

| Contract / risk | Executable evidence |
| --- | --- |
| Nested app imports, exact bare precedence, explicit local/root/absolute fallback | `idea-transformer/tests/package_loader.rs::registered_bare_names_win_and_explicit_app_paths_keep_their_meaning`, real ProjectLoader, undecorated-loader control. |
| Embedded relative candidates, cross-package imports, missing/escaping paths | `embedded_relative_candidates_stay_in_the_catalog_and_root_imports_return_to_app`; aggregate `tests/package_sources.rs` checks the exact three-edge chain and source constants. |
| Invalid/duplicate names/extensions, byte boundary and original IO cause | `registration_limits_and_fallback_errors_are_explicit`; namespace impersonation rejected by `namespace_collision_is_not_accepted_as_an_alias`. |
| Merge/references/diamonds and catalog isolation | `merge_references_diamonds_and_independent_catalogs_keep_composer_contracts`; aggregate tests add a real Profile nickname, retain source metadata/order, and pin G19 `Profile!` replacement. |
| Cycles/depth/bytes/files/import/expanded budgets terminate | `package_cycles_and_depth_limits_have_external_watchdog` launches the ignored child under the existing external watchdog; `watchdog_forced_kill_negative_control` kills/reaps its own blocked process. No in-executor timeout claim. |
| Catalog actually used, not just a standalone helper | Aggregate `tests/cases/codegen.rs::generate_provider_uses_shipped_catalog_for_nested_application_sources`: real generate event/provider, four source models and Auth encrypted metadata, owned schema.json; no registration IO. This is composition/provider proof, not G50 model emission. |
| Existing API output/runtime unchanged | Existing API normalization/emitted-source/generated-consumer tests pass in the compatible workspace graph; no new projection or descriptor test fixture remains. |
| Packaging/provenance and feature leaks | Package listings include the assets; all facade/consumer graph assertions, affected build-host execution and no-feature WASM checks; root and consumer lock assertions below. |

## Commands and results

All Cargo/proof/clean payloads are recorder-wrapped at the canonical main store. Outer form: `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label 'g49 …' --kind … --scope … --cache warm -- python3 /private/tmp/g49/run.py <command>`. Full commands, logs, wall times and Cargo-stage subtotals are in the record identified by each suffix. Proof scripts/results are disposable `/private/tmp/g49` evidence, not shipped runtime files.

| Record suffix / command group | Result |
| --- | --- |
| `d1993c8387b7`, `2769cdf555c2`, `5905152ecb85`, `72068d29649a`, `e7eae1767d9c` — checkpoint provenance, real-type probes, parity and KB | Historical checkpoint evidence preserved below; no accepted parity fix inferred from diagnostics. |
| `c99a8bca62be` — implementation preparation | Copied exact aggregate asset. Its projection/omission fixtures were subsequently removed under G50. |
| `e4b8ddc8c62d`, `ce44cc4e769d` — scoped edit-loop nextest | First failed on a test-only error conversion, corrected with the existing IO cause constructor. Second exposed a wrong test expectation: G19 retains the local Profile position; expected order corrected, Composer unchanged. |
| `08cd24cf67be` — scoped nextest for transformer + aggregate `full,codegen` | **139 passed / 10 skipped**, including the new resolver/provider proofs and existing compile-fail suite without overwrite. |
| `3a8e2291d5fc` — fmt; `db3ed202535a` — rounds.py | Formatting and all three distinct test-verified rounds passed; details below. |
| `9c482d9d364c` — docs.py | Guide compiled/ran and session docs passed; aggregate new claim anchor failed after rustfmt wrapped the expression. Corrected the anchor; closing rerun passes. |
| `4cea5f248948` — gate.py | **8 stages passed**: fmt, default/full workspace Clippy `-D warnings`, nextest and doctests, full Rustdoc `-D warnings`. Default **993 passed / 89 skipped**, full **1115 passed / 96 skipped**; default/full doctests **968 / 973 passed**, none ignored/failed. **1288.523 s wall / 606.42 s reported Cargo stages**; those are separate measures. |
| `79aabcde5459` — metrics.py | Parsed default/full doctest totals and recorder metadata from the actual gate log. |
| `390e1c3bcaa3` — closing.py | Docs and facade checks passed; build-host consumer graph correctly refused its stale lock under --locked. Refreshed only that lock under the existing authorization. |
| `bb105ad6c537`, `16eb4baa932f` — consumer-lock.py | Preliminary metadata --no-deps inspected the manifest without resolving it; the single actual offline resolution used `cargo tree --offline`, seeded from root. Immediately afterward: root bytes/699 tuples identical, consumer tuples a root subset. Build-host registry entries 196 → 276; no tuple new to root. All subsequent Cargo calls locked/offline. |
| `4e2af6e3fe88` — closing.py rerun | **Passed**: docs/compiled guide, every facade/consumer graph assertion, native/hydrate/CSR checks, build-host fmt/Clippy/build/finite run/freshness --check, isolated aggregate codegen, session/API/aggregate no-feature WASM, all three package-asset listings and exact root/consumer lock assertions. |
| `f5af14df7b2c` — coverage.py | **All four crates pass ≥70%** independently. **320.400 s wall / 281.37 s reported Cargo stages**. Full owned-file totals below. |
| `639241766fa0` — consumer-drift.py | **Passed** exact generated-diff assertion; all eight consumers (fmt/Clippy/build/finite run, build-host --check), direct/facade inventory equality and complete graphs.py --check, including updated website SSR/hydrate/CSR. **1107.215 s wall / 1058.18 s reported Cargo stages**. |
| `97c65cc80dcc` — final.py | **Passed**: exact 29-file authorized scope, git diff --check, 104 changed-document local links, source/runtime non-change assertions, root/consumer lock checks, all four coverage floors, compiled guide and both docs checkers, canonical KB and final recorder totals. Final receipt-only KB revalidation uses recorded label `g49 completed prerequisite receipt KB revalidation`. |

Full compatible feature selection: `framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,framework-api-codegen,framework-csrf-server,framework-email-server,stackpress-framework/codegen,stackpress-framework-schema/providers,stackpress-framework-sql/sqlite-maintenance,stackpress-framework-sql/codegen,stackpress-framework-sql/server`. Workspace nextest uses `--all-targets --locked --offline --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)'`. No renderer all-features. Live DB environment gates were unset and remain skipped; this task changes no SQL/runtime behavior and starts no services.

Implementation target: **target/g49**, two jobs, warm sccache, debug=0, incremental=0, native **aarch64-apple-darwin**, Rust/Cargo **1.96.0**. Implementation uses the standing **24 GiB cap / 23.4 GiB guard**, replacing the checkpoint's self-imposed 12/11.4 cap. Full gate peak **10,626,209,596 bytes**; coverage peak **15,068,558,435 bytes (14.03 GiB)**; final consumer replay peak **19,625,009,706 bytes (18.28 GiB)**, all below the guard. Old effort targets, shared debug dependency artifacts and coverage reports are retained; initial recorded clean/llvm-cov clean are in checkpoint record 2769cdf555c2. The existing recorded consumer runner removed only its completed `consumer-aggregate-api-disabled`, `consumer-aggregate-facade` and `consumer-aggregate-website` outputs at its internal 17 GiB threshold; final graph checks rebuilt the required website target.

## Refactoring rounds, docs and coverage

1. **Complexity:** reviewed exact-match precedence, embedded traversal, fallback selection, inclusive byte bounds and error/limit order. No further behavior-preserving code edit justified. Resolver suite rerun: seven tests pass, child intentionally ignored except under its parent watchdog.
2. **Responsibility/abstraction:** reviewed generic SourceLoader decorator versus explicit owner catalog and real generate-provider boundary; no parser/package knowledge leaks, global state or model-generation fallback. No further code change justified. Three selected aggregate composition/provider tests pass.
3. **Consistency/Rustdoc/comments:** reviewed public constructors/errors, static ownership, constants, feature boundaries and branch comments. Transformer 14 + aggregate 2 doctests pass. Final docs cleanup preserves prior claims' compact formatting and numbers the new loader reference section; no behavioral change. Corrected new claim anchor and final docs checks cover this cleanup.

Docs: new loader guide's complete Rust example compiled and executed in a disposable copy of the existing API-checkpoint consumer graph, with its lock byte-identical. Session check: **21 pages / 17 Rust fences / 39 claims / four negative controls**. Aggregate check: **19 pages / 11 fences / 63 claims / seven negative controls**. Existing aggregate/session fences were not edited; the new prose references source/tests and the separately gated G50 limit. API has no package docs/check.py; its README, constant/alias and existing compiled tests are covered. Transformer has no docs/check.py; the new guide compile/doctest and final local-link checks provide its evidence.

Independent `cargo llvm-cov -p <crate> [--features …] --locked --offline --fail-under-lines 70 --json --output-path /private/tmp/g49/<crate>-coverage.json -- --test-threads 2` runs measure each crate's own `src/` files. Coverage does not include browser execution or doctests (those have separate compile/test evidence); there are no new exclusions. Final results:

| Crate / selected features | Covered / executable lines | Line coverage |
| --- | --- | --- |
| idea-transformer, default | 2100 / 2249 | **93.37%** |
| framework-session, `auth-server,leptos-ssr` | 5223 / 5711 | **91.46%** |
| framework-api, `sql,codegen,ui-ssr` | 3709 / 4147 | **89.44%** |
| framework, `full,codegen` | 1076 / 1113 | **96.68%** |

New `source/package.rs`: **84 / 87 lines, 96.55%**. No claimed line coverage for assets/docs alone. All four per-crate JSON reports and owned-file totals are retained under `/private/tmp/g49/coverage-results.json`.

KB checks `95e2b128ce48`, final `97c65cc80dcc`: canonical sibling-link layout **0 errors / 36 warnings**; raw worktree **146 sibling-path errors / 36 warnings**. Only nonexistent external sibling paths with existing canonical targets are remapped. Compared with the checkpoint, one additional preferred-size warning reflects this coherent chronological receipt; it and the verbatim findings remain below the hard 500-line cap. Final scope/link/KB assertions passed after coverage and the authorized fixture replay.

## Lock/manifest delta, scope and limits

Only manifest change: aggregate Cargo.toml's existing `codegen` feature now activates its **already declared optional** session/API dependencies to read their constants. No new dependency declaration/version, root facade/manifest edit, external package or root lock edge is needed. Root lock SHA-256 is `c2dc3328d42d2a943aaf437532047be7203d8cdfa6666fc8ecf1626c75ead683`; all **699 registry tuples** and root bytes remain unchanged. Seven aggregate consumer locks are unchanged; build-host used its one authorized offline resolution and remains a root subset (196 → 276 registry entries, 80 already-present root tuples). No registry tuple was added to the workspace. Build-host lock SHA-256: `aab24ce76947eb1d474a23f6b159187be1b041af7cb576b523a347c0e682c6b2` → `f0912cf01aa29e95c417bc98213d0567512bd06fe875f41c2777451cbce7ef08`.

Exact consumer workspace delta: add `stackpress-framework-api` and `stackpress-framework-session` package entries/edges to its aggregate entry; registry tuple set gains 80 existing-root tuples and loses none. The consumer lock diff retains their full name/version/source/checksum records (**849 added / seven removed lines**, including necessary version disambiguation of existing dependency references). Package listings emit the existing missing-package-metadata warning; no license field/file was edited. The two generated consumer files below are retained under the coordinator's subsequent exact-file grant.

### Authorized inherited-drift fix — generated consumer output

Final check `d20980045813` correctly failed its 27-file scope assertion on two additional files written by the existing build-host consumer's normal Install operation: `consumers/generated/ui/note.rs` and its `.stackpress-generated.json`. The new output adds `row_actions::detail_action` and the sticky action header/cell already emitted by unchanged `framework-admin/src/codegen/templates/table.rs.tpl:20–26` (and asserted by its `templates/tests.rs:53–55`). This is inherited shared-UI fixture drift, not a G49 resolver/model change; the build-host's input uses direct ProjectLoader, not PackageLoader. Its generate → freshness --check roundtrip passed in record 4e2af6e3fe88 after installing that output; this is not evidence that the pre-existing checked-in fixture was fresh.

The generated Note file changes from **9589 bytes / b555ca4b04c0400d20bab2e6fbb7892dfe86b84433bdf18c34fd242c5acb682f** to **9851 bytes / d03d6ffc3758ee80f98981c8d3336d871bc6fbb09d14b0413fef98b3a77f45e4**; the manifest changes only that size/hash. The full workspace gate preceded the fix; the subsequent **all-eight consumer replay and final graphs** compile/run the updated fixture, including finite website SSR and hydrate/CSR compilation. This generated consumer fixture has freshness/compile/finite-use evidence, not a separate line-coverage percentage hidden within aggregate library coverage.

The restoration/baseline negative-control payload was rejected and did not execute. The coordinator then supplied the reason: the shared-UI merge missed regenerating this fixture, and restoring it would retain stale output. **Exact-file grant: keep both official build-host outputs in this task**, prove current --check, pin this sole row-actions difference, and replay all eight consumers plus graphs.py --check. No baseline freshness failure is claimed as executed evidence; the cause is established by the unchanged HEAD template, the emitted diff and the coordinator's confirmation. No generator source or template was edited for this fix.

Exact generated Note diff hunk (the proof asserts byte equality to HEAD with only these two replacements):

```diff
@@ -195,9 +195,14 @@ pub fn table(
             .into_iter()
             .map(|control| view! { <TableCell>{control}</TableCell> })
             .collect_view();
-        rendered.push(view! { <TableRow row_key=key.clone()>{cells}</TableRow> });
+        let action = stackpress_framework_admin::row_actions::detail_action("note", &["id"], &seed);
+        rendered.push(view! { <TableRow row_key=key.clone()>{cells}<TableCell sticky_right=true>{action}</TableCell></TableRow> });
     }
-    let headers = vec![view! { <TableHeader>{"title"}</TableHeader> }].collect_view();
+    let headers = vec![
+        view! { <TableHeader>{"title"}</TableHeader> },
+        view! { <TableHeader sticky=true sticky_right=true>{().into_any()}</TableHeader> },
+    ]
+    .collect_view();
     let rows = rendered.collect_view();
     Ok(view! { <Table caption="Note"><TableHead><TableRow row_key="header">{headers}</TableRow></TableHead><TableGroup>{rows}</TableGroup></Table> }.into_any())
 }
```

Corresponding manifest hunk; every other manifest property is asserted identical:

```diff
@@ -63,8 +63,8 @@
     },
     "ui/note.rs": {
       "owner": "framework-admin",
-      "sha256": "b555ca4b04c0400d20bab2e6fbb7892dfe86b84433bdf18c34fd242c5acb682f",
-      "bytes": 9589
+      "sha256": "d03d6ffc3758ee80f98981c8d3336d871bc6fbb09d14b0413fef98b3a77f45e4",
+      "bytes": 9851
     },
     "ui/pages.rs": {
       "owner": "framework-admin",
```

Exact changed-file list (including this receipt and the accepted design-only G50 documents):

```text
.agents/specs/00001-stackpress-rust-foundation/tasks/00030-g49-schema-idea-files.md
.agents/specs/00001-stackpress-rust-foundation/tasks/00030-g49-schema-parity-findings.md
.agents/specs/00001-stackpress-rust-foundation/tasks/00031-g50-generated-framework-models.md
.agents/specs/00001-stackpress-rust-foundation/tasks/00031-g50-generator-inventory.md
crates/framework/framework-api/README.md
crates/framework/framework-api/src/codegen.rs
crates/framework/framework-api/src/lib.rs
crates/framework/framework-session/docs/api/storage.md
crates/framework/framework-session/docs/claims.json
crates/framework/framework-session/schema.idea
crates/framework/framework-session/src/lib.rs
crates/framework/framework/Cargo.toml
crates/framework/framework/consumers/build-host/Cargo.lock
crates/framework/framework/consumers/generated/.stackpress-generated.json
crates/framework/framework/consumers/generated/ui/note.rs
crates/framework/framework/docs/claims.json
crates/framework/framework/docs/guides/codegen.md
crates/framework/framework/src/lib.rs
crates/framework/framework/src/native/commands/schema.rs
crates/framework/framework/stackpress.idea
crates/framework/framework/tests/cases/codegen.rs
crates/framework/framework/tests/package_sources.rs
crates/idea/idea-transformer/docs/README.md
crates/idea/idea-transformer/docs/api/source-loader.md
crates/idea/idea-transformer/docs/guides/package-imports.md
crates/idea/idea-transformer/src/lib.rs
crates/idea/idea-transformer/src/source/mod.rs
crates/idea/idea-transformer/src/source/package.rs
crates/idea/idea-transformer/tests/package_loader.rs
```

No session/API model descriptor/storage/auth behavior was changed. The package resolver composes source; downstream generators still have the source-construct gaps listed in 00031. Browser evidence is graph/compilation only; no browser matrix or live DB claim is made for this source-import change. G50's generated model migration, external-reference capability, signup collaborators and runtime/admin/custom-app proofs remain unreleased. Stop after prerequisite verification; coordinator acceptance is separate.

## Historical checkpoint — retained for provenance

**Everything below records the earlier checkpoint, not current recommendations or pending owner questions.** G49 subsequently approved the actual aggregate filename and a no-alias loader; G50 superseded its projection/parity work. The original source diagnostics and commands are retained losslessly for review. Current delivered scope, decisions and verification are above.

Historical status: **Checkpoint ready; awaiting coordinator review. Resolution/public API changes and final parity-test implementation were not released by that checkpoint.**

Authority: [G49](../decisions.md) and the coordinator's task-00030 brief. Baseline confirmed **a01940d**, initially clean. Scope remains the three requested items; generating session Profile/Auth from Idea is excluded. No commit, manifest/lock edit, listener, install or network access.

## Source inventory and item 1 evidence

Read-only upstream: `/Users/cblanquera/server/projects/stackpress/stackpress`, commit **a71d683051ba8350fdd12d6b5a33f268fdcc285f**. Each file below equals its committed Git blob; provenance checks did not rely on an uncommitted upstream edit. SHA-256 is over exact bytes, including whitespace and final-newline state.

| Upstream path | Bytes | SHA-256 | Rust disposition at checkpoint |
| --- | --- | --- | --- |
| `packages/stackpress-session/schema.idea` | 8005 | `395c0f46e6543f258ea97a74a543d9306676fcafef3bf71d0dd02deafeaac84a` | Copied byte-for-byte to [framework-session/schema.idea](../../../../crates/framework/framework-session/schema.idea). |
| `packages/stackpress-api/schema.idea` | 8576 | `5bddbdfa9c58be140cdfdab4c9cb9ec670e3dec4fc625c098c97ea5a6a2adebf` | Already exactly equal to [framework-api/schema.idea](../../../../crates/framework/framework-api/schema.idea); untouched. |
| `packages/stackpress/stackpress.idea` | 32 | `3195ad4ed5d4e74bb08b9c7903a93f66765ebe395cb6d3587614cf19857ecb26` | Actual aggregate source; awaiting filename ruling before copying. |
| `packages/stackpress/schema.idea` | — | — | **Does not exist.** Aggregate `package.json` explicitly ships `stackpress.idea`, whose entire content is `use "stackpress-api/schema.idea"` with no final newline. |

Recommendation: ship the aggregate's actual `stackpress.idea` in `crates/framework/framework/`, preserve `stackpress/stackpress.idea` as its canonical import, and provide the requested `stackpress/schema.idea` spelling as an explicit catalog alias if the coordinator wants that compatibility spelling. Do not invent an independently edited schema, duplicate generated models, or place an asset in the root facade without a grant. No aggregate asset has been created yet.

## How imports resolve today

| Existing owner | Verified contract / consequence |
| --- | --- |
| [idea-parser](../../../../crates/idea/idea-parser/src/lib.rs) | Pure parsing/compilation retains `use`; no package/file IO. Compatibility `finalize(file, source, inputs)` does not load imports; it is not a composition API. |
| [SourceLoader](../../../../crates/idea/idea-transformer/src/source/mod.rs) | Existing public seam returns bounded owned text, syntax, logical name and canonical SourceId. No parser replacement is needed. |
| [ProjectLoader](../../../../crates/idea/idea-transformer/src/source/native.rs) / [paths](../../../../crates/idea/idea-transformer/src/source/paths.rs) | Explicit absolute project root. Ordinary specifiers, including `stackpress-session/schema.idea`, are importer-relative filesystem paths. `@/` uses the project root; absolute and parent-relative paths retain their existing meaning. Exact file, optional extension candidates, then index candidates. Canonical symlink identities, bounded reads and original IO causes already exist. No Node/Cargo package lookup or embedded catalog. |
| [MemoryLoader](../../../../crates/idea/idea-transformer/src/source/memory.rs) | Same relative candidate semantics over explicit virtual files/aliases. It can supply today's finite proof, but a root insertion alone is not a package import from every nested application directory. |
| [Composer](../../../../crates/idea/idea-transformer/src/compose/mod.rs) / [idea generate](../../../../crates/idea/idea/src/lib.rs) | Existing import traversal, G19 merge, per-run snapshots/provenance, cycle/identity checks and acquisition/expansion budgets; explicit reference-aware finalization. `generate` already accepts a SourceLoader. Keep these semantics. |
| [Aggregate build host](../../../../crates/framework/framework/src/native/commands/schema.rs) | `commands::schema::rust::provider` constructs ProjectLoader and calls existing generate. This is the natural place to install the approved package catalog for ordinary aggregate generation. |
| [API codegen](../../../../crates/framework/framework-api/src/codegen.rs) | `config()` removes the initial use with `replacen` (line 137), renders a scalar Profile from the hand-written descriptor, parses the replacement source, then applies API-local wildcard/inverse adapters. Existing normalization test pins exactly three models. |

## Proposed resolution mechanism and public surface — not implemented

Add an **opt-in SourceLoader decorator** in `stackpress-idea-transformer`; keep ProjectLoader, MemoryLoader, the parser and Composer unchanged. Framework package knowledge belongs in the owning packages/aggregate build host, not in the foundational Idea crates.

Proposed additive public surface (names/signatures for review):

```rust
// stackpress_idea_transformer; construction owns the explicit catalog, no global cache.
pub struct PackageLoader<L> { /* fallback and compiled source catalog */ }
impl<L: SourceLoader> PackageLoader<L> {
    pub fn new(fallback: L) -> Self;
    pub fn with_source(self, name: &str, text: &'static str) -> Result<Self, LoadError>;
    // Only needed if the compatibility spelling above is approved.
    pub fn with_alias(self, alias: &str, canonical: &str) -> Result<Self, LoadError>;
}
// SourceLoader implementation; existing Composer/generate accept it unchanged.
```

Owning packages expose additive data constants `stackpress_framework_session::SCHEMA_SOURCE`, `stackpress_framework_api::SCHEMA_SOURCE` and `stackpress_framework::SCHEMA_SOURCE` with include_str of their own shipped asset. Keep existing `framework_api::codegen::SOURCE` as a compatible alias. Constants alone require no parser/native dependency or runtime plugin selection.

The aggregate adds `commands::schema::rust::package_loader<L: SourceLoader>(fallback: L) -> Result<PackageLoader<L>, LoadError>` under its existing native `codegen` feature. It registers the session, API and aggregate texts and is used by the existing generate provider. Direct library applications may assemble the same table without selecting the aggregate. Illustrative application use after approval:

```rust
let loader = stackpress_framework::commands::schema::rust::package_loader(
    ProjectLoader::new(project_root)?,
)?;
let schema = Composer::new(loader, CompositionLimits::default()).compose("schema.idea")?;
```

Lookup contract:

1. Exact registered bare package names resolve from the compiled catalog, independently of the importing app directory. An explicitly relative `./stackpress-session/schema.idea` remains a filesystem request; `@/`, absolute and ordinary unregistered imports delegate with unchanged candidate/error behavior. A registered bare name wins over a same-spelled local path; this selection is explicit when constructing the decorator.
2. Embedded package-relative imports resolve within the catalog namespace; bare imports can reach other registered packages. A missing embedded relative source is an error, not a synthetic host filesystem lookup. No package-manager search, process cwd/env, network, mutable singleton or dynamic plugin selection.
3. Canonical package identities are independent of disk paths. Aliases share one SourceId; conflicting names/aliases and fallback identities colliding with the reserved catalog namespace fail explicitly. Bound text before cloning. Composer still owns traversal limits, repeated-identity verification, import edges, typed references, merge order and cycle diagnostics.
4. Sources are compile-time data; runtime registration, G42 handler ordering/Abort and G43 policies are unchanged. No global guard or route arbitration is introduced.

Dependency/feature plan: only existing workspace edges. API `codegen` needs the existing transformer. Session parity tests need the existing parser/transformer as a dev edge. Aggregate `codegen` can select the owning session/API data constants without activating their runtime plugins or API's codegen/UI definitions. Keep default facade and runtime/browser normal graphs unchanged. Assert all **699 registry tuples identical**, and every refreshed standalone lock a root subset, after a coordinator-released offline resolution; no manifest/lock changes have been made at this stop.

Error compatibility: retain API `config()`'s existing `Result<SchemaConfig, IdeaError>` signature. Recover an original parser IdeaError from the typed composition cause chain; map package-resolution/limit failures to the existing source-free validation/limit shape with a logical filename and absent byte coordinates. Do not invent a source span or change shared error APIs. Application Composer/generate callers retain the full TransformError cause chain.

## API normalization compatibility — decision required

The real-type probe captured current `codegen::config()` and `normalize(definitions())` outside the repository. Normalized baseline SHA-256 (pretty JSON): **bb415715b587108af42cafb8bedad2b1b35f247d9156e051594d8dca1d196397**; models **Application, Session, Profile**. Existing [API test](../../../../crates/framework/framework-api/tests/sql.rs) `full_idea_normalization_retains_management_fields_and_external_profile` pins three.

Using unmodified API SOURCE and original session text with today's Composer/MemoryLoader succeeds and produces **Application, Session, Profile, Auth**. Full Profile includes reverse auth/applications/sessions edges and management metadata absent from today's scalar-only external projection. Directly normalizing the unadapted result returns exactly:

```text
SchemaError { kind: Invalid, context: "unknown query path", cause: None, path: ["model", "Application"], location: None }
```

That first failure is expected before the existing API-local wildcard adapter, not a new parser defect. Further selected-source capabilities include Auth hashed/encrypted attributes, which the existing shared normalizer explicitly rejects (`codegen/metadata.rs`); they must not be silently removed from the composed source or implemented as scope creep. Session-only source also names Application/Session reverse types: composition retains them, while full semantic normalization needs their definitions.

**Recommendation:** compose the complete imports without deleting the use or hand-rendering replacement source, then apply a named, documented API-local projection to preserve today's exact three-model normalized management contract. Keep the existing wildcard/inverse adapters. The projection selects the existing scalar external Profile shape and excludes Auth from this API-only output; full source composition remains available to application loaders. Pin byte-equivalent normalized JSON against the captured baseline and a structural checked-in oracle. Generating session models remains outside G49.

Alternative: normalize all four models/full metadata, which changes the requested unchanged-schema contract and requires a separate capability/behavior release. Do not assume that choice. The coordinator must confirm the explicit projection boundary before implementation.

## Item 3: descriptor parity evidence and unresolved differences

The disposable real-type probes compare **528** mapped properties: **444 equal / 84 unequal**. Complete source/descriptor JSON values for all unequal properties are in [the parity findings](00030-g49-schema-parity-findings.md); load that file when classifying or implementing the maintained parity test. No source or descriptor was repaired, and no expected-difference allowlist was blessed. This is a raw-source/common-contract comparison, not full shared-normalizer equivalence: derived input/generated/nonempty metadata and app-owned access/component policies need an explicitly agreed comparison boundary in the maintained test.

| Result family | Finding |
| --- | --- |
| Model/field inventory, types and defaults | Both models; ordered Profile 13/Auth 11 fields; 20 stored columns. Scalar/list types, scalar nullability, SQL names, literal/provider defaults, id/active/unique/searchable/sortable/timestamp flags, Auth.profile local/foreign keys and codec families agree. |
| Reverse-list required flags | Profile.auth, applications, sessions: source `true`, descriptor `false` — three differences. |
| Column diagnostic representation | Profile.id and Auth.id: no source is.required attribute, descriptor generic `Value is required.`. Auth.verified: source `Verified is required`, descriptor no required_message — three differences. These do not by themselves prove differing runtime validation. |
| Model metadata | 26 retained-attribute, 25 documentation, 20 presentation and 7 assertion-rule differences. Counts overlap: many source flags/defaults are already represented in dedicated descriptor properties or Column behavior; missing Model metadata is not automatically a missing runtime check. |

Recommendation: coordinator/owner classify these before accepting a passing full-parity test. Implement strict comparisons for the common storage/relationship contract and explicit, reviewed handling for metadata omissions; **do not silently snapshot the differences as accepted parity or change either side**. If full descriptor metadata parity is required, that completion needs an explicit release; reverse requiredness/verified diagnostics remain separate behavior questions. The maintained test, final docs and closing rounds/gates follow that ruling.

## Risk and proving plan after release

| Risk | Mechanism / proving test |
| --- | --- |
| Package names depend on app nesting or working directory | Test root/nested app imports, aggregate→API→session chain, direct constituents, package alias identity and explicit local override. Use real ProjectLoader plus compiled text, without any upstream checkout present. |
| New resolver weakens existing source semantics | Existing native/memory/import/merge suites; mixed filesystem/package diamonds, cycles, importer-relative paths, unresolved references, duplicate catalog names, missing source, alias/canonical collision and repeated-ID drift. Keep every existing candidate/error rule. |
| Large source/cycle hangs or unbounded clone | Per-source/total/file/depth/import/expanded budgets with embedded text. External process watchdog and forced-kill negative control for recursive/cycle/depth cases; finite tests, no listener. |
| API output changes while imports now load | Checked-in baseline normalized schema and byte comparison plus existing API emitted-source/generated consumer tests. Full composition must retain four source models before the explicitly approved API projection. |
| Parity test hides source/descriptor drift | Exhaustive typed comparisons, all literal/provider defaults and relations, retained metadata separately, and negative controls changing a type/default/attribute/relation. Report unmatched fields/attributes, never filter unknown metadata silently. No automatic source/descriptor edits. |
| Runtime/browser graphs gain build-host dependencies | Default + each affected feature, wasm no-feature and existing hydrate/CSR graphs; no resolver/codegen edge in runtime-only graphs; unchanged disabled-facade graph. |
| Cargo packaging omits source or source provenance drifts | Package listing and isolated source-asset include_str compilation from disposable copies; exact hashes and upstream commit recorded above. No generated Profile/Auth and no license changes. |

Closing gate after implementation: recorder-wrapped `--locked --offline` scoped nextest/Clippy/doctests/Rustdoc, compatible workspace gate (two test threads, excluding tcp_watchdog), affected standalone consumers/graphs, docs claim/link checks and aggregate compiled fences if changed, three distinct test-verified refactoring rounds, and ≥70% coverage per changed implementation crate. Named checkpoint differences are not claims of an implemented fix or coverage.

## Checkpoint commands, boundaries and changed files

All executed Cargo/proof/clean commands used the benchmark recorder at the canonical main store. Disposable probes reused the existing session direct and API-checkpoint-tool manifests/dependency selections in `/private/tmp/g49` with only path locations rebased; their original locks remained byte-identical. No offline resolution was needed. Target `target/g49` has distinct consumer subtargets, two jobs, warm sccache, debug=0, incremental=0; cap 12 GiB / guard 11.4 GiB, peak **1.196 GiB**. Old effort targets were retained.

| Recorded step | Result |
| --- | --- |
| `d1993c8387b7` — source provenance/preparation | Source Git blobs/bytes/hashes match; session copied; API equality and actual aggregate filename checked. |
| `2769cdf555c2` — `cargo clean --locked --offline`; `cargo llvm-cov clean --workspace --locked --offline`; two `cargo run --manifest-path /private/tmp/g49/{direct,source}/Cargo.toml --locked --offline` | Both real-type finite probes passed; descriptors/config/baseline/composed source captured, no resolution or IO services/listener. **162.294 s command wall time / 154.24 s reported Cargo stages** (36.24 + 118 s). |
| `5905152ecb85`, final expanded `72068d29649a` — parity report / lock assertion | All 528 mapped comparisons reported and changed-default negative control passed as a **diagnostic probe**, not full parity; 699 external registry tuples identical, both consumer locks subsets and byte-identical. |
| `72068d29649a` — KB / exact scope assertion | Canonical sibling-link layout: **0 errors / 35 warnings**. Raw worktree: **146 sibling-path errors / 35 warnings**; only missing external sibling targets were remapped to the canonical checkout for the canonical run. New receipt/findings links and Agent File caps pass; exact three-file scope asserted. |

Recorder invocation correction: an initial preparation attempt used unsupported `--cache none`; argparse rejected it before executing or creating a record. It was rerun with `--cache unknown`. No Cargo/proof payload ran unrecorded.

Checkpoint review rounds: complexity — reuse SourceLoader/Composer rather than a new parser; responsibility — generic adapter in transformer, explicit catalog in build host; consistency — preserve public paths/errors and upstream bytes. **No behavior-preserving code edits were justified or performed at this design stop.** The required implementation rounds, maintained parity test, coverage and package docs gate are pending release; disposable compilation is not a completed implementation gate.

Exact repository changes at this checkpoint:

- `crates/framework/framework-session/schema.idea` — new exact upstream asset.
- `.agents/specs/00001-stackpress-rust-foundation/tasks/00030-g49-schema-idea-files.md` — this checkpoint/receipt.
- `.agents/specs/00001-stackpress-rust-foundation/tasks/00030-g49-schema-parity-findings.md` — all reported comparison differences; 373 lines retained together to preserve coherent verbatim review.

No Rust source, test, existing documentation, manifest, lock, facade or historical Agent record changed. Stop here for the aggregate filename/alias ruling, API projection approval and classification of parity differences.


## Coordinator verification

Committed as `9be543c` (29 files: shipped `framework-session/schema.idea` and `framework/stackpress.idea`, idea-transformer `PackageLoader`, aggregate catalog/`package_loader`, docs, regenerated aggregate consumer Note UI fixture, receipts 00030/00031). Independent fresh-target gate `/private/tmp/g49-verify/run.sh`, 2026-11-03, exit 0: Cargo.lock identical to `2abc9f1`; fmt; default and full-feature Clippy -D warnings; default 993 passed / 89 skipped; full 1,115 passed / 96 skipped; coordinator tcp_watchdog; doctests and Rustdoc -D warnings; aggregate `graphs.py --check`, all eight `consumers/run.py` consumers and `build-host -- --check` freshness (which fails on the previous main, confirming the fixed drift); session/aggregate docs checks and proofs ledger; tree clean. Merged into main.
