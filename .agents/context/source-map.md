# Source map for the Rust implementation

Start with [the release crate inventory](source-structure.md) for all 23 library/package owners, features and facade paths. Paths here are relative to this checkout; source and manifests stay in place. Root membership comes from [Cargo.toml](../../Cargo.toml), not from directory naming.

## Locate the semantic owner

| Work | Open first |
| --- | --- |
| Values, parsing, events, priority queues, templates | [lib source](../../crates/lib/src), [lib tests](../../crates/lib/tests) |
| Startup/config/plugins/lifecycle or buffered HTTP | [server source](../../crates/server/server/src), [server docs](../../crates/server/server/docs/README.md) |
| Supplied-IO HTTP/1 host, TLS, deployment, assets | [HTTP source](../../crates/server/http/src), [HTTP docs](../../crates/server/http/docs/README.md) |
| Parser reuse/grammar | [parser source](../../crates/idea/idea-parser/src), [extraction provenance](../../crates/idea/idea-parser/PROVENANCE.md) |
| Composition/generation/output ownership | [transformer source](../../crates/idea/idea-transformer/src), [Idea wrapper](../../crates/idea/idea/src) |
| Query/dialect/transactions versus generated persistence | [SQL source](../../crates/sql/sql/src), [framework SQL](../../crates/framework/framework-sql/src) |
| Shared components versus host page policy | [Leptos source](../../crates/leptos/src), [framework view](../../crates/framework/framework-view/src) |
| Schema/source families and emitted code | [schema source](../../crates/framework/framework-schema/src), [admin codegen](../../crates/framework/framework-admin/src/codegen) |
| Account, API, CSRF, email and aggregate composition | [framework crates](../../crates/framework), [aggregate native composition](../../crates/framework/framework/src/native/mod.rs) |
| Application-owned CLI | [CLI source](../../tools/cli/src), [capability-host example](../../examples/capability-host) |

Trace declaration/config → semantic owner → generator/runtime producer → generated output → consumer → adapter → verification. Fix the producer of generated code and use its official generator plus freshness check. Corresponding crate `docs/`, `tests/`, `examples/` and `proofs/` are implementation-specific evidence; compilation alone does not establish every platform/provider claim.

## Other root workspace members

Every row is unpublished (`publish = false`, directly or inherited), with no facade re-export. This table plus the 23 release rows in source-structure covers all root packages. Features list non-`default` keys; `(default)` identifies selected defaults.

