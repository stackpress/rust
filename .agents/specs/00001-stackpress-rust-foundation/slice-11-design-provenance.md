# Slice 11 source provenance and checkpoint evidence

Status: Draft / Not Frozen. Worktree root /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9, confirmed clean af9cc01b9d9ab9ab73af5998ad8a8376bb6b20f7 before drafting. Current files and read-only upstream were used; no other trial branch, old target output or generated cjs/esm implementation was consulted.

## Source package boundary

Canonical read-only package: /Users/cblanquera/server/projects/stackpress/stackpress/packages/stackpress, package version 0.10.8. Enumerated the whole non-generated package including config/type barrels, client helper files, scripts/events, HTTP/WHATWG/backend/view/browser wrappers, UnoCSS, reset/import stylesheets, schema include, build configs, README/license and package exports/dependencies/scripts/peers. There is no tests directory in this snapshot; npm script globs are not test evidence.

The symbol ledger covers **77 named/default declaration rows, 2144 symbol occurrences including repeated aliases/types** from all src/**/*.ts files. It is an inventory of identifiers, not an archive/copy of implementation code. Completeness checks compare the exact generated rows against current source; family dispositions route every row to reuse, substantive bridge, explicit non-port or named follow-up. Root/subpath alias differences are preserved.

## Behavior anchors

| Source anchor | What was inspected |
| --- | --- |
| stackpress/src/plugin.ts:14–25 | Nine synchronous constituent calls inside async exported function, exact order; source adds no guard. |
| stackpress/src/client/types.ts:84–101 | Config intersection of optional source sections with arbitrary app data; no aggregate runtime plugin-toggle map. |
| stackpress/src/events.ts:1–18; scripts.ts:1–17 | Twelve event names, eleven scripts, query difference, delegated owner packages. |
| stackpress/src/index.ts; types.ts | Complete root type/value fan-out and aliases; root API type omissions vs subpaths; events/scripts namespace export. |
| stackpress-server/src/plugin.ts:10–17 | Listen registers develop/emit/serve. |
| stackpress-schema/src/plugin.ts:16–38 | Client module default, nullable import, Config priority 10; generate Listen contribution; separate idea transform hook. |
| stackpress-language/src/plugin.ts:12–68; Language.ts | Missing config inactive; locale key/default/map; data then prefix update, reconstruct request and raw nested route resolve; labels/translations/cookie state. |
| stackpress-sql/src/plugin.ts:27–56 | Operator event order, nullable client/model traversal, typeof listen callback, separate idea transform. |
| stackpress-view/src/plugin.ts | Config-presence gate, production/development provider, build command, dev routes and separate idea transform. |
| stackpress-session/src/plugin.ts | Existing auth/email/identity source composite order, already ported in Slice 9. |
| stackpress-admin/src/plugin.ts:15–31 | Route-phase nullable client traversal and per-model admin functions; no source unconditional universal admin route. |
| stackpress/stackpress.idea:1 | API built-in schema include. |
| stackpress/package.json | Thirty advertised export keys, npm metadata/peers/package scripts; not the same set as all src files. |

Existing Rust inventory is anchored to current manifests/lib/plugin/codegen files linked in [constituents](slice-11-design-constituents.md), existing #14 guides, generated admin pages_host.rs and the accepted Slice 9/10 design/task receipts. Source support libraries and runtime registrars are explicitly distinguished. Proposed bridges are not mislabeled existing functionality.

## Advertised subpaths

Require/import targets below are source packaging metadata. They do not imply a port of Node/React/CJS/ESM or a newly implemented Rust helper. src/client/index and events/scripts files are inventoried even when no matching standalone export key is advertised.

