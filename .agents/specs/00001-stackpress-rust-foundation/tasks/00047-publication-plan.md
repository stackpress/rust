# Task 00047 — crates.io publication plan

Status: release preparation authorized by G59/G60/G61 and Task 00050; publication remains unauthorized. The original audit below is preserved as historical evidence. Snapshot: local `71c4118`, branch `trial/3-publication-plan`, 2026-11-10. Original audit findings are from that checkout and offline probes. G59/Task 00049 updates the candidate scope to 23 and removes the premade-model generator blocker; retained measured rows remain the original audit evidence.

Stackpress can be released as a family of 23 packages, with the foundation packages first and the main `stackpress` facade last. Task 00050 resolves manifest versions, licence declarations, package metadata and README gaps; its verification results are linked below. Publication still waits for integrated package-test self-containment, the separate coverage lane and coordinator/owner release checks. Examples and consumer fixtures remain private to the repository.

G59/G60/G61 and the coordinator brief select the release scope, lockstep version and licences. Complete the remaining integrated checks and crate ownership review before the owner publishes in the order below. The original plan was read-only; Task 00050 is the separately authorized implementation. Neither task authorizes an actual upload.

## Authority and evidence

The [quality policy](../../../context/testing-and-quality.md) sets the 90% publication floor; [source structure](../../../context/source-structure.md) preserves independent crates and an acyclic facade. Current manifests/source take precedence over historical implementation-status prose. The coordinator supplied earlier 404 results for `stackpress`, `stackpress-lib`, `stackpress-server`, and zero search results for “stackpress”; these suggest availability, reserve nothing, and were not rechecked here. Check **all 23 names** again immediately before release.

[Audit resources](../../../resources/publication-plan-00047/inventory.json) contain every package/dependency/feature declaration. [Packaging evidence](../../../resources/publication-plan-00047/packaging.json) retains all 24 file lists, exact commands, exit codes, missing-index errors and size calculations. [External file references](../../../resources/publication-plan-00047/external-reads.json) retain source locations and resolved destinations; load these for individual repair work. [Optional feature wiring](../../../resources/publication-plan-00047/feature-paths.json) identifies the direct feature-to-path dependencies.

## Complete root workspace inventory

There are 51 explicitly listed members plus the root package: 52 packages total, 23 proposed releases and 29 unpublished examples. “Publishable” describes the intended role, not readiness. At the audit snapshot all inherited `publish = false`; every workspace version was `0.1.0`. Task 00050 now explicitly enables the 23 approved candidates and keeps the workspace default false.

| Package | Directory | Classification | Reason |
| --- | --- | --- | --- |
| `stackpress` | `.` | facade | Re-exports selected independent libraries through one namespace. |
| `stackpress-lib` | `crates/lib` | publishable library | Transport-neutral foundation and event/request/response contracts. |
| `stackpress-server` | `crates/server/server` | publishable library | Capability host and lifecycle independent of application entrypoints. |
| `stackpress-server-http` | `crates/server/http` | publishable library | Optional owned HTTP/1 host and TLS adapter. |
| `stackpress-cli` | `tools/cli` | tool/binary | CLI adapter library; `autobins = false`, so no standalone executable is shipped. |
| `stackpress-example-capability-host` | `examples/capability-host` | example/consumer/fixture | Executable demonstration/fixture, not a reusable release API. |
| `stackpress-idea-parser` | `crates/idea/idea-parser` | publishable library | Independent Idea parser and AST. |
| `stackpress-idea-transformer` | `crates/idea/idea-transformer` | publishable library | Composition, transformation and managed output contracts. |
| `stackpress-idea` | `crates/idea/idea` | facade | Turnkey parser/transformer wrapper. |
| `stackpress-example-profile-codegen` | `examples/generated-profile/codegen` | example/consumer/fixture | Application-owned generator/build tool. |
| `stackpress-example-generated-profile` | `examples/generated-profile/app` | example/consumer/fixture | Generated application consumer. |
| `stackpress-sql` | `crates/sql/sql` | publishable library | Independent query, dialect and database contracts. |
| `stackpress-sql-sqlite` | `crates/sql/sql-sqlite` | publishable library | SQLite execution and maintenance adapter. |
| `stackpress-example-sql-profile` | `examples/sql-profile` | example/consumer/fixture | Executable demonstration/fixture, not a reusable release API. |
| `stackpress-sql-postgres` | `crates/sql/sql-postgres` | publishable library | PostgreSQL driver adapter. |
| `stackpress-sql-mysql` | `crates/sql/sql-mysql` | publishable library | MySQL driver adapter. |
| `stackpress-framework-view` | `crates/framework/framework-view` | publishable library | Page/render, preference and flash contracts with optional adapters. |
| `stackpress-leptos` | `crates/leptos` | publishable library | Reusable Leptos controls and explicit rendering modes. |
| `stackpress-framework-language` | `crates/framework/framework-language` | publishable library | Pure locale registry and aliases. |
| `stackpress-example-profile-ui-contracts` | `examples/profile-ui/contracts` | example/consumer/fixture | Example-specific contracts/generated schema. |
| `stackpress-example-profile-ui-shared` | `examples/profile-ui/shared` | example/consumer/fixture | Example-specific shared views/generated application code. |
| `stackpress-example-profile-ui-web` | `examples/profile-ui/web` | example/consumer/fixture | Application-owned HTTP/SSR host. |
| `stackpress-example-profile-ui-client` | `examples/profile-ui/client` | example/consumer/fixture | Browser entrypoint proving an example application. |
| `stackpress-example-profile-ui-native` | `examples/profile-ui/native` | example/consumer/fixture | Profile example native/Tauri consumer. |
| `stackpress-example-profile-ui-gateway` | `examples/profile-ui/gateway` | example/consumer/fixture | Profile example gateway consumer. |
| `stackpress-example-review-preview` | `examples/review-preview` | example/consumer/fixture | Executable demonstration/fixture, not a reusable release API. |
| `stackpress-example-ui-stream-shared` | `examples/ui-stream/shared` | example/consumer/fixture | Example-specific shared views/generated application code. |
| `stackpress-example-ui-stream-client` | `examples/ui-stream/client` | example/consumer/fixture | Browser entrypoint proving an example application. |
| `stackpress-example-ui-stream-web` | `examples/ui-stream/web` | example/consumer/fixture | Application-owned HTTP/SSR host. |
| `stackpress-example-ui-gallery-shared` | `examples/ui-gallery/shared` | example/consumer/fixture | Example-specific shared views/generated application code. |
| `stackpress-example-ui-gallery-client` | `examples/ui-gallery/client` | example/consumer/fixture | Browser entrypoint proving an example application. |
| `stackpress-example-ui-gallery-web` | `examples/ui-gallery/web` | example/consumer/fixture | Application-owned HTTP/SSR host. |
| `stackpress-example-ui-preferences-shared` | `examples/ui-preferences/shared` | example/consumer/fixture | Example-specific shared views/generated application code. |
| `stackpress-example-ui-preferences-client` | `examples/ui-preferences/client` | example/consumer/fixture | Browser entrypoint proving an example application. |
| `stackpress-example-ui-preferences-web` | `examples/ui-preferences/web` | example/consumer/fixture | Application-owned HTTP/SSR host. |
| `stackpress-framework-schema` | `crates/framework/framework-schema` | publishable library | Schema contracts, providers and optional generation. |
| `stackpress-framework-sql` | `crates/framework/framework-sql` | publishable library | Generated stores, migrations and SQL integration. |
| `generated-workspace-tool` | `examples/generated-workspace/tool` | example/consumer/fixture | Application-owned generator/build tool. |
| `generated-workspace-app` | `examples/generated-workspace/app` | example/consumer/fixture | Generated application consumer. |
| `generated-workspace-contracts` | `examples/generated-workspace/contracts` | example/consumer/fixture | Example-specific contracts/generated schema. |
| `data-quickstart` | `examples/data-quickstart` | example/consumer/fixture | Executable demonstration/fixture, not a reusable release API. |
| `stackpress-example-ui-format` | `examples/ui-format` | example/consumer/fixture | Executable demonstration/fixture, not a reusable release API. |
| `stackpress-framework-admin` | `crates/framework/framework-admin` | publishable library | UI role adapters and optional generation. |
| `stackpress-framework-api` | `crates/framework/framework-api` | publishable library | API mechanics, optional SQL/UI and generation. |
| `stackpress-framework-csrf` | `crates/framework/framework-csrf` | publishable library | CSRF contracts and optional server integration. |
| `stackpress-framework-email` | `crates/framework/framework-email` | publishable library | Email contracts, transports and optional server hooks. |
| `stackpress-framework-session` | `crates/framework/framework-session` | publishable library | Identity, permissions and optional account/server/UI adapters. |
| `stackpress-framework` | `crates/framework/framework` | publishable library | Compile-time framework composition and runtime registration. |
| `generated-admin-tool` | `examples/generated-admin/tool` | example/consumer/fixture | Application-owned generator/build tool. |
| `generated-admin-shared` | `examples/generated-admin/shared` | example/consumer/fixture | Example-specific shared views/generated application code. |
| `generated-admin-client` | `examples/generated-admin/client` | example/consumer/fixture | Browser entrypoint proving an example application. |
| `generated-admin-web` | `examples/generated-admin/web` | example/consumer/fixture | Application-owned HTTP/SSR host. |