| Manifest / path | Package | Responsibility | Local features |
| --- | --- | --- | --- |
| [examples/capability-host](../../examples/capability-host/Cargo.toml) | `stackpress-example-capability-host` | In-memory profiles application proving compiled Stackpress plugin composition | `experimental` |
| [examples/generated-profile/codegen](../../examples/generated-profile/codegen/Cargo.toml) | `stackpress-example-profile-codegen` | Independent pre-build codegen host for the generated Profile example | `profiles` (default) |
| [examples/generated-profile/app](../../examples/generated-profile/app/Cargo.toml) | `stackpress-example-generated-profile` | Generated public Profile contracts and one capability implementation | `profiles` (default) |
| [examples/sql-profile](../../examples/sql-profile/Cargo.toml) | `stackpress-example-sql-profile` | Explicitly installed SQLite profiles application with direct, CLI and buffered HTTP parity | none |
| [examples/profile-ui/contracts](../../examples/profile-ui/contracts/Cargo.toml) | `stackpress-example-profile-ui-contracts` | Client-safe Profile source reuse and typed search wire contracts | none |
| [examples/profile-ui/shared](../../examples/profile-ui/shared/Cargo.toml) | `stackpress-example-profile-ui-shared` | One Profile component for SSR, hydration and packaged CSR | `ssr`, `hydrate`, `csr` |
| [examples/profile-ui/web](../../examples/profile-ui/web/Cargo.toml) | `stackpress-example-profile-ui-web` | Send page loader over the existing generated Profile capability | `demo-customization`, `sqlite` |
| [examples/profile-ui/client](../../examples/profile-ui/client/Cargo.toml) | `stackpress-example-profile-ui-client` | Host-free hydration and Tauri CSR entrypoints | `hydrate`, `csr` |
| [examples/profile-ui/native](../../examples/profile-ui/native/Cargo.toml) | `stackpress-example-profile-ui-native` | Permitted local Profile search with finite cancellation | `app`, `demo-customization`, `review-preview` |
| [examples/profile-ui/gateway](../../examples/profile-ui/gateway/Cargo.toml) | `stackpress-example-profile-ui-gateway` | Finite example-only HTTP/1 gateway for the shared Profile UI | `demo-customization`, `sqlite` |
| [examples/review-preview](../../examples/review-preview/Cargo.toml) | `stackpress-example-review-preview` | Debug-only Slice 8 owner review host | `review-preview` |
| [examples/ui-stream/shared](../../examples/ui-stream/shared/Cargo.toml) | `stackpress-example-ui-stream-shared` | Shared in-order SSR demonstration | `ssr`, `hydrate` |
| [examples/ui-stream/client](../../examples/ui-stream/client/Cargo.toml) | `stackpress-example-ui-stream-client` | Streamed replay and real hydration admission | `hydrate` |
| [examples/ui-stream/web](../../examples/ui-stream/web/Cargo.toml) | `stackpress-example-ui-stream-web` | Real Hyper duplex streamed SSR proof without listeners | none |
| [examples/ui-gallery/shared](../../examples/ui-gallery/shared/Cargo.toml) | `stackpress-example-ui-gallery-shared` | Slice 8 bounded shared UI proof | `ssr`, `hydrate`, `csr` |
| [examples/ui-gallery/client](../../examples/ui-gallery/client/Cargo.toml) | `stackpress-example-ui-gallery-client` | Slice 8 bounded shared UI proof | `hydrate`, `csr` |
| [examples/ui-gallery/web](../../examples/ui-gallery/web/Cargo.toml) | `stackpress-example-ui-gallery-web` | Slice 8 bounded shared UI proof | none |
| [examples/ui-preferences/shared](../../examples/ui-preferences/shared/Cargo.toml) | `stackpress-example-ui-preferences-shared` | Finite Slice 8 preferences proof | `ssr`, `hydrate`, `csr` |
| [examples/ui-preferences/client](../../examples/ui-preferences/client/Cargo.toml) | `stackpress-example-ui-preferences-client` | Finite Slice 8 preferences proof | `ssr`, `hydrate`, `csr` |
| [examples/ui-preferences/web](../../examples/ui-preferences/web/Cargo.toml) | `stackpress-example-ui-preferences-web` | Finite Slice 8 preferences proof | none |
| [examples/generated-workspace/tool](../../examples/generated-workspace/tool/Cargo.toml) | `generated-workspace-tool` | Independent app-owned generator for the three-model data proof | `database-tools` |
| [examples/generated-workspace/app](../../examples/generated-workspace/app/Cargo.toml) | `generated-workspace-app` | Real generated three-model native data consumer | `matrix`, `names` |
| [examples/generated-workspace/contracts](../../examples/generated-workspace/contracts/Cargo.toml) | `generated-workspace-contracts` | Pure consumer of the exact generated native application's contracts | none |
| [examples/data-quickstart](../../examples/data-quickstart/Cargo.toml) | `data-quickstart` | Idea-to-SQLite getting-started example with explicit schema history | `native`, `evolved`, `expanded` |
| [examples/ui-format](../../examples/ui-format/Cargo.toml) | `stackpress-example-ui-format` | Finite shared rich-content and deterministic formatting proof | `ssr`, `hydrate`, `csr`, `rich`, `format` |
| [examples/generated-admin/tool](../../examples/generated-admin/tool/Cargo.toml) | `generated-admin-tool` | Generated admin tool proof | `format` |
| [examples/generated-admin/shared](../../examples/generated-admin/shared/Cargo.toml) | `generated-admin-shared` | Generated admin shared proof | `format`, `ssr`, `hydrate`, `csr` |
| [examples/generated-admin/client](../../examples/generated-admin/client/Cargo.toml) | `generated-admin-client` | Generated admin client proof | `format`, `hydrate`, `csr` |
| [examples/generated-admin/web](../../examples/generated-admin/web/Cargo.toml) | `generated-admin-web` | Generated admin web proof | `review-preview`, `format` |

