# Slice 9 source provenance and coverage map

Status: Revised design only under G43, captured 2026-10-30 Asia/Manila. [The checkpoint](slice-9-design-decisions.md) routes the complete design. Current explicit instructions and G39–G43 govern; imported TypeScript reference prose is source-system evidence only.

## Checked source roots

| Source | Revision / boundary |
| --- | --- |
| Current Rust worktree | `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9`, HEAD `d54faed9f3d1054e6a6e34c18fd2fd76f78ca4eb`, branch `trial/3-slice9-session-csrf-email`; initially clean. No other trial branch or stale target output consulted except the explicit G43 authority path below. |
| Upstream Stackpress | `/Users/cblanquera/server/projects/stackpress/stackpress`, HEAD `a71d683051ba8350fdd12d6b5a33f268fdcc285f`; selected package/aggregate/helper paths were clean at capture. Read-only. All three package manifests report 0.10.8. |
| Authorized G43 authority | `/Users/cblanquera/server/projects/stackpress/rust/.agents/specs/00001-stackpress-rust-foundation/decisions.md:82`, trial/3 at `3cf407f`; SHA256 `99ed6b0d24f12e9fb7d769ebe796da70aa864606a9d3e8d700d17c21dd4fae04`. Read-only, explicitly directed by coordinator. |
| Source path notation | `S/` = upstream `packages/stackpress-session/src/`; CSRF/email paths are explicitly named in inventories. Physical upstream root was resolved independently because `../stackpress` from a Paseo worktree is not the canonical sibling checkout. |
| Forbidden/unneeded roots | `../idea` was not touched; `../frui` was not needed. No source checkout was changed or copied into repository resources. |

Required readings: root AGENTS; context index, source-structure, source-catalog, project-direction, rust-code-style, testing-and-quality; coordinated-crate-development §5.1 and build/benchmark workflows; task 00020; G39–G42 and authorized G43; issue #14 SQL/presentation guides. Agent File creation/routing rules and chrisai-docing API-reference guidance govern this draft. The old source-structure catalog still lists removed UI/preferences-era roles; the later G39/G40/G41 decisions and current source take precedence.

KB anchors: [Session and language source contracts (00028)](../../references/00028-upstream-session-language-contracts.md), [00041 framework supplements](../../references/00041-framework-package-supplements.md), [00023 configuration](../../references/00023-upstream-configuration-catalog.md), [00032 interface exposure](../../references/00032-upstream-interface-exposure-examples.md). 00023 calls access an allowlist; actual Session.authorize empty-map short-circuit permits all. G43 resolves the contract in favour of the actual source; missing access installs no global guard. All package flows/pages and SQL adapter are in scope now.

## Coverage and evidence boundaries

The source inventory includes every root/subpath export and exported auth/profile/session type, relevant deep exported helper, every config field/read, all registered event/route/view entries, generated support methods/column defaults and security-relevant behaviours. Page/UI-specific exports have their own routed table. Non-code package build/manifest/CSS conventions are dispositioned rather than mistaken for Rust requirements. Source test scripts reference tests, but none exist in the three maintained package trees at capture; no upstream test/build was run.

The overlap table maps current lib cookie Session, view access/policy/preferences/flash, admin public CSRF fields/native fixture guard and both examples to keep/reuse/move. No production move is needed. G43 settles source behaviour. The revised API/compiled-adapter/lane sketches await mechanics review; the former opaque-reference, hardening and deferred-flow recommendations are superseded. Only JWT dependency choice and the exact optional-phone inconsistency remain owner questions. Disposable tests prove only the library operations actually executed. No actual Slice 9 protocol, account implementation, browser graph, provider delivery or coverage result is claimed.

All complete-draft topics were assembled outside the repository before partitioning, then split by cohesive topic. A temporary coverage inventory records topic hashes/required identifiers and dispositions of superseded recommendations; the previous design is preserved outside the repository for lossless comparison. The partition check compares bytes, routes every topic from the entry, verifies inventory/decision/risk/acceptance identifiers and checks final line caps. No historical Agent File is edited merely to link a new draft.

## Maintained upstream file hash inventory