Nested workspaces are **not release candidates**. Keep their existing `publish = false` or inherited nested-workspace false; do not flip them while enabling the libraries.

| Nested location | Packages/fixtures staying unpublished | Reason |
| --- | --- | --- |
| `examples/blog` | `blog-tool`, `blog-web`, `blog-app`, `blog-assets`, `blog-store`, `blog-client`, `blog-xtask` | Separate seven-member example workspace; application plugins and dev/build tooling. |
| `examples/api-checkpoint/{tool,web,client}` | `api-checkpoint-tool`, `api-checkpoint-web`, `api-checkpoint-client` | Three standalone proof consumers. |
| `examples/leptos-independent` | `leptos-independent` | Explicitly excluded standalone library-consumption example. |
| `crates/framework/framework/consumers/{api-disabled,build-host,direct,disabled,facade,full,headless,website}` | `slice11-{api-disabled,build-host,direct,disabled,facade,full,headless,website}` | Eight standalone feature/build-host consumers. |
| `crates/framework/framework-session/consumers/{browser,direct,disabled,facade,plugin}` | `slice9-account-browser`; four separate `slice9a-consumer` packages | Account feature/boundary proofs. |
| `crates/framework/framework-email/consumers/{direct,disabled,event}` | Three separate `slice9b-consumer` packages | Email feature/registration proofs. |

No other tracked Cargo manifests were found outside those groups and the root inventory. Generated Rust fixture trees without their own Cargo manifests remain fixtures, not additional packages.

## Dependency graph and publication order

Names below omit the `stackpress-` prefix except the root `stackpress`. `?` marks an optional normal dependency; target-specific native dependencies are included. An arrow is **dependent → prerequisite**. At the audit snapshot every listed edge had `path` and no `version`; there are 86 normal dependency declarations, 65 optional. No candidate has a build dependency or `build.rs`. Dev-dependencies are excluded from this ordering.

| Order | Package suffix | Internal prerequisites |
| --- | --- | --- |
| 1 | `leptos` | None |
| 2 | `framework-language` | None |
| 3 | `sql` | None |
| 4 | `lib` | None |
| 5 | `idea-parser` | None |
| 6 | `sql-sqlite` | `sql` |
| 7 | `sql-postgres` | `sql` |
| 8 | `sql-mysql` | `sql` |
| 9 | `server` | `lib` |
| 10 | `idea-transformer` | `idea-parser`, `lib` |
| 11 | `server-http` | `lib`, `server` |
| 12 | `cli` | `server`, `lib` |
| 13 | `framework-view` | `leptos?`, `framework-language?`, `lib?`, `server?` |
| 14 | `framework-csrf` | `lib`, `server?` |
| 15 | `framework-email` | `lib?`, `server?` |
| 16 | `idea` | `idea-parser`, `idea-transformer` |
| 17 | `framework-schema` | `idea-parser?`, `idea-transformer?`, `lib?` |
| 18 | `framework-sql` | `framework-schema`, `sql`, `lib?`, `server?`, `idea-transformer?`, `sql-sqlite?` |
| 19 | `framework-admin` | `lib?`, `framework-schema`, `leptos`, `idea-transformer?` |
| 20 | `framework-session` | `leptos?`, `framework-view?`, `framework-schema?`, `sql?`, `framework-sql?`, `lib?`, `server?`, `framework-csrf?`, `framework-email?` |
| 21 | `framework-api` | `lib`, `framework-schema`, `leptos?`, `framework-admin?`, `framework-view?`, `idea-transformer?`, `server`, `framework-csrf?`, `framework-session?`, `framework-sql?`, `sql?`, `idea-parser?` |
| 22 | `framework` | `lib?`, `server?`, `framework-schema?`, `framework-csrf?`, `framework-session?`, `framework-api?`, `sql?`, `framework-view?`, `framework-sql?`, `idea?`, `framework-admin?` |
| 23 | `stackpress` | `lib`, `server?`, `server-http?`, `idea-parser?`, `idea-transformer?`, `idea?`, `sql?`, `sql-sqlite?`, `sql-postgres?`, `sql-mysql?`, `framework-view?`, `leptos?`, `framework-language?`, `framework-schema?`, `framework-sql?`, `framework-admin?`, `framework-session?`, `framework?`, `framework-api?`, `framework-csrf?`, `framework-email?` |