`profile-ui/native` contains the Tauri proof, while its gateway is example-only. `review-preview` is debug review tooling. Neither establishes current blog desktop support. UI examples keep shared source and separate client/web artifacts; generated-workspace and generated-admin keep independent pre-build tools and generated consumers.

## Nested workspaces and consumers

These manifests declare their own `[workspace]`, or belong to the blog workspace rather than the root workspace. They are unpublished and have no facade re-export. Each standalone workspace owns its lock; blog members share the blog workspace lock. `examples/leptos-independent` is explicitly excluded by the root manifest; it proves use without a Stackpress host. API checkpoint tool/web/client each declare a standalone workspace.

| Manifest / path | Package | Responsibility | Local features |
| --- | --- | --- | --- |
| [crates/framework/framework/consumers/api-disabled](../../crates/framework/framework/consumers/api-disabled/Cargo.toml) | `slice11-api-disabled` | api-disabled dependency/registration consumer | `api` |
| [crates/framework/framework/consumers/build-host](../../crates/framework/framework/consumers/build-host/Cargo.toml) | `slice11-build-host` | build-host dependency/registration consumer | none |
| [crates/framework/framework/consumers/direct](../../crates/framework/framework/consumers/direct/Cargo.toml) | `slice11-direct` | direct dependency/registration consumer | `api` (default) |
| [crates/framework/framework/consumers/disabled](../../crates/framework/framework/consumers/disabled/Cargo.toml) | `slice11-disabled` | disabled dependency/registration consumer | none |
| [crates/framework/framework/consumers/facade](../../crates/framework/framework/consumers/facade/Cargo.toml) | `slice11-facade` | facade dependency/registration consumer | `api` (default) |
| [crates/framework/framework/consumers/full](../../crates/framework/framework/consumers/full/Cargo.toml) | `slice11-full` | full dependency/registration consumer | `api` (default) |
| [crates/framework/framework/consumers/headless](../../crates/framework/framework/consumers/headless/Cargo.toml) | `slice11-headless` | headless dependency/registration consumer | none |
| [crates/framework/framework/consumers/website](../../crates/framework/framework/consumers/website/Cargo.toml) | `slice11-website` | website dependency/registration consumer | `aggregate-browser`, `api`, `ssr`, `hydrate`, `csr` |
| [crates/framework/framework-email/consumers/direct](../../crates/framework/framework-email/consumers/direct/Cargo.toml) | `slice9b-consumer` | direct dependency/registration consumer | none |
| [crates/framework/framework-email/consumers/disabled](../../crates/framework/framework-email/consumers/disabled/Cargo.toml) | `slice9b-consumer` | disabled dependency/registration consumer | `plugins` |
| [crates/framework/framework-email/consumers/event](../../crates/framework/framework-email/consumers/event/Cargo.toml) | `slice9b-consumer` | event dependency/registration consumer | none |
| [crates/framework/framework-session/consumers/browser](../../crates/framework/framework-session/consumers/browser/Cargo.toml) | `slice9-account-browser` | browser dependency/registration consumer | `hydrate`, `csr` |
| [crates/framework/framework-session/consumers/direct](../../crates/framework/framework-session/consumers/direct/Cargo.toml) | `slice9a-consumer` | direct dependency/registration consumer | none |
| [crates/framework/framework-session/consumers/disabled](../../crates/framework/framework-session/consumers/disabled/Cargo.toml) | `slice9a-consumer` | disabled dependency/registration consumer | `session` |
| [crates/framework/framework-session/consumers/facade](../../crates/framework/framework-session/consumers/facade/Cargo.toml) | `slice9a-consumer` | facade dependency/registration consumer | none |
| [crates/framework/framework-session/consumers/plugin](../../crates/framework/framework-session/consumers/plugin/Cargo.toml) | `slice9a-consumer` | plugin dependency/registration consumer | none |
| [examples/api-checkpoint/client](../../examples/api-checkpoint/client/Cargo.toml) | `api-checkpoint-client` | Hydrate/CSR browser entry | `aggregate-browser`, `hydrate`, `csr` |
| [examples/api-checkpoint/tool](../../examples/api-checkpoint/tool/Cargo.toml) | `api-checkpoint-tool` | App-owned source-family generator | none |
| [examples/api-checkpoint/web](../../examples/api-checkpoint/web/Cargo.toml) | `api-checkpoint-web` | Native API/account/admin host | none |
| [examples/blog/client](../../examples/blog/client/Cargo.toml) | `blog-client` | Separately compiled hydration and CSR browser entries | `hydrate`, `csr` |
| [examples/blog/plugins/app](../../examples/blog/plugins/app/Cargo.toml) | `blog-app` | Blog plugin with native pages and portable Leptos views | `ui`, `server`, `ssr`, `hydrate`, `csr` |
| [examples/blog/plugins/assets](../../examples/blog/plugins/assets/Cargo.toml) | `blog-assets` | Blog source and compiled browser asset plugin | none |
| [examples/blog/plugins/store](../../examples/blog/plugins/store/Cargo.toml) | `blog-store` | File SQLite connection plugin | none |
| [examples/blog/tool](../../examples/blog/tool/Cargo.toml) | `blog-tool` | Independent source-model generator for the Stackpress blog boilerplate | `native-proof`, `server` |
| [examples/blog/web](../../examples/blog/web/Cargo.toml) | `blog-web` | File SQLite Stackpress blog example and finite application commands | `server` (default), `extension-proof` |
| [examples/blog/xtask](../../examples/blog/xtask/Cargo.toml) | `blog-xtask` | Browser builds and live development for the Stackpress blog example | none |
| [examples/leptos-independent](../../examples/leptos-independent/Cargo.toml) | `leptos-independent` | Independent components/rendering consumer | `ssr`, `hydrate`, `csr`, `async`, `rich`, `format`, `stream` |
| [fuzz](../../fuzz/Cargo.toml) | `stackpress-fuzz` | Finite parser/CSV/query-form fuzz targets | none |

