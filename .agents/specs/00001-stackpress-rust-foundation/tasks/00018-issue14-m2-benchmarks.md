# Issue 14 M2 benchmark ledger

Persistent store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Each record ID owns `runs/<ID>.json` and `.log`, including exact command, source head, status and timing. Task drivers and detailed child logs are retained at `/private/tmp/stackpress-issue14-m2`.

The outer driver is `python3 /private/tmp/issue14-m2.py <edit|gate> <argv...>`, with the maintained 8 GiB supervisor. Ordinary-capacity recovery runs its explicit clean-only driver directly so recovery can start from a guard-stopped target. Cargo/proof subprocesses are wrapped once at the outer boundary; nested commands are not double-counted.

Environment: sccache/default SCCACHE_DIR/20G cache; jobs=2; dev/test debug=0; offline; locked where applicable. Edit tier uses explicit Cargo profile incremental=true (coordinator-approved correction for sccache 0.19); gate/coverage incremental=0. One warm `target/issue14` root, nested native coverage namespace. The initial empty target is labelled cold even though sccache is shared. No renderer all-features build.

## Recorded attempts

Times are seconds. Command time and observed Cargo-stage subtotals are distinct. Browser builders retain their Cargo output in child logs, so the store subtotal is incomplete for those rows; `—` never means no build happened. Expected negative compilations are contained in successful proof records.

| Record ID | Label suffix | Result | Command | Cargo stages |
| --- | --- | --- | ---: | ---: |
| `20261029T103630.357744Z-19bb3fabcf1f` | edit baseline warm sccache | failed | 1.129155 | — |
| `20261029T103724.696546Z-ada104b1da58` | edit baseline profile incremental sccache | failed | 131.022865 | — |
| `20261029T104021.106048Z-1abe9f446624` | edit official template generation stable check | passed | 136.707537 | 121.08 |
| `20261029T104441.718758Z-b438cb41007b` | edit format source | passed | 3.447343 | — |
| `20261029T104458.173001Z-7e02f49b898f` | edit selected crates and dependents | passed | 86.813841 | 42.61 |
| `20261029T104717.725250Z-b8fab11b1b29` | gate normal graph boundaries | passed | 9.109391 | — |
| `20261029T104812.170987Z-6388c64449b1` | gate full workspace no tcp watchdog | passed | 549.637605 | 247.96 |
| `20261029T110044.069495Z-17aac4010c84` | gate near-cap stale incremental release | failed | 1.875329 | — |
| `20261029T110102.045964Z-e40ee1a259d2` | gate admin roles SSR hydrate CSR browser | passed | 283.655231 | — |
| `20261029T110131.866281Z-d5bcef3aba2a` | gate near-cap incremental release rounded threshold | failed | 1.693472 | — |
| `20261029T110255.727925Z-4cc514a683b4` | gate inherited KB diagnostic baseline | passed | 1.489902 | — |
| `20261029T110557.113512Z-f767d080407c` | gate isolated entry point feature selection | passed | 154.621320 | — |
| `20261029T110732.144146Z-6e1d44a044ea` | gate admin CRUD transfer and races browser | passed | 137.125855 | — |
| `20261029T110917.162745Z-e2ea18ae4df8` | gate admin formatted replacement SSR hydrate CSR browser | passed | 338.388843 | — |
| `20261029T110940.870524Z-2128ed62fc66` | gate coverage tool nextest interface | passed | 1.441349 | — |
| `20261029T111306.680929Z-b3b3c210080b` | gate near-cap release after browser builds | passed | 2.882153 | — |
| `20261029T111341.525687Z-17e3c0c57790` | gate formatted native quality and generator freshness | failed | 186.189587 | 119.10 |
| `20261029T111813.785378Z-e6d70b8b38b1` | gate corrected generator executable freshness | failed | 11.150994 | 1.89 |
| `20261029T111955.276679Z-6d637749ff63` | gate generator package source isolation and freshness | passed | 51.227791 | 31.81 |
| `20261029T112100.886154Z-0b308aac6b1f` | gate generated consumer bootstrap removal and locked compile | failed | 18.067927 | — |
| `20261029T112511.292136Z-402c30c18811` | gate locked consumer explicit format resolution | failed | 20.059595 | — |
| `20261029T112737.596408Z-b19ce0fa2372` | gate native consumer bounded metadata and compile | passed | 66.401289 | — |
| `20261029T112948.488921Z-50b80f4394c2` | gate near-cap native package variants release | passed | 3.327970 | — |
| `20261029T113055.885723Z-6e866802a6c5` | gate production coverage default native nextest | passed | 208.553825 | 148.00 |
| `20261029T113539.561416Z-c3acad473f6e` | gate near-cap completed browser cache release | passed | 1.707034 | — |
| `20261029T113611.270508Z-6444cd2bfb84` | gate production coverage formatted native nextest | failed | 1.705977 | — |
| `20261029T113654.926323Z-11316a3ba3cc` | gate formatted coverage compatible retained profiles | failed | 91.412138 | 73.00 |
| `20261029T114103.099740Z-02f695c31208` | gate guarded capacity recovery ordinary variants | passed | 2.837555 | — |
| `20261029T114137.016605Z-8a2cb6c0e20b` | gate formatted coverage after capacity recovery | passed | 45.011143 | 0.48 |
| `20261029T114309.807858Z-4353b074e197` | gate per crate generated production coverage accounting | passed | 11.428082 | — |
| `20261029T115132.293481Z-f26cdc5db109` | gate final receipt scope lock and retained target audit | passed | 5.281884 | — |

