# Task 00012: SQL crate documentation and ambient-TLS fix receipt

2026-10-27, branch `trial/3-s7-sql-m0`. The requested `git merge trial/3` fast-forwarded
to base/HEAD `4b1b077b4aa8c51cb76c8a2bea1ae1a1695ea79e` before this work.
Evidence for [task 00012](00012-slice-7-production-servers.md); independent coordinator
review remains pending. The released docs follow-up and subsequently released scoped
PostgreSQL fix are complete. Generated-store integration remains deferred.

## Scope and authority

Used the repository [chrisai-docing skill](../../../skills/chrisai-docing/SKILL.md),
its guided-learning, API-reference and formatting guidance, the coordinated workflow,
SQLite docs pattern and SQL core tone. Live public types, implementations, native suites
and CODECS.md take precedence over proposal-only API sketches.

The original docs-only scope created the two README/docs trees, changed one cross-reference
line in SQL core docs and added this pre-approved receipt. The coordinator then explicitly
released the discovered PostgreSQL guard correction, a regression and this receipt update.

Production changes are limited to PostgreSQL options construction; tests add one private
environment module and its declaration. MySQL production code, SQL core implementation,
SQLite, manifests, Cargo.lock, facade exports and other lanes are unchanged.
No dependency change, driver fork, listener, installation, commit, push or publication.

The MySQL conservative reuse policy and PostgreSQL domain wire-type boundary are
coordinator decisions, not owner decisions. Documentation preserves both.

## Documentation delivered

Each crate has 13 pages: root README; docs index, getting started and concepts;
guides for configuration switching, transactions/cancellation and isolated testing;
API index plus options, TLS, database, codecs and limits/errors pages.

| Crate | Entry | Rust snippets compiled |
| --- | --- | --- |
| stackpress-sql-postgres | [PostgreSQL documentation](../../../../crates/sql/sql-postgres/docs/README.md) | 8 / 8 |
| stackpress-sql-mysql | [MySQL documentation](../../../../crates/sql/sql-mysql/docs/README.md) | 8 / 8 |

Pages cover explicit defaults/bounds, verified TLS/private CA/plaintext, eager Engine open,
secret resolution, config-only selection among compiled adapters, strict native codecs and
typed NULL, canonical JSON text, unsupported types, domains, unsigned/bool behavior,
native arity/outcomes, exclusive transactions, rollback-only state, consuming completion,
unknown commit outcome, no automatic retry, primary/cleanup error precedence, PostgreSQL
DISCARD ALL, MySQL's exact allowlist/hidden-trigger assumption, pool bounds, deadlines,
close/drain, error disclosure, opt-in fixtures and platform/integration limits.

The config schema is application code, not an invented package API. It constructs real
options to obtain validated defaults and passes the selected password separately.
Its SQLite variant constructs SqliteOptions rather than assuming direct deserialization
defaults. The PostgreSQL options/TLS/quick-start pages now describe the enforced ambient
rule; they no longer present the discovered defect as a current limitation.
MySQL docs accurately state that SQLx appends NO_ENGINE_SUBSTITUTION/PIPES_AS_CONCAT
to the server mode; the adapter does not force STRICT_TRANS_TABLES/NO_BACKSLASH_ESCAPES.

## Ambient guard correction and source audit

[PostgreSQL options](../../../../crates/sql/sql-postgres/src/options.rs) previously checked
`PGPgROOTCERT`, `PGPgCERT` and `PGPgKEY` instead of SQLx's actual TLS names.
The corrected guard rejects `PGSSLROOTCERT`, `PGSSLCERT`, `PGSSLKEY` and existing
`PGOPTIONS` before native options construction. Presence, including an empty/non-UTF-8
value, is sufficient. Each static `Configuration` context names the variable without
formatting its value. The rule applies with every TLS mode.

Audited SQLx `sqlx-postgres =0.9.0`, `src/options/mod.rs` lines 56-100, its setters
and `options/pgpass.rs`. All 12 constructor reads are accounted for:

