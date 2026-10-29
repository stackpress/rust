# Issue 17 M1 benchmark ledger

Complete implementer-attempt ledger for `trial/3-issue17-leptos-library` at base `0c07794b7233ddd5df64ade802cff52e29f2a69a`. Read with the [M1 receipt](00017-issue17-m1.md). Persistent store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Each ID owns `runs/<ID>.json` and `.log`; no prior record was edited or deleted.

All Cargo, finite proof and cache-clean commands ran through the recorder and bounded supervisor. Existing Python proof drivers wrap nested Cargo only once; their subcommand argv/results remain in the retained evidence logs. Disposable M1 drivers are under `/private/tmp/stackpress-issue17-m1`; browser/build/coverage artifacts and reports are under its `evidence` directory.

## Counter changes and timing meaning

- Task attempts: **+42** (16 failed, 26 passed); kinds: build 2, check 13, clean 6, clippy 1, coverage 2, doc 1, other 3, test 14.
- Attempts with visible compilation: **+15**. Observed Cargo events: 1093 Compiling, 840 Checking, 51 Documenting. Hidden redirected output is not reconstructed as recorder events.
- Declared cold full-workspace rebuild counter: **+0**. The common wrapper recorded `scope=selected, cache=unknown`; actual workspace tests/doctests/lint and workspace Rustdoc are identified in their labels and argv/logs. This counter does not mean no workspace compilation occurred. Mixed proof stages and unknown cache declarations cannot establish a declared cold full-workspace rebuild.
- Accumulated finished command time: **3362.436640 s**. Observed Cargo-stage subtotal: **731.490000 s**, present in 12/42 finished attempts; all other Cargo-stage fields remain null. These are distinct measurements. Hidden gallery/Profile/format/admin build stages, nested compilation and tests make command time longer; Cargo stage sums are rounded and may overlap.
- Highest supervisor-observed aggregate target size: **6,406,104 KiB** (6.109 GiB), within the 8 GiB cap. Native, WASM, Profile-WASM and LLVM outputs all stay under the single `target/issue17` root. Recorded clean phases make room; the final clean removes that root. Persistent records and evidence survive.

## Every attempt

Times are seconds. `—` means unavailable, never zero. C/K/D are observed Cargo Compiling/Checking/Documenting event counts.

