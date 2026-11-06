# Blog example — clean-checkout verification

Status: **clean-copy gate passed; ready for coordinator review.** Follow-up to independent verification of merged b5e011f. The prior warm-state proof missed ignored prerequisites; its historical results below are superseded for reproducibility. No commit.

- **Scope:** only `examples/blog/README.md`, `proofs/{gate,locks,readme,nickname}.py` and this receipt. No Rust/app/framework behavior, manifests, locks, schema or license changes.
- **Gate prerequisites:** official generate-app/native generation, extension emit/check before native compilation, actual browser assets before the native HTTP asset test, explicit probe build before Playwright, and nickname preparation before freshness/browser checks. Each bootstrap failure stops dependent phases. The dedicated target directory exists before its first size sample.
- **Locks:** `locks.py check` reads local committed HEAD:Cargo.lock, compares root bytes and699 tuples, and checks the blog437-tuple subset; no resolution marker or Cargo resolution. One-shot seed markers remain restricted to the separate seed operation.
- **Copy-out:** README path setup is derived only from existing repo-local edges in the current checkout; path existence/containment and exact reversible replacement are asserted. No ignored review patch. Copied/source/root manifests and locks remain byte-identical except those copied path values.
- **Exact proof:** detached `b5e011f6d3fdaec7dce3da21b247df3bfd94b62f` at /private/tmp/blog-clean-b5e011f-01 plus scoped uncommitted patch SHA256 `70374e7af9d2c2a721d6101b9b5096840b23551ae0a14525ffe85ad8a8214dde`; no untracked source files. Both .build and targets were absent. The runner extracted and executed the README Verify shell block verbatim, with sccache/two jobs/incremental0/debug0 and locked/offline Cargo. Evidence: `.build/clean-copy/b5e011f-01/{source.json,fix.patch,verify.sh,gate}`.
- **Results:** all26 gate phases pass in one fresh run;49/49 native tests including the extension/Catalog/Date/cookie/asset regressions, fmt, native and separate-renderer Clippy -D warnings, Rustdoc -D warnings, doctests (0 examples), both byte/mtime freshness checks and native/browser graphs. Six public matrix cases, both authenticated flows and both regenerated nickname flows pass against real in-process HTTP/SQLite using cached Playwright only.
- **Coverage:** blog-tool 102/113 (90.27%); blog-web 2553/3568 (71.55%); blog-shared 3536/3705 (95.44%); shipped generated source/UI counted. Blog-client retains its G56 thin-entry/IO exemption, exercised by real browser tests; no coverage scope was narrowed for this follow-up. Raw LLVM export warnings, if present, remain in the recorder log.
- **Disk/cleanup:** maximum sampled phase footprint10.131GiB, below12GiB. Prior merged-lane targets were retired before the proof. This detached checkout's entire target and worktree were removed after preserving logs/reports/screenshots; no other lane's target was touched.
- **Benchmark:** `20261106T123127.526279Z-8c3a90e35648` passed; command wall1255.947899s versus completed Cargo stages1082.09s, not session elapsed. Persistent repository store `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; cold dedicated target, warm global sccache. No listener/install/network/resolution/commit/push.
- **Review:** dependency ordering, generated/native/browser responsibilities and source/lock side effects were checked against the actual clean run; README now has one exact Verify sequence and documents the prerequisite for direct narrow integration tests. Original G52/G56/G57 contracts and omitted delivery/desktop/MCP boundaries remain unchanged.
- **Closing audit:** `20261106T125657.183522Z-bd82c72484e0` passed; exactly the five approved files plus receipt changed, tested patch unchanged, root/source manifests/locks/schema and all25 upstream source files unchanged; disposable target/worktree absent. Inherited KB146 errors/43 warnings, no new errors (not clean validation). Follow-up through this audit:5 attempts/5 passed, 1 with compilation,0 root-workspace builds; command wall1262.177704s versus reported Cargo stages1082.09s. Receipt-only validation follows.

All earlier status statements below are dated provenance; this clean-copy result is the current reproducibility claim.

## Superseded checkpoints

Moved verbatim to [00037 superseded checkpoints](00037-blog-example-history.md).

## Historical design checkpoint and original evidence

The complete 2026-11-04 feature inventory, trimmed G52 layout, walkthrough, milestone/lock plan and historical evidence below are retained for source coverage. All statuses, M3-pending statements, old root hashes and old gate results in this historical section are superseded by the current receipt above. They are not current implementation claims.

---

## Historical checkpoint: blog boilerplate design

Status: **started; M0 scaffold implemented, generation proofs blocked by source-role normalization; stopped for coordinator review**.
Date: 2026-11-04. Implementer: Codex; coordinator: Claude; product decisions: owner.
Task 00036 is reserved for G50 M3. The coordinator released only the M0 files below; later milestones remain unreleased.

## Task summary and authority

Owner request: “can you make a blog example like on the original stackpress templates folder that showcases how to create a project (like a boilerplate example)?”
Design a copyable Rust web application that teaches importing the framework schema, selecting ordinary plugin functions, generating the application's models, using file SQLite, and sharing Leptos UI. Preserve source behavior under G42/G43 except the explicit G52 app changes; follow G47/G49/G50 and G51. M0 proves generation risk only, not a working website or implementation acceptance.

- Inspected Rust HEAD: `c8fc4d918b4e4ca40345264d6c35029be929a80f`; initial working tree clean.
- M0 release baseline: `9314c3e90afe8b5dc37be81f8b14ea2950761db3`, containing the accepted G52 decisions. Only this untracked receipt existed before M0 edits. No other trial branch is an input.
- `T` below means `/Users/cblanquera/server/projects/stackpress/stackpress/templates/blog`, read-only at `a71d683051ba8350fdd12d6b5a33f268fdcc285f`. All `T/...` citations are files in that exact source tree. Inspected all **25 tracked source/config/test files** outside `public/` and `client-source/`; neither output directory is source to copy.
- Rust paths below are relative to this worktree; shorthand `framework-*/...` expands to `crates/framework/framework-*/...`, and `framework/...` to `crates/framework/framework/...`. Existing source is evidence of an API, not a claim that its tests were rerun here. G50 M3 is pending in another worker; no inspection of that worker's branch, target output, or unmerged code informed this receipt.
- Read [context](../../../context/index.md), [programming](../../../workflows/rust-programming.md), [coordination](../../../workflows/coordinated-crate-development.md), [fast builds](../../../workflows/fast-build-and-test.md), [style](../../../context/rust-code-style.md), [quality](../../../context/testing-and-quality.md) and Agent File creation/ingestion rules. Stackpress router selected the local Rust workflow; ChrisAI documentation guided the walkthrough order.
- Current prerequisites: [G47 receipt](00026-g47-config-placement.md), [G49 receipt](00030-g49-schema-idea-files.md), [G50 design](00031-g50-generated-framework-models.md), [M2 receipt](00035-g50-m2.md), and [decisions](../decisions.md). Proposed behavior stays here, with no promotion to Context.

## Feature map

Legend: **E** = existing reusable Rust mechanism; **M3** = prerequisite promised by G50 M3, not available at this HEAD; **A** = small proposed app adaptation; **G** = named gap requiring omission/decision/separate framework work. A row can include several states when metadata exists but runtime/UI integration does not.

### Schema constructs

Every row in this table originates in `T/schema.idea`. Repeated annotations are grouped explicitly; field names, defaults, enum values, nullable/list distinctions, descriptions, examples and presentation props remain source data unless an owner-approved deviation is named below.

| Source construct | Rust equivalent, status and proving anchor |
| --- | --- |
| `use "stackpress/stackpress.idea"` | **E**: shipped `crates/framework/framework/stackpress.idea`, importing API then session schemas; `crates/idea/idea-transformer/src/source/package.rs` and aggregate `framework/src/native/commands/schema.rs::rust::package_loader`. Application loader wraps ProjectLoader with PackageLoader; do not flatten/copy the four framework models. |
| `PublishStatus`: DRAFT, REVIEW, PUBLISHED, ARCHIVED | **E** source enum normalization/codecs in `crates/framework/framework-schema/src/codegen/source_graph.rs` and `src/source/`; source store emitter `crates/framework/framework-sql/src/codegen/source.rs`. **M3** concrete select/admin roles. |
| Extend `Profile` with `articles Article[]`, `comments Comment[]`, both labeled | **E** G19 composition + generated inverse relation metadata; `crates/framework/framework-models/tests/extended/schema.idea`, `tests/extensions.rs`. **M3** the selected extended family must reach session/auth/API/admin/export; do not substitute default Profile. Stock signup still accepts only name/type/roles under Q50-SIGNUP. |
| Category model labels Category/Categories, icon sitemap, display `{{name}}` | **E** full metadata via source normalizer/emitter above. **M3** source admin display; app shell uses the owner's Lucide family rather than importing Font Awesome. |
| Category `id String @id @default("cuid()")`; `name String @searchable`; `slug String @unique`; `banner String?`; `description Text?` | **E** generated source schema/store/value providers: `framework-models/src/generated/source/profile.rs` is the existing CUID/default pattern; `framework-schema/src/source/`, `framework-sql/src/source/`. **M3** generated forms/list/detail. Preserve required messages on name/slug, field string/slug/url/textarea choices and image dimensions. |
| Category `active Boolean @default(true) @active`, `created Datetime @default("now()") @sortable`, `updated Datetime @default("now()") @timestamp @sortable` | **E** native source defaults, active filtering and timestamp writes in `framework-sql/src/source/{store,actions,event_defaults}.rs`; generated family tests `framework-models/tests/{native,sql}.rs`. **M3** switch/date renderers. |
| Category `catalogs Catalog[]`; `profile Profile @relation({local "profileId" foreign "id"})` | Catalog inverse is **E**. **G52 / A** removes the dangling Profile edge because categories are unowned; no profileId field/seed is added. Framework `check_relations` remains unchanged; a negative control must still reject reintroducing the invalid edge. |
| Catalog labels Catalog/Catalogs, icon link, display `{{category.name}} - {{article.title}}`; query `*`, `category.*`, `article.*`, `article.profile.*` | **E** source wildcard/path metadata and relation loading in `framework-schema/src/codegen/source_graph.rs`, `framework-sql/src/source/{select,query}.rs`; **M3** source display/admin integration. |
| Catalog `categoryId String @id`, `articleId String @id` | **E** compound identity/DDL/CRUD via source stores, `framework-models/tests/{sql,extensions}.rs`. Preserve both required diagnostics, relation-input URLs/templates, list/view relation templates. **M3** concrete lookup/template adapters; no synthetic Catalog ID. |
| Catalog `category Category` and `article Article`, explicit local/foreign mappings | **E** owning joins and FK definitions in `framework-sql/src/source/{store,statement,select}.rs`; **M3** corresponding generated admin pages. |
| Article labels Article/Articles, icon file, display `{{title}}`, query `*`, `profile.*` | **E** source descriptors and joins; **M3** source page/catalog output. |
| Article `id String @id @default("cuid()")`; `profileId String`; `title String @searchable`; `slug String @unique`; `banner String?` | **E** source CRUD/validation/query metadata. Required Profile/Title/URL Slug messages and profile relation field/filter URLs (`?json&q={{query}}`) retained. **M3** concrete source adapters. |
| Article `contents Text? @field.editor(...) @view.html` | **E** Idea composition retains metadata; underlying Text storage exists. **G-M0-PROPS / M3**: real native normalization currently rejects the editor props before emission. Rich editor/HTML display also need later proof. Preserve all14 options: history, font, size, format, paragraph, blockquote, style, color, highlight, text, textStyle, align, list, code. No metadata stripping or typed fallback. |
| Article `keywords String[]`, `tags String[]`, both `@default([])`, field/list/view taglist | **E** source arrays/defaults. **M3** source props warning=true, pill=true, className=`frui-mr-md` need compatible Leptos roles, not merely typed collection availability. |
| Article `references Hash? @default({})`, field metadata add="Add Reference", view metadata padding classes | **E** source Hash codec and defaults. **M3** source-compatible metadata UI; the current typed admin metadata adapter accepts string values, so do not claim all source Hash values are supported by it. |
| Article `status PublishStatus @default("DRAFT")`, required status, select options all four values | **E** composed enum/default data and reusable enum storage. **G-M0-PROPS / M3** native normalization rejects `field.select`'s options prop before emission; source-select support must retain these values. |
| Article `published Datetime?`, `active`, `created`, `updated` | **E** nullable date and the same active/default/timestamp/sort mechanisms as Category; **M3** date/switch UI. Published is editable, not automatically set by choosing PUBLISHED. |
| Article `profile Profile` on profileId→id, `comments Comment[]`, `catalogs Catalog[]` | **E** generated forward/reverse query metadata and eager relation data; **M3** admin/source contract projection. |
| Comment labels Comment/Comments, icon comment, display `{{comment}}`, query `*`, `article.*`, `profile.*` | **E** descriptors/query graph; **M3** source display/admin. |
| Comment `id String @id @default("cuid()")`, `articleId String`, `profileId String`, `comment Text?` | **E** source fields/CRUD. Preserve required article/profile messages, article/profile relation field/filter URLs, list/view templates, textarea and list/view text. **M3** source role adapters. |
| Comment `active`, `created`, `updated`, owning `article` and `profile` mappings | **E** same source defaults/active/timestamps/FKs/query joins; **M3** admin integration. No handwritten replacement Comment store. |
| All model/field `@label`, `@labels`, `@icon`, `@display`, `@query`, `@description`, `@examples` | **E** complete retention by Idea composition; full blog source-graph/native emission is pending G-M0-PROPS below. **M3** deliberately selected public UI projection. Preserve even copy/paste descriptions such as Article.profileId's “auth” wording; metadata cleanup is not this task. |
| Repeated `@list.clip({length 10 hellip true})`, `@field.string`, `.slug`, `.url`, `.textarea`, `.relation`, `.editor`, `.taglist`, `.metadata`, `.select`, `.datetime` | **E** source vocabulary/props metadata in `framework-schema/src/codegen/{source_roles,source_graph}.rs`. **M3 / G-UI** concrete availability, including compound-key relations; not a switch to the stricter typed generator. |
| Repeated `@list.string`, `.image` (20×20), `.date("m d, Y h:iA")`, `.template`, `.taglist`, `.text`; `@view.string`, `.image` (100×100), `.date`, `.template`, `.html`, `.taglist`, `.metadata`, `.yesno`, `.text`; `@filter.switch`, `.relation` | **E** metadata; **M3 / G-UI** source role emission, date-format semantics and all five placements. `framework-admin/docs/api/catalog.md` currently says rich/media/relation services unavailable; `source_roles.rs` sets source definitions available=false. A successful native schema generation is not successful admin generation. |

### Config sections and environment variants

`T/config/common.ts` owns every common section below. G47's concrete precedent is `examples/sql-profile/{src/config.rs,config/develop.json,config/build.json}` and its `tests/cases/configuration.rs`; the typed guide is `crates/server/server/docs/guides/typed-config.md`. Values belong in app JSON, computed/environment/template/native bindings in `src/config.rs`. No Rust config module is dynamically executed from JSON.

| Source section | Current Rust owner and proposed mapping |
| --- | --- |
| Config type, Cascade type, cwd/build/assets/modules constants | **E/A** typed app config and PathBufs; root-resolved `.build`, source `assets/`, generated `dist/`; CASCADE delete / RESTRICT update passed through existing DDL options. `modules` is a Node path, not a Rust dependency resolver. `framework/docs/api/configuration.md`, `framework-sql/src/source/maintenance.rs`. |
| `seed.database`, `seed.session`; `.env.sample` DATABASE_SEED/SESSION_SEED (`abc123`) | **E/A** read only explicitly in app config; native typed providers. Keep sample development seeds documented, not serialized to page snapshots. Existing `framework-session/src/{session.rs,auth/config.rs}` and SQL source constructors take the corresponding seed. No automatic dotenv dependency. |
| admin name Admin, base /admin, six menu items | **E/A/M3** `framework-admin` pages + app LayoutPanel/menu. Categories, Articles, Profiles, Auth, Apps, Sessions retain their search paths and match prefixes. Catalog/Comment are still generated models although absent from this static menu. M3 must wire source admin to the same family. |
| api expires = 365 days in milliseconds, scope `user` label/description | **E** `framework-api/src/config.rs::{ApiConfig,ApiScope}` retains milliseconds and source scope behavior. **M3** generated extended Profile/Application/Session collaborators. |
| GET /api/auth/search → auth-search, public, cors=true | **E/A** `framework-api/src/{config,endpoints,routes}.rs`; copied endpoint config, actual generated events after M3. Do not add credential admission to this public example. |
| GET /api/profile/search, /api/profile/detail/:id, /api/profile/get/:key/:value → profile-search/detail/get, app, scope user | **E/A/M3** same endpoint adapters with app family. Existing source API checks/any-match scopes stay unchanged. |
| GET /api/my/address and /api/my/files → profile-detail, session, scope user | **E/A/M3** preserve these illustrative aliases; they do not imply Address/File models or real address/file services. |
| api webhooks: auth-signout → POST http://localhost:3000/api/webhook, empty validity/data | **E/A** inline hook in `framework-api/src/webhooks/{mod,fetch,reqwest}.rs`, priority -200. No receiver exists in T. Local runnable example uses an explicitly configured app-owned sink/recording Fetch; endpoint port is substituted from managed origin only when opting into a real receiver. See G-WEBHOOK. |
| auth base /auth, redirect /, 2fa issuer Stackpress, default roles USER | **E/M3** `framework-session/src/auth/{config,plugin,events}.rs`, `framework-session/src/auth/pages/mod.rs`, `src/presentation.rs`, shared `src/views/`. M3 changes model collaborators, not auth policy. |
| auth menu username/email/phone signin, signup; footer terms-of-use/privacy-policy | **E/A** shared account views and app presentation config (`framework-session/docs/guides/shared-ui.md`). Preserve labels/routes; no new legal-document content promised. |
| auth password min8/max32/upper/lower/number/special | **E** existing auth config/assertion helpers and tests; sample credentials created directly by generated auth-create remain the source fixture, not a new signup policy. |
| brand Stackpress, /logo.png, /icon.png, /favicon.ico | **E/A** public Presentation/Leptos head props. Author/source declared assets for Rust build; do not copy T/public output. Branding artwork choices need no framework work. |
| client lang/module/package/revisions/build/tsconfig/prettier | **E/A/M3** CompiledClient and independent app generator replace dynamic JS module import. `framework/src/native/{client,mod}.rs`, `framework-models/src/bin/generate.rs`. Native revision history/output paths are explicit; JS language/compiler/Prettier knobs have no Cargo equivalent. |
| cookie path /; csrf name csrf | **E** existing server cookie policy and `framework-csrf/src/{lib,plugin}.rs`; only source-selected processes validate CSRF. No new global request guard. |
| database seed/migrations/schema/populate | **E/A** `framework/src/native/commands/sql.rs`, `framework-sql/src/{setup.rs,source/maintenance.rs}` and generated stores; app supplies Scripts/History, file paths and ordered event/data records. Generate does not install/migrate/populate. Detailed seed ledger below. |
| email SMTP example host smtp.example.com:587, secure=false, username/password | **E/A** `framework-email/src/config.rs`, `docs/guides/transports.md`. Runnable local default uses existing JSON transport; SMTP object retained as an opt-in documented example, with no delivery acceptance or network in automated proofs. See G-EMAIL. |
| language key locale, default en_US; EN and TH; two translations each | **E/A** `framework/src/native/language.rs` source cookie/query/prefix selection; root-local `crates/leptos/src/language.rs`. Retain en_US/th_TH and exact Sign In→Signin / Home Page→Home Page or Home Pagesss strings. Do not reinterpret them as normalized preference locales. See G-LANGUAGE. |
| server host127.0.0.1/port3000/cwd | **E/A** source config + `crates/server/http/{src/,examples/deployment.rs}`. Runtime command overrides the example port with devmetrics `{port}`, and derives the matching origin. Never start port3000 directly. |
| session key session, seed; ADMIN/MANAGER/USER/GUEST route grants | **E/M3** `framework-session/src/{session.rs,permissions.rs}` and auth-server composition. Preserve original grants: common /, /articles/**, /auth/**, /api/**; ADMIN /admin/**; MANAGER /admin/auth/**; USER no admin; GUEST only auth signup/signin variants/signout plus /mcp. No default-deny beyond selected source policy. |
| terminal label [BLOG], idea path | **E/A** CLI output label/app config; Rust aggregate root is `cli.idea` (not a new `terminal` root), per `framework/docs/api/configuration.md`. `tools/cli/src/` parses commands and `-b`; app supplies Environment argv/cwd. |
| view noview=json, base=/, notify bottom-right/5000/dark with click/hover/drag/progress settings | **E/A/G** source render skip in `framework/src/native/view.rs`; app LayoutPanel, notifications/Toast in `crates/leptos/src/{components/layout.rs,components/controls.rs,components/overlay.rs,state.rs}`. React-toastify options are not a drop-in Rust type; document unsupported drag/progress/timing behavior rather than advertise it. |
| mcp common object | **G-MCP** six tools and source inconsistencies recorded below; no shipped Rust MCP registrar/crate found in root Cargo.toml or crates/tools source inventory. Config metadata is not an implementation. |

The variants themselves have observable differences that must remain visible in the README reference:

| Source file | Rust equivalent / gap |
| --- | --- |
| `T/config/develop.ts` | **E/A** `config/develop.json` + config::shared + compiled app builder; mode development/build dir. Includes mcp. Source Vite clientRoute, templates, React optimizeDeps, Frui/Stackpress/Uno CSS and watch settings become explicit Leptos builds/assets (G-ASSETS). |
| `T/config/build.ts` | **E/A** `config/build.json`; production build mode, assets/client/page destinations become dist + native SSR binary. It includes csrf, excludes mcp, and builds assets before serving. |
| `T/config/production.ts` | **E/A** `config/production.json`; explicit compiled assets plugin and source render provider, includes mcp. Source retains the same Vite/internal/static access grants as develop; no silent security cleanup. Rust has no Vite route inventory to expose. |
| `T/config/client.ts` | **E/A/M3** finite generator destination switch equivalent to client-source output; native source and portable UI partitions compiled separately. Source changes lang to ts, output/module/package to client-source, omits csrf and MANAGER access; record these differences, not an invented identical config. |
| develop/production access append lists | **A** source grants /@vite/client, /@react-refresh, /@fs/**, /node_modules/**, /__uno.css, /plugins/**, /react.svg plus GET assets/client/images/styles/favicon.ico/favicon.png. Rust grants the corresponding compiled assets paths; obsolete Node routes have no handlers. This is a platform mapping, not route arbitration. |
| `T/config/desktop.ts` | **G52 / omitted first release**: original app id io.stackpress.blog/name/version, 1200×800/min800×600/title and local app-data blog.sqlite are historical reference only. No Rust desktop config/package is planned in this release. |
| `T/config/desktop.routes-exact.ts`, `desktop.routes-wildcard.ts` | **G52 / omitted**: source all-routes(default empty list), exact / + /articles/:slug, wildcard / + /articles/** + /api/** retained in this feature inventory only. No desktop fixture files are created. |
| desktop nested server/window/security/menu/updater/data/build/package sections (`T/config/desktop.ts`) | **G52 / omitted** source inventory: runtime=http; server127.0.0.1:0/open=/; app name/title Stackpress Blog/version1.0.0; externalNavigation=deny, allowDevTools=true; menu.enabled=true; updater.enabled=false/menu=true; data.directory=userData/database=blog.sqlite; .build/desktop main.js/preload.js/manifest.json; electron-builder output .build/releases. A later separate task can assess Tauri; no current build/package parity is claimed. |

### Plugins, pages, views, hooks and helpers

The original plugin array is a compile-time contribution inventory in Rust, not a runtime plugin loader. Registration functions remain `fn(&mut ServerBuilder) -> Result<()>`; generated model traits are collaborators already owned by G50, not an extra plugin contract.

| Source path / feature | Rust equivalent and disposition |
| --- | --- |
| `T/package.json`: app, store, stackpress, stackpress-ai, stackpress-desktop | **E/A/M3/G** compiled app/store/aggregate functions; MCP unsupported and desktop skipped under G52. Existing shape: `framework/consumers/support/native.rs` and `framework/docs/getting-started.md`. Preserve Config→Listen→Route and priority/FIFO/Abort; no route uniqueness system. |
| `T/plugins/store/{connect,plugin}.ts` | **E/A** SQLite Engine service using `crates/sql/sql-sqlite/` and `examples/sql-profile/src/`; default `.build/blog.sqlite`, explicit setup only. Original PGlite is intentionally replaced per owner/G46; MySQL/PostgreSQL remain optional documented adapter choices, not blog proof requirements. No PGlite connector/dependency. |
| store `connection.before` query console logging | **G(a)** no equivalent universal Engine before callback identified (`crates/sql/sql/src/engine.rs`). Omit with note; normal app diagnostics can log its explicit commands. Exact interception of every generated query would be a separate SQL hook task, not a fake callback. |
| `T/plugins/assets/plugin.ts` production-only fallback file/MIME streaming | **E/A** existing `crates/server/http/src/assets.rs` AssetBundle and `crates/server/http/examples/deployment.rs` explicit bundle registration; app builds manifest from authored assets. State that arbitrary T/public directory lookup is replaced by the existing Rust asset bundle contract; no new framework file-serving policy. |
| `T/plugins/app/plugin.ts`: home/article route plus later view contributions | **E/A** app plugin queues ordinary handlers and source render adapters; `framework/src/native/view.rs` Provider and `framework-view/src/leptos/`. Handler and view contributions can share a route. |
| `T/plugins/app/pages/home.ts` | **G52 / A** deliberately changes the ineffective top-level status assignment into a real eq.status=PUBLISHED filter in the future app home handler. Add published descending only if sort absent, raw-emit article-search, then view props. Store/framework query behavior stays unchanged (`packages/stackpress-sql/src/transform/{events/search,store/select}.ts`; Rust `framework-sql/src/source/query_input.rs`). Test published inclusion and draft/review/archived exclusion, preserving caller sort. README must cite G52 for this intentional template deviation. Home handler is outside released M0. |
| `T/plugins/app/pages/article.ts` | **E/A** nested resolve article-search with eq.slug, early return on non200/missing/empty, then comment-search eq.articleId; merge first result with comments or []. Do not add status=PUBLISHED, change missing-article semantics into an invented error, or make comments failure fail the page. |
| `T/plugins/app/plugin.ts` response→error hook | **E/A** ordinary lifecycle in `crates/server/server/src/lifecycle.rs` and raw nested dispatch. App translates source res.error check to the existing typed response/error offer; prove response/error/Abort ordering and avoid recursively reentering top-level handling. No global protection/hardening. |
| `T/plugins/app/pages/error.ts` | **E/A/G-ERROR** skip terminal requests (false/Abort), nonGET and existing body; server.mode and status response; snippets skip first frame/nonabsolute/missing files, attach previous/main/next line plus a caret when coordinates permit. json opts out; .js uses console.log and .css a comment, both dropping stack and preserving status; otherwise render HTML with view props and `me` session. Existing Rust Error source locations/frames differ from JS Trace. See gap for snippets. |
| `T/plugins/app/views/home.tsx`: Body/Head/Page | **E/A** shared Leptos body/head/LayoutPanel + leptos_meta; topbar/brand, publication hero, Published/With Visuals/Latest metrics, featured first row/empty state, recent cards, four orientation chips, closing reading section. Preserve banner/title/author/date/read-time/summary links and empty fallbacks. Heads retain title/description, OG/Twitter/icon/style behavior through app assets. |
| `T/plugins/app/views/article.tsx`: Body/Head/Page | **E/A** shared Leptos reading layout/backlink, title/date/author/read-time, first3 tags + first2 keywords capped at4, optional banner, HTML contents, comments/count/reader/text/empty fallbacks. HTML rendering follows the source; do not add a sanitizer or treat it as plain text. Head metadata/style references remain app-owned. |
| `T/plugins/app/views/error.tsx`: Body/Head/Page | **E/A/G-ERROR** 404 Not Found vs Oops, description, production-sensitive debug display, trace/snippet presentation and head metadata. Reuse LayoutPanel, root language/theme contexts; icon family Lucide. Native provenance is not a JS stack reconstruction claim. |
| `T/plugins/app/helpers.ts`: stripHtml, formatArticleDate | **A** ordinary pure app helpers: tag stripping + whitespace collapse/trim; UTC en-US year/short-month/day, missing date→Unscheduled. Test exact strings and source truthiness (including timestamp0), not browser-local timezone. Existing temporal source values and UI formatting are reusable mechanisms, not equivalent helper semantics by themselves. |
| helpers getFeaturedArticle, getArticleSummary, getReadingTimeLabel | **A** first/null + remainder; empty content fallback, default160 chars or provided limit and trailing `...`; ≥1 minute at180 words/minute. Preserve JS UTF-16 slicing where observable rather than accidentally changing Unicode truncation through Rust byte indexing. |
| helpers getContentSignals, getTopicHighlights, getCommentCountLabel | **A** row count, truthy-banner count, latest valid date; exact four source orientation phrases; No comments yet/1 comment/N comments. No new comment creation UI: source only displays comments. |
| app desktop:menu contribution with WeakSet guard | **G52 / omitted**: source id blog:open-latest-posts, help menu, label Open Latest Posts, priority20 and per-context idempotence remain reference only. No native menu code in this release. |
| app `blog:desktop-open-latest-posts` event | **G52 / omitted**: source response `{desktopEvent:"blog:desktop-open-latest-posts"}` does not navigate or query “latest”; no equivalent event is registered in the web-only release. |
| `T/tests/helpers.ts` | **A** seven meaningful test counterparts: featured/remainder, HTML summary+empty fallback, minimum reading time, metrics/latest date, four phrases, summary limit, zero/one/many comments. Source test script glob is `tests/**/*.test.ts`, which does not select this filename; Rust tests must actually discover/run them, and record the source discrepancy. |

### Ordered seed records

All **15 records**, in this order, are in `T/config/common.ts::database.populate`; Rust uses `src/seed.rs` to build the same event/data sequence and existing aggregate populate dispatch (`framework/src/native/commands/sql.rs`). Dates/computed expiry are native values at the app boundary. IDs left absent in T stay generated. No idempotent upsert, automatic purge, or startup seeding is introduced.

| # | Source event / record | Rust proof obligation |
| --- | --- | --- |
| 1 | profile-create: developer / Developer / person / [ADMIN] | Generated extended Profile, same fields; authenticate against this actual record. |
| 2 | auth-create: developer profile, username developer, secret developer | Actual generated auth-create hashing/result behavior; not signup. |
| 3 | auth-create: developer profile, email developer@shoppable.ph, secret developer | Same source email credential data. |
| 4 | application-create: Developer App; developer; scopes [profile-write,auth-read]; now+365d | Native clock-controlled expiry and generated application; no invented user scope. |
| 5 | category-create: anime, slug anime, Anime, All about anime, source pinimg banner URL | Preserve source record; categories are unowned under G52. |
| 6 | category-create: gaming, slug gaming, Gaming, All about gaming, source wallpapers banner URL | Same; no profileId supplied in source. |
| 7 | article-create: pokemon, developer; Pokémon turns 30 title/long slug; source g.foolcdn banner; three HTML paragraphs; keywords pokemon/nintendo; tags Satoshi Tajiri/Arjan Mann; references.source nature.com; PUBLISHED; 2024-02-27T12:00:00Z | Preserve all data from common.ts, including Unicode/HTML/array order and date, with no remote fetch during proof. |
| 8 | article-create: minecraft, developer; Minecraft 15 title/slug; source minecraft.net banner; two HTML paragraphs; keywords xbox/pc; tags Mojang/Microsoft; references.source minecraft.net; PUBLISHED; 2024-11-15T12:00:00Z | Same fidelity; default home ordering puts Minecraft first. |
| 9 | catalog-create: anime→pokemon | Compound key + real FK relation. |
| 10 | catalog-create: gaming→minecraft | Compound key + real FK relation. |
| 11 | comment-create developer→pokemon: Great article! | Generated ID/defaults; retain text. |
| 12 | comment-create developer→pokemon: Congrats on 30 years of Pokémon! | Two Pokemon comments, no invented sort. |
| 13 | comment-create developer→minecraft: Minecraft is still amazing after 15 years! | Generated join/profile display. |
| 14 | comment-create developer→minecraft: Looking forward to the next 15 years of Minecraft! | Two Minecraft comments. |
| 15 | application-create: Example App; developer; same scopes and now+365d | Two distinct applications, even though they share scopes. |

The endpoint scope is `user`, while both seeded applications have `profile-write`/`auth-read`: seeded credentials must demonstrate the source denial, not be “fixed” silently. A separate explicitly labeled test application with `user` can prove the positive endpoint path without changing the 15-record seed. Populate's independent child responses continue after a non-OK result under the existing source contract; record failures rather than claim a transactional seed batch.

### Commands, build tools and MCP

The CLI/event bridges already exist, but an empty `Bindings` service is not a runnable CLI. `crates/framework/framework/docs/api/{commands,sql,view,codegen}.md` and `src/native/commands/` define the existing seams. `tools/cli/docs/guides/application-binary.md` owns `main_with` and `-b` layering.

| `T/package.json` script or supporting file | Rust equivalent / ownership |
| --- | --- |
| generate, generate:client | **E/A/M3** app tool uses PackageLoader/Composer, `Schema::normalize_source`, source schema/store/admin emitters and OutputWorkspace; check-only mode. Existing source tool: `framework-models/src/bin/generate.rs`; existing app independent tool: `examples/generated-admin/tool/src/{main,lib}.rs`. Client variant is output partition/destination selection, not TS compilation. |
| migrate | **E/A** existing command writes ordered SQL files from History; app supplies durable source revisions + adjacent planner. Do not claim writing SQL also applies it. |
| push | **E/A** existing source wrapper selects install for insufficient revision history, otherwise upgrade; app supplies Scripts/History from generated schemas. First-run setup and later reviewed upgrade are explicit; preserve warning/force and failure semantics. |
| populate | **E/A** existing sequential event/data dispatcher, using all15 source records. Not implicit in serve/develop. |
| purge | **E/A** existing generated/aggregate purge, SQLite DELETE mapping; explicit trusted destructive command only, not part of ordinary restart. |
| query | **E/A** existing trusted native SQL command, final nonempty argv query semantics and output provider. No new browser SQL endpoint. |
| emit | **E/A** existing CLI dispatch/output binding; direct/CLI/buffered-HTTP parity of article-search and error flows. |
| serve | **E/A** existing serve bridge bound to `stackpress-server-http::HttpServer`, signal/drain/Engine+Hooks shutdown. Deployment example is the reference; actual development starts only through devmetrics. |
| build | **E/A** separate SSR compile and hydrate/CSR WASM+cached wasm-bindgen, app authored CSS/asset manifest, existing BuildProvider if exposing build event. Current framework does not supply a bundler. |
| dev, dev:start | **A** finite build followed by managed serve; explicit rebuild/restart walkthrough. A reusable Vite-like watch/HMR engine is **G(c)**, not implemented by retaining the develop event name. |
| dev:clean | **A** no node_modules/.vite in Rust; optional removal only of owned generated dist/cache after explicit selection. No cargo clean on every dev iteration. |
| preview | **A/G** same built-asset managed serve with build/develop data. T references config/preview but no such source file exists; document this source gap, do not imply parity with a working preview bootstrap. |
| desktop:build, desktop:dev, desktop:package | **G52 / omitted**: desktop is a later separate task. No corresponding command is exposed in the first release. |
| test | **E/A** cargo nextest native behavior + separate doctests + cached Playwright. Source seven helper cases supplemented by real model/config/SQLite/lifecycle/UI proofs. |
| `T/tsconfig.json`, `uno.config.ts`, package dependencies/devDependencies | **A/G-ASSETS** Cargo target/feature graphs replace NodeNext/tsx/ts-node/React tooling; finite asset build replaces Vite/Uno Wind3+Stackpress presets. No npm install, runtime transpilation, generated client-source copy, or new JS compiler needed. |

MCP source detail (`T/config/common.ts::mcp`, `T/package.json`, `T/config/{develop,production,client,build}.ts`):

- name Stackpress Blog MCP, version1.0.0, /mcp, stateful. Package/plugin are already listed; develop, production and client include mcp, build omits it. The comment saying “not enabled by default” is stale relative to these files.
- `article_search`: public GET → article-search; q string/limit number schema, empty data. Its description says published, but data contains no published filter; do not enforce one based on prose.
- `article_create`: app POST → article-create, scope articles.write; required title/slug/content/profileId/keywords/tags, optional banner/status. **content differs from Article.contents**; preserve as a documented inconsistency, not a runnable promise.
- Plugin-mode tools: article-detail-tool (default public), article-update-tool (app/articles.write), comment-search-tool (default public), comment-create-tool (user/comments.write). No matching handwritten tool handlers appear in T/plugins; they depend on stackpress-ai generation/runtime outside this template.
- **G52 / G-MCP(a)**: MCP is not a release blocker. The future README documents it as unsupported and retains the original config as reference, without mounting /mcp. Real reusable MCP is **(c)** a separate framework task only if requested; an ordinary REST tool shim is not a substitute.

## Proposed layout

Use a standalone, copyable **nested Cargo workspace** at `examples/blog/`, like the standalone aggregate consumers. This avoids edits to the root Cargo.toml/Cargo.lock and keeps generator/runtime/browser graphs distinct. All versions/edition/MSRV/lints are declared locally from the current repository convention, using existing path dependencies; license metadata is untouched. It is still part of this monorepo, not a new framework or package loader.

```text
examples/blog/
  Cargo.toml; Cargo.lock; .gitignore; .env.sample; README.md
  schema.idea                         # import framework, enum, local extensions/models
  config/{develop,build,production}.json
  docs/{features,commands}.md          # detailed matrix/reference after first success
  assets/{global.css,article.css,brand.svg}   # authored inputs, never T/public copies
  bootstrap.js; build.py; proof.py
  tool/Cargo.toml; tool/src/{main,lib}.rs
  tool/tests/main.rs; tool/tests/cases/generation.rs
  shared/Cargo.toml; shared/src/{lib,page,helpers,language}.rs
  shared/src/views/{mod,home,article,error,layout}.rs
  shared/src/generated/               # M3 portable/public output; never native source schemas
  shared/tests/main.rs; shared/tests/cases/{helpers,render}.rs
  client/Cargo.toml; client/src/lib.rs # hydrate OR csr, explicit root disposal
  web/Cargo.toml
  web/src/{lib,main,config,client,render,seed,history}.rs
  web/src/plugins/{mod,app,store,assets,commands}.rs
  web/src/plugins/app/{home,article,error}.rs
  web/src/generated/                  # native source schemas/stores/family/admin bindings
  web/tests/main.rs
  web/tests/cases/{config,lifecycle,storage,pages,auth_api,commands,export,history}.rs
  tests/browser.cjs; proofs/{graphs,readme,locks}.py
  .build/                            # ignored: SQLite/revisions/migrations/owned proof files
  dist/{hydrate,csr}/                 # ignored build artifacts + asset manifest