A topological sort of all 23 nodes succeeds: **no normal/build dependency cycles**. Leaves are `leptos`, `framework-language`, `sql`, `lib`, `idea-parser`; they can publish in any order. The table is one valid serial order, including every optional capability. An already published `.crate` file in a local target does not populate the registry index.

There are 41 internal dev-dependency declarations, including PostgreSQL/MySQL tests depending on unpublished `generated-workspace-app`, and email/session test relationships. Dev-dependencies do not define release ordering; path-only dev-dependencies can be omitted from the normalized published manifest. Never publish an example just to satisfy them. Rework/externalize repository-only tests or version publishable dev-dependencies as appropriate, then test the extracted package separately.

## Packaging blockers and proposed repairs

Keep the root workspace's `publish = false` as the default. After owner approval, replace `publish.workspace = true` with `publish = true` only in the 23 approved package sections. All other packages remain false. This avoids accidentally releasing the 29 examples.

Every prerequisite in the graph needs a compatible registry version, including optional and target-specific entries: for example `{ path = "../../lib", version = "0.1.0" }`. Preserve existing `optional`, `features` and `default-features` settings. Publishing strips the local path and resolves the version from crates.io. Disabled features do not excuse missing versions. The five leaves have no such normal/build edits; all other candidates do. The feature-wiring evidence lists exact feature names; examples include facade `framework-*`/`leptos-*`, schema `codegen`/`source`, view `leptos-*`/`preferences`/`server`, session `native`/`sql`/`auth-server`/`leptos-*`, API `native`/`sql`/`codegen`/`ui-*`, aggregate `full`/`codegen`, with dev-only crate fixture regeneration examples.

The following are crate-boundary problems that adding versions cannot fix. Source locations below are relative to each crate; complete literal occurrences are in the linked external-reference evidence. `include`/`exclude` cannot package `../` sibling files. Tests that are kept only at repository level must still contribute to release integration evidence; omitting them from an archive is not a coverage waiver.

| Candidate | Files outside its directory | Required release preparation |
| --- | --- | --- |
| `stackpress-server` | `src/core_tests.rs` and concurrency/ownership tests import `crates/lib/tests/support/watchdog.rs`. | Bring the test helper within a supported test boundary. Library build alone does not expose this. |
| `stackpress-cli` | Seven unit/integration references to the lib watchdog; entry tests create `../../target/cli-entry-fixtures`. | Fix helper ownership; use a writable test temp location when testing extracted packages. PTY Python probes are inside `tests/support` and must remain available if those tests ship. |
| `stackpress-sql` | `tests/{harness,network}.rs` import sibling `crates/sql/test-support/harness.rs`. | Self-contain or move repository-only network tests. Preserve `examples/support/mod.rs`, which Rustdoc examples include from within this crate. |
| `stackpress-sql-sqlite` | Maintenance unit tests import `examples/generated-workspace/support/watchdog.rs`. | Self-contain the helper for the maintenance test lane. |
| `stackpress-sql-postgres` | Unit/integration harness imports sibling SQL test support; dev fixture uses unpublished generated workspace app. | Self-contain supported crate tests; retain live-URL gating and process support. |
| `stackpress-sql-mysql` | SQL test harness; PostgreSQL `src/tests{.rs,/support/*}`; framework-SQL `g37-ddl.json`; unpublished generated workspace dev fixture. | Self-contain supported crate tests; do not assume the PostgreSQL source checkout exists beside the installed dependency. |
| `stackpress-leptos` | Rich-content unit test embeds `examples/ui-format/corpus.json`. | Give the test a package-local corpus while preserving its shared-source provenance. |
| `stackpress-framework-view` | Cleanup test imports the lib watchdog. | Repair test helper ownership. Retain local embedded JS and example assets. |
| `stackpress-framework-schema` | Source-generation tests embed sibling session/API schemas and its local `tests/extension-oracles.json` (relocated unchanged in G59). | Self-contain these test inputs or retain them in a separate repository integration lane. |
| `stackpress-framework-sql` | Migration/tests import example watchdog, SQL examples support, generated-workspace four Idea inputs, session/API schemas. | Self-contain package tests; preserve codegen templates and generation/regression data. |
| `stackpress-framework-admin` | Source tests embed session/API schemas. | Self-contain test inputs; preserve codegen `.tpl` and source-role behavior. |
| `stackpress-framework-api` | G59 uses local generated models/presentation and local live support. Other package-boundary inputs still require the extracted-package check. | Separate archive tests from repository consumer proofs; retain local `schema.idea`, UI CSS/SVG and template/fixture inputs used by shipped examples. |
| `stackpress-framework-session` | G59 uses local generated model families and local live support. Schema vectors still need a supported package boundary. | Self-contain supported package tests; retain `schema.idea`, views/styles, icon notices and local account/interop example inputs. |
| `stackpress-framework` | G59 localizes live helpers. Its generation tests read `consumers/build-host/source.idea`. | Self-contain or move repository integration tests: Cargo omits nested packages, so that consumer's input is absent from the package list. |

