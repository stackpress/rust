# Issue 14 M1 benchmark ledger

Persistent store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
Each ID below owns `runs/<ID>.json` and `.log`, including exact argv, worktree/base, environment, status and output. All records have `issue14 M1` labels on this worktree. The selected/consumer scope is deliberate; no workspace rebuild was requested or counted.

Recorded child command: `python3 /private/tmp/issue14-m1.py <stage>`, which delegates to the maintained `examples/ui-preferences/proofs/supervise.py` with `--target <worktree>/target/issue14 --seconds 1800`. Installed Node 22.14.0 and Playwright are used without installation. `CARGO_TARGET_DIR`, Profile native/WASM selectors and the nested LLVM target all remain within the one 8 GiB root. Incremental/debug info are off, jobs are two, Cargo is offline and applicable commands are locked. Drivers/evidence are retained at `/private/tmp/stackpress-issue14-m1`.

## Attempts

Times are seconds. `—` means no observed Cargo Finished stage, not zero. The subtotal may omit Cargo stages captured by existing asset/browser scripts; their complete child logs remain in the evidence root. Expected compile-fail cases are contained in successful graph/selection proofs.

| Record ID | Stage / kind / scope / cache | Result | Command | Cargo stages |
| --- | --- | --- | ---: | ---: |
| `20261029T061017.453893Z-977abc861ec0` | clean / clean / selected / unknown | passed | 1.162796 | — |
| `20261029T061028.934423Z-b60a97825c15` | llvm-clean / clean / selected / unknown | passed | 2.138708 | — |
| `20261029T061042.727311Z-4155b6195110` | baseline / test / selected / cold | passed | 150.689005 | 132.00 |
| `20261029T061508.610758Z-c8f69ad092d2` | native / test / selected / warm | passed | 87.956884 | 60.00 |
| `20261029T061757.366203Z-614aaa791f81` | quality / clippy / selected / warm | passed | 97.044637 | 90.47 |
| `20261029T062007.184970Z-297a0c866229` | assets / build / selected / warm | passed | 170.189753 | 59.21 |
| `20261029T062350.873054Z-e1d321611e97` | browsers / test / selected / warm | passed | 155.256877 | — |
| `20261029T062716.127069Z-22d8bf178e7f` | review / test / selected / warm | failed: disposable descriptor | 18.562381 | 12.36 |
| `20261029T062916.457281Z-b611d89ddc2f` | review corrected / test / selected / warm | passed | 12.133942 | 0.76 |
| `20261029T062957.299825Z-0ebe14e333a0` | graphs / check / consumer / warm | passed | 23.153995 | — |
| `20261029T063131.592064Z-f97fe66322e3` | compatibility / test / consumer / warm | passed | 23.324882 | 18.70 |
| `20261029T063233.581406Z-e0474799092f` | coverage / coverage / selected / cold | failed after 61 passing tests: reporter assumption | 94.227637 | 84.00 |
| `20261029T063559.331763Z-8ddde81b4a59` | coverage-report / coverage / selected / warm | passed; retained profiles | 2.443928 | — |
| `20261029T063925.754372Z-ae28f50d6d6f` | cleanup / clean / selected / unknown | passed | 6.707004 | — |
| `20261029T064332.496505Z-0f20e107a831` | audit / other / other / unknown | passed | 1.134217 | — |

These fifteen completed attempts total **846.126646 s command time**, **457.50 s observed Cargo-stage subtotal**, thirteen passes and two resolved harness failures. Seven attempts emitted `Compiling`; zero declared cold full-workspace rebuilds. Attempt counters count outer invocations, not crates/rustc calls; nested commands are not double-wrapped.

Final audit passes exact fifteen-path scope, all **45 local links**, Markdown/whitespace, unchanged root lock, absent target and no matching task native process. Raw KB validation still exits 1 with **146 inherited errors / 27 existing warnings**, including the already-accepted M0 design length advisory; no M1 error or warning is added. Full diagnostics and scope result are retained as `kb.log` and `audit.json` in the evidence root. This final ID/timing annotation adds no link or contract change.

## Exact scope and evidence

- Baseline selected view + ui-preferences web + enabled review-preview all-target suites: 63 pass results and 44 response pins. Native implementation adds Profile web/shared and `view/presentation,native-file`: 98 pass results. View stream support is selected transitively by review-preview; no renderer all-features invocation.
- Quality: affected selected Clippy `-D warnings`, doctests, view `presentation,leptos-ssr` Rustdoc `-D warnings`, isolated view web-preferences/web-flash/both/presentation Clippy and formatting. All eight doctests pass. Markdown complete direct/customized hosts both execute.
- Assets: official preferences builder and official Profile hydrate/CSR builders/manifest checks, plus two mode-specific WASM Clippy selections. Browser driver logs include four preferences flows/40 first-paint cases, two Profile backends with nine invocations each and six native-CSR cases.
- Review proof uses four actual composed-host presentation browser cases with checked assets, a persistent pipe and duplex HTTP. It deliberately does not rebuild all unrelated review UI assets. Failed short-name immutable descriptor is retained in the failed record; revalidation corrected the disposable fixture.
- Graphs: 17 target-selected normal/no-proc-macro graphs, three isolated consumer import selections and the expected existing native-on-WASM adapter refusal. Root Cargo.lock is byte-identical; all 616 external tuples match the task base. Isolated consumer locks contain only approved tuples; no generate-lockfile.
- Coverage native selections: `stackpress-framework-view` + `stackpress-example-ui-preferences-web`, features `view/presentation,native-file,leptos-ssr`, all targets. First measurement passed all 61 tests and retained profiles. Reporter resume contains actual LLVM export argv/raw JSON/LCOV with automatic example exclusion removed. Production per-file counts/exclusions: `coverage.json` (view 1548/1634, web 267/296). No numeric WASM coverage.
- Peak allocated aggregate target: **5,359,168 KiB / 5.11 GiB**, below 8 GiB. Exact owned target deleted; reviewed Profile assets copied to evidence before generated dist cleanup. Persistent records are retained.
- No server/http TCP test selection, including `tests/tcp.rs::tcp_watchdog`; no workspace tests, listener or install. View's port-free stream child is exercised by its watchdog. A preliminary read-only `git diff --check` ran outside the recorder; the final audit repeats it recorded. It performed no build and is excluded from these attempt counters.

The [M1 receipt](00018-issue14-m1.md) gives behavior, limits, cleanup and the stop gate. Formal review rounds and SQL/admin/generation implementation remain unreleased.