The ledger covers 86 session, 9 CSRF and 10 email maintained files, plus aggregate plugin and the schema hashing/encryption helper (107 files total). Generated ESM/CJS/node_modules trees are excluded. Hashes below identify source; they are not copied code. Ledger digest is SHA256 over sorted `relative-path + NUL + hex-sha256 + newline` records: `3975d985bbe8e6c697f860374958edf483aef88cad64e273078eccbc35921ac5`.

| Path under upstream root | SHA-256 |
| --- | --- |
| `packages/stackpress/src/plugin.ts` | `0c3da5b8a8a6ac91c737fb9fad7d81da03d3e2d46ffb9dd57a9ebcc7736d3ee8` |
| `packages/stackpress-csrf/LICENSE` | `033ff066a8d5efa2a11686a2b2175af303cda460f7563a48b7d1d4041f43076c` |
| `packages/stackpress-csrf/README.md` | `000f898485a5afab6e2815a0cf3119b5ee055a9f4b2cbc89f63ba421a8a4a9c7` |
| `packages/stackpress-csrf/package.json` | `eb5786bc66b45bdc154cb4f0309c6b70551ffb8d2509980c6d742ee99d9bea2d` |
| `packages/stackpress-csrf/src/index.ts` | `e507e2675ce953f028ffa8b41691834a878df781ce12d6824f3a6875f63bb979` |
| `packages/stackpress-csrf/src/plugin.ts` | `55f589f6d9d9c63d37e751ca1e550b932c4b893de3f65f263a2590286e5792e2` |
| `packages/stackpress-csrf/src/types.ts` | `b91fe8d5743f30e67d7db132665706c657ddca5e1d69bf7b5bdce8ee93a45a8c` |
| `packages/stackpress-csrf/tsconfig.cjs.json` | `9654bef8efd9f4e8458d2f05ea32f12f992e0a7629afff00791bba455e4c236f` |
| `packages/stackpress-csrf/tsconfig.esm.json` | `e03d5aacac619a22a59df062953346cdc7ef4ffa994401197194a048c3b974aa` |
| `packages/stackpress-csrf/tsconfig.json` | `ce9fb1747b4f575516033445521cccebe9397e6e2fe62ac2ed39601191db21ec` |
| `packages/stackpress-email/LICENSE` | `033ff066a8d5efa2a11686a2b2175af303cda460f7563a48b7d1d4041f43076c` |
| `packages/stackpress-email/README.md` | `30ba7ca6cc4b14c1aec0370a1385240e68f327e675320ebe129b9c5fb2b695c0` |
| `packages/stackpress-email/package.json` | `9188ef014d7b8269e1b2d3da1eb93620f7766c8b5c89d958c1637c46b3dd14a3` |
| `packages/stackpress-email/src/events.ts` | `4061c0714fc3fff61e65cd05d916332629bcede7f39df0fd55aac8cf7dddc65d` |
| `packages/stackpress-email/src/index.ts` | `c2c2bc0bc4af7070ca8b0e04bae3038f6f8b83633d3722ac63d8da1af1df1462` |
| `packages/stackpress-email/src/plugin.ts` | `4ce2b3cc8a1ad5348309415ab715d278cb1d4882c1b9cf956cb5ace527410759` |
| `packages/stackpress-email/src/types.ts` | `bf715b884db12f3dc3512df9aac2166dc003f3f28d78250e2e31eb2623991a66` |
| `packages/stackpress-email/tsconfig.cjs.json` | `9654bef8efd9f4e8458d2f05ea32f12f992e0a7629afff00791bba455e4c236f` |
| `packages/stackpress-email/tsconfig.esm.json` | `e03d5aacac619a22a59df062953346cdc7ef4ffa994401197194a048c3b974aa` |
| `packages/stackpress-email/tsconfig.json` | `ce9fb1747b4f575516033445521cccebe9397e6e2fe62ac2ed39601191db21ec` |
| `packages/stackpress-schema/src/helpers.ts` | `8d68bb1842b92a76517481382fa549c952be0fb12c5866d7593ad980df6e83c3` |
| `packages/stackpress-session/LICENSE` | `033ff066a8d5efa2a11686a2b2175af303cda460f7563a48b7d1d4041f43076c` |
| `packages/stackpress-session/README.md` | `227e8eb3ceb9a86dfae96b0b50a5d201a3c8a82b34075f9f9be2a00ec1df902c` |
| `packages/stackpress-session/package.json` | `540e86d646a21c67fc745b58cfb468df767d524612053fd890ab3d858b1660b7` |
| `packages/stackpress-session/schema.idea` | `395c0f46e6543f258ea97a74a543d9306676fcafef3bf71d0dd02deafeaac84a` |
| `packages/stackpress-session/src/Exception.ts` | `0c6379adeee3f013c9464e97156a294c39c3a23165f3d5d95e95c734c01ed5ad` |
| `packages/stackpress-session/src/auth/AuthActions.ts` | `33e9c030c8dde3f0f6c08efee55427351cab0bdb97fb7694d49bb7ad76273bab` |
| `packages/stackpress-session/src/auth/AuthSchema.ts` | `467196daeaabac13df63aa84603681c15f6794e1344b1665859a065a51b84b6d` |
| `packages/stackpress-session/src/auth/AuthStore.ts` | `5b58563b995d7b0524c9e55a07a61538153f8754c021f83b4764ae4a2e846bf1` |
| `packages/stackpress-session/src/auth/columns/ActiveColumn.ts` | `8b3c63572c9f118bc765e7238c0171d22082a456eff02a99a9f949edb7ceeadc` |
| `packages/stackpress-session/src/auth/columns/ConsumedColumn.ts` | `3308d01f77f5bc288d6bab081fd37c334821bb96673a71b995e3ce5944c63a66` |
| `packages/stackpress-session/src/auth/columns/CreatedColumn.ts` | `7edb3f64623c26ff67065f08c57e303c2fa36cec35f934853b881565b4231779` |
| `packages/stackpress-session/src/auth/columns/IdColumn.ts` | `6d7dc77eace4f8596a7aca6c6ca33d3a0fbd2b06e6c2b8f6626843b6f5e16d1c` |
| `packages/stackpress-session/src/auth/columns/ProfileIdColumn.ts` | `dc18c23a8ab5ddc0c8f3a32ba633d1ff619b85ceeecd5dcc1a4d9689dc7d3bf4` |
| `packages/stackpress-session/src/auth/columns/SecretColumn.ts` | `8aa86d03190acbb1759eebfb4160443bc53cc52f469532bb449b657a1d44f538` |
| `packages/stackpress-session/src/auth/columns/TokenColumn.ts` | `c8b1c8a2dba959eb42073998cf4f313f53c07b3f8225a0846fd77ede289d5195` |
| `packages/stackpress-session/src/auth/columns/TypeColumn.ts` | `7029d8e5da555a2c5119f43f281cde8b1616928c353d0601e5c6303c50d907e1` |
| `packages/stackpress-session/src/auth/columns/UpdatedColumn.ts` | `3159b7819d185b98aa1cffc2cb62244968745da631353bbf650a6ea1a9b87149` |
| `packages/stackpress-session/src/auth/columns/VerifiedColumn.ts` | `ba10bf46fc7d18dae3d56a777a5f78949a80c6c480507e96261172f5e887edf8` |
| `packages/stackpress-session/src/auth/events/signin.ts` | `b2c92c01ea41bfca4f9a90b21acf3dd57f5609cac85d19790c103c35e4a3ccea` |
| `packages/stackpress-session/src/auth/events/signout.ts` | `8b00667f3ccf8caac7d0c71ae2be964c3c22a59a43f15b5b88c243d1b92ec792` |
| `packages/stackpress-session/src/auth/events/signup.ts` | `db7be4e9174a25f0ac090c9f20ceffb43726ce50e2030ad007393d931682557f` |
| `packages/stackpress-session/src/auth/helpers.ts` | `ee5aa4389b3478741ed35bd3c5b584163176dfab03419897d805e09520f42751` |
| `packages/stackpress-session/src/auth/pages/signin/2fa.ts` | `efac34dac0e9d67165a11269946a83b2d1b40d37ee9e0758901b77f98de664a3` |
| `packages/stackpress-session/src/auth/pages/signin/email.ts` | `147c76378a43fddf8ccdcc05352909bbd7a68f4fcb039a16a066f401d222efbc` |
| `packages/stackpress-session/src/auth/pages/signin/link.ts` | `18cf40e4b1d68b795375760e0e7cf6ea06000d28de8cbf576ff3553711d58f9e` |
| `packages/stackpress-session/src/auth/pages/signin/otp.ts` | `13173fcf0974158bfeffddf34b2e95b717b625e80e3c050f9fc3092cecda057e` |
| `packages/stackpress-session/src/auth/pages/signin/phone.ts` | `6f4e7560ae5f5e0a21cd8f5b400cffbee769d525b3fb3cd10c31957dd28fa0df` |
| `packages/stackpress-session/src/auth/pages/signin/username.ts` | `403b07cbe1f8d4fdac21c387a6ce161ca929645bb0074b6c218523381c0392a6` |
| `packages/stackpress-session/src/auth/pages/signin.ts` | `6fa2690ed305de6c82f0a8d38b5a9b3493bf13d56f9b654b32e0d851a0039668` |
| `packages/stackpress-session/src/auth/pages/signout.ts` | `e342d92666110124e91b7acb021e05ada97510f496643d86bebe2bc8b8632846` |
| `packages/stackpress-session/src/auth/pages/signup.ts` | `ca05a3c6541f1a53059bb8e771142b18e6f16682c58df22bc45597eb72117c0a` |
| `packages/stackpress-session/src/auth/plugin.ts` | `d3923a0bc22e7a8776b6f9e33469af14983ebbd49bfb8799b3f54851872cc18a` |
| `packages/stackpress-session/src/auth/templates.ts` | `ed85cb2306886f00616aadd488d6b46f8b0fd41418a76b1b2478c02987dbc871` |
| `packages/stackpress-session/src/auth/types.ts` | `8d96720af2f26d6d468f54890144086a3657e7fd99caeac48700cbe6d86ba30f` |
| `packages/stackpress-session/src/auth/views/signin/2fa.tsx` | `ff264413dc8afc39af217ad277834f8c64e6669076b4aaf3cea35875540a0413` |
| `packages/stackpress-session/src/auth/views/signin/email.tsx` | `8db9b2d548fc30b5de07ef73847e5e9b2b26036a4017a0709ca1aeb73ed88225` |
| `packages/stackpress-session/src/auth/views/signin/index.tsx` | `e8ecf086ced8b1dbbad5b70a011f780906fac0a8df710511b8cbcb8e79afdb34` |
| `packages/stackpress-session/src/auth/views/signin/link.tsx` | `d3d7b1fc1cac7e06f99dfb6fa960ef5dafa65ff021295ba720fc4c1123decbfa` |
| `packages/stackpress-session/src/auth/views/signin/otp.tsx` | `53019f2a934a8d1cbc7e7dfffc101776d9233df2ae64fd07e8383f2f26e63ec5` |
| `packages/stackpress-session/src/auth/views/signin/phone.tsx` | `e40c2b52fa5db6e191e89289e6645651088670b76ec5503a0ca84ff83ace6f27` |
| `packages/stackpress-session/src/auth/views/signin/username.tsx` | `1baf3d0fc05a05266d4beaf27b98cd70cf715056f10d75e28d2c9533f743db9a` |
| `packages/stackpress-session/src/auth/views/signup.tsx` | `97609bebc55994addc87d6125504cb816386f64a903a4fb476a673874adde87f` |
| `packages/stackpress-session/src/index.ts` | `98eea9f115028cf01ff5b6fdacb1faa2e847378081980c159526e2f643bb6546` |
| `packages/stackpress-session/src/plugin.ts` | `aade8d295a5910f7ccc4f9ac56a7c7a6ac7a8da44bd8b544c8259859bbc4a84f` |
| `packages/stackpress-session/src/profile/ProfileActions.ts` | `e93dae3447842e526aab10229c6b9e562483a083badbd9e328e254e56f29e5e5` |
| `packages/stackpress-session/src/profile/ProfileSchema.ts` | `0307a179a8471b59941367f588a3febd8e09eee5fe2fd71aadcc194afc27ea3d` |
| `packages/stackpress-session/src/profile/ProfileStore.ts` | `6546282d5795d58915d83678f2f002512db8ec29ac76ee6c5ec4e9617c47350a` |
| `packages/stackpress-session/src/profile/columns/ActiveColumn.ts` | `8b3c63572c9f118bc765e7238c0171d22082a456eff02a99a9f949edb7ceeadc` |
| `packages/stackpress-session/src/profile/columns/CreatedColumn.ts` | `7edb3f64623c26ff67065f08c57e303c2fa36cec35f934853b881565b4231779` |
| `packages/stackpress-session/src/profile/columns/IdColumn.ts` | `6d7dc77eace4f8596a7aca6c6ca33d3a0fbd2b06e6c2b8f6626843b6f5e16d1c` |
| `packages/stackpress-session/src/profile/columns/ImageColumn.ts` | `5dbbcb71f3928991de506ab1779d2ef7778b8a16397a41cbc664e4fe84e40040` |
| `packages/stackpress-session/src/profile/columns/NameColumn.ts` | `9b19b0820b4e47c55446af14da01ccacd98f9bc203afebb91ff9d937e090c439` |
| `packages/stackpress-session/src/profile/columns/ReferencesColumn.ts` | `8e123fd7ca2b6fbe5be858880bc4b32a0307a7a3250744b2a9683270507ab1fb` |
| `packages/stackpress-session/src/profile/columns/RolesColumn.ts` | `6f08d5278afdbad27ef211ef933cdcdbe92442ea70d6a2d943fce6169c10ec87` |
| `packages/stackpress-session/src/profile/columns/TagsColumn.ts` | `4fe9e58918e4c96135721ba594bbbf594207b62c37c27b2e4f4c171273d0a96e` |
| `packages/stackpress-session/src/profile/columns/TypeColumn.ts` | `3558d6d1344f59b56a051d53eecf18fa6cea46786ea56c7fbbb52725986eb977` |
| `packages/stackpress-session/src/profile/columns/UpdatedColumn.ts` | `3159b7819d185b98aa1cffc2cb62244968745da631353bbf650a6ea1a9b87149` |
| `packages/stackpress-session/src/profile/types.ts` | `68670a7d27b9f7217518c92f40c707dda9ee61318e0d1ac558f90c451baa03d8` |
| `packages/stackpress-session/src/session/Session.ts` | `b6d71977591e879be76a69178321326566278f104b49ec2d550dd60d4b94d5f4` |
| `packages/stackpress-session/src/session/events/authorize.ts` | `602544d99fcb635766064dd3f20230b46633274b36d576f5f6aecc9a8b23052e` |
| `packages/stackpress-session/src/session/events/session.ts` | `5c9605d097f68c295dae8094d9a76064bcdf365db2ba47e6c666471f5ca1c5cb` |
| `packages/stackpress-session/src/session/helpers.ts` | `1de1e1387471b4fb6d1bf744cfe2002366a0209211ac63acbbad7364a44dd233` |
| `packages/stackpress-session/src/session/pages/2fa/detail.ts` | `2a500a576c626004432f35ba4656d1cfd2b7190aafd4257239ec1a531f5da416` |
| `packages/stackpress-session/src/session/pages/2fa/remove.ts` | `2bc77cfb960789a92b2c107402a548d9a36d988ea60bf075d68a700cafaba315` |
| `packages/stackpress-session/src/session/pages/account.ts` | `b773d002de5b7805a5fd2f1e46f1ca6964301f71465abddb2f59523f4166b644` |
| `packages/stackpress-session/src/session/pages/authorize.ts` | `98349d96668ae2d84d724a95d3c20a889f8f0a1ebd232ce94233dd147a151856` |
| `packages/stackpress-session/src/session/pages/export.ts` | `9d8274cbe2de6c9bc23e7ccb479a743be3ca11168d287979f15a3f4e1d99d04a` |
| `packages/stackpress-session/src/session/pages/password.ts` | `7af71c23b7c8494d22ac51890137bef6e594f244cda62e5f613f7b8df617339b` |
| `packages/stackpress-session/src/session/pages/remove.ts` | `9fc2252feb9bfbdf69dd60f2225b44e41c647f5ea7544b05818dc87c38318a3f` |
| `packages/stackpress-session/src/session/pages/update.ts` | `792349a9897add17cb809b9d02e4cad261f686739d43e851033a76137b56b572` |
| `packages/stackpress-session/src/session/plugin.ts` | `3ef88db19285c961ad5a5af1d74ef75c3bb6a98341e758402790f17a01730e78` |
| `packages/stackpress-session/src/session/types.ts` | `1fef1591358951d5d634cc1ad615e8ba8de1a5c79cf1d479135130506d26dea2` |
| `packages/stackpress-session/src/session/views/2fa/detail.tsx` | `fbd8f9b8fed250bfd30707ddbe708bc9d9d7eba604e0085c1adaf59d16ef8f1d` |
| `packages/stackpress-session/src/session/views/2fa/remove.tsx` | `4712c6c959280907eee79179820a0f0846d0756e17bc468d4f1432a1efbd188a` |
| `packages/stackpress-session/src/session/views/account.tsx` | `3cce92a71b31b8705d6094d8bf789ab3da8b4ad37e91de10447ce21829234e5d` |
| `packages/stackpress-session/src/session/views/export.tsx` | `e6417f3f0580ec506317ef3176f908a730ec95c034be892da4c02c1b9d69aa2b` |
| `packages/stackpress-session/src/session/views/helpers.tsx` | `4241ff8e3da45bac483335db17a848a775eef2feb412e09917ab18fd5745d67e` |
| `packages/stackpress-session/src/session/views/password.tsx` | `f984a3f269fd49ba035f2945bd63b36c9a20e6176cf1bccf80d83ed3569e5384` |
| `packages/stackpress-session/src/session/views/remove.tsx` | `ecaf3252b26a0f13cfbc185733daf7c48c818ceb4c3c1687bd5dd5c490211655` |
| `packages/stackpress-session/src/session/views/security.tsx` | `65abf6e31507045d6b0b9fd750b5cbbf749bc772c6bd211c428876daa9ca28b9` |
| `packages/stackpress-session/src/session/views/update.tsx` | `523a809cc60a0a00bdcc4311f0772d204b8d036c8ada154381ad1ea8a34d0ed1` |
| `packages/stackpress-session/src/types.ts` | `5495ac0e4a646c3008c8f535a5a32596f78970783d45d1304fa79632b9720fb1` |
| `packages/stackpress-session/styles.css` | `4b59ed916296b70ba50ca9e59e274986a1fb0dff2f3eb19f3333cc41d11a0d8a` |
| `packages/stackpress-session/tsconfig.cjs.json` | `9654bef8efd9f4e8458d2f05ea32f12f992e0a7629afff00791bba455e4c236f` |
| `packages/stackpress-session/tsconfig.esm.json` | `e03d5aacac619a22a59df062953346cdc7ef4ffa994401197194a048c3b974aa` |
| `packages/stackpress-session/tsconfig.json` | `ce9fb1747b4f575516033445521cccebe9397e6e2fe62ac2ed39601191db21ec` |


