# Task 00052 — release coverage and self-contained tests

Status: final gate green; all five crates exceed the 90% live-inclusive publication floor. Branch `release/coverage`,
base `7c014bb8e9215fac8a1498871ddf3ee0bd321a07`. Scope and floor follow
[00047](00047-publication-plan.md): each of PostgreSQL, MySQL, framework SQL,
admin and email must reach 90% source lines. Native-driver claims must be
**live-inclusive**, with a separate non-live number. All five meet the floor in this declared scope.

## Changes and ownership

Added behavior/error/branch tests for both drivers' configuration, SQL/value
conversion and error mapping; SQL projection, generic store/actions, numeric
prefix conversion and host operation assembly; admin appearance, fieldsets,
media/templates, metadata and controls; email options, content and TLS construction.
The new gated M1 publication groups cover native output budgets, MySQL integer
widths and loss/recovery of an owned connection. The live replay passed.

Generated-store scenarios moved unchanged to the unpublished generated-workspace
app's `driver_stores` custom harness and `tests/support/driver_stores/stores/`.
SQLite runs offline; PostgreSQL/MySQL are live-only. Workspace all-targets coverage
includes this lane. The length generation/SQLite proof moved to
`tools/release/proofs/lengths/`, with path-only caller changes. Four email source
oracle generators moved byte-identically to `tools/release/proofs/email-source/`;
the package still replays its unchanged JSON fixtures. Incoming guides/receipt
links were updated, including one CSRF documentation link.

`tools/release/test-copies-coverage.json` registers 24 exact local helpers/fixtures
and two intentionally adapted generated-store native fixture owners. Each entry
records canonical SHA-256, destination and provenance, plus divergence/copy hash
where needed. The other lane owns the shared checker. A recorded scan verified
all 26 entries and crate-local literal Rust references: PostgreSQL 25, MySQL 25,
SQL 78, admin 20, email 17 after the integration move. The shared canonical harness, process helper and G54
helper remain byte-identical to HEAD; no API/session/aggregate copy is registered here. Session assets were moved, not copied.

Approved manifest edits: remove `generated-workspace-app` from both drivers'
dev-dependencies; add both drivers and the already-locked SQLx PostgreSQL/MySQL
features to the unpublished app, with `driver_stores` harness=false; switch email's
reqwest feature as below, and move email test dependencies as listed below. Root lock
changes are three added and eleven removed workspace edges only; all
**699 registry tuples are unchanged**. A direct `cargo metadata --offline` attempt
failed on an uncached all-target dependency; the coordinator restated the ban.
No further metadata command was issued. Root/nested locks were checked unchanged
at that point; subsequent nested changes are only the approved TLS refresh below. The later publication-order
resolution preserved every nested lock byte again.

## Approved email bug fix

| Before → after | Cause and decision | Proof |
| --- | --- | --- |
| Native HTTP content client panicked before URL parsing → retained malformed-URL regression returns its concrete parse error; HTTPS client construction succeeds | Email used `rustls-no-provider`. API's `rustls` feature masked the missing provider through Cargo feature unification. Coordinator approved email `rustls`, preserving gzip/deflate/cookies and reqwest's source accept-invalid handling, instead of a private certificate verifier. No global provider installation or new registry tuple. | Email-only default/server suites: 39/52 passed after relocation, including the unchanged regression and HTTP/SMTP construction; API-alone default suite: 62 passed. |

Lettre audit: `tokio1-rustls-tls` enables Ring; cached 0.11.23 `build_rustls` uses
`builder_with_provider`, with a compiled Ring fallback. The new no-network test
constructs implicit, required STARTTLS and opportunistic TLS parameters. No Lettre
production fix was needed. The approved graph assertion now checks `reqwest rustls`.

All three email consumer locks required the approved root-seeded offline refresh:
direct **181 → 203**, event **181 → 203**, disabled **181 → 203** registry tuples.
Each resolution immediately asserted root byte identity, unchanged 699 root tuples,
and consumer-registry subset. Remaining 18 tracked nested locks stay byte-identical
to HEAD. Detailed counts, identities and hashes: `target/00052/email-lock-refresh.json`.

## Review rounds and evidence

Three test-verified rounds covered logic/error precedence, responsibility/ownership,
and consistency/docs. The original baseline, findings and complete benchmark ledger
are preserved in [non-live development evidence](../../../resources/release-coverage-00052/non-live-development.md).

The final ordinary live-inclusive workspace measurement uses cargo-llvm-cov 0.8.6,
Rust 1.96.0 and LLVM 22.1.2 on aarch64-apple-darwin with the exact compatible set.
All 1311 tests passed, 60 skipped; no custom export or new exclusion was used.

| Crate | Live covered / source lines | Live-inclusive | Non-live |
| --- | --- | --- | --- |
| PostgreSQL | 842 / 889 | **94.7132%** | 41.5073% |
| MySQL | 986 / 1049 | **93.9943%** | 44.6139% |
| Framework SQL | 7803 / 8629 | **90.4276%** | 90.0104% |
| Framework admin | 3929 / 4364 | **90.0321%** | 90.0321% |
| Framework email | 2471 / 2742 | **90.1167%** | 90.1167% |

The SQL/admin source denominators are unchanged. Email's existing HTTP builder was
extracted for direct construction testing, adding three executable source lines.
No new exclusion was used. Evidence summary and report hash are recorded in
`.agents/resources/release-coverage-00052/non-live-summary.json`.
After the publication-order moves, both relocated lanes ran under the same
compatible workspace coverage (9 passed, two live cases skipped). The accumulated
export retains the non-live counts; first-party production source
was unchanged. Its report hash and complete accumulation record chain are in
`.agents/resources/release-coverage-00052/post-move-non-live-summary.json`.