No escaped literal include/path references were found for `stackpress`, `lib`, `server-http`, `idea-parser`, `idea-transformer`, `idea`, `framework-language`, `framework-csrf` or `framework-email`. No candidate has a build script with hidden filesystem reads. The audit also checked filesystem/command calls: library runtime file paths are caller/config supplied, rather than fixed reads of sibling repository files. This is a static audit plus packaging probes, not proof of every runtime configuration.

`framework-email` embeds `data/nodemailer-{services,mime}.json`; Leptos/session/API embed local SVG/CSS and some source schemas/templates are embedded. Retain these files in package allowlists. HTTP `examples/deployment.rs` uses package-local `examples/assets` at runtime. Do not exclude those assets while continuing to advertise that example.

## Package contents and size evidence

At the original audit snapshot, all 24 `cargo package --list` invocations returned 0. The table below retains the measured rows for the 23 remaining G59 candidates; it is not a refreshed packaging run. In the disposable tracked-file copy, only root publish inheritance and local dependency versions were changed; example/consumer package sections were kept false. Every local path entry was given `0.1.0` mechanically, including dev entries for the probes; this does not recommend registry versions for unpublished fixture packages. No licence/metadata/include repairs were made. The copy had its own temporary Git index, no commit, and `--allow-dirty` was required. It has been deleted.

For each package: `cargo package -p NAME --list --offline --allow-dirty`, then `cargo package -p NAME --no-verify --offline --allow-dirty`. Five leaves produced actual archives; 19 stopped on the first missing internal registry package listed below. **No-verify never proves compilation.** Leaf `cargo publish -p NAME --dry-run --offline --allow-dirty` returned 101 for all five: `attempting to make an HTTP request, but --offline was specified`. Cargo stopped before any HTTP request; these are not successful dry-runs. Non-leaf dry-runs were inapplicable because their prerequisites have never been published.

Sizes in bytes: **A** is actual compressed `.crate`; **E** is an indicative gzip/tar reconstruction from the file list, not a Cargo archive. Listed payload for E uses original manifests and the workspace lock as placeholders for synthetic files; dependency resolution will change these. A rows give actual uncompressed archive payload. Compare final compressed archives with the stated **10 MB** registry limit and confirm current policy/any account limit with the owner online. None of the measured archives exceeds it; estimates do not certify unpublished archives.

| Package suffix | Listed files | Payload bytes | Compressed bytes | `--no-verify` result / first missing registry package |
| --- | --- | --- | --- | --- |
| `leptos` | 193 | 1,051,297 | 279,326 A | 0 — packaged |
| `framework-language` | 5 | 7,268 | 3,003 A | 0 — packaged |
| `sql` | 71 | 794,440 | 160,405 A | 0 — packaged |
| `lib` | 77 | 855,854 | 215,857 A | 0 — packaged |
| `idea-parser` | 70 | 852,222 | 103,089 A | 0 — packaged |
| `sql-sqlite` | 49 | 495,633 | 120,494 E | 101 — `sql` |
| `sql-postgres` | 57 | 501,298 | 128,742 E | 101 — `sql` |
| `sql-mysql` | 38 | 418,511 | 107,380 E | 101 — `sql` |
| `server` | 63 | 786,012 | 194,135 E | 101 — `lib` |
| `idea-transformer` | 58 | 592,887 | 151,174 E | 101 — `idea-parser` |
| `server-http` | 52 | 508,337 | 134,459 E | 101 — `lib` |
| `cli` | 44 | 549,592 | 143,666 E | 101 — `lib` |
| `framework-view` | 82 | 595,812 | 160,033 E | 101 — `framework-language` |
| `framework-csrf` | 16 | 232,561 | 59,047 E | 101 — `lib` |
| `framework-email` | 76 | 868,595 | 196,047 E | 101 — `lib` |
| `idea` | 9 | 232,656 | 57,661 E | 101 — `idea-parser` |
| `framework-schema` | 89 | 784,744 | 187,978 E | 101 — `idea-parser` |
| `framework-sql` | 165 | 2,086,231 | 324,209 E | 101 — `framework-schema` |
| `framework-admin` | 74 | 635,902 | 162,510 E | 101 — `framework-schema` |
| `framework-session` | 151 | 1,235,818 | 264,704 E | 101 — `framework-view` |
| `framework-api` | 92 | 675,353 | 166,631 E | 101 — `framework-admin` |
| `framework` | 117 | 1,129,446 | 218,955 E | 101 — `framework-sql` |
| `stackpress` | 641 | 18,413,274 | 5,010,947 E | 101 — `framework` |

At the audit snapshot the facade listed 641 files, including **498 `.agents` files**, 117 example files and 15 tool files. Its approximately 5.01 MB compressed estimate is dominated by unrelated material; three raw coverage JSON files alone occupy about 10 MB uncompressed. Use a reviewed facade allowlist such as `src/**`, `tests/**`, `Cargo.toml`, `README.md` and owner-approved licence files. Re-run the list after this plan/evidence exists; package size grows with repository artifacts otherwise.

For libraries, start with `src/**`, `Cargo.toml`, package README/docs and owner-approved licence/notice files; add actual `schema.idea`, `stackpress.idea`, `third-party/**`, `data/**`, codegen templates and test/example inputs according to usage. If tests/examples ship, retain their entire supporting tree and repair escaped reads first. An `exclude` alternative may remove `consumers/**`, caches and irrelevant proof/generated snapshots, but an allowlist is easier to review. Never blanket-exclude `src/generated/**`, `.tpl`, `.idea`, SVG/CSS/JS or fixtures still referenced by code. Framework-SQL has 165 listed files and about 2.09 MB indicative payload; trim unused snapshots only after identifying which tests and consumers still need them.

Binaries/examples require explicit decisions. CLI is an adapter library, not an installable `stackpress` command. Auto-discovered examples: HTTP `deployment`; SQL `schema_diff`, `build_inspect`, `execute_plan`, `capabilities`; SQLite `m0-host`; API `composite_admin_fixture`; session `jwt_interop`. Declared feature-gated examples: view `issue17_parity` (`leptos-stream`), session `account_host` (`leptos-ssr`). Verify every retained example under its required features from an extracted archive. No other candidate binary/example targets were found.

## Metadata and current licence declarations