```

G52 makes this layout web-only; desktop is a later separate step. No empty MCP/email plugin crate: select the existing email constituent or an app binding when needed. `src/plugins/app.rs` is a module, not a crate. Native generated and portable output stay separate because current source schemas are native-only (`framework-models/src/lib.rs`). M0 creates only the tool crate, using ignored `.build/generated` for a proof-only native feature; the default generator compiles without that tree. Later M3-dependent app/public roots are not created or advertised now.

App API sketch (proposed app-owned names, not compiled framework signatures):

```rust
pub async fn build_application(layers: ConfigLayers) -> BuildResult<Server>;
pub fn shared() -> Result<ConfigLayers>; // in web/src/config.rs
pub fn register(app: &mut ServerBuilder) -> Result<()>; // each app/store/assets module
// Named async handlers retain (&mut Request, &mut Response, Invocation<'_>) -> Result<Flow>.
// web/src/client.rs selects the generated family through M3's reviewed CompiledClient seam.
// tool::generate(...) installs/checks output; shared::views consumes only public page data.
```

`shared()` layers plugin defaults < shared values < `-b` file < explicit code overrides, matching G47. Config startup opens only the selected existing database for runtime; an explicit setup command owns creating/installing a new file. The generator never opens SQLite. The serve binding owns the listener/runtime; drain first, then auth Hooks shutdown and Engine.close. App plugins do not start listeners during registration/startup.

## Developer walkthrough

The README should deliver a visible seeded home page before the architectural tour, then explain each source file just used. Commands below are **planned example interfaces**, not commands run at this checkpoint. Every Cargo/proof invocation in development must be wrapped once by build-benchmark.py; README/proof helpers must keep that operational recipe visible.

1. **Create a project:** copy only examples/blog source/config/manifests/lock into a new folder, excluding .build/dist/target. In-monorepo copies can retain documented relative paths; outside the repo, adjust the centralized workspace dependency paths in Cargo.toml to the local checkout. Cargo path fields are literal paths, not environment-variable substitutions. No published package version or nonexistent scaffold CLI is promised. Show edition2024/MSRV1.96 and where to rename package/brand. Do not introduce license edits.
2. **Read schema.idea:** show `use "stackpress/stackpress.idea"`, PublishStatus and Article→Profile, then all four blog models and Profile's added inverse collections. Explain that imported core models remain generated and selected by the same app. Category has no owner relation under G52; this is the only M0 schema deviation from the template.
3. **Generate:** planned `cargo run --manifest-path examples/blog/Cargo.toml -p blog-tool --locked --offline -- generate`, then the same command with `--check`. Tool compiles without generated application output; rerun/check must be deterministic and check must write nothing. Explain native vs public output and the M3 prerequisites without exposing native schema values to the client.
4. **Prepare SQLite explicitly:** select `-b examples/blog/config/develop.json` and run the planned blog CLI setup/push command on `.build/blog.sqlite`. First setup installs from generated DDL; show created tables and explicit no-startup-install behavior. `migrate` writes SQL; reviewed `push`/upgrade applies it. Do not tell readers to use destructive source install as a universal safe upgrade.
5. **Populate:** planned blog CLI `emit populate`; show Developer username/password fixture, two categories, two articles, two links, four comments and two applications. State repeated populate follows create/uniqueness failures rather than silently replacing rows; show how to inspect per-event failures.
6. **Build and run:** finite build.py compiles SSR and one browser renderer, generates JS/WASM/CSS/manifest; inspect devmetrics list, then start only the prebuilt binary with `devmetrics start --summary "Stackpress blog walkthrough" -- '<absolute-blog-web-binary> --config <absolute-develop.json> --port {port}'`. The binary must accept both config and port; report the returned URL/expiry and stop via devmetrics. No direct cargo-run listener, unmanaged watcher or background `&` recipe.
7. **Verify first success:** open /, see Minecraft featured then Pokémon, follow an article and its two comments. Inspect `?json`, HTTP/error behavior and the unchanged unfiltered-by-status detail route. Explain the deliberate G52 home change: the app handler supplies eq.status=PUBLISHED so drafts/review/archived entries do not appear on home. Store/framework filtering is unchanged. Then explain plugin registration, view props, assets and shared Leptos components in encounter order.
8. **Admin/auth/API:** sign in with the seeded username, visit /admin/article/search and categories/profiles/auth/apps/sessions; create/edit an Article and verify page data. Show API `user` scope denial for the original seeds and a separately created app with that scope for the positive case. Explain menu visibility vs actual source process access. Never replace the generated admin with handwritten CRUD to bypass an M3 gap.
9. **Add a column:** extend imported Profile with `nickname String?`, regenerate + check, write/review/apply the explicit nullable-column migration, update/query the nickname through registered CRUD/API/admin and export, and exercise auth collaborators. Demonstrate source data retained across restart. Stock signup still does not auto-forward nickname. Failed/unsupported migration planning becomes a named gap, not a drop/reinstall workaround.

Desktop is skipped for this first release under G52 and can be assessed as a later separate step; it has no paths, config or commands in the walkthrough. MCP remains unsupported with the original config retained as reference in the future README.

## Gaps and owner questions

Classification requested by the brief: **(a)** omitted with an explicit note, **(b)** small app-level adaptation, **(c)** framework capability needing a separate task. A proposal below is not permission to omit a mandatory G50 prerequisite.

| Gap | Classification, recommendation and boundary |
| --- | --- |
| G-M3: family binding/source admin/full graph | **M3 prerequisite**, not blog work. `00031` M3 includes source UI/admin emission, session/API replacement and CompiledClient reconciliation. Wait for coordinator-verified integration and exact public signatures; prove the complete blog graph, not only Profile.nickname. If an adapter is absent after M3, record **(c)** follow-up; do not modify forbidden crates or downgrade to strict/handwritten CRUD. |
| G-M0-PROPS: native normalization rejects blog role props | **Observed framework gap (c)** within G50 source-role work: `framework-schema/src/codegen/source_roles.rs` does not override the standard empty-prop definitions for field.editor/field.select; `registry.rs::validate_prop` rejects their object keys. Article.contents's14 Boolean editor options and Article.status's options array both reproduce `unknown component prop`. This blocks native source output before any admin/UI emission. Blog schema is unchanged beyond G52; no filtering/registry override is installed in the app. Coordinator must route this to the framework/M3 owner, then rerun M0. |
| G-UI: source rich editor, relation lookup, image/HTML/template/tag/date roles | Current source role metadata is unavailable; existing typed `framework-admin/docs/api/catalog.md` explicitly lacks several adapters. M3 may cover them, but blog-specific props/compound keys still require proof. **(b)** app catalog/presentation bindings only for existing public adapters; an absent reusable concrete adapter is **(c)**. Do not silently mark roles unavailable, remove them or substitute plain text and claim parity. |
| G-SCHEMA: Category.profile missing local field | **Resolved by G52 (b)**: remove that edge; categories are unowned. No framework relation-validation relaxation or new profileId scalar/seed. |
| G-HISTORY: durable revisions and adjacent migrations | **(b)** explicit app History provider in web/src/history.rs using generated native table definitions, existing table diff/DDL/source setup functions (`framework/src/native/commands/sql.rs`, `crates/sql/sql/src/table.rs`, `framework-sql/src/source/maintenance.rs`). No generator dependency enters runtime. Existing `framework-sql/src/migrations/` is the separate typed SQLite maintenance path, not automatically the source History provider. Prove nullable Profile addition with preserved data. Broader unsupported transforms are **(c)**, not a new migration ledger/runner or automatic reconstruction. |
| G-MCP / stackpress-ai | **Resolved by G52 (a)**: not a blocker; README marks unsupported and retains original config as reference. **(c)** real MCP only as a separately requested task. No dependency/install/route in this task. |
| G-DESKTOP packaging and API/auth parity | **Resolved by G52 (a)**: skipped for the first release. No desktop stage, crate, config, menu, test or packaging path is part of this plan. A later separate task can use `examples/profile-ui/native/` as reference. |
| G-WEBHOOK | Webhook dispatch already **E**. **(b)** recording Fetch or explicit app receiver binding proves inline payload/predicate/status/order. **(a)** default live outbound call to hardcoded localhost3000, which has no source receiver; retain exact original config in docs. No queue/retry/signature/egress restrictions added. A real opted-in local receiver must use the managed port; external delivery remains unclaimed. |
| G-EMAIL | Email/SMTP adapters already **E**. **(b)** existing JSON transport for self-contained auth/email demonstrations; preserve SMTP example as opt-in. **(a)** live SMTP/provider acceptance. Do not clear shared responses or change hooks to make email send: framework-email documents the source auth-update-body skip. |
| G-LANGUAGE | **(b)** source native locale selection bridged explicitly to checked root-local phrase map for Leptos. Preserve exact en_US/th_TH and phrase fallback. Browser state isolation is required; no new preference canonicalization/negotiation rule. **(a)** full Thai translation catalog, Intl plural/collation parity and translated article content, none provided by source. |
| G-ASSETS: Vite/UnoCSS/Frui CSS | **(b)** authored CSS + existing Leptos styles, finite Cargo/wasm-bindgen pipeline and AssetBundle. **(a)** Node-specific optimizeDeps, TSX transpilation and React refresh. A reusable live watcher/HMR/bundler is **(c)** if desired later. Do not copy generated public/client-source trees; external seed banners are fixture data and are intercepted in browser proofs. |
| G-ERROR: JS stack/source snippets | **(b)** app error presentation of existing `crates/lib/src/error.rs` source locations/invocation frames; guarded local file snippets only when meaningful source coordinates exist. **(a)** exact JS method/stack/char reconstruction where Rust lacks it. Show production vs development/404/json/JS/CSS branches faithfully; no framework error/disclosure policy changes. |
| G-NOTIFY / store tracing / preview file / test glob | **(a)** unsupported react-toastify details, exact universal query-before hook, nonexistent source config/preview and unselected tests noted above. **(b)** readable existing Rust notifications, finite rebuild/serve and discovered Rust helper tests. Missing generic capabilities only become **(c)** tasks if separately requested. |

All owner questions are answered by **G52**: Q-BLOG-1 removes Category.profile; Q-BLOG-2 documents unsupported MCP without blocking release; Q-BLOG-3 skips desktop entirely for the first release. G52 also explicitly requires a real PUBLISHED equality filter in the later app home handler. No product decision is pending at M0.

## Staged plan

Only M0 is released. Paths in each row are relative to examples/blog/; managed roots are complete generator-owned output inventories, not a license to patch them. Root manifests/lock, framework-session/API/models/schema/SQL/aggregate and the admin generator remain outside blog ownership. M5 desktop is dropped under G52; other stage numbers are retained for review continuity.

| Stage / dependency | Exact proposed files | What proves the stage, then stop |
| --- | --- | --- |
| M0: independent bootstrap/generation-risk proof; **released before G50 M3** | Cargo.toml, Cargo.lock, .gitignore, .env.sample, schema.idea; tool/Cargo.toml, tool/src/{main,lib}.rs, tool/tests/main.rs, tool/tests/cases/generation.rs; proofs/locks.py | One root-seeded offline lock resolution and immediate invariants; PackageLoader imports unchanged framework source; all8 models, enum and relations compose. Generate ignored .build/generated, compile it with tool's native-proof feature in the nested workspace, default generator independent of output. Repeat/check-only/manual-drift/deleted-output controls. No source admin/API/session binding workaround. |
| M1: config/store/commands/seed; base pieces **can start before M3**, final family-bound registration **waits M3** | web/Cargo.toml, web/src/{lib,config,seed,history}.rs, web/src/plugins/{mod,store,commands}.rs; config/{develop,build,production}.json; web/tests/main.rs, web/tests/cases/{config,storage,commands,history}.rs; proof.py | Real file SQLite fresh setup + ordered15 seed events + restart persistence; exact counts/joins/defaults/composite key/unique/errors; startup has no installation/population effects. Real -b precedence/invalid values, generate vs migration vs push separation, adjacent nullable-column migration, purge/query only on owned fixtures. Missing History/provider must fail visibly, not silently do nothing. |
| M2: integrated app family/admin/auth/API/export; **must wait for coordinator-verified M3** | web/src/{client,generated/}; shared/{Cargo.toml,src/lib.rs,src/generated/}; web/tests/cases/{auth_api,export}.rs; shared output index plus tests consume M3's actual generated public types | Regenerate/check all partitions, same Profile type in session and API, one CRUD inventory, extended nickname through create/read/update/auth collaborator/API joins/admin/export; stock signup ignores extra input per decision. Exact source grants/scope success+denial, missing-config/Engine/client skips and G42 order/Abort. Real SQLite writes, not mock-only or hand-bound default models. |
| M3: public pages/SSR/browser pipeline; **waits M2 and M3 UI contracts** | web/src/{main,render}.rs, web/src/plugins/{app,assets}.rs, web/src/plugins/app/{home,article,error}.rs; shared/src/{page,helpers,language}.rs, shared/src/views/{mod,home,article,error,layout}.rs, shared/tests/main.rs, shared/tests/cases/{helpers,render}.rs; client/{Cargo.toml,src/lib.rs}; assets/{global.css,article.css,brand.svg}, bootstrap.js, build.py, proofs/graphs.py; web/tests/cases/{lifecycle,pages}.rs | Seven helper counterparts plus boundaries; native SSR, wasm hydrate and CSR compiled separately with no --all-features. Actual native/browser normal-edge graphs: no SQL/server/mail/identity/codegen in browser, no generator in runtime. Real registered page handlers + SQLite rows, error/terminal/nonGET/body/json/JS/CSS branches, existing source HTML semantics, matching SSR/hydration props. Managed HTTP smoke only through devmetrics. |
| M4: browser/readme proof; **waits M3** | tests/browser.cjs; proofs/readme.py; README.md; docs/{features,commands}.md; refine proof.py | Cached Playwright only; both browser modes, 1280/375px and 200% narrow zoom, no-JS first paint, keyboard/accessibility/overflow, hydration errors and disposal. Home featured/order/empty, article/comments/missing path, sign-in/admin edit/API denial+success, locale/theme/head/assets/json and nickname migration/export. Browser requests go through finite buffered HTTP + actual SQLite as in generated-admin/proofs/http_browser.cjs, remote images fulfilled locally. Exact README finite commands run in a disposable copied app; separately report real managed URL/expiry/HTTP smoke and stop only owned server. |
| M6: review rounds and closeout; **waits released implementation stages** | Only authored files already listed above; receipt update after coordinator grant | ≥3 distinct test-verified rounds: complexity, responsibility/abstraction, consistency/Rustdoc/comments, plus dedicated ownership/async/safety review. Meaningful reruns per findings, no ceremonial edits. Final compatible workspace/all-target native nextest + separate docs, fmt, Clippy -D warnings, warning-clean Rustdoc, every touched generator/graph/browser proof; ≥70% per changed implementation crate including executed generated code, ≥90% only if preparing publication. Coordinator independently re-verifies; owner visual review is separate. |

Build/proof execution protocol for later releases:

- All commands go once through `.agents/scripts/build-benchmark.py run --label ... --kind test|check|clean|coverage|doc|other --scope selected|workspace|consumer|other --cache warm|cold|unknown -- ...`; use `check` for compile/lint and `other` for build helpers, keeping the brief's valid kinds. An outer Python proof records nested Cargo once; no double counting.
- First actual compile effort gets isolated target/blog native/browser/consumer subdirectories, recorded cargo clean and cargo llvm-cov clean --workspace for owned targets only; never inspect/reuse stale unrelated output. Then retain warm targets, sccache, debug=0, bounded two jobs/workers. At gates incremental=0; edit loop unset incremental. Default cap12GiB unless coordinator grants another cap; report size and disk constraints before exceeding it.
- Use cargo nextest for native tests and cargo test --doc separately. Every resolving Cargo command after the one lock-seeding operation uses --locked --offline; fmt has no resolution flags. Do not use --all-features on renderer graphs; build SSR, hydrate and CSR separately. No installs/network or alternate trial branches.
- The nested example's gate selects `--manifest-path examples/blog/Cargo.toml --workspace`; a root-workspace gate alone would not test these crates. Root regression commands use the coordinator's compatible feature sets and explicitly exclude listener-binding fixtures such as tcp_watchdog under this brief. Report those exclusions; actual network behavior is covered separately by the devmetrics-managed smoke, never a hidden test listener.
- Playwright path is the cached `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`, verified before launch; reuse its cached Chromium. Reference `crates/framework/framework/proofs/browser.cjs`. No npx download, system Chrome dependency or browser server.
- Every hang-class native/CLI/pool/hook test has an external parent deadline, readiness seam and exact owned-child kill/reap with a stuck-child negative control; avoid scheduler sleeps/in-executor-only timeouts. Close Engine/Hooks explicitly. Browser proofs have finite deadlines too.
- Coverage must include authored example paths (llvm-cov's default examples exclusion needs an explicit report filter, as G47 discovered), generated executed code and declared target features. Native coverage is not browser/Tauri proof. Skip unavailable platform/provider gates with an explicit result, never a fabricated pass.

Risk inventory: incorrect Category relation (owner resolution + generation failure control), default/extended family split (type equality and end-to-end nickname), unavailable source roles (generate+compile+browser each), seed scope mismatch (positive and source-denied credentials), migration data loss (real existing-file upgrade + failure cases), browser native leaks (normal-edge graphs + snapshot inspection), lifecycle double dispatch (ordered event/error traces), uncertain writes/deadlock/shutdown (existing source semantics + parent watchdog), stale binary reuse (per-consumer target paths), and lock drift (immediate invariant checks). Each is assigned to the stage that first exposes it above.

## Lock plan

The root has **699 external registry tuples** at this HEAD. Baseline root Cargo.lock SHA-256: `65861e753f71bc227304186fc5a69a05354bb3ff63fd97938688b49964a95169`. Preserve root bytes exactly throughout blog work; the example has its own workspace lock. No root Cargo.toml/Cargo.lock writes, cargo update/generate-lockfile, installs, license metadata changes or dependency fetches.

1. Before a stage's resolution, snapshot root bytes and the ordered `(name, version, source, checksum)` registry list; assert699. Inventory selected dependencies/features against that baseline, including dev dependencies. Existing names alone do not prove a feature needs no missing transitive tuple. Desktop/Tauri dependencies are absent from this web-only example.
2. Root-seed `examples/blog/Cargo.lock` once, then run **one authorized offline resolution**: `cargo tree --manifest-path examples/blog/Cargo.toml --workspace --offline -e normal --prefix none`, with stdout to DEVNULL. The coordinator requires this established command; do not use cargo metadata/update/generate-lockfile. The wrapper checks invariants immediately in `finally`, even on failure. No second uncontrolled resolution or online retry.
3. Immediately assert every external registry tuple of the example lock is an exact subset of the captured699, and root lock bytes/SHA remain identical. Pruning unused registry tuples in the standalone lock is expected; new versions/sources/checksums/tuples are not. Inventory differences are workspace package entries/edges only. Reject drift before another build.
4. Thereafter all example/native/browser/consumer commands are locked/offline. Add temporary consumers to the declared example graph where possible; if a genuinely standalone probe needs its own lock, root-seed it once with the same immediate subset/root-byte assertions, then locked/offline. Never accept Cargo silently mutating root while testing a path dependency.
5. If a required later M3 integration changes the manifest graph after initial resolution, the coordinator must explicitly release that workspace-edge amendment with the same tuple/root-byte proof. No new external tuples are authorized by this task. A missing cached crate/tool is a limitation; stop the dependent proof rather than install or substitute a dependency.

## Implementation notes

Historical design checkpoint: read-only inspection and this receipt only; no implementation/build/listener occurred then. The following M0 work is separately released at 9314c3e; G52 settles every owner question. No M1+ implementation is authorized. Root/framework/upstream/license/Context/task-tracker files remain outside scope.

The historical design baseline hashed all2705 tracked files/all25 template source files and captured raw KB diagnostics. Its `/private/tmp/stackpress-blog-checkpoint/build-benchmarks` store was a process mistake corrected by the coordinator: **all M0 and later records must use `--store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`**. The temporary historical records are retained, not treated as repository-store runs.

### M0 current work

Created the released nested workspace/tool manifests, generator library/CLI, one integration-test binary, lock wrapper, .gitignore and .env.sample. The tool uses the existing public PackageLoader/source emitters, with no framework edits or M3 dependencies. The native-proof feature includes only ignored generated output and is not selected for generation. No default source model list is copied into runtime logic.

Repository-store baseline record `20261104T062150.017207Z-d0c80eea015c` captures HEAD9314c3e, all2705 tracked hashes, root SHA/699 tuples, and KB146 errors/40 warnings. The first schema-copy command was rejected before execution because the same exact line occurs in Article; its replacement scopes Category, asserts one local match, preserves complete Article/Comment blocks and proves exactly one removed line (`c65480ef5ff2` below). The first resolution command was also rejected before execution because it used cargo metadata; the corrected wrapper uses the coordinator's established cargo tree command. Neither denied child ran or counts as a resolution attempt.

Exactly **one root-seeded offline resolution** succeeded (`10e44dee2f53`): the standalone lock contains **100 registry tuples**, all exact subsets of the root699. The root lock SHA remains `65861e753f71bc227304186fc5a69a05354bb3ff63fd97938688b49964a95169`; root Cargo.toml and all framework sources are unchanged. All subsequent Cargo commands used --locked --offline; no install/network occurred. `.build/lock-baseline.json` is the exclusive attempt marker, not a generated application database or migration history.

The default tool compiled with no generated tree. Composition proves all8 models, PublishStatus values, imported Profile.auth plus app-added articles/comments, Article.profile local/foreign keys, Catalog compound identity/nested query and Comment's two owning edges. Public `compose` returns the actual composed SchemaConfig; source normalization is deliberately a separate emission step, so these are not falsely presented as normalized/native runtime proofs.

**M0 is incomplete.** The initial suite failed5/5 at native source normalization. A disposable, output-free per-attribute diagnostic isolated exactly the two source-role selections named G-M0-PROPS above. After separating the composition test from normalization, the final unfiltered suite is **1 passed /4 failed /0 skipped**. Failures occur before writer publication, so none of the planned deterministic/repeat, drift, deletion or stale-schema controls has reached its proving assertions. The generated native family has not been emitted or compiled; native-proof stays unrun. No reduced schema, metadata filtering, handwritten model or framework edit works around the gap.

Diagnostic files remain under `/private/tmp/stackpress-blog-m0/`; they never publish generated output and use only fresh M0 rlibs, with a unique-artifact assertion. Test commands run under a finite external process-group deadline with owned kill/reap. No timeout tripped. The nested target is retained warm at `target/blog-m0/native`; coverage cleanup used the separate task-owned target. Environment: sccache, cache20G, incremental0, dev debug0, two build jobs/test workers. Completed-command target peak/current about280MiB, below12GiB; no unrelated target was read/cleaned.

Quality evidence: default all-target compile and Clippy with -D warnings pass; warning-clean Rustdoc passes; doctests pass with **0 examples** (not behavioral evidence). All four authored Rust files pass direct rustfmt --check with skip_children=true, because the optional proof module's managed output is absent and must not be hand-formatted. One format check initially found a wrap introduced by the earlier formatter; corrected and rechecked. No implementation coverage, native generated-code coverage or three-round closeout is claimed: the initial behavior proof has not passed, and later rounds/stages remain unreleased.

### M0 exact file ownership and next action

The11 released new paths are Cargo.toml, Cargo.lock, .gitignore, .env.sample, schema.idea; tool/Cargo.toml; tool/src/main.rs; tool/src/lib.rs; tool/tests/main.rs; tool/tests/cases/generation.rs; proofs/locks.py, all beneath examples/blog/. This receipt is the12th intended repository path. No README, web/shared/client crate, config directory, desktop path, root manifest/lock, framework crate, generated root or license metadata was authored. No staging/commit, listener, database, browser or M1+ work.

Coordinator next action: resolve G-M0-PROPS in the appropriate G50 framework lane and integrate its verified source-role contract; then release a rerun of the same full-schema M0 generation, native-proof compile and freshness/drift/deletion controls. This is a framework dependency, not an unanswered owner product question. The future README must still record the G52 home eq.status deviation and unsupported MCP reference; no home handler is in this release.

### M0 repository-store command evidence

Full IDs, argv, logs and environment are under `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/`. Wall seconds and observed Cargo-stage seconds are separate; `—` means no Cargo stage was observed.

| Record ID | Result / proof | Wall s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261104T062150.017207Z-d0c80eea015c` | Baseline scope/lock/KB captured | 0.830532 | — |
| `20261104T062851.985343Z-c65480ef5ff2` | Category-only one-line delta passed | 0.038581 | — |
| `20261104T063114.497202Z-10e44dee2f53` | Sole offline resolution + immediate invariants passed | 0.522638 | — |
| `20261104T063141.373681Z-dbf3d9ab4ad6` | Native target cleanup passed | 0.137193 | — |
| `20261104T063151.404626Z-64b52f3a9aa4` | Coverage target cleanup passed | 0.488706 | — |
| `20261104T063200.515363Z-461574a3157a` | Default all-target tool compile passed | 26.460172 | 26.41 |
| `20261104T063309.248300Z-4300b7a9a0ce` | Initial suite: 0 passed /5 failed at props | 21.006882 | 18.15 |
| `20261104T063517.459789Z-19678521271a` | Authored-file formatting applied | 0.048894 | — |
| `20261104T063613.976208Z-65292abd3167` | Composition1/1 + attribute locator passed | 3.642264 | 1.34 |
| `20261104T063719.015361Z-509aeb33921a` | Default all-target Clippy passed | 0.569982 | 0.49 |
| `20261104T063739.101353Z-05ef8c72a72e` | Warning-clean Rustdoc passed | 1.762508 | 1.73 |
| `20261104T063811.640052Z-0ea2206e86ab` | Final suite: 1 passed /4 failed, none skipped | 1.337516 | 0.11 |
| `20261104T063857.062282Z-d18c92f21ec1` | Doctests: 0 examples, passed | 0.575905 | 0.25 |
| `20261104T063950.258574Z-200d8009602a` | Format check failed on main.rs wrapping | 0.079536 | — |
| `20261104T064009.437822Z-8f337844941f` | Corrected format check passed | 0.086283 | — |

