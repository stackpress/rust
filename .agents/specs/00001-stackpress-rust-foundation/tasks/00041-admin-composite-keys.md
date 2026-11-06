# Task 00041 — source-admin composite keys

Status: implemented and locally verified; ready for coordinator review. Base: 7d47ef6, initially clean. No commit.

Source reference: original Stackpress a71d683051ba8350fdd12d6b5a33f268fdcc285f, read-only. Paths below are under packages/stackpress-admin/src/transform unless stated otherwise.

## Identity audit and changes

The runtime reads the actual ordered key fields from the source graph's generated PageSpec. It never assumes that a creation result has an id member. Generated model/UI fixtures come from maintained schema.idea and official public emitters; no model descriptor was hand-written.

| Page / route | Checked / before → after | Original anchor |
| --- | --- | --- |
| Routes | Literal :id → one named segment per declared key. Copy's canonical `create/<keys>` route is added; existing Rust `copy/<keys>` remains an alias. | routes.ts:11–13,41–73,188–204 |
| Create | Committed composite row followed by 400 → search redirect; a single declared key redirects to `detail/<value>`. Only non-object results produce Unknown creation response results, even with noview. | pages/create.ts:92–95,153–174 |
| Copy | Same result-shape/key-count rule as create. Only @field inputs are forwarded, retaining the source's key/default behavior. | pages/copy.ts:99–105,158–198 |
| Detail | Complete ordered keys flow through the real detail event and all item action links. | pages/detail.ts:165–175; views/detail.ts:212–226 |
| Update | req.id redirect/browser submission → every declared key. Redirect projection reads only keys, preserving unrelated native updated Dates. | pages/update.ts:89–92,147–160 |
| Remove | Compound route selects the exact row; success still redirects to search. Source invalid-CSRF retry literally uses id and remains unchanged. | pages/remove.ts:142–167, especially153–154 |
| Restore | Compound route and success detail redirect use all keys. Source invalid-CSRF literal id retry is unchanged. | pages/restore.ts:89–92,154–179 |
| Search links | Literal row.id fallback → ordered key path. Existing generated trailing table actions already support all keys and remain unchanged. | views/search.ts:304; pages/search.ts:147–153 |
| Import / export | No key-specific rewrite needed: import forwards full rows/mode to batch; export preserves record fields/order. Default upsert with supplied absent keys still returns null; explicit create mode inserts. | pages/import.ts:79–84; stackpress-sql/src/transform/events/batch.ts:84–98; pages/export.ts:148–158 |

Remaining literal id uses select the framework Profile/Application collaborators' actual core field, or preserve the original CSRF retry described above. No new guard, route arbitration or SQL/model policy.

## Maintained proofs and rounds

Generated Catalog has categoryId + articleId and no id; Slugged has one key named slug. Real SQLite setup and registered handlers prove create → search, detail/update/remove/restore with both keys, isolation from rows sharing either key, copy, import modes, fallback/trailing search links and unchanged single-key behavior. Both immediate and queued registrars run. Separate registered-handler controls pin object/array/null results with/without noview; numeric path interpolation matches the existing source table adapter.

Original Rust expectations, Node goldens and .stderr files are unchanged.

| Refactoring round | Finding / result |
| --- | --- |
| Complexity | Pin result-object validation before noview and redirect by declared key count; route/redirect branches covered by real handlers. |
| Responsibility | One private portable identity helper serves native redirects and UI links; fixture file ownership is armed before asynchronous setup. |
| Consistency / Rustdoc | Document canonical copy paths and generated modules; match source numeric interpolation and preserve explicit key order. Narrow tests passed; final fmt/Clippy below passed. |

## Final gate

Recorder: canonical build-benchmarks store. Warm target/g50-m0 and sccache, jobs2, debug0/incremental0; standalone consumer targets rebuilt separately. Cargo uses --locked --offline; nextest uses --test-threads 2 and excludes tcp_watchdog. No source edits after final-tree fingerprint 20261106T064201.813827Z-93839aadcaea.

