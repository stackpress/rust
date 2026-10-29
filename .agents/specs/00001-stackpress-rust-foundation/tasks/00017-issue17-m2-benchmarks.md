# Issue 17 M2 benchmark ledger

Complete implementer-attempt ledger for `trial/3-issue17-leptos-library`, based on `68f38b7731f13b23f894bd4bd82d617b1a8bbfb7`. Read with the [M2 receipt](00017-issue17-m2.md). Persistent store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Each ID owns `runs/<ID>.json` and `.log`, including argv, environment, host/toolchain, exit status and output. No prior record was edited or deleted.

All Cargo, proof and clean commands used the recorder once around the outer finite supervisor/proof driver. Maintained proof entry: `examples/leptos-independent/proof.py`; disposable drivers: `/private/tmp/stackpress-issue17-m2`; reports/copied artifacts: its `evidence` directory. Browser/build/doc/graph subcommand argv remain in the driver or retained logs. Captured output in graph/import cases is retained separately under `evidence/graphs` rather than reconstructed as recorder compilation events.

## Counters and timing

- Attempts: **+21** (failed 5, passed 16). Kinds: build 1, check 8, clean 3, coverage 1, test 8. Scopes: consumer 13, selected 5, workspace 3. Cache declarations: cold 3, unknown 2, warm 16.
- Attempts emitting visible compilation: **+9**. Observed events: 887 Compiling, 297 Checking, 2 Documenting.
- Declared cold full-workspace rebuild counter: **+0**. Builds/tests select the isolated consumer or named packages, not the entire root workspace. Coverage is cold for its previously empty nested LLVM target; other native/WASM/doc output is retained under the same aggregate target root. Initial clean commands are not builds.
- Accumulated finished command time: **911.835373 s**. Observed Cargo-stage subtotal: **514.730000 s**, available in 8/21 attempts. Null remains unavailable. Wall time includes test/proof/browser/bindgen work; observed rounded Cargo stages are neither compiler CPU time nor interchangeable with wall time. Concurrent attempts overlap, so summed durations are not elapsed calendar time.
- Highest supervisor-observed aggregate task target: **4,054,436 KiB (3.867 GiB)**, below 8 GiB. All compilation/LLVM output is under `target/issue17`; final cleanup removes it. Reports, screenshots and copied finite review artifacts remain outside that target.

## Every attempt

Times are seconds. `—` means unavailable. C/K/D are observed Compiling/Checking/Documenting counts. The metadata record supplies full commands; labels describe the actual gate. Internal expected invalid-mode/import failures are assertions inside a passing graph audit and are separate from failed outer attempts.

| Record ID | Result | Kind / scope / cache | Intent | Command | Cargo stages | C/K/D |
| --- | --- | --- | --- | ---: | ---: | ---: |
| `20261029T013951.452261Z-125ac5fbc41e` | passed | clean / workspace / unknown | fresh task target cleanup | 1.036643 | — | 0/0/0 |
| `20261029T014003.087403Z-e1da97112b02` | passed | clean / workspace / unknown | fresh LLVM cleanup | 1.053564 | — | 0/0/0 |
| `20261029T014243.897333Z-c894acd18264` | passed | check / consumer / cold | offline independent lock resolution | 1.066089 | — | 0/0/0 |
| `20261029T014315.771541Z-aa21b05e51f2` | failed | check / consumer / cold | independent default and SSR API compile | 75.438566 | — | 112/134/0 |
| `20261029T014517.570159Z-fc633ea2727b` | passed | test / consumer / warm | independent native lifecycle and helpers | 49.904819 | 46.820000 | 156/0/0 |
| `20261029T014846.463842Z-d89c1a482325` | passed | test / consumer / warm | independent native modes lint doctests docs fmt | 78.414626 | 71.440000 | 58/12/1 |
| `20261029T015150.405116Z-1572c453e27d` | passed | build / consumer / warm | independent separate base helper hydrate CSR artifacts | 146.833733 | 139.920000 | 252/0/0 |
| `20261029T015635.528852Z-c1d3d5a71f7a` | failed | test / consumer / warm | independent browser identity helpers interactions disposal | 11.678267 | — | 0/0/0 |
| `20261029T015909.599893Z-cb8cab715e76` | failed | test / consumer / warm | independent browser boot diagnostics | 11.810147 | — | 0/0/0 |
| `20261029T020055.217764Z-de2e181c0f93` | failed | test / consumer / warm | independent hydration panic location | 14.946834 | 2.560000 | 1/0/0 |
| `20261029T020304.299708Z-b90642a8bc0f` | failed | test / consumer / warm | independent metadata marker and matching browser artifacts | 25.459266 | 13.500000 | 6/0/0 |
| `20261029T020554.309630Z-37ffa322d5f7` | passed | test / consumer / warm | independent complete browser matrix with composition settlement | 9.567148 | — | 0/0/0 |
| `20261029T020744.926677Z-2574a299e209` | passed | check / selected / warm | selected graphs invalid modes isolated import absence | 194.731562 | — | 0/0/0 |
| `20261029T021216.165086Z-bd20c6ed6eb6` | passed | test / consumer / warm | final independent input-state native browser and WASM lint gates | 94.739191 | 70.390000 | 9/112/1 |
| `20261029T021524.611842Z-799c78d15691` | passed | check / consumer / warm | literal library and view Markdown examples in isolated consumers | 49.730308 | 45.970000 | 1/39/0 |
| `20261029T021709.098154Z-f2ab8ff507ed` | passed | coverage / consumer / cold | independent native production coverage two feature scopes | 130.865486 | 124.130000 | 292/0/0 |
| `20261029T021742.855148Z-5cdde78d52b2` | passed | check / selected / warm | documentation structure links anchors lock and whitespace | 1.147290 | — | 0/0/0 |
| `20261029T022153.598127Z-133c8e86e67b` | passed | check / selected / warm | final scope fmt locks migration and inherited KB audit | 3.406440 | — | 0/0/0 |
| `20261029T022713.681750Z-11163981c456` | passed | check / selected / warm | M2 receipt scope fmt locks and exact KB delta | 3.503659 | — | 0/0/0 |
| `20261029T022753.383173Z-a443a7650f44` | passed | clean / workspace / warm | M2 final owned target removal | 5.455431 | — | 0/0/0 |
| `20261029T022810.276589Z-64fa29d38a96` | passed | check / selected / warm | M2 cleanup lock retained evidence and owned process audit | 1.046304 | — | 0/0/0 |

## Failed attempts and interpretation

The first fixture compile failed because a generated component property needed Rustdoc under the fixture’s missing-docs denial; it was documented before successful tests. Initial browser attempts failed during hydration. A fixture-only panic hook exposed the missing upstream metadata HEAD marker; adding the consumer-owned marker and rebuilding matching artifacts fixed startup. The next attempt passed all base cases, then exposed that the synthetic IME test asserted before its documented next-task settlement. Correct event sequencing and waiting for that task fixed the helper case. All 16 cases passed afterward, then passed again after adding an action that proves early-draft signal adoption and the visible Safety samples label. No generic library behavior was changed to make these proofs pass.

The final raw KB validator exits 1 with the inherited worktree path diagnostics disclosed in the receipt. The enclosing audit passes only by comparing the exact diagnostic set against the recorded baseline and rejecting new or own-file errors; it does not report raw KB validation as passing. Failed outer attempts remain in the table and totals.

Final records cover receipt/scope/lock/fmt validation, exact owned-target removal, retained evidence and absence of matching task proof/native processes. No target is retained for M3. Coordinator independent verification and owner/platform acceptance remain pending.