## API acquisition evidence added for G43

The API source was consulted for its actual caller acquisition, not to move task 00021 into Slice 9. OAuth resolves me; session/app endpoints resolve their own SQL records/scopes, and only API-session records set profileId. The API sketch records these distinct paths and the proposed stackpress-lib native request carrier. Hashes cover maintained src context; acquisition reasoning cites helpers/plugin/oauth/token lines explicitly.

| Path under upstream root | SHA-256 |
| --- | --- |
| `packages/stackpress-api/src/Exception.ts` | `c40e8f34c75227b01e1effd7ed5e914a7ea0511ed8c206cbe048fa5d663d81b9` |
| `packages/stackpress-api/src/helpers.ts` | `4748a377184f34038aec20353e5d6331575cdfb395013bedf6613466eda81a7b` |
| `packages/stackpress-api/src/index.ts` | `234ccd65796fe1cef621c6d634c1ba70d4b189259b71df777a72261fbfefbeca` |
| `packages/stackpress-api/src/pages/oauth.ts` | `52bc29b52c36825d6d7be7a1f68a7c62de1d82b7087f0bfcc9f2cf31b6d5ce6a` |
| `packages/stackpress-api/src/pages/token.ts` | `4a9b185f4a64adfc5046e3eddfd576bc302db621d6a23d5b0c1253662a97a7a8` |
| `packages/stackpress-api/src/plugin.ts` | `a63a25af8cd1288420fc9bb7451230573acaa60b99e9103f6720a3c5d353e600` |
| `packages/stackpress-api/src/types.ts` | `124747ac4b0e6abc4297bea08b2ec5ee8904820f3cf1da646d5879d017ee7ba9` |
| `packages/stackpress-api/src/views/oauth.tsx` | `e2750f64ff717179909c4dca2020f61448eede64bc66a9f3ad8be164cd98b4cd` |