Historical audit: all 23 had a `description`. All 23 omitted explicit `repository`, `readme`, `documentation`, `keywords` and `categories`; none inherits those from workspace metadata. Cargo automatically finds `README.md` in these 11: root `stackpress`, `server-http`, `sql-postgres`, `sql-mysql`, `leptos`, `framework-admin`, `framework-api`, `framework-csrf`, `framework-email`, `framework-session`, `framework`. The other 12 have neither a package README nor a readme field. Add package-specific documentation and explicit metadata before release; absent repository/docs/keywords/categories are discoverability/documentation gaps, not all hard upload errors. Select valid registry categories and confirm the canonical repository and public documentation URLs online.

At the audit snapshot, `stackpress-idea-parser` alone declared `license = "Apache-2.0"` and contains its own `LICENSE`. The other **22 candidates declare neither `license` nor `license-file`**, including the root. Root `LICENSE` is titled “STACKPRESS COMMUNITY LICENSE”, January 2026; it is included in the root package list but is not automatically a licence declaration inherited by child packages. Licence choice, compatibility decisions and all licence edits belong to the owner. This plan makes no recommendation of a licence.

| Candidate directory | Third-party licence/notice files currently present and listed |
| --- | --- |
| `crates/leptos` | `third-party/lucide/{LICENSE,FEATHER-LICENSE,NOTICE}`; `third-party/frui/{LICENSE,NOTICE}`; local SVGs. |
| `crates/framework/framework-session` | `third-party/lucide/{LICENSE,FEATHER-LICENSE,NOTICE}`; `third-party/frui/{LICENSE,NOTICE}`; local SVGs. |
| `crates/framework/framework-api` | `third-party/lucide/{LICENSE,FEATHER-LICENSE,NOTICE}`; local SVGs. |
| `crates/framework/framework-email` | `data/nodemailer-LICENSE` alongside the two data JSON files. |
| All remaining candidate package directories | No third-party notice/licence files in their own package file lists. Parser's own `LICENSE` and root `LICENSE` are reported above. |

Lucide's vendored text contains ISC terms and a Feather-derived MIT section; `FEATHER-LICENSE` repeats that section. Frui and Nodemailer files contain their own supplied terms. Preserve these exact files and provenance when changing package contents; they do not declare the whole crate's licence.

## Coverage against the publication floor

One accumulated native workspace measurement used `cargo-llvm-cov 0.8.6`, Rust/Cargo 1.96.0, LLVM 22.1.2 and `aarch64-apple-darwin`; sccache, incremental/debug off and two Cargo build jobs. The [exact compatible SSR features](../../../resources/publication-plan-00047/features.txt) match the expanded native source/codegen/server lane documented in task 00035. No `--all-features`, custom exclusion regex or source edits were used. No `STACKPRESS_TEST_*` environment variables were inherited.

The main nextest sweep passed **1,244 tests**, with 99 skipped. Its initial isolated-watchdog command failed because this tool disallows `--no-report` together with `--no-clean`; the first standalone export also had no file summaries without the selected features. A targeted continuation used `--no-clean` **without** `--no-report`, retained all profiles, passed the one watchdog and exported with the same features. Total: **1,245 executed tests passed**, with 98 tests left skipped overall; the continuation's 1,342 skipped count includes the already executed main sweep and ignored cases, rather than additional missing tests. The failed recorder attempt remains preserved, not rewritten as a pass.

The [per-crate coverage report](../../../resources/publication-plan-00047/coverage-summary.json) records each file's counts, and the compressed original LLVM export (5.5 MB, not committed; kept locally under the ignored `target/publication-plan-00047/coverage.json.gz`) retains raw counters. Attribution sums each package's mapped `src/**` file summaries, including shipped generated models and any in-source unit code counted by the tool. Separate integration tests/examples/consumer fixtures are not attributed as implementation, although their executions exercise it. No new exclusion was added to improve a percentage. This is a declared native source-scope baseline, not proof of every feature/platform or release integration.

| Package suffix | Covered / total executable lines | Percent | Gap to 90% (percentage points) | More covered lines needed, fixed denominator |
| --- | --- | --- | --- | --- |
| `stackpress` | 0 / 0 | N/A | Unmeasured | N/A |
| `lib` | 4,205 / 4,423 | 95.07% | 0.00 | 0 |
| `server` | 2,347 / 2,504 | 93.73% | 0.00 | 0 |
| `server-http` | 1,717 / 1,816 | 94.55% | 0.00 | 0 |
| `cli` | 1,306 / 1,402 | 93.15% | 0.00 | 0 |
| `idea-parser` | 2,261 / 2,391 | 94.56% | 0.00 | 0 |
| `idea-transformer` | 2,104 / 2,249 | 93.55% | 0.00 | 0 |
| `idea` | 41 / 41 | 100.00% | 0.00 | 0 |
| `sql` | 3,248 / 3,394 | 95.70% | 0.00 | 0 |
| `sql-sqlite` | 740 / 808 | 91.58% | 0.00 | 0 |
| `sql-postgres` | 227 / 889 | 25.53% | 64.47 | 574 |
| `sql-mysql` | 414 / 1,049 | 39.47% | 50.53 | 531 |
| `framework-view` | 1,791 / 1,844 | 97.13% | 0.00 | 0 |
| `leptos` | 7,928 / 8,737 | 90.74% | 0.00 | 0 |
| `framework-language` | 54 / 58 | 93.10% | 0.00 | 0 |
| `framework-schema` | 6,298 / 6,840 | 92.08% | 0.00 | 0 |
| `framework-sql` | 7,530 / 8,629 | 87.26% | 2.74 | 237 |
| `framework-admin` | 3,818 / 4,364 | 87.49% | 2.51 | 110 |
| `framework-api` | 2,234 / 2,458 | 90.89% | 0.00 | 0 |
| `framework-csrf` | 67 / 69 | 97.10% | 0.00 | 0 |
| `framework-email` | 2,423 / 2,739 | 88.46% | 1.54 | 43 |
| `framework-session` | 3,706 / 3,977 | 93.19% | 0.00 | 0 |
| `framework` | 1,173 / 1,227 | 95.60% | 0.00 | 0 |

**Five measured crates miss the floor:** SQL PostgreSQL, SQL MySQL, framework SQL, framework admin and framework email. Eighteen meet 90% only in this measured scope. Combined source counts are **56,714 / 63,016 = 89.9994%**, which is still below 90% even though two-decimal rounding displays 90.00%. A passing combined total would not waive a per-crate gap.