| Record ID | Result | Intent | Command | Cargo stages | C/K/D |
| --- | --- | --- | ---: | ---: | ---: |
| `20261028T232834.659732Z-7d4d640d1463` | passed | initial Cargo clean | 1.058691 | — | 0/0/0 |
| `20261028T232845.761504Z-266a5bb6ff3e` | passed | initial LLVM clean | 1.075528 | — | 0/0/0 |
| `20261028T233019.263205Z-3cd5e51fff5b` | failed | before extraction parity | 78.217773 | — | 185/0/0 |
| `20261028T233217.158556Z-a1999bc9a0ab` | passed | before extraction parity corrected fixture | 4.136799 | 2.710000 | 1/0/0 |
| `20261028T233253.742545Z-1c9c59b7932b` | failed | generic seam proof | 1.058776 | — | 0/0/0 |
| `20261028T233305.504561Z-07daa7a4e1de` | failed | generic seam proof corrected manifest | 16.475519 | — | 22/0/0 |
| `20261028T233335.139220Z-fb1a48f1f599` | passed | generic seam proof public replay error | 4.158013 | 2.600000 | 1/0/0 |
| `20261028T233745.393151Z-a6b7c7d53de3` | failed | extracted library and view check | 27.944650 | — | 0/130/0 |
| `20261028T233857.000096Z-c4e0820c2e99` | failed | view compile and immutable lock audit | 4.192126 | — | 0/2/0 |
| `20261028T233928.147832Z-aa9d13bf584b` | failed | view compile corrected source extraction | 5.201428 | — | 0/2/0 |
| `20261028T233953.381613Z-6c13c4bea70b` | passed | view optional integration compiled | 2.106073 | 1.390000 | 0/1/0 |
| `20261028T234036.463678Z-e13705c795a2` | failed | native renderer suites and parity | 46.922046 | — | 113/0/0 |
| `20261028T234216.407737Z-d2d7a3f2aa0c` | failed | native suites corrected corpus path | 22.968596 | — | 2/0/0 |
| `20261028T234341.414708Z-aadf9a3162a6` | passed | native suites adapter test boundary | 26.088768 | 13.460000 | 5/0/0 |
| `20261028T234556.434432Z-ca4b98cb986b` | failed | generators and workspace tests lint docs | 32.489272 | 30.370000 | 57/0/0 |
| `20261028T234707.180005Z-ca8fe9ca7824` | failed | official regeneration and workspace checks | 529.314681 | 209.330000 | 262/153/0 |
| `20261028T235111.499649Z-eed61858cfc3` | failed | migrated documentation links | 1.116394 | — | 0/0/0 |
| `20261028T235727.304274Z-3f6eeb9f0400` | passed | warning-clean native matrix and final parity | 139.980721 | 120.150000 | 14/164/3 |
| `20261029T000034.880918Z-839e7b3c1259` | passed | executed independent Markdown example | 22.217954 | 20.410000 | 12/0/0 |
| `20261029T000220.342725Z-8de13b66a5a1` | passed | release native verification cache | 5.443257 | — | 0/0/0 |
| `20261029T000235.015600Z-4b4c5f42352f` | passed | actual separate browser mode builds | 821.899135 | — | 0/0/0 |
| `20261029T000535.337199Z-2343bd0397ee` | passed | corrected migration documentation links | 1.093797 | — | 0/0/0 |
| `20261029T000708.236843Z-bdb4cc42c973` | passed | format final G41 mapping regression | 1.110681 | — | 0/0/0 |
| `20261029T002035.529762Z-578bcd43e53e` | failed | browser regressions gallery Profile preferences stream format | 1.196626 | — | 0/0/0 |
| `20261029T002119.122923Z-34a624fa4a3d` | passed | browser regressions gallery Profile preferences stream format installed browser | 336.591841 | — | 0/0/0 |
| `20261029T002310.553657Z-02020aff1761` | failed | M1 lock assets migration dependency boundaries | 1.126968 | — | 0/0/0 |
| `20261029T002331.358563Z-377ed754dcb6` | failed | M1 lock assets migration dependency boundaries corrected notice paths | 2.255701 | — | 0/0/0 |
| `20261029T002411.342869Z-03903a41a997` | passed | M1 lock assets and target runtime boundaries | 4.496296 | — | 0/0/0 |
| `20261029T002713.124926Z-8f039946a702` | passed | optional safety and composed review host lint docs | 56.747613 | 54.070000 | 18/137/1 |
| `20261029T002825.357361Z-7463cb73dcdf` | passed | clear sole cache before generated-admin matrix | 6.549745 | — | 0/0/0 |
| `20261029T002840.844288Z-17360f8bd1fd` | passed | generated-admin separate hydrate CSR role CRUD format plugin transfer builds | 455.707226 | — | 0/0/0 |
| `20261029T002852.158557Z-4ccddaafd78e` | failed | post extraction KB diagnostic audit | 1.076275 | — | 0/0/0 |
| `20261029T003449.572335Z-d6b748d7e0e1` | passed | final source migration and documentation scope audit | 1.077318 | — | 0/0/0 |
| `20261029T003641.655083Z-e9880cc23f91` | passed | generated-admin role CRUD formatted plugin transfer race browser regressions | 283.964574 | — | 0/0/0 |
| `20261029T003842.919402Z-e413bb7b43eb` | passed | workspace warning-denied Rustdoc | 66.846562 | 65.000000 | 15/251/47 |
| `20261029T004150.414162Z-c9c12af70ae1` | passed | clear sole cache before coverage instrumentation | 4.273992 | — | 0/0/0 |
| `20261029T004203.902305Z-fba27fc5c609` | failed | per changed crate native production coverage default and formatted | 135.155561 | 105.000000 | 247/0/0 |
| `20261029T004451.293358Z-360efbf7d96b` | passed | resume native coverage with supported multi-pass report flags | 199.544142 | 107.000000 | 139/0/0 |
| `20261029T004935.031627Z-d6ef1890d226` | passed | post coverage final fmt and source scope | 3.189383 | — | 0/0/0 |
| `20261029T005140.339443Z-1632d088a425` | passed | receipt and KB exact diagnostic comparison | 1.122597 | — | 0/0/0 |
| `20261029T005151.732276Z-942ecebff7e6` | passed | M1 final owned target removal | 4.199689 | — | 0/0/0 |
| `20261029T005230.207703Z-9cb147b5fec5` | passed | M1 cleanup lock and owned process audit | 1.043853 | — | 0/0/0 |

## Interpreting failed attempts

Prototype/signature/source-move failures were corrected before final native evidence. Generator ownership refusal was corrected by restoring task-only managed edits and regenerating with the writer. Clippy type complexity and relocated Markdown links were corrected. Browser dependency failure was an absent browser revision; an already installed matching Playwright/browser pair completed all suites. The coverage CLI conflict was corrected without losing the successful first-pass profiles. Two graph/asset audit attempts were corrected for actual notice filenames and exclusion of host proc-macro tooling from browser runtime edges.

KB validation remains a separately disclosed failure: 146 inherited errors plus 25 protected historical links to the removed package. It is not reported as clean code verification; the M1 receipt maps the relocations. Failed attempts remain in all totals. CLI argument parsing that rejected an invalid benchmark kind ran no Cargo/proof and created no attempt record.

Verification notes: coordinator re-verification is pending. This is implementer telemetry, not product/platform acceptance or a performance comparison with another branch.