| Check | Result | Record |
| --- | --- | --- |
| fmt --check | passed | 20261106T064208.201746Z-448df6b1f3a9 |
| Workspace Clippy default / compatible, -D warnings | both passed | 20261106T064218.044708Z-31a4f8dd8c86 / 20261106T064225.558302Z-ac9711c27d8e |
| API/admin nextest default / compatible | 63 passed/3 skipped; 158 passed/9 skipped | 20261106T064242.166840Z-05ceb3dc1909 / 20261106T064250.017580Z-a15a6f3848cf |
| Composite models / UI freshness | both passed | 20261106T064321.228958Z-97d2b75d17ef / 20261106T064328.288583Z-ca901ac70d28 |
| API checkpoint --check | passed | 20261106T064411.149395Z-66af2ccac5e9 |
| All eight aggregate consumers, including build-host --check | passed; direct/facade inventories match full | 20261106T064439.607761Z-cc89d09996bd |
| API browsers: standalone hydrate/CSR; aggregate hydrate/CSR | 13 each, 52 passed, original expectations unchanged | 20261106T070432.416843Z-a05fe58a53ba / 20261106T070452.601610Z-83ac58ac241a / 20261106T070508.118490Z-5e9773bb9a0d / 20261106T070528.043250Z-1024d63af35d |
| API coverage | 90.89% (2234/2458 src lines; only implementation crate changed) | 20261106T071011.752095Z-870ea611f273 |
| Scope / unchanged final tree | passed; no manifests, locks, existing goldens, .stderr or Leptos changes; nothing staged | 20261106T070711.157845Z-a6f29b681c65 |
| KB canonical layout | 0 errors / 42 warnings canonical; 146 raw sibling-path errors | 20261106T071714.713112Z-ca66e2b14917 |

Fresh native/WASM artifacts were built and bindgen processed in /private/tmp/task41/browser. Cached Playwright path was asserted exactly; own headless Chromium, fresh contexts and intercepted requests, no listener. Browser/WASM work began after both nextests completed. Exact command argv, command/build timings and artifact-build record IDs are indexed in /private/tmp/task41/evidence.json and the canonical recorder.

The first compatible gate caught a new whole-request JSON projection rejecting the native updated Date (97 passed/1 failed, 20261106T063854.470111Z-67168cbdbb5c). Key-only projection fixed it; the original single-id expectation stayed unchanged; narrow confirmation passed (20261106T064117.033666Z-81669623bceb), then the complete scoped gate restarted. An initial checkpoint -p invocation targeted a non-workspace package; the corrected standalone --manifest-path command above passed without a source edit.

Resource cleanup removed only completed owned coverage/trybuild outputs, preserving root debug dependencies (20261106T063402.162836Z-f0595a3c5250). Consumer runner removed each completed owned subtarget under its existing size policy. Final target 20.06 GiB (<23.4 guard); disk 121.38 GB free. Final assertions: 20261106T071559.970702Z-70db06e2d858; all 699 registry tuples and every manifest/lock are unchanged..

## Exact changed files

M = modified; A = new. Framework-admin implementation is unchanged; its suites and existing compound-row-link tests passed.

```text
A .agents/specs/00001-stackpress-rust-foundation/tasks/00041-admin-composite-keys.md
A crates/framework/framework-api/examples/composite_admin_fixture.rs
M crates/framework/framework-api/src/views/admin.rs
A crates/framework/framework-api/src/views/identity.rs
M crates/framework/framework-api/src/views/management.rs
M crates/framework/framework-api/src/views/mod.rs
A crates/framework/framework-api/tests/composite_admin.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/.stackpress-generated.json
A crates/framework/framework-api/tests/fixtures/admin-composite/models/mod.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/source/_catalog.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/source/catalog.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/source/mod.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/source/slugged.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/stores/catalog.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/stores/family.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/stores/mod.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/models/stores/slugged.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/.stackpress-generated.json
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/mod.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/ui/catalog.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/ui/host.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/ui/mod.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/ui/pages.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/presentation/ui/slugged.rs
A crates/framework/framework-api/tests/fixtures/admin-composite/schema.idea
M crates/framework/framework-api/tests/main.rs
```
