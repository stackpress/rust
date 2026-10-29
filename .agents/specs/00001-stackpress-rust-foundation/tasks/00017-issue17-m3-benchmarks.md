# Issue 17 M3 benchmark ledger

Complete implementer-attempt ledger for `trial/3-issue17-leptos-library`, based on `dcca20a6d9d66da4475bd295dcea79ae082a0f56`. Read with the [M3 receipt](00017-issue17-m3.md). Persistent store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Each ID owns `runs/<ID>.json` and `.log`, including argv, environment, host/toolchain, exit status and output. No prior record was edited or deleted.

All Cargo, proof and clean commands used the recorder once around the outer finite supervisor/proof driver. Maintained proof drivers remain in the affected examples; the independent entry is `examples/leptos-independent/proof.py`; disposable drivers: `/private/tmp/stackpress-issue17-m3`; reports/copied artifacts: its `evidence` directory. Browser/build/doc/graph subcommand argv remain in the driver or retained logs. Captured output in graph/import cases is retained separately under `evidence/independent/graphs` rather than reconstructed as recorder compilation events.

## Counters and timing

- Allocated records: **+27** (failed 2, passed 25), plus **one pre-allocation CLI rejection** described below. Kinds: build 2, check 2, clean 5, clippy 2, coverage 2, other 2, test 12. Scopes: consumer 2, selected 21, workspace 4. Cache declarations: cold 4, unknown 3, warm 20. No Cargo/proof child ran in that rejected invocation.
- Attempts emitting visible compilation: **+14**. Observed events: 1207 Compiling, 603 Checking, 6 Documenting.
- Declared cold full-workspace rebuild counter: **+0**. Builds/tests select the isolated consumer or named packages, not the entire root workspace. Coverage is cold for its previously empty nested LLVM targets. Recorded cleans separate native/coverage, ordinary browser and generated-admin phases within the same aggregate target root. Initial clean commands are not builds.
- Accumulated finished command time: **3881.836393 s**. Observed Cargo-stage subtotal: **1145.480000 s**, available in 15/27 attempts. Null remains unavailable. Wall time includes test/proof/browser/bindgen work; observed rounded Cargo stages are neither compiler CPU time nor interchangeable with wall time. Concurrent attempts overlap, so summed durations are not elapsed calendar time.
- Highest supervisor-observed aggregate task target: **7,166,240 KiB (6.834 GiB)**, below 8 GiB. All compilation/LLVM output is under `target/issue17`; final cleanup removes it. Reports, screenshots and copied finite review artifacts remain outside that target.

## Every attempt

Times are seconds. `—` means unavailable. C/K/D are observed Compiling/Checking/Documenting counts. The metadata record supplies full commands; labels describe the actual gate. Internal expected invalid-mode/import failures are assertions inside a passing graph audit and are separate from failed outer attempts.