### Blog layout

[Blog workspace manifest](../../examples/blog/Cargo.toml) owns `tool`, `web`, `client`, `plugins/app`, `plugins/assets`, `plugins/store`, and `xtask`. The [README](../../examples/blog/README.md) describes the copyable application and generation commands. Its plugin crates are `blog-app` (native pages and shared Leptos views), `blog-assets` (authored/static and compiled browser assets) and `blog-store` (file SQLite). `web/src/client.rs` selects registration functions. The thin client selects UI plus one renderer without native store/assets dependencies.

`tool` generates from the app's schema and framework `.idea` imports into `.build/app-generated/`; generated output is not the durable owner of a fix. `xtask` supplies `cargo xtask build-browser` and the live-reload dev loop; the [Cargo alias](../../examples/blog/.cargo/config.toml) selects `blog-xtask`. Any command that starts a development listener, including `cargo xtask dev`, must use an available session installation of the devmetrics skill, as required by [root AGENTS.md](../../AGENTS.md).

### Verification tools

- [tools/cli](../../tools/cli) is the release CLI library, not a generic stock executable.
- [tools/release](../../tools/release) owns the explicit release list, packaging/facade checks, package-test boundary and copy checks, dependency-licence snapshot, dry runs and [release gate](../../tools/release/gate.py).
- [tools/examples/blog](../../tools/examples/blog/README.md) holds contributor-only generation, storage, browser, graph, lock, copied-app and coverage proofs; these tools are separate from the copied boilerplate. Browser proofs use cached Playwright and in-process routing.
- [fuzz](../../fuzz/README.md) is a separate unpublished workspace with `idea_parser`, `admin_csv` and `lib_query_form` targets, its own lock, seed corpora and finite nightly cargo-fuzz workflow.
- [Agent scripts](../scripts) include workspace/provenance validation and the [build benchmark recorder](../scripts/build-benchmark.py). Record builds/proofs through the [benchmark workflow](../workflows/build-benchmarks.md).

For upstream TypeScript source locations, load [the historical source-provenance reference](../references/00044-preimplementation-source-provenance.md) only when tracing an original contract or capture; those sibling paths describe the canonical checkout layout and were not re-inspected for this sync. [Upstream contract router](upstream-stackpress.md) preserves the full imported APIs.
