# Issue 15 M2 benchmark attempts

Supporting evidence for the [M2 receipt](00016-issue15-m2.md).
Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Matching exact commands, JSON and logs are under `runs/<ID>.*`.
Counters: 24 attempts; 19 passed, 5 failed, zero running; 10 attempts emitted Compiling; zero cold workspace compilation attempts/full workspace rebuilds.
Accumulated command time **620.945771 s**; observed Cargo Finished stages **360.07 s**. Missing stages are unavailable, not zero. Preferences build and Profile driver Cargo logs are retained separately; the recorder subtotal cannot see their redirected stages. Native/LLVM artifact trees share one capped target. Cache labels are declarations; the LLVM subtree was initially fresh. No controlled performance comparison is claimed.
First attempt `2026-10-28T14:49:59.491616+00:00`; final recorded completion `2026-10-28T15:25:39.201300+00:00`. This span includes implementation/reading time; accumulated command time is a different measure.

| ID | Attempt | Result | Command s | Cargo s |
| --- | --- | --- | ---: | ---: |
| `20261028T144959.491616Z-5f24b50518f9` | initial Cargo clean | passed | 0.116020 | unavailable |
| `20261028T145026.456134Z-eebe724fc11c` | initial LLVM clean | passed | 0.565770 | unavailable |
| `20261028T145057.823065Z-4985e952c51b` | pre-M1 early-draft baseline | passed | 122.537302 | 118.02 |
| `20261028T145920.729302Z-f41f71efcdba` | host registration formatting | passed | 2.122029 | unavailable |
| `20261028T145933.134746Z-228f82bfe129` | registration compile | passed | 12.501307 | 11.47 |
| `20261028T150041.368960Z-f73aa7bcde24` | composed example compile | passed | 26.115732 | 25.47 |
| `20261028T150155.606353Z-838b571f82fa` | registration tests | failed | 17.906263 | 14.49 |
| `20261028T150304.816100Z-684e56ce77c3` | host tests corrected fixtures | failed | 3.188367 | 1.68 |
| `20261028T150349.870027Z-d8523e89e16b` | presentation tests complete | passed | 4.272384 | 1.82 |
| `20261028T150445.641956Z-e5a43c035a57` | final format | passed | 3.184390 | unavailable |
| `20261028T150502.059792Z-b6b03017c56b` | current example assets | failed | 2.152419 | unavailable |
| `20261028T150600.685378Z-2174a824b0ed` | supervisor transient unlink regression | passed | 3.211113 | unavailable |
| `20261028T150614.177685Z-fff67908989f` | regenerate current browser assets | passed | 52.220403 | 4.32 |
| `20261028T150747.239097Z-d8598e69b584` | standard preferences browser driver | passed | 17.823867 | unavailable |
| `20261028T150828.471479Z-9dcbdfc7716c` | native checks and generators | failed | 7.361741 | 4.16 |
| `20261028T150900.530335Z-1b2f435a03bf` | native checks independent flash fix | failed | 5.282818 | 2.00 |
| `20261028T150947.917865Z-b7e2471c441e` | final native checks with composed host | passed | 168.202352 | 143.29 |
| `20261028T151303.625078Z-02559458d050` | standard Profile browser driver | passed | 58.804163 | unavailable |
| `20261028T151503.114814Z-0f33dc148a06` | golden compatibility after registration | passed | 13.013109 | 11.47 |
| `20261028T151629.703113Z-2c04f6c99418` | standard SQLite Profile browser driver | passed | 61.334313 | unavailable |
| `20261028T151755.357991Z-6bc00b04e2a1` | native view coverage | passed | 22.972460 | 19.12 |
| `20261028T152006.222039Z-4a9eb6c04e41` | final prerequisite coverage and source proofs | passed | 11.964604 | 2.76 |
| `20261028T152121.193712Z-bbeffcd26c89` | delete target and owned generated outputs | passed | 3.705609 | unavailable |
| `20261028T152538.552714Z-d377f5ba0521` | final scope and cleanup audit | passed | 0.387236 | unavailable |

All failed attempts are retained: the first two test attempts used fixture assumptions inconsistent with existing config/mount/no-listener semantics; the asset attempt hit du's disappearing Cargo archive race; one feature check exposed the unused flash-only retained origin; the next caught a missing borrow in a test header. Each was corrected and superseded by passing final proofs. The historical early-draft child assertions intentionally fail; their enclosing baseline proof passed because the exact expected reset was observed at both mounts.

Evidence root: `/private/tmp/stackpress-issue15-m2-20261028`. `evidence/cleanup.json` records 4,118,616 KiB before deletion; `owned-processes.json` proves recorded supervised children absent. All artifacts/reports/screenshots needed for review remain; the target, throwaway source and generated compiled assets were deleted.