| Ambient names | Disposition |
| --- | --- |
| `PGSSLROOTCERT`, `PGSSLCERT`, `PGSSLKEY`, `PGOPTIONS` | Rejected: no public clearing operation |
| `PGHOST`, `PGHOSTADDR`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE` | Explicit connection setters overwrite |
| `PGSSLMODE` | Selected TLS enum overwrites |
| `PGAPPNAME` | Fixed `stackpress` application name overwrites |
| `PGPASSFILE` / default passfile | Separate loader bypassed by `new_without_pgpass` |

No additional unoverridden ambient input was found in that options source. Shape-only
`validate()` remains independent of environment; open enforces the guard.
Machine-readable inventory: `/private/tmp/s7-sql-docs/sqlx-ambient-audit.json`.

## Regression and test-verified review rounds

[Regression module](../../../../crates/sql/sql-postgres/src/tests/environment.rs)
runs the exact unit test in eight isolated children: four variables, each with a synthetic
nonempty value and an empty value. It removes actual/legacy guard variables in each child's
Command environment and never calls global set_var/remove_var or unsafe code.

Each child asserts options construction rejects before native IO, retains Configuration,
names its selected variable and hides the synthetic value in Display/Debug. Parent output
contains names/empty-value flags only. A five-second watchdog kills/reaps a stuck child;
null stdio avoids pipe backpressure. This regression creates no database or connection.

- Before the fix, `fix-before.log` failed all eight cases: TLS names escaped rejection;
  PGOPTIONS was rejected but its old generic context did not identify the variable.
- After the fix, `fix-after.log` passed. The final full offline suite also passes it.
  `fix-regression.json` records the before/after boundary.
- Initial Clippy rejected the test's err().expect() idiom. An explicit match replaces it,
  avoiding expect_err's unexpected-success formatting of secret-bearing driver options.
  The original lint failure remains in `fix-clippy.log`; final Clippy is clean.

Review rounds after the passing baseline:

1. Logic/complexity: verified presence semantics, all four fixed contexts, existing
   Configuration classification and pre-construction ordering; regression replay passes.
2. Responsibility/abstraction: guard remains with options policy; environment isolation
   and watchdog remain private test support. No new public abstraction needed.
   `fix-final-offline.log` passes after the explicit-match cleanup.
3. Consistency/Rustdoc/comments: aligned docs with the enforced rule and audited source,
   retained static secret-free contexts and explanatory comments. Final fmt, Clippy,
   Rustdoc, doctests and all 16 Markdown consumer snippets pass.
4. Safety/soundness: reviewed no parent environment mutation, no native regression IO,
   exact child selection, bounded kill/reap and diagnostic disclosure. Final offline and
   native M0 replays pass; no further edit was justified.

## Verification and commands

Evidence is under `/private/tmp/s7-sql-docs/`. Rust/Cargo: `+1.96.0`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`. Fresh cargo/llvm-cov workspace cleans
preceded building; the same single worktree target path served all phases and the
disposable consumer. Its first offline resolution created only its private lockfile.

| Final check | Result / log |
| --- | --- |
| Full offline tests, both gates unset | 18 unit tests, SQLite reference, 14 explicit native ignores, 8 doctests; `fix-final-offline.log` |
| Both crates, all-target Clippy -D warnings | Passed; `fix-final-clippy.log` |
| Doctests, gates unset / set | 4 per crate each; `fix-final-doctests-offline.log`, `fix-doctests-gated.log` |
| Native M0, both gates set | Six lifecycle checks per backend; `fix-native-m0.log` |
| All Markdown Rust examples + facade aliases | 16 / 16 and facade compile; `fix-consumer-check.log`, `examples.json` |
| JSON config parsing / unknown adapter rejection | Both passed; `config-postgres.log`, `config-mysql.log` |
| TOML/JSON syntax and Markdown scan | 26 pages, 73 links/anchors, no issues; `markdown-check.log` |
| Rustdoc warnings denied / workspace fmt | Passed; `fix-rustdoc.log`, `fix-fmt.log` |
| Copy parity, hashes, whitespace and scope | Passed; `docs-sha256.json`, `final-audit.json` |