The root facade reports no executable lines: **0/0 is not 100% and not a numeric pass**. Record explicit facade feature/re-export/consumer integration evidence and an owner decision on applying the line floor to that non-executable package. The PostgreSQL/MySQL low counts reflect the absence of opt-in live database lanes; meet the floor with meaningful live/backend tests and error-path coverage, not synthetic numerator padding. Close SQL/admin/email gaps with their actual source-level untested paths. Additional-line estimates assume the current denominator stays fixed.

Doctests, browser WASM hydrate/CSR, installed Tauri/platform adapters, live PostgreSQL/MySQL, TLS or optional branches outside the selected features and extracted registry-only release integration were not established by this measurement. Stable native coverage is not a waiver for those support claims. Generated source stays counted; no hidden generated-tree exclusion was applied. Publication remains blocked until the selected crates and applicable release integration meet the policy.


## Versioning and toolchain recommendation

Recommend a **lockstep initial `0.1.0` release** for the 23 selected packages: the current workspace already shares this version, and facade re-exports plus generated schema/store types closely couple the family. This is a proposal for the owner, not an accepted versioning policy. Independent versions reduce unnecessary bumps for stable low-level libraries but require a maintained compatibility matrix, separate changelogs and careful facade/generated-output compatibility checks.

A requirement `version = "0.1.0"` is Cargo's compatible range below `0.2.0`, not an exact pin. If source-level contracts change incompatibly, use `0.2.0` under a stated pre-1.0 policy. Patch releases should preserve the documented public/generated behavior. Re-exported types retain their defining crate identity: incompatible copies of contracts, SQL types or Leptos types can make otherwise similar values unusable together. Coordinate changes to those crates, dependents and facade exports/features; do not promise that bumping only the facade fixes them.

Every package declares Rust **1.96** and edition **2024** through workspace inheritance. Edition 2024 itself needs at least Rust 1.85, but the declared 1.96 floor governs these packages. The installed compiler/Cargo are 1.96.0; the coverage run therefore measures on that floor. Run the full selected release matrix on 1.96 and the current supported compiler, checking resolved dependency MSRVs too; a lockfile can hide a newer compatible dependency requirement. Do not lower the manifest floor merely because the syntax compiles. Edition changes and MSRV increases need release notes and an explicit compatibility policy.

For each subsequent release: classify compatibility changes; choose the next coordinated version; update workspace/package versions and every relevant normal/build/optional/target/dev version requirement; refresh the lock deliberately; regenerate using official generators and freshness checks; rerun archive, per-crate coverage, feature/platform and integration gates; record changelogs/artifact hashes; create the owner-approved release tag and publish prerequisites before dependents. Independent versioning would require replacing some workspace version inheritance and tracking which facade version supports each component.

## Owner runbook — future online release

These are instructions to execute only after the owner releases implementation and publication. The offline probes above do not satisfy these gates.

1. Decide the exact 23-package scope (or a smaller dependency-closed first wave), licence declarations/files per package, initial version policy, registry account/owners and token permissions. Decide which package tests/examples/docs are supported; G59 keeps generation in repository-owned test/application tools. Recheck all names, ownership, 10 MB limits, registry category values, Rust/Cargo policy and repository/docs visibility online. Do not assume the three previously checked names reserve the whole family.
2. In a separately authorized implementation change, leave workspace publish false and enable only approved candidates; add the 92 versioned normal dependency entries described above; repair cross-crate file reads without adding dependency cycles; resolve dev-only fixture strategy; add owner-approved licence metadata/files, package README/repository/docs/keywords/categories and reviewed include lists. Preserve third-party notices, shipped templates/data and renderer/dependency boundaries. Review all manifest and lock diffs.
3. Use a task-owned target and the installed recorder. Before release checks, set up the gate environment below and run repository tests, lint, Rustdoc, doctests, generator freshness, target boundaries and per-package archive tests. For this first release, leaves can fully package before publication; dependent package resolution needs earlier versions in the registry. A local path-based build is insufficient. Re-run each dependent's package verification and dry-run after its prerequisites are indexed.

```sh
export RUSTC_WRAPPER=sccache SCCACHE_CACHE_SIZE=20G
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/target/00047-release"
export CARGO_LLVM_COV_TARGET_DIR="$CARGO_TARGET_DIR/llvm-cov"
TASK47_STORE=/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks
TASK47_FEATURES=$(cat .agents/resources/publication-plan-00047/features.txt)
bench() { local task47_kind=$1 task47_scope=$2; shift 2; python3 .agents/scripts/build-benchmark.py --store "$TASK47_STORE" run --label "00047 release $task47_kind $task47_scope" --kind "$task47_kind" --scope "$task47_scope" --cache unknown -- "$@"; }
bench clean workspace cargo clean
bench clean workspace cargo llvm-cov clean --workspace
bench other workspace cargo fmt --all --check
bench clippy workspace cargo clippy --workspace --all-targets --locked --features "$TASK47_FEATURES" -- -D warnings
bench test workspace cargo nextest run --workspace --all-targets --locked --features "$TASK47_FEATURES" --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)'
bench test workspace cargo nextest run --workspace --all-targets --locked --features "$TASK47_FEATURES" --test-threads 1 -E 'test(tcp_watchdog)'
bench test workspace cargo test --workspace --doc --locked --features "$TASK47_FEATURES"
bench doc workspace env RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features "$TASK47_FEATURES"
```

The feature set is the documented compatible native SSR lane, never renderer `--all-features`. Repeat default-feature gates and separate browser hydrate/CSR/WASM consumer checks using each crate's documentation; retain live database/backend and native-platform lanes required by the intended support claim. Do not start dev listeners directly; use the managed devmetrics workflow for any preview. These plan checks are future work, not results of this task. Repeat the coverage accumulation below, then inspect each row of the generated per-crate report. Require **each crate** plus release integration to reach 90%; an aggregate `--fail-under-lines 90` alone can hide a failing crate. Resolve the facade's 0/0 treatment explicitly. Keep all generator `--check` proofs and third-party data parity checks.