## Current Rust overlap anchors

These source hashes identify inspected responsibility boundaries, not build artefacts.

| Path under current Rust root | SHA-256 |
| --- | --- |
| `crates/lib/src/session.rs` | `8f6e2c588b9dbf1eb8254f186d0efa5f87d867dead166ed36f0c541574e851a9` |
| `crates/lib/src/pattern.rs` | `26ae9efc923d974874014fe8c49813d9078b84806089c8a022c7129a9eb76eb8` |
| `crates/framework/framework-view/src/access.rs` | `2035db5630837243975428ed69a916ac784d92ef9019410b2b463fe063b595bb` |
| `crates/framework/framework-view/src/policy.rs` | `b0736881ff97e091cd47bebab57df88423f729980cdf425616a02530d0402c63` |
| `crates/framework/framework-view/src/preferences/host.rs` | `63383dc3a0abacb26ff8d067f15d206cf767efac13c6c1a353bcf5790e069fa8` |
| `crates/framework/framework-view/src/preferences/mod.rs` | `c4a621964c764896757cf5715f491450d457894772e635687e5242fe9ec176c3` |
| `crates/framework/framework-view/src/flash.rs` | `29e0e9e498be9df6362d6bfdb3b460dd0f0c0f2549e168a8d8ef6ac28b21df1e` |
| `crates/framework/framework-view/src/flash/host.rs` | `400c88a8df87b274afabba3716eea57d66a1a5caac2b6a49b0b469200d3d98a1` |
| `crates/framework/framework-admin/src/pages.rs` | `0d5250cc703714f53a1d7a031c2e801d9e6a67a2b944078a166d54c1d4500c2d` |
| `examples/generated-admin/web/src/host.rs` | `badd386ccc749ec8fda49dbb01ac8ea58c122d638a5dbb57d6dffbcffd5c43ba` |
| `examples/generated-admin/web/src/host/transfer.rs` | `01bf8c7773464f82f0d3e76d1de2373fbfdef5bdb615f1fd299ef35f717bb449` |
| `examples/review-preview/src/gateway.rs` | `51589a728bc2e36d568be58aa74c25a7c99b01b7046e5ad35d35d0bff869f5b0` |
| `examples/review-preview/src/setup.rs` | `60defcc9bbc10d01f23fcc8fb93288512010506f4298a4d3726f1e0a552275ec` |
| `examples/ui-preferences/web/src/lib.rs` | `cc80a69a7d031f17c91d7b0b4d22c6d629c78051c242636156c16c3fec802596` |
| `examples/ui-preferences/web/src/policy.rs` | `b5262973a34d1515050df1f1a3c7c028de8a48d11615651446aac993b76b8e6e` |
| `crates/server/server/src/context.rs` | `1b70cafdfbc0d0d688ec8ff45db54fc2013da4d39e56c99bb2265d631a8e1335` |

## Validation scope

Baseline `validate-agent-workspace.py` output was saved outside the repository before drafting: **146 errors / 27 warnings**, dominated by pre-existing source links resolving against the worktree's different parent layout. These are not corrected in this task. Closeout reports any new diagnostic separately; KB validation is never Rust code verification.

Disposable probe evidence and exact command IDs/hashes are in [the assessment](slice-9-design-assessment.md). No Cargo command targeted a repository manifest. No listener, upstream write, commit/push, install or GitHub write occurred. Coordinator communication is reserved for the final handoff, as directed.
