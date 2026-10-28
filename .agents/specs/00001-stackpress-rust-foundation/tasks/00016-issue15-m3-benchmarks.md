# Issue 15 M3 benchmark ledger

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
Exact commands, environment, logs, stages and test-result rows: `/private/tmp/stackpress-issue15-m3-20261028/evidence/benchmark-final.json`.

| Record | Label | Kind / scope / cache | Status | Command seconds | Observed Cargo seconds |
| --- | --- | --- | --- | ---: | ---: |
| `20261028T155204.835671Z-f17254fd5282` | issue15 M3 initial Cargo clean | clean / selected / unknown | passed (0) | 0.135 | unavailable |
| `20261028T155222.767073Z-5d6c3c2ab794` | issue15 M3 initial LLVM clean | clean / selected / unknown | passed (0) | 0.520 | unavailable |
| `20261028T155248.347942Z-43a82f1b780c` | issue15 M3 round 1 complexity verification | test / selected / cold | passed (0) | 117.613 | 96.000 |
| `20261028T155528.140278Z-6bb507f03798` | issue15 M3 round 2 responsibility verification | test / selected / warm | passed (0) | 76.402 | 55.690 |
| `20261028T161211.170093Z-4ca67db19eaf` | issue15 M3 round 3 documentation and consistency verification | test / selected / warm | passed (0) | 140.467 | 112.460 |
| `20261028T161537.857492Z-63256f41f482` | issue15 M3 round 3 final documentation corrections | doc / selected / warm | passed (0) | 79.669 | 59.070 |
| `20261028T161724.376481Z-c74f51a8cbef` | issue15 M3 round 4 safety boundary verification | test / selected / warm | passed (0) | 72.634 | 49.270 |
| `20261028T161855.142219Z-a20925c8e780` | issue15 M3 real preferences and Profile client regeneration | build / selected / warm | passed (0) | 150.301 | 74.860 |
| `20261028T162137.183059Z-23dbffd7d72d` | issue15 M3 real port-free browser regression matrix | test / selected / warm | passed (0) | 161.131 | unavailable |
| `20261028T162431.132328Z-05dbbff4860a` | issue15 M3 cold selected LLVM coverage including presentation hosts | coverage / selected / cold | passed (0) | 94.011 | 84.000 |
| `20261028T162713.446252Z-32aafa233f64` | issue15 M3 final features generators and source checks | check / selected / warm | passed (0) | 108.049 | 87.830 |
| `20261028T163038.331437Z-ce47495c48e3` | issue15 M3 final asset checks and target cleanup | clean / selected / warm | passed (0) | 4.820 | unavailable |

Counters: `{"attempts": 12, "cache": {"cold": 2, "unknown": 2, "warm": 8}, "cold_workspace_rebuilds": 0, "command_seconds": 1005.751568, "compiling_attempts": 8, "kind": {"build": 1, "check": 1, "clean": 3, "coverage": 1, "doc": 1, "test": 5}, "observed_cargo_build_seconds": 619.18, "scope": {"selected": 12}, "status": {"passed": 12}}`.

Only outer proof drivers are recorded. Browser-driver Cargo and preferences build stages redirect to retained child logs; they are not inferred from command time. Cargo seconds are the sum of observed Finished stages, not CPU time or total end-to-end time. Cold coverage means its initially empty LLVM subtree inside the one capped target; it is a selected build, not a workspace rebuild.

The earlier coordinator-denied KB rewrite and round-3 command did not execute and have no benchmark record. Historical KB files were never changed. The golden aliases were restored before the recorded round-3 checks; all later attempts above ran against those historical inputs unchanged. Static validator/source inspection runs do not invoke Cargo. No failed/running record is omitted.
