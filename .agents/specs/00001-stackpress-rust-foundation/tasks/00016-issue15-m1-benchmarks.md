# Issue 15 M1 benchmark attempts

Supporting evidence for the [M1 receipt](00016-issue15-m1.md).

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Each ID owns matching JSON/log files under `runs/`.
Counters: 50 attempts; 31 passed, 19 failed, zero running; 21 attempts emitted Compiling; zero declared successful cold workspace rebuilds.
Accumulated command time 1090.275798 s; observed Cargo Finished stages 499.17 s. Hidden build-script logs have no observed Cargo stages; unavailable is not zero. Times include nested commands and are operational evidence, not a controlled benchmark.

Cache labels are declarations: the coverage run used a fresh LLVM subtree beneath the retained task target; the standalone formatter ran after cleanup and used no artifact cache. No cache-speed comparison is claimed.

| ID | Attempt | Result | Command s | Cargo s |
| --- | --- | --- | ---: | ---: |
| `20261028T131449.009003Z-2b21a8f518ee` | initial Cargo clean | passed | 0.108799 | unavailable |
| `20261028T131456.172319Z-c03f4bcc36d9` | initial LLVM clean | passed | 0.514876 | unavailable |
| `20261028T131503.697090Z-4e3db145e0c0` | old-code golden capture | passed | 71.830368 | 70.00 |
| `20261028T131637.892022Z-a0a1a9b8186c` | old-code pinned golden verification | passed | 3.125601 | 1.83 |
| `20261028T132633.124069Z-0847bcf35e24` | minimal workspace lock metadata | passed | 1.071504 | unavailable |
| `20261028T132949.848511Z-5cffc88e77e7` | minimal offline lock resolution | passed | 1.078583 | unavailable |
| `20261028T133010.767906Z-dce3d7d549ca` | view migration compile | failed | 11.366033 | unavailable |
| `20261028T133149.755735Z-2dd72d1c24d7` | migrated presentation and example checks | failed | 31.147005 | unavailable |
| `20261028T133312.163031Z-09bb5248b3dc` | selected migrated tests | failed | 1.094521 | unavailable |
| `20261028T133332.285587Z-8665fbbc5103` | selected migrated tests minimal lock update | failed | 47.936779 | unavailable |
| `20261028T133510.764782Z-7db65cb3917c` | migrated golden compatibility | passed | 12.540823 | 11.17 |
| `20261028T133604.415024Z-d7d8dfa40afe` | migrated tests corrected assets | failed | 20.015896 | 3.14 |
| `20261028T133641.206786Z-d8d97337e1ac` | real preferences hydrate CSR assets | passed | 75.656585 | unavailable |
| `20261028T133832.395488Z-af7c3c5ab646` | real example first-paint matrix | failed | 1.087779 | unavailable |
| `20261028T133836.590940Z-680b20ea28f3` | preferences real browser flows | passed | 7.380652 | unavailable |
| `20261028T133937.774306Z-337585ca687a` | corrected real first-paint matrix | passed | 9.488055 | unavailable |
| `20261028T134007.139791Z-e35f1cb6c4d7` | format migration | passed | 2.089527 | unavailable |
| `20261028T134038.264519Z-e164d095ae22` | profile hydrate assets | passed | 9.487344 | 8.42 |
| `20261028T134151.037239Z-2d12e2766575` | profile CSR assets | passed | 8.471363 | 7.46 |
| `20261028T134237.102744Z-c10dca09a6bf` | native tests docs lint and capability graphs | failed | 33.867712 | unavailable |
| `20261028T134256.858104Z-e652f393ef5d` | native CSR saved and unset themes | passed | 4.297527 | unavailable |
| `20261028T134350.233850Z-f6778513126f` | corrected native matrix and generated checks | failed | 31.687397 | 11.01 |
| `20261028T134456.822633Z-be2b0916a17f` | gallery real asset regeneration | passed | 36.903321 | unavailable |
| `20261028T134623.567683Z-e2a44711a3ce` | native verification after metadata compatibility fix | failed | 104.041164 | 76.69 |
| `20261028T134637.433839Z-8b5ab3369efc` | gallery browser frozen theme regression | failed | 4.219798 | unavailable |
| `20261028T134801.381571Z-9f99537209b1` | profile real SSR hydration handoff | failed | 2.169353 | unavailable |
| `20261028T134941.900713Z-2eb1b2204b57` | build updated profile finite renderer | passed | 23.459726 | 22.50 |
| `20261028T135044.603564Z-6d804af22b65` | gallery checked theme handoff browser | passed | 46.652565 | unavailable |
| `20261028T135051.081898Z-dfd6da569945` | lock asset and capability invariants | passed | 3.232701 | unavailable |
| `20261028T135136.345633Z-527079d77d1d` | updated profile SSR hydration handoff | failed | 2.164160 | unavailable |
| `20261028T135237.595962Z-1cc0ecfcf24e` | format completed handoff fix | passed | 2.006712 | unavailable |
| `20261028T135324.329563Z-a26a96a14ac4` | final native workspace docs WASM and generated checks | failed | 66.643453 | 21.34 |
| `20261028T135458.811859Z-611deffba36f` | final workspace docs WASM with native dependency | failed | 3.277366 | unavailable |
| `20261028T135613.223410Z-c5c1faa70737` | minimal native lock edge and final checks | failed | 128.612651 | 86.74 |
| `20261028T135854.512775Z-71f1869f5faa` | final WASM lint facade and asset generators | passed | 98.420782 | 54.73 |
| `20261028T135909.719509Z-bd581c29bc12` | corrected Profile first-theme handoff | failed | 2.260296 | unavailable |
| `20261028T140136.269954Z-862e94bbdc5c` | final draft adoption native and WASM checks | passed | 47.535823 | 23.87 |
| `20261028T140231.066312Z-fe456c9ba2d1` | Profile first theme and retained early draft | passed | 3.361112 | unavailable |
| `20261028T140243.274936Z-a3bb2b7789e9` | native framework coverage | failed | 92.961969 | 84.00 |
| `20261028T140502.063465Z-220e47f0e8cb` | Profile frame focus draft matrix | passed | 2.344377 | unavailable |
| `20261028T140506.275107Z-f90fc994f986` | native command adapter regression | passed | 4.504441 | 3.45 |
| `20261028T140533.283496Z-144a93aea241` | selected coverage export | failed | 1.143405 | unavailable |
| `20261028T140549.043108Z-28397508baf0` | selected coverage report corrected flags | passed | 1.134667 | unavailable |
| `20261028T140609.619936Z-e093795c486f` | final pinned golden compatibility | passed | 12.329702 | 10.95 |
| `20261028T140614.102939Z-e689cb131119` | final native CSR settings browser | passed | 4.575776 | unavailable |
| `20261028T140832.887633Z-27db21ba86ce` | warning-clean golden consumer | passed | 3.499351 | 1.87 |
| `20261028T141049.979833Z-fdeae9e74777` | final scope syntax and whitespace audit | failed | 1.243662 | unavailable |
| `20261028T141113.267427Z-450c00dbddfa` | corrected removal-aware final audit | passed | 1.140177 | unavailable |
| `20261028T141229.308199Z-e19cf010935a` | final target and generated output cleanup | passed | 5.009683 | unavailable |
| `20261028T141542.334119Z-235a137318f4` | format standalone golden fixture | passed | 0.082876 | unavailable |