| Record ID | Result | Kind / scope / cache | Intent | Command | Cargo stages | C/K/D |
| --- | --- | --- | --- | ---: | ---: | ---: |
| `20261029T025006.943797Z-f5b4a75d8759` | passed | clean / workspace / unknown | fresh task target clean | 1.045634 | — | 0/0/0 |
| `20261029T025023.454456Z-669f0024094c` | passed | clean / workspace / unknown | fresh LLVM target clean | 1.046875 | — | 0/0/0 |
| `20261029T025044.488357Z-a7da30be146b` | passed | test / selected / cold | baseline renderer suites and pinned outputs | 142.510054 | 131.540000 | 294/0/0 |
| `20261029T025434.198356Z-f96fa81773fb` | passed | test / selected / warm | round1 worker control flow native suites exact parity | 40.432292 | 30.290000 | 4/0/0 |
| `20261029T025727.279589Z-2d470b825b1a` | failed | test / selected / warm | round2 ownership shell and admission native parity | 19.730606 | — | 2/0/0 |
| `20261029T025801.505997Z-26b522a00dac` | passed | test / selected / warm | round2 native parity corrected test API | 30.136000 | 17.100000 | 4/0/0 |
| `20261029T025907.876736Z-b50acf9fba40` | passed | test / consumer / warm | round2 independent consumer shell native lifecycle | 119.438199 | 112.120000 | 56/198/1 |
| `20261029T030517.535037Z-a6a8bdbc4d06` | passed | test / selected / warm | round3 Rustdoc comments and event readability native parity | 43.980123 | 31.140000 | 4/0/0 |
| `20261029T030641.762759Z-646cbd4d5b84` | passed | test / selected / warm | round3 native facade admin generators doctests lint docs | 163.061192 | 130.330000 | 134/73/4 |
| `20261029T031154.602029Z-4bedce883099` | passed | test / selected / warm | round4 exact observable capacity safety and pinned outputs | 10.780326 | 4.800000 | 2/0/0 |
| `20261029T031408.286661Z-a8f2b4ca27f3` | passed | test / selected / warm | final native consumers SSR admin G41 independent lint | 75.715280 | 57.710000 | 12/7/1 |
| `20261029T031610.938829Z-7203d081b1b9` | passed | check / selected / warm | final graph API gates and literal Markdown examples | 315.632733 | 117.550000 | 25/128/0 |
| `20261029T032255.954977Z-bb8f5a6fc2b3` | failed | coverage / selected / cold | changed crate native production coverage and Send proof | 237.547052 | 218.000000 | 432/0/0 |
| `20261029T032916.698473Z-1d7980d6028c` | passed | coverage / consumer / warm | resume consumer coverage with retained native and SSR profiles | 71.625951 | 65.000000 | 111/0/0 |
| `20261029T033336.062451Z-96a55dfdf1ee` | passed | test / selected / warm | round4 followup page progress barrier normal Send and byte parity | 12.166477 | 5.750000 | 2/0/0 |
| `20261029T033539.442792Z-dbaf992f3672` | passed | check / selected / warm | final documentation structure links and lock audit | 1.286589 | — | 0/0/0 |
| `20261029T033619.630614Z-f24ec349c3eb` | passed | clippy / selected / warm | safety followup warning denied test lint | 9.453275 | 8.470000 | 0/2/0 |
| `20261029T033937.527142Z-e65d6d19a10f` | passed | clean / workspace / warm | release native and LLVM cache before browser matrix | 9.001788 | — | 0/0/0 |
| `20261029T034012.420653Z-774bda8e7bb7` | passed | build / selected / cold | actual separate browser mode artifacts all touched UI families | 1192.340150 | 153.520000 | 125/0/0 |
| `20261029T040318.118681Z-e6eaef95c3e1` | passed | test / selected / warm | finite browser regressions affected UI and independent helper | 389.689186 | — | 0/0/0 |
| `20261029T041124.516120Z-cefd8df6a1ac` | passed | clippy / selected / warm | warning denied browser mode lint for Leptos and view | 63.364362 | 62.160000 | 0/195/0 |
| `20261029T041356.380118Z-2a052950639c` | passed | clean / workspace / warm | release ordinary browser cache before generated admin matrix | 8.711273 | — | 0/0/0 |
| `20261029T041427.762176Z-6f383096b836` | passed | build / selected / cold | generated admin separate role CRUD format plugin transfer artifacts | 539.864675 | — | 0/0/0 |
| `20261029T042411.750009Z-beac328204ad` | passed | test / selected / warm | generated admin role CRUD format plugin transfer race browser regressions | 375.769944 | — | 0/0/0 |
| `20261029T043100.988845Z-a5ab9b96b86d` | passed | other / selected / warm | final receipt scope immutable policies locks parity fmt and inherited KB audit | 3.221179 | — | 0/0/0 |
| `20261029T043133.971808Z-ba59254264fe` | passed | clean / selected / warm | final exact owned target removal | 3.229962 | — | 0/0/0 |
| `20261029T043227.587641Z-8705b0ab8801` | passed | other / selected / unknown | final cleanup reaping retained artifacts and unchanged locks audit | 1.055216 | — | 0/0/0 |

## Failed attempts and interpretation

Round 2 first failed because the new admission-order test named a nonexistent ViewError variant and assumed PageSnapshot was Clone. The fixture now uses the established InvalidSnapshot category and constructs a fresh snapshot for each adapter; production error policy was unchanged. The first coverage invocation completed native Leptos/view tests and measurements and the consumer SSR pass, then rejected the unsupported --no-clean/--no-report flag combination. The resumed pass used --no-clean with JSON output, preserving all completed native and SSR profiles. Both failures and their partial timings remain in this ledger. Any later failed attempt is identified by its exact label and retained log in the table.

The first final-cleanup-audit recorder invocation used cache=none. The recorder parser exited 2 before allocating an ID or spawning the supervisor/proof: supported values are cold, warm and unknown. It was corrected to unknown and the recorded audit passed. The captured diagnostic and exact attempted argv are retained in /private/tmp/stackpress-issue17-m3/evidence/cleanup-recorder-cli-rejection.txt. This extra invocation has no recorder timing, compilation events or fabricated record ID and is outside the numeric record subtotals above.

The final raw KB validator exits 1 with the inherited worktree path diagnostics disclosed in the receipt. The enclosing audit passes only by comparing the exact diagnostic set against the recorded baseline and rejecting new or own-file errors; it does not report raw KB validation as passing. Failed outer attempts remain in the table and totals.

Final records cover receipt/scope/lock/fmt validation, exact owned-target removal, retained evidence and absence of matching task proof/native processes. No task target is retained after M3. Coordinator independent verification and owner/platform acceptance remain pending.