Through these15 records: **12 passed /3 failed**,3 attempts with observed compilation,0 full-root-workspace rebuilds; cumulative command wall57.587592s, observed Cargo-stage sum48.48s. These are command totals, not elapsed session time or a runtime benchmark. The closing scope/lock/KB audit after this receipt edit is recorded separately in the same repository store and reported in the final handoff.

## Verification notes

The historical checkpoint below verifies the original design document only. Current M0 results are above:1 composition pass,4 generation failures, no native output proof, coordinator review pending. Old receipt results are not represented as current implementation tests. The closing audit must preserve all2705 tracked baseline hashes, allow exactly12 new released paths, verify the pinned one-line Category delta/root lock/registry subset, and introduce no KB errors beyond the inherited146.

- Baseline record `20261104T060259.839639Z-57b1cb404032`: source/lock/scope inventory passed; raw validator independently returned exit1 with **146 existing errors /39 warnings**, chiefly inherited sibling-checkout paths. This is not a clean KB result.
- Source-path/feature audit passed: **104 existing cited paths**, all25 tracked template source files mapped, all15 seed records present. Scope audit passed: **2705 baseline tracked files byte-identical**, all25 upstream template source files unchanged, root lock bytes and ordered699 registry tuples identical. Only this untracked receipt was added; nothing staged.
- KB delta: **146 inherited errors /40 warnings**, exit1. Error messages match baseline exactly; the sole additional warning is this cohesive task file exceeding preferred200 lines while remaining below the500-line hard limit. There are no new link errors. No existing KB diagnostic was repaired outside this scope.
- Two exploratory audits failed and remain recorded: the first exposed one wrong auth/pages citation plus checker handling of shorthand/member paths; the second incorrectly inventoried an untracked .DS_Store. The citation and disposable checker were corrected; the final audit uses the exact tracked-source baseline and passes. Neither failure involved Cargo or implementation tests.
- Keep this cohesive checkpoint in one requested task file even if it exceeds the preferred200 lines; final hard limit500 applies. No Reference File split is needed unless that hard limit is exceeded.

