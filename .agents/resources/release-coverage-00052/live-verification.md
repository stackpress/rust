# Task 00052 live verification

The coordinator authorized live replay after verifying PostgreSQL 12.2 on
localhost:5432 and MySQL 8.0.27 on 127.0.0.1:3306, passwordless, with no existing
`stackpress_*` databases. Tests create unique names without IF NOT EXISTS, arm
cleanup only after their own CREATE succeeds, reap owned children and DROP only
that exact name on success, failure and unwind. Catalog evidence never adopts or
sweeps databases. Administrative and parent watchdog limits are unchanged.

## First live replay and coverage

The five crates' new/existing driver suites and relocated generated-store lane
passed **56 tests**. All **18 CREATE/DROP pairs** match, nine per server, and both
final inventories are empty. Exact names, log line pairs and SHA-256:
[first-live-replay.json](first-live-replay.json).
Record `20261111T063908.309538Z-e5caa13c198c`: command 28.340393 s,
observed Cargo stages 9.70 s.

The ordinary `cargo llvm-cov nextest` workspace report, exact compatible feature
set in [features.txt](features.txt), passed **1311 tests**, with 60 skipped. It uses
neither a custom export nor new exclusions. The former load-sensitive examples
and TCP watchdog passed in the ordinary sweep. All eight moved email integration
cases ran from session; these own SQLite files and have no live SMTP/HTTP service.
The framework live-search suites also ran with both URL gates.

| Crate | Covered / source lines | Live-inclusive | Non-live |
| --- | --- | --- | --- |
| PostgreSQL | 842 / 889 | 94.7132% | 41.5073% |
| MySQL | 986 / 1049 | 93.9943% | 44.6139% |
| Framework SQL | 7803 / 8629 | 90.4276% | 90.0104% |
| Framework admin | 3929 / 4364 | 90.0321% | 90.0321% |
| Framework email | 2471 / 2742 | 90.1167% | 90.1167% |

Counts sum every mapped `src/**` file in the ordinary text report, matching the
publication plan's attribution. Record `20261111T063957.121800Z-d7376e4de16d`:
command 401.500712 s; observed Cargo 127.13 s. Its **40 CREATE/DROP pairs**, 20 per
server, all match; both final catalogs are empty. The raw benchmark log hash,
per-crate counts and exact pairs are in
[initial-live-inclusive-summary.json](initial-live-inclusive-summary.json).
Final gate coverage below was measured fresh after the approved corrections.

## Approved standalone findings

The final gate first stopped at default all-targets Clippy in API. Removing email's
later dev edges exposed generated-fixture source exports previously supplied by
workspace feature unification. No driver runtime failure or test oracle change
occurred. The coordinator approved minimal earlier dev-feature declarations, plus
a sweep with each current candidate selected alone.

| Finding and reproduction | Correction | Proof |
| --- | --- | --- |
| API default generated aliases cannot find framework SQL NativeStore/NativeActions/SessionModels/ApiModels. `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`. | API dev framework-sql features `["codegen", "server", "source"]`, preserving server. | API-alone default Clippy and 62 nextest tests pass; `20261111T064932.008810Z-f36011964810`. Failed gate record `20261111T064721.774641Z-000eaf34b611` is retained. |
| Session default generated aliases have the same absent source exports. `cargo clippy -p stackpress-framework-session --all-targets --locked --offline -- -D warnings`. | Session dev framework-sql features `["codegen", "server", "source"]`. | Standalone default freshness test passes: `20261111T065659.804943Z-bb0e70973601`; standalone Clippy passes in final scan. |
| Leptos default lib-test compiles returns_focus with no caller. `cargo clippy -p stackpress-leptos --all-targets --locked --offline -- -D warnings`. | Exact caller union `#[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]`: real DOM caller is wasm-only and the existing policy test is SSR-only. No assertion/suppression/behavior change. | Default all-targets Clippy, unchanged native_dialog_mode_policies SSR test, hydrate and CSR wasm library Clippy pass: `20261111T065551.777435Z-ea4dca81a53e`. |

All **23 current publication candidates** pass standalone default all-targets
Clippy, not a combined package selection. [Standalone rows](standalone-candidate-clippy.json)
retain all scan records, including failures before correction. The 00047 baseline
also named framework-models, retired by G59; its attempted package lookup is
recorded and it is absent from the current manifest inventory.

The same final gate resumes after corrections. A changed Rust cfg justifies
revalidating formatting and compatible Clippy; otherwise passed phases are kept.
Every failed attempt stays in the benchmark ledger. No deadline, assertion,
golden, oracle, stderr fixture or JSON vector was changed.

## Completed final gate

All 21 required phases pass. Default nextest: 1081 broad + one isolated watchdog
+ two serial finite examples = **1084**. Compatible: 1308 + one + two = **1311**.
Both Clippy selections, doctests and Rustdoc with warnings denied pass. All eight
consumers, build-host --check, generated-workspace/admin freshness, email fixture
freshness, documentation links/claims and feature-graph combinations pass.

The final ordinary compatible workspace coverage executes **1311** tests, with
60 skipped. No special export, new exclusion or modified oracle was used.

| Crate | Final covered / lines | Final live-inclusive | Non-live |
| --- | --- | --- | --- |
| PostgreSQL | 842 / 889 | 94.7132% | 41.5073% |
| MySQL | 986 / 1049 | 93.9943% | 44.6139% |
| Framework SQL | 7803 / 8629 | 90.4276% | 90.0104% |
| Framework admin | 3929 / 4364 | 90.0321% | 90.0321% |
| Framework email | 2471 / 2742 | 90.1167% | 90.1167% |

Final coverage record: `20261111T073556.918740Z-10ca57ba6cd0`; command
528.159718 s, observed Cargo stages 263.57 s.
Its exact counts, ordinary report log/hash, CREATE/DROP pairs and final inventory
record are in [final-live-inclusive-summary.json](final-live-inclusive-summary.json).
All **114** gate database CREATE/DROP pairs match; final PostgreSQL `[]` and MySQL
`[]` are read-only evidence. The prior first replay and initial coverage retain
their separate 18 and 40 pairs. No database was adopted or swept.

[Complete gate records](final-gate-summary.json) retain each command, its status,
log SHA-256 and separate command/Cargo timings, including the stopped default
Clippy attempt. The loop resumed after approved corrections. Root lock and all
21 tracked nested locks stayed byte-identical throughout the live gate; the
root registry remains exactly the original 699 tuples. Other 18 nested locks
remain identical to HEAD; only the three previously approved email lock refreshes
differ from HEAD. Recorded cleanup removes only this effort's completed build
outputs. No source, oracle or test expectation was deleted for coverage.

Final collection/invariant proof: `20261111T074519.668636Z-592295afe2fb` passed.
All 26 registered copies, 19 moved email files, four moved oracle bodies, existing
stderr fixtures and shared canonical helpers retain their required identities.
Crate-local Rust literal references: PG 25, MySQL 25, SQL 78, admin 20, email 17.
The target directory finishes at approximately 22 GiB.

Final KB comparison: `20261111T074605.245500Z-d51868af8e63` passes the
inherited-findings assertion. The underlying validator still exits 1 with exactly
146 inherited errors and 44 inherited warnings; no new finding was added.