| Package export key | Target(s) |
| --- | --- |
| ./language/types | require: ./cjs/language/types.js, import: ./esm/language/types.js |
| ./language | require: ./cjs/language/index.js, import: ./esm/language/index.js |
| ./lib/types | require: ./cjs/lib/types.js, import: ./esm/lib/types.js |
| ./lib | require: ./cjs/lib/index.js, import: ./esm/lib/index.js |
| ./schema/types | require: ./cjs/schema/types.js, import: ./esm/schema/types.js |
| ./schema | require: ./cjs/schema/index.js, import: ./esm/schema/index.js |
| ./http | require: ./cjs/server/http.js, import: ./esm/server/http.js |
| ./whatwg | require: ./cjs/server/whatwg.js, import: ./esm/server/whatwg.js |
| ./server/types | require: ./cjs/server/types.js, import: ./esm/server/types.js |
| ./server | require: ./cjs/server/index.js, import: ./esm/server/index.js |
| ./session/types | require: ./cjs/session/types.js, import: ./esm/session/types.js |
| ./session | require: ./cjs/session/index.js, import: ./esm/session/index.js |
| ./mysql | require: ./cjs/sql/mysql.js, import: ./esm/sql/mysql.js |
| ./pglite | require: ./cjs/sql/pglite.js, import: ./esm/sql/pglite.js |
| ./pgsql | require: ./cjs/sql/pgsql.js, import: ./esm/sql/pgsql.js |
| ./sqlite | require: ./cjs/sql/sqlite.js, import: ./esm/sql/sqlite.js |
| ./sql/types | require: ./cjs/sql/types.js, import: ./esm/sql/types.js |
| ./sql | require: ./cjs/sql/index.js, import: ./esm/sql/index.js |
| ./view/client | require: ./cjs/view/client.js, import: ./esm/view/client.js |
| ./view/types | require: ./cjs/view/types.js, import: ./esm/view/types.js |
| ./view | require: ./cjs/view/index.js, import: ./esm/view/index.js |
| ./plugin | require: ./cjs/plugin.js, import: ./esm/plugin.js |
| ./types | require: ./cjs/types.js, import: ./esm/types.js |
| ./unocss | require: ./cjs/unocss.js, import: ./esm/unocss.js |
| . | require: ./cjs/index.js, import: ./esm/index.js |
| ./tsconfig/cjs | ./tsconfig/cjs.json |
| ./tsconfig/esm | ./tsconfig/esm.json |
| ./assets.d.ts | ./tsconfig/assets.d.ts |
| ./reset.css | ./reset.css |
| ./stackpress.css | ./stackpress.css |

## Whole-package SHA-256 ledger

These 43 identities pin the actual inspected package source/config/assets without copying their code into .agents/resources. Other constituent plugin paths above are read-only behavioral references, not additional files attributed to the aggregate package.

| Relative package file | SHA-256 |
| --- | --- |
| LICENSE | 033ff066a8d5efa2a11686a2b2175af303cda460f7563a48b7d1d4041f43076c |
| README.md | d1a3b52a0b95f30b0c2a7957d26491cb8c64d99e54f206b1877d03422d5c7612 |
| package.json | 9d30f15005dd2ab22a5f9240b58e394ce07980d8078e0282849365d050cfcb4e |
| reset.css | 8c22bfd736984092684c12d482ca49838f08a0069687fa9801b1ba2a972b598c |
| src/api/index.ts | a2bc11a53bdc9322c74171a5d0cfb306e2a60f61eab27f496726a7df14c0f5f2 |
| src/api/types.ts | 22024a39836774199c3adb3083844f842d441790d9a478219cd18bb6d5974988 |
| src/client/index.ts | d5fd737009ef84d9dd17ebac27e1ce79d00e789d1ee9000c18edca1e38a7e7fb |
| src/client/types.ts | d312787f347cb705f37345c3f6f26328f4da30ac1edddd2ab8e4f9c009082d8b |
| src/events.ts | 3ef571f8b2ba274da7f657d6107d6912ec236b0e4d4326c49bb01e6c23d1599a |
| src/index.ts | f985c625db86622fe19498aea722691db72cf6556ccf26aefc904f9043a95621 |
| src/language/index.ts | fd6357d4bb00f3b9580af3dc38f586e63e6a5045756c42a933381bdfaf45aaef |
| src/language/types.ts | acc89658779f526daf9adc5d1f8e05ed580e0ccd309a8221dbb42f65937c56d8 |
| src/lib/index.ts | a498829dcf483732484a2553cb5ddedec05bb0f5e17f9e863dc0291c48de506f |
| src/lib/types.ts | 0962aa481c0f6b9431bd0d032e96d9d7214c5e67514b25f3356da41f93551e65 |
| src/plugin.ts | 0c3da5b8a8a6ac91c737fb9fad7d81da03d3e2d46ffb9dd57a9ebcc7736d3ee8 |
| src/schema/index.ts | cf4d7cfd55c5a4f47f8963b80dfb0465a8fa41be86c8a6297bbec144d833f538 |
| src/schema/types.ts | fb0223505460d154c9960186c421f0494139986a591bcc89db67382e1ae296f1 |
| src/scripts.ts | b8b1dc8a3954c9d8c3115f9896af8cb104f8d56881f6a928fb1a81f3284f37eb |
| src/server/http.ts | cb837bbac28c509ea856dd4bb3fba42933798dd8597b418322f3c2e5c96debef |
| src/server/index.ts | 4ec36a974a2792b534a0ee0dcac12ed1f963886edb13a3b303d9d91752cf755b |
| src/server/types.ts | 9aa1af3cb0aece121e3ee6c27cd7ba35c8d479e1a2aff2c51659ddc4b595afdc |
| src/server/whatwg.ts | 0aa8fab0f5c61f2b5b4d53a1cd7a5eec6793308f5b4b93a91034e63a788516da |
| src/session/index.ts | 35d85c43f878db895c98564c16bc2c29ec743936e88d7e7ccd32401aa0216133 |
| src/session/types.ts | 10034fb4213f284ee1b9cfd55df581722c9d5ee71ca901692e2e95f98b3f707a |
| src/sql/index.ts | a44c0a8cc4f4531fe06ec496cd00ec04a1c5467f0a4670a4d31f195d111cd9f5 |
| src/sql/mysql.ts | c726e222843e7b2b33f6b1f6914d98732ff0b8e0544864dc52b6da637b9749e0 |
| src/sql/pglite.ts | 07769347e3c5eecba6f984c5c67cb198077829bc76bc7710139ce07f45067e60 |
| src/sql/pgsql.ts | cc41d5d77f2c2784e2317cf493dbe93b9095604701a5443a95c036c0a0eacbd1 |
| src/sql/sqlite.ts | 54280f42d1b886ba17da185195638d40eaa0ea8e5608afb23e3f49f1b33ae6ab |
| src/sql/types.ts | c7603b985990be60ca2ad3df830b97ff91b3c1644bc9e9e2d72cbcc3720b2368 |
| src/types.ts | 231dc814f30d9b30b85bf68d6c5631cea8d80ff78ad767b828f04261c2b696e1 |
| src/unocss.ts | b365256184c17a7f82d91b08555914a36772fd73be0967272458dba9fec7efef |
| src/view/client.ts | 287ddc0d2b852e934901f72415f5da62950b5dab2ff0ae8b16b145cbdb0270ae |
| src/view/index.ts | 25db8c0708ff78a8e419645b1663780c77d70e054ad3654f199aacfe52b7fe3c |
| src/view/types.ts | 9b73b008213cdaf54ef797bee87b4546a77a495407cb14d4cd427f6a2bb38084 |
| stackpress.css | a36b781bf45f9123b85a6032e55c2ac680491843e776776a852b762cb1d6fb86 |
| stackpress.idea | 3195ad4ed5d4e74bb08b9c7903a93f66765ebe395cb6d3587614cf19857ecb26 |
| tsconfig/assets.d.ts | 315250a851f7b989911c9cc418b9d5dd754ccc12203b088bbaf5de0c88ac24a4 |
| tsconfig/cjs.json | a8ec331825b58e12227bd105919e744f75349ad02360b119679abfce5229c3e8 |
| tsconfig/esm.json | 7cbee4b76a7597ccaaa4d459236305343c6b70c0b1f21d632f1cdc89a4e9b8ca |
| tsconfig.cjs.json | 9654bef8efd9f4e8458d2f05ea32f12f992e0a7629afff00791bba455e4c236f |
| tsconfig.esm.json | e03d5aacac619a22a59df062953346cdc7ef4ffa994401197194a048c3b974aa |
| tsconfig.json | ce9fb1747b4f575516033445521cccebe9397e6e2fe62ac2ed39601191db21ec |