Benchmark evidence through the scope audit (all kind=check, scope=other, cache=unknown; Cargo-stage time **unavailable** for every row):

| Record ID | Outcome | Command wall seconds |
| --- | --- | ---: |
| `20261104T060259.839639Z-57b1cb404032` | Baseline inventory passed; raw KB failure captured separately | 1.002707 |
| `20261104T061202.469606Z-ede4bd997a76` | Exploratory path audit failed, corrected | 0.033904 |
| `20261104T061227.798613Z-176effdf397b` | Exploratory file inventory failed, corrected | 0.057198 |
| `20261104T061237.338486Z-e56647169b2d` | Tracked-source/path/feature audit passed | 0.030831 |
| `20261104T061341.175366Z-5de422a54f5f` | Scope/lock/KB-delta audit passed | 1.265132 |

Through these five records: **+5 attempts, 3 passed /2 failed, 0 with compilation, 0 full rebuilds**, cumulative command wall2.389772s. A closing combined audit after this evidence edit is recorded in the same store; its ID/timing stays in that run's JSON/log and the final handoff. This is documentation validation only. No Rust test/compile/browser or installed-platform gate is claimed.

## Acceptance notes

Acceptance criteria: **none for design/M0 generation proof**. M0 stops for coordinator verification; later UI stages need owner review of rendered home/article/error/admin surfaces. Desktop is skipped under G52. Machine verification is not product acceptance, publication or deployment.

## Coordinator verification

Committed as `888055d` and `d716d67`, then merged into main as `b5e011f` and `03a299e`. Reviewed:
- 91 authored files, about 470 KB; generated output and `.build/` ignored;
- the nested workspace, which root does not include;
- the blog lock: a 437-tuple subset of root;
- G52 changes, and the G56 exemption scope;
- the localhost origin fix and the review-approved manifest patches.

My first clean-checkout run failed: the gate depended on `.build` leftovers (extension fixture, probe binary, nickname inputs, lock marker). `d716d67` fixed that.

Independent run of the README "Verify" sequence on a detached clean checkout of `03a299e`, with no prior `.build`, 2026-11-06: all 26 gate phases passed.
- native tests: 49 passed;
- format, freshness, native/hydrate/CSR Clippy, doctests, Rustdoc, graphs;
- public, authenticated and copied-out nickname browser flows;
- coverage: blog-tool 90.27%, blog-web 71.55%, blog-shared 95.44%; blog-client G56-exempt;
- lock invariants; tracked tree unchanged afterwards.