The disposable `/private/tmp/s7-sql-docs/consumer/` uses equivalent absolute path deps
and CARGO_TARGET_DIR pointing at this worktree's target. Visible snippets contain no
hidden Rustdoc wrappers; only compilation scaffolding is added to the extracted copies.
Config JSON/TOML parses, and the application config executables ran without opening a DB.

```sh
cargo +1.96.0 test -p stackpress-sql-postgres -p stackpress-sql-mysql --offline --locked
cargo +1.96.0 clippy -p stackpress-sql-postgres -p stackpress-sql-mysql --all-targets --offline --locked -- -D warnings
cargo +1.96.0 test -p stackpress-sql-postgres -p stackpress-sql-mysql --doc --offline --locked
cargo +1.96.0 test -p stackpress-sql-postgres -p stackpress-sql-mysql --test m0 --offline --locked
cargo +1.96.0 check --manifest-path /private/tmp/s7-sql-docs/consumer/Cargo.toml --bins --offline --locked
RUSTDOCFLAGS='-D warnings' cargo +1.96.0 doc -p stackpress-sql-postgres -p stackpress-sql-mysql --no-deps --offline --locked
cargo +1.96.0 fmt --all -- --check
```

Connection gates were supplied privately; URLs/credentials are not retained. Native
fixture subprocesses clear the four blocked ambient fields and use explicit test plaintext.
Final offline/Clippy/ungated-doctest durations: 6.913 / 1.385 / 2.109 s.
Gated doctests/native M0: 3.637 / 2.113 s. Durations are observations, not throughput claims.

Initial docs-only proof logs remain: `consumer-check.log`, `doctests-offline.log`,
`doctests-gated.log`, `native-m0.log`, `rustdoc.log`, `fmt.log`. Initial gated
doctests/M0 took 3.219 / 4.325 s. New first-success programs were compiled, not run
against a deployment. Rustdoc no_run examples remain compile-only with gates set;
doctests create no DB. Numeric coverage and native M1/M2 were not remeasured in this
scoped fix; their [merged M2 evidence](00012-receipt-sql-m2.md) is not fresh evidence.

## Exact fixture ledger

Only M0 native runners created databases. Every unique CREATE armed exact-name cleanup
only after success. Existing parent watchdogs kill/reap, close owned connections and DROP
only that name; cleanup has its independent 9 s IO / 10 s wall deadline.
No enumeration, catalog analysis, role/global mutation, unrelated session termination,
adoption, prefix sweep or PostgreSQL FORCE drop occurred. All four DROP acknowledgements:

| Backend | Exact database | Log / confirmed |
| --- | --- | --- |
| MySQL | `stackpress_s7_m0_6876_18e23fc00acdc170` | `native-m0.log` / yes |
| PostgreSQL | `stackpress_s7_m0_6878_18e23fc066f4ce08` | `native-m0.log` / yes |
| MySQL | `stackpress_s7_m0_16e1_18e240a946a3cf70` | `fix-native-m0.log` / yes |
| PostgreSQL | `stackpress_s7_m0_16e3_18e240a971964190` | `fix-native-m0.log` / yes |

Offline SQLite reference files below were also removed with their sidecars, confirmed
in `fix-offline.log` / `fix-final-offline.log`. Both used the OS temporary directory:
`stackpress_s7_m2_sqlite_b8c_18e24096708b9678.sqlite` and
`stackpress_s7_m2_sqlite_1060_18e2409ead8d5590.sqlite`.
There was no unconfirmed owned cleanup or failed native run.

## Closeout and limitations

The initial docs clean removed 2736 files / 561.3 MiB. Final cleanup after the scoped fix:
removed 2784 files, 533.7MiB total. The worktree target is absent; logs/disposable source remain outside it.

The Agent Workspace validator retains exactly its baseline 146 errors / 22 warnings,
principally external sibling-source links and existing length/link-text warnings.
No new receipt diagnostic was introduced; baseline repair is outside this write set.

TLS interoperability, other platforms/server versions, external poolers, hidden MySQL
trigger/session effects and generated-store integration retain their existing limitations.
No whole-Slice-7 acceptance or publication readiness is claimed. Stop for coordinator review.