```sh
TASK47_COVERAGE="$CARGO_TARGET_DIR/release-coverage.json"
bench coverage workspace cargo llvm-cov nextest --workspace --all-targets --locked --features "$TASK47_FEATURES" --no-report --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)'
bench coverage selected cargo llvm-cov nextest --workspace --all-targets --locked --features "$TASK47_FEATURES" --no-clean --json --output-path "$TASK47_COVERAGE" --test-threads 1 -E 'test(tcp_watchdog)'
python3 .agents/resources/publication-plan-00047/summarize-coverage.py "$TASK47_COVERAGE" --output "$CARGO_TARGET_DIR/per-crate-coverage.json"
```

The second invocation retains counters and writes the report; do not combine `--no-clean` with `--no-report`. The summarizer reports gaps and does not silently mark a gate passed.

4. Authenticate interactively after the release gates and ownership checks. Keep the token out of shell history, artifacts and recorder arguments:

```sh
cargo login --registry crates-io
```

5. Publish serially in this exact order. The loop stops on any failure. At each iteration inspect the file list and generated archive, run the normal package build and a dry-run, upload, then verify the indexed version. Confirm downloaded archive hash/content, repository/docs links and an independent registry-only consumer before advancing. Inspect the intended package/features instead of adding `--no-verify` or `--allow-dirty` to bypass a release failure.

```sh
set -e
for crate in \
  stackpress-leptos \
  stackpress-framework-language \
  stackpress-sql \
  stackpress-lib \
  stackpress-idea-parser \
  stackpress-sql-sqlite \
  stackpress-sql-postgres \
  stackpress-sql-mysql \
  stackpress-server \
  stackpress-idea-transformer \
  stackpress-server-http \
  stackpress-cli \
  stackpress-framework-view \
  stackpress-framework-csrf \
  stackpress-framework-email \
  stackpress-idea \
  stackpress-framework-schema \
  stackpress-framework-sql \
  stackpress-framework-admin \
  stackpress-framework-session \
  stackpress-framework-api \
  stackpress-framework \
  stackpress
do
  bench other selected cargo package -p "$crate" --list --locked
  bench other selected cargo package -p "$crate" --locked
  # Review target/00047-release/package/$crate-0.1.0.crate before continuing.
  bench other selected cargo publish -p "$crate" --dry-run --locked --registry crates-io
  bench other selected cargo publish -p "$crate" --locked --registry crates-io
  cargo info --registry crates-io "$crate@0.1.0"
  # Verify the archive and a fresh registry-only consumer before advancing.
  printf 'Type verified after completing those checks: '
  read -r TASK47_VERIFIED
  test "$TASK47_VERIFIED" = verified
done
```

`cargo info` may initially fail while the index propagates: stop, wait, and verify; resume at the first unpublished package. A registry-only consumer must use `NAME = "0.1.0"` with no local paths/patches, native SSR plus separate applicable browser features, and the documented MSRV. Include facade feature/re-export and generated-output compilation. Verify account ownership after the first successful upload; create the immutable release tag and release notes as part of the approved release process. A successful upload is public even if a later crate fails, so retain the upload receipt and resume from actual registry state rather than rerunning blindly.

6. For a bad release, stop the remaining order. Record the version and affected consumers; publish a corrected new version and bump affected dependents/facade. `cargo yank --registry crates-io --vers 0.1.0 CRATE` removes that version from normal new resolution; it is **not deletion** and does not remove existing downloaded copies or lockfile use. `cargo yank --undo --registry crates-io --vers 0.1.0 CRATE` reverses a mistaken yank. Published bytes/version identifiers cannot simply be overwritten. Confirm current exceptional deletion policy with the registry if ever needed; do not base rollback on deletion. Remove the task target after recording all evidence.

## Interim alternative: Git dependencies

After the owner pushes an approved commit/tag, consumers can use a repository dependency without waiting for crates.io preparation:

```toml
[dependencies]
stackpress = { git = "https://github.com/stackpress/rust", rev = "<pushed-commit-sha>", features = ["server"] }
# Or select a component package from the same repository/revision:
stackpress-lib = { git = "https://github.com/stackpress/rust", tag = "<pushed-release-tag>" }
```

Choose exactly one of `rev`, `tag` or `branch` per dependency. A pushed immutable revision/tag gives reproducibility; a branch follows subsequent updates when the consumer refreshes its lock. Align direct component dependencies to the same revision as the facade to avoid duplicate type versions. Git checkout includes sibling source/schema files, but this does not resolve publication coverage or licence decisions, and downstream crates.io packages cannot depend on Git-only unpublished crates.

Local evidence only: origin is `https://github.com/stackpress/rust.git`; stored `origin/main` is `8dd37cb`, and local `71c4118` is seven commits ahead of that stored tracking ref. The coordinator states local main is ahead of GitHub and nothing has been pushed. No fetch/network was made, so the remote's live state was not verified. A local SHA is unusable to external Git consumers until that object is actually pushed. No tag, commit or push was made by this task.

## Receipt and remaining decisions

This is a planning/evidence deliverable for coordinator and owner review. Only this task document and its linked raw resource folder were added. All previously tracked manifests, locks, licence files, Rust source and other tracked files remain unchanged. No commit, push, publish, login, install or network operation was made. No development listener was started.

Permanent recorder store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. The [copied task record metadata](../../../resources/publication-plan-00047/benchmark-records.json) and [provenance](../../../resources/publication-plan-00047/provenance.json) retain commands/environment, tool versions and report/archive hashes; canonical `.log` files remain in that store. Durations below are end-to-end **command seconds / observed Cargo build-stage seconds**; “not observed” is not zero.

| Record ID | Outcome and scope | Seconds (command / Cargo stages) |
| --- | --- | --- |
| `20261110T052202.465173Z-c26257bdaca6` | passed: Fresh target clean | 0.155 / not observed |
| `20261110T052218.971750Z-7b8cfbd034e6` | passed: Fresh coverage clean | 1.331 / not observed |
| `20261110T052321.095290Z-e85ab646da66` | passed: Packaging audit completed; 19 package and five dry-run child failures recorded | 19.580 / not observed |
| `20261110T052231.966992Z-ffc853c892ff` | failed: 1,244 tests passed; wrapper failed on incompatible isolated-watchdog flags; initial export had no file summaries | 1129.352 / 343.000 |
| `20261110T054210.214595Z-3c93c9ad69ed` | passed: Targeted continuation: one watchdog passed, existing profiles retained and usable report exported | 15.112 / 2.150 |
| `20261110T054423.495383Z-435222790fc9` | passed: Final task-target cleanup | 5.888 / not observed |