Only the native aarch64 Apple host and released compatible features are measured;
these numbers do not establish other execution targets or all branch outcomes.

All proofs use the required benchmark recorder/store, sccache, nonincremental builds,
no dev debug info and `target/00052`. Timing distinctions and every record are in the
linked development evidence.

## Publish-order follow-up

Moved the five email integration modules to `framework-session/tests/email/integration/`,
with the model generator, authored idea, managed output and page vectors under
`tests/email/fixtures/`. `tests/email_integration.rs` owns registration. All 19 moved
files are unchanged except the module's vector include path and generator's local
fixture base. The same eight tests pass under both `auth-server` and `leptos-ssr`;
the moved regeneration example's `--check` passes. Incoming email/session docs,
claim paths and aggregate coverage-map evidence now resolve to the new lane.

Data-aware diagnostics moved byte-identically to the app's driver-store lane;
SQL/native-only diagnostics stay driver-local as `sql_diagnostics.rs`, with the
same category/context output. MySQL has its own registered exact copy. The shared
canonical harness/process/G54 sources remain byte-identical to HEAD.

Stripped package copies pass PG 19/MySQL 18 non-live tests (nine live cases skipped
each), and email 39 default/52 server. Root/nested bytes are unchanged; standalone
registries remain root subsets. Details: `.agents/resources/release-coverage-00052/package-order-proof.json`.

The first relocated auth-server build exposed the missing CSRF/server test feature;
no test assertion changed. The coordinator approved the earlier test-only edge below,
then both renderer configurations passed. Documentation checks passed: email/CSRF
27 pages and 18 claims; session 22 pages and 42 claims; aggregate coverage map;
email graph's four default/server combinations. Post-move Clippy covers both drivers,
email, session and the unpublished app, all targets with warnings denied.

Exact authorized manifest lines (no other manifest ownership assumed):

- Both drivers remove the same dev-dependency:
  `generated-workspace-app = { path = "../../../examples/generated-workspace/app" }`.
- The unpublished app adds these dev lines and test target:

```toml
stackpress-sql-postgres = { path = "../../../crates/sql/sql-postgres" }
stackpress-sql-mysql = { path = "../../../crates/sql/sql-mysql" }
sqlx = { version = "=0.9.0", default-features = false, features = ["runtime-tokio", "postgres", "mysql", "tls-rustls-ring"] }

[[test]]
name = "driver_stores"
harness = false
```

- Email replaces `rustls-no-provider` with `rustls` on this normal dependency:

```toml
reqwest = { version = "=0.13.4", default-features = false, features = ["rustls", "gzip", "deflate", "cookies"] }
```

- Email removes these now-unused dev lines and its relocated example:

```toml
stackpress-framework-sql = { path = "../framework-sql", features = ["codegen", "server"] }
stackpress-idea-transformer = { path = "../../idea/idea-transformer" }
stackpress-framework-schema = { path = "../framework-schema", features = ["codegen", "providers"] }
stackpress-framework-csrf = { path = "../framework-csrf", features = ["server"] }
stackpress-framework-session = { path = "../framework-session", features = ["auth-server"] }
stackpress-sql = { path = "../../sql/sql" }
stackpress-sql-sqlite = { path = "../../sql/sql-sqlite" }
stackpress-framework-view = { path = "../framework-view", features = ["leptos-ssr"] }
stackpress-leptos = { path = "../../leptos", default-features = false, features = ["ssr"] }

[[example]]
name = "regenerate_email_models"
path = "tests/fixtures/regenerate.rs"
```

- Session adds these earlier dev dependencies and the moved example:

```toml
stackpress-framework-csrf = { path = "../framework-csrf", version = "0.1.0", features = ["server"] }
stackpress-framework-view = { path = "../framework-view", version = "0.1.0", features = ["leptos-ssr"] }
stackpress-leptos = { path = "../../leptos", version = "0.1.0", default-features = false, features = ["ssr"] }

[[example]]
name = "regenerate_email_models"
path = "tests/email/fixtures/regenerate.rs"
```

## Live verification and final gate

The coordinator authorized the verified PostgreSQL 12.2/MySQL 8.0.27 services.
First live replay: 56 passed; 18 exact CREATE/DROP pairs, nine/server. The ordinary
workspace coverage run passed 1311 cases with 40 matching pairs, 20/server.
Both runs ended with PostgreSQL `[]` and MySQL `[]`. Exact names, log lines,
hashes, commands and retained failures are in [live evidence](../../../resources/release-coverage-00052/live-verification.md).

The standalone sweep exposed API/session dev-feature gaps and a Leptos cfg mismatch.
Each correction was coordinator-approved; every existing assertion is unchanged.
API and session dev edges now both read exactly:
`stackpress-framework-sql = { path = "../framework-sql", features = ["codegen", "server", "source"] }`.
Repro: `cargo clippy -p stackpress-framework-api --all-targets --locked --offline -- -D warnings`
(and `-p stackpress-framework-session`). Leptos repro uses `-p stackpress-leptos`;
its exact caller cfg is `#[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]`.
All 23 current candidates pass standalone default all-targets Clippy. API's 62
default tests, session freshness, unchanged Leptos SSR policy test and both wasm
renderer Clippy checks pass. The retired models baseline entry is explained in the evidence.

Final gate: all 21 phases pass, including both workspace selections, all eight consumers,
freshness/docs and final ordinary coverage. All 114 gate CREATE/DROP pairs match; both catalogs are empty.
No golden, oracle body, JSON vector or `.stderr` changed. No install, development
listener, commit, push or publication occurred. KB findings remain the inherited
146 errors and 44 warnings; final comparison is recorded in the evidence.
