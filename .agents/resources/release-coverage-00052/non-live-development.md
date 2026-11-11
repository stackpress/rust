# Task 00052 non-live development evidence

Historical proof details are separated from the current receipt so the receipt
keeps the release result, ownership decisions and exact manifest edits together.

## Review rounds and evidence

1. Logic/complexity: checked projection, coercion, host authority and error precedence;
   the new test exposed the email provider bug, which was stopped/reported and then
   fixed under explicit coordinator approval. Email-only 60 tests and API-alone
   construction passed; 30 focused coverage tests passed after the fix.
2. Responsibility/abstraction: removed the generated-store runner's unused cleanup
   flag; backend parents retain exact CREATE/DROP authority. Strengthened new SQL
   tests to assert table identity, bind data and filters. Removed a copied MySQL
   no-op PostgreSQL retry path. Seventeen targeted tests passed.
3. Consistency/docs: formatted new code, corrected fixture comments, updated test
   layout/live-gate guides and oracle/length links, and registered the copies.
   Existing email/CSRF docs checker: 27 pages, 21 Rust fences, 18 claim anchors passed.
   Fresh compatible workspace replay passed 1267 broad tests, the isolated TCP
   fixture, then four serial example/final-branch cases. Selected-five Clippy
   passed with warnings denied. The final inventory helper compiles and the
   generated-store SQLite case passed; neither server was contacted.

Untouched selected-five baseline: 194 passed, 17 skipped; PG 227/889 (25.53%),
MySQL 414/1049 (39.47%), SQL 6695/8629 (77.59%), admin 3569/4364 (81.78%),
email 2423/2739 (88.46%). SQL/admin selected-five scope lacks other consumers and
is not directly comparable to the publication plan's workspace baseline.
The first workspace non-live export before later gap tests was PG 369/889 (41.51%),
MySQL 468/1049 (44.61%), SQL 7646/8629 (88.61%), admin 3893/4364 (89.21%),
email 2460/2739 (89.81%). Accumulated reports across later source edits were diagnostic.

## Initial proof records

All commands use `.agents/scripts/build-benchmark.py`, the required shared store,
sccache, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two build jobs, and
`target/00052`. `command_seconds` is end-to-end; `cargo_build_seconds` includes
only observed Cargo stages and may be null for capturing outer scripts.

| Selected record ID | Result |
| --- | --- |
| `20261111T022037.031219Z-63cf78317d40` | Untouched selected-five non-live baseline. |
| `20261111T023901.737970Z-e4f5e55dce74` | First broad non-live run: 1254 passed, two unrelated example child timeouts retained. |
| `20261111T025924.265154Z-9b0751d4d9c6` | Both examples passed alone/serially, 2.663 s and 2.042 s; no deadline/code change. |
| `20261111T030405.865843Z-32e626aa8cd9` | Retained email panic reproduction: 27/28 focused cases passed. |
| `20261111T030922.444485Z-8e7b0fe040a5` | API-alone webhook client construction passed. |
| `20261111T031014.663337Z-636911c75ba7` | Email alone: 60/60 passed. |
| `20261111T031141.567835Z-be1133cbe91a` | Thirty focused coverage tests passed after TLS fix. |
| `20261111T031335.371634Z-29b98244cd9c` | Approved root-seeded email lock refresh passed. |
| `20261111T031410.684569Z-781a70a3a860` | Length proof failed without its documented owned TMPDIR; retained. |
| `20261111T031717.131191Z-ff1068f7fd8b` | Documented TMPDIR replay: generation, freshness, SQLite, snapshots passed. |
| `20261111T031815.445132Z-2fb655988421` | Seventeen responsibility-round tests passed. |
| `20261111T032157.115912Z-56716115f626` | Existing email/CSRF documentation checker passed after incoming-link correction. |
| `20261111T032311.431626Z-8a2668a0fcb6` | Copy/boundary/canonical/699-tuple assertions passed. |
| `20261111T032057.997615Z-7dff6c65e695` | Fresh broad coverage: 1267 passed; command 504.061 s, observed Cargo stages 143 s. |
| `20261111T033037.784897Z-072ffcf97a78` | Authorized isolated TCP watchdog passed. |
| `20261111T033229.784361Z-3f9102430749` | Serial examples and final SQL/admin branches: 4 passed; command 16.592 s, build 3.02 s. |
| `20261111T032515.854297Z-dc4ad0717be7` | Three standalone email consumers: format/Clippy/execution passed; command 243.825 s, observed Cargo stages 236.91 s. |
| `20261111T033340.920210Z-5641e801a23a` | Final inventory helper compile and generated-store SQLite case passed. |
| `20261111T033525.565776Z-ac4df11a405a` | All five owned crates: selected compatible features, all-targets Clippy with -D warnings passed. |
| `20261111T033529.218163Z-c24392f33a85` | Final copy identities, moved oracle bodies, fixture/stderr and other 18 nested locks verified. |
| `20261111T032746.490006Z-3bebc438105e` | KB validator comparison: unchanged 146 inherited errors and 44 warnings. |

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.


## Package-order evidence

Standalone temporary package copies remove both forbidden later dev edges from
each driver. Their all-targets non-live suites pass: PG **19**, MySQL **18**;
each reports nine native cases skipped. Email's package now has only Tokio as a
dev-dependency; its standalone copy passes **39** default and **52** server tests,
including the retained HTTP regression and HTTP/SMTP construction. Resolved
registry tuples are subsets of root 699; root and every tracked nested lock stayed
byte-identical during this proof. Result rows are retained in
`.agents/resources/release-coverage-00052/package-order-proof.json`.


## Publication-order proof records

| Follow-up record | Result |
| --- | --- |
| `20261111T043429.623643Z-590be9853467` | One offline workspace-edge resolution; 699 registry tuples and nested bytes preserved. |
| `20261111T043455.037744Z-851323a82f73` | Format after relocation. |
| `20261111T043505.217052Z-a1808ac95389` | Drivers 37, app SQLite 1, email default 39/server 52 passed; retained missing-CSRF-feature compile failure. |
| `20261111T043645.508975Z-10c29129a6af` | Stripped package-copy suites passed. |
| `20261111T043658.111663Z-2f7ff665742a` | 26 copies, 19 moved email files, diagnostics and local-reference identities passed. |
| `20261111T043725.704989Z-26460b18594a` | Relocated email 8 + 8 and model freshness passed. |
| `20261111T043729.547137Z-740ae27ef062` | Updated incoming docs, claims, aggregate map and email feature graph passed. |
| `20261111T043854.270519Z-e6b056ce7ddd` | Compatible workspace coverage replay of moved lanes: 9 passed, two live skipped; all three framework floors retained. |
| `20261111T043923.214346Z-d76a5ec90e8d` | Post-move affected all-targets Clippy with -D warnings passed. |


## Final relocation integrity and KB records

The length warning on the expanded receipt was resolved by this lossless split;
the receipt retains every exact manifest edit while this file owns development logs.

| Record | Result |
| --- | --- |
| `20261111T044126.575384Z-d569d4e46820` | Copy/move identities, local references, 699 tuples, format and diff checks passed. |
| `20261111T044129.972077Z-91161099911e` | Retained KB comparison failure: receipt length warning, resolved by routing this ledger separately. |
| `20261111T044213.762199Z-21a8f5a5ee6b` | KB validator still exits 1 with exactly the inherited 146 errors and 44 warnings; no new finding. |