Task-only counters: 0 → 6 attempts (5 passed, 1 failed); these are not global ledger totals. Main plus continuation coverage command time was 1144.463 s, with 345.150 s of visible Cargo stages. Nested compiler output can be captured by tests, so the latter is incomplete build evidence, not compiler CPU time or a complete compilation total. The package wrapper's success means its audit finished, not that all child commands passed.

Sampled target peak: 7,880,978,432 bytes (7.34 GiB), below the 12 GiB cap; the sample is not a continuous maximum. The throwaway copy and `target/00047` were deleted after report/archive evidence was retained. Benchmark records remain. Review checked inventory completeness, dependency ordering/cycles, file-boundary ownership and runbook shell syntax; no Rust implementation/refactoring was authorized.

Agent Workspace validation retained the **146 inherited errors and 43 inherited warnings**, adding only the preferred-length warning for this coherent under-500-line plan (389 lines). No new hard errors or broken links were introduced. The inherited diagnostics were preserved because repairing unrelated KB references is outside this plan-only task. The release loop and all shell examples passed `zsh -n`; inventory and dependency-order checks covered 53/53 original root packages and 24/24 original candidates. G59 now has 52 root packages and 23 candidates.

The next owner decisions are release scope, licence declarations, initial version policy, the test packaging boundaries, and facade line-floor treatment. A separately released implementation task should close the five coverage gaps and packaging/metadata issues, then run actual release integration before publication.


## Coordinator review

Reviewed 2026-11-10 and committed as a plan only. No manifest, lock, licence or source changes. The raw 5.5 MB LLVM export was kept out of git, as described above. Owner decisions are still needed before any release work: the release scope (the 23 candidates), lockstep 0.1.0 versioning, licence declarations, how to treat the 0/0 facade, and fixing the five crates below 90% plus retained test packaging boundaries.

## Task 00050 release preparation and owner runbook

Task [00050](00050-release-manifests.md) implements the approved 23-candidate scope
at lockstep `0.1.0`. The workspace default remains `publish = false`; each candidate
explicitly opts in. Internal candidate normal/build/optional/target dependency paths
now also declare `version = "0.1.0"`, preserving their feature options. Earlier-order
dev-dependencies are versioned; eight later-order dev edges and unpublished example
dev-dependencies stay path-only. Normalized-archive checks prove Cargo strips them.
The [exact edges and provisional test exclusions](../../../resources/release-manifests-00050/dev-cycle-policy.json) are recorded for merge reconciliation. Examples and consumers remain unpublished.

The owner clarified G60 through the coordinator on 2026-11-11: **the facade uses the
Stackpress License**. Root `LICENSE` stays unchanged. It and the ten framework copies
are declared with `license-file = "LICENSE"`. The other twelve candidates declare
Apache-2.0 and ship byte-identical copies of the parser's Apache `LICENSE`. No alternate
facade Apache file is created. Lucide/Feather, frui and Nodemailer notices are retained.

Manifest, licence, README, repository, keyword/category and package-allowlist blockers
are implemented in Task 00050. Twelve new crate READMEs pass twelve actual doctests.
The eight selected category slugs are confirmed in the [cached registry source map](../../../resources/release-manifests-00050/category-sources.json); no unconfirmed slug is used.
The [package evidence](../../../resources/release-manifests-00050/packages.json) records
file counts, bytes, licence/metadata checks and file-reference failures. Both normal-lockfile workspace package passes and all five leaf dry-runs passed.
The [actual per-crate archive sizes and counts](../../../resources/release-manifests-00050/package-sizes.md) replace the historical estimates for release verification; every archive and uncompressed payload is below 10 MB. Test self-containment and coverage remain separate
worker deliverables; this lane does not close those blockers.

**G61 interpretation:** build the default, every one of the 79 declared facade features
individually, and the compatible full native SSR set. This is not a `2^N` powerset.
Renderer hydrate/CSR/SSR selections remain separate and `--all-features` is forbidden
on renderer crates. The consumer imports all 87 advertised public re-export paths;
the default normal tree must contain only `stackpress-lib` among Stackpress dependencies.
Run all three facade requirements with `python3 tools/release/check_facade.py`.
The [tooling guide](../../../../tools/release/PACKAGING.md) describes the package and
facade proofs and the temporary exact test-reference pending list.

The owner runs this preparation sequence from a clean, integrated checkout using
Cargo 1.96 and the already installed tools. It performs no registry writes:

```sh
export RUSTC_WRAPPER=sccache SCCACHE_CACHE_SIZE=20G
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/target/release-0.1.0"
export CARGO_LLVM_COV_TARGET_DIR="$CARGO_TARGET_DIR/llvm-cov"
# Point at an already installed Playwright module; this performs no installation.
export BLOG_PLAYWRIGHT=/absolute/path/to/installed/playwright
RELEASE_STORE=/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks
python3 .agents/scripts/build-benchmark.py --store "$RELEASE_STORE" run \
  --label '0.1.0 integrated release gate' --kind test --scope workspace --cache unknown \
  -- python3 tools/release/gate.py
```

The gate's package driver runs `cargo package --workspace` excluding every one of
the 29 non-candidates, first with `--no-verify`, then with verification. It records
actual archive bytes and counts, checks normalized path-only dev stripping, and runs
`cargo publish --dry-run --locked` for lib, sql, leptos, idea-parser and framework-language.
The five initial offline probes required HTTP; authorized anonymous registry reads
made all five final probes pass without authentication or upload. Workspace packaging
stays offline and retains archive lockfiles; `--exclude-lockfile` is not used.
Do not use `--pending-tests` for the integrated release gate. Resolve per-crate coverage in the other worker's receipt.

After all integrated checks, coverage, names, registry ownership and token permissions
are confirmed, the owner may explicitly authorize publication. Follow steps 4–5 of
the serial publication runbook above (the complete 23-package order is unchanged).
Those interactive login/upload commands are instructions for that future owner action;
Task 00050 runs neither login nor any publish without `--dry-run`.