## Checkpoint validation

Baseline proof record **20261030T161707.414654Z-890f3bd8353e** stores the tracked-file hash freeze, all 43 source hashes and raw/canonical KB diagnostics under /private/tmp/slice11-design. Raw worktree baseline: **146 missing-sibling-path errors, 28 warnings**. Canonical-layout baseline: **0 errors, 28 warnings**, resolving 116 unique sibling targets (some linked repeatedly).

Canonical-layout checking runs the current validator/current worktree Agent Files, remapping only unresolved links outside this worktree under its sibling parent to the equivalent existing path beneath /Users/cblanquera/server/projects/stackpress. It does not modify the validator, source/Agent File contents or filesystem, and does not read a different trial branch's Rust implementation. Internal links stay pointed at this worktree. Both raw and normalized results are retained; raw diagnostics are not described as clean.

Final checkpoint checks: all new links/heading anchors/fences/line caps and export rows; all 43 upstream hashes unchanged; all preexisting tracked files unchanged except one exact appended task-status row; allowed new files only slice-11-design-*.md and tasks/00023-slice-11-aggregate.md; no production/manifest/lock/facade change. Raw KB diagnostic set must remain byte-for-byte equivalent, canonical layout must remain **0 errors / 28 warnings**. The final recorder log is identified in task Implementation notes after completion; this is implementer evidence, not coordinator verification.

No Cargo build, check, test, cleanup, tree or probe compilation, listener, install, lock resolution, commit/push, GitHub write, other-agent message or source asset redistribution ran. Proposed API examples are intentionally uncompiled checkpoint sketches. Optional later risky probes go outside the repository and require source-driven evidence; new crates.io/network needs explicit approval. The shared benchmark store is retained unchanged apart from new recorder records. No source archive, historical Agent File repair or context promotion was made.