**31 completed records: 21 passed, 10 failed/interrupted; 2565.403982 s command time and 785.93 s observed Cargo-stage subtotal.** These are outer attempts, not crate/rustc counts or unique tests. The successful default workspace gate has 755 test passes and 942 doctest passes. Separate selected scopes have 105 edit-loop and 122 formatted/native passes; coverage has 129 default and 133 formatted passes. Browser scope is 88 cases.

## Corrections and retained evidence

- The first edit attempt was rejected by installed sccache's incremental environment check; the accepted profile settings resolved it. The next preliminary run overlapped implementation before generation and failed because the new generated API was not emitted yet. It is not claimed as a baseline pass. Official regeneration resolved the missing APIs.
- Two defensive cleanup attempts declined without deleting anything: target size had already dropped below their threshold through normal proof-package cleanup. Subsequent capacity releases ran only near the ceiling.
- Formatted/native tests, Clippy and Rustdoc passed before a bundled freshness command exposed the known warm-target source-path artifact issue. Removing only the entry executable was insufficient; matching the maintained proof driver's package/library cleanup resolved it. Both roots then regenerate twice/check identically, and the workspace tool check reports no differences.
- Locked independent consumer checks required explicit approved-lock resolution after changing formatter features. Host-filtered metadata avoids unrelated uncached embedded-target dependencies. The completed proof uses only approved external tuples and needs no download.
- The installed coverage tool rejects `--no-report` with `--no-clean`. The compatible final-report invocation preserves passing default profiles. One formatted build then reached the size guard after linking, before tests. Ordinary first-party cache recovery preserved all profiles and completed instrumented objects; retry passed 133 tests without rebuilding them (0.48 s observed Cargo stage).
- An initial recorder CLI spelling used unsupported kind `generation`; it was rejected before execution and created no record. It was rerun with kind `other`.
- The inherited generated-profile consumer's disposable lock generation/temporary target is explicitly accepted by the coordinator. The proposed follow-up is not applied.

Coverage excludes only test implementation and includes shipped generated source, with physical-source ownership and platform limitations detailed in the [receipt](00018-issue14-m2.md). Runtime/generator graphs, feature refusals, generator output hashes, compile-consumer proofs, browser JSON/screenshots and per-file coverage gaps are retained outside the build target. Full raw KB diagnostics preserve the inherited baseline.

The supervisor's observed peak is **8,389,216 KiB** (8 GiB + 608 KiB); the guard aborted that attempt. After capacity recovery the target remains warm, with all default/formatted profiles and common dependencies. No whole-target deletion at M2. Final audit/ledger annotations add no behavior or contract change.

Final audit passes exact 26-path scope, 15 local links, unchanged 616 external tuples, all 88 browser records, per-crate coverage floors and retained warm target; no owned native process remains. Installed tools: Rust 1.96.0, nextest 0.9.150, cargo-llvm-cov 0.8.6, sccache 0.19.0, Node 22.14.0, wasm-bindgen 0.2.129. KB remains 146 inherited errors / 27 existing warnings.
