# Slice 9 Lane A M3 implementation receipt

Status: M3 implemented, pending independent coordinator verification. Stop at M3. Base/HEAD is 9fcc456755d546eb3b68362ddd26e60e49956ee1, branch trial/3-slice9-session-csrf-email. G42/G43/G44 remain binding. This receipt does not release M4, integration, publication or owner visual acceptance.

## Delivered surface and ownership

Only framework-session production/tests/docs/vendor assets, its root Cargo.lock edges and this new receipt changed. No other crate, root facade, shared carrier, schema provider or historical Agent File was edited. The [account reference](../../../../crates/framework/framework-session/docs/account.md) documents configuration, plugin/direct use and source behavior; [auth](../../../../crates/framework/framework-session/docs/auth.md) remains the M2 event contract.

Stable public paths under stackpress_framework_session:

- `account::{account, load_account_profile, update, update_auth, password, assert_password, remove, export, csv, enroll, remove_factor, provisioning_uri, qr_data_url}` under auth-server.
- `account::events::{register, auth_search, auth_update, auth_create, auth_remove, profile_detail, profile_update, profile_remove}`. Existing `auth::events::{search, update}` paths delegate to the same generated-style adapters; no second SQL policy.
- `views::{Page, PageProps, Config, Menu, PasswordRules, PasswordCheck, PasswordEvaluation, AccountPageProps, AuthSignupFormProps, body, head, page}`. Named signup/signin/username/email/phone/otp/link/two_factor/profile/update/security/password/export/remove/enroll/remove_factor modules each expose body/head/page.
- Direct forms signup_form/username_form/phone_form/email_form; helpers use_account_base/account_csrf_fields/account_back_header/account_head/account_layout/password_strength/password_checklist/evaluate_password. Rust arguments/owned AnyView children replace React helper-prop types; custom head/layout work without plugins. `auth::totp::generate_secret(length)` uses the original alphabet and literal length.
- `presentation::{Presentation, load, render}` under leptos-ssr. Existing `profile::ProfileSchema`, Column/Value/RecordSchema names and method surface remain available.

The ordinary auth and independent identity registrars now contribute every original view and account process when their source config gates are active. There are 29 view contributions at -100: 16 auth, including the last view-only POST signin/:type fallback, and 13 account. Eight GET/five POST account processes accompany the M2 auth processes. Several handlers can cooperate on one route/event; priority then registration order and any-handler Abort remain intact. Missing session/auth config disables only the original constituent; missing session.access adds no request guard.

Generic SQL events remain explicitly selected, with explicit Engine/table installation. No registrar installs a database, listener, global operation policy or conflict arbiter. Lane B is consumed only through its frozen email-send/registrar contracts. The finite host uses its real JSON transport; no framework-email/framework-csrf edit occurred. Live-provider delivery is not inferred.

## Source parity and mechanics

Read-only original source HEAD is a71d683051ba8350fdd12d6b5a33f268fdcc285f. Anchors below are relative to packages/stackpress-session/src unless otherwise stated. Reproduced original oracles cover 77 account invocation traces, 72 password-strength vectors, the joined removal interaction, plus unchanged M2 57 auth traces/two templates/four TOTP vectors and M1 380 column/17 email/scalar/returned-row vectors.

| Source behavior / anchor | Rust mapping and proof |
| --- | --- |
| session/pages/account.ts; loadAccountProfile | JWT/guest check, profile-detail then auth-search; linked methods expose only id/type/token/verified; source missing-profile 401. Real SQLite and original invocation traces. |
| session/pages/update.ts | Existing identifier edits need no current password or csrf.valid call. Adding a missing method requires the original password-only match/errors. Profile then username/email/phone writes, blank-token skips, first returned failure and partial effects retained. No verification reset/JWT refresh. Missing phone retains the source update trim error; G44 changes signup only. |
| session/pages/password.ts | Original current/new rule precedence, any matching auth hash, sequential updates to all rows. Returned per-row errors are ignored before source flash/redirect. No new CSRF, transaction or revocation. |
| session/pages/export.ts | Profile-only warning/download, filename and text/csv header; insertion-order rows, object/list/Date flattening and formula-prefix handling. Original CSV compared byte-for-byte; no sorted comparison. SQLite pins reverse-lexical nested JSON insertion order. |
| session/pages/remove.ts:28,62–76; stackpress-sql/src/transform/events/remove.ts:79–81 | Parent profile body remains; fresh auth-remove responses perform removals, parent-bound profile-remove passes through. Original joined proof and real SQLite agree: credentials removed, profile still active, success flash/signout redirect. Preserved pending owner answer below. |
| session/pages/2fa/detail.ts | Request/stored/generated secret, presence-based regenerate, source eager fallback generation, issuer/label encoding, code/CSRF gates and update/create effects. Source url/secret/profile/authId remain deliberately public. Source post-write flash/redirect retains returned errors. |
| session/pages/2fa/remove.ts | Truthy confirmed works on GET or POST, without csrf.valid; search status rather than result length is checked, ID-only generated removal ignores extra profileId/type predicates. Exact errors/flash/redirect/CSRF generation remain. |
| auth/plugin.ts:86–167; session/plugin.ts | Every GET/POST view at -100; last generic POST signin/:type fallback, concrete-view precedence, augmentation, equal-priority FIFO, Abort and missing-config modes have actual registrar/route tests. |
| auth/views/signup.tsx; every auth/session view and session/views/helpers.tsx | Source field requirements/input spread/error binding, menu/footer redirects, pass/otp/magic controls, weighted UTF-16 checklist (72 vectors), heads, navigation, custom AccountHead/Layout, CSRF fields and every form. Source UI/server special-character classes intentionally differ. Enrollment retains both readonly and hidden secret form values. |

Mechanics necessities: compiled exhaustive Page/functions replace dynamic imports and React hooks; owned public DTOs, root-local Leptos state and explicit native resource/timezone/SSR services replace implicit JS globals. Lucide replaces FontAwesome and native Leptos controls replace Frui components. PH default, source country/dial order and full-value phone submission use approved pinned metadata; existing Leptos PhonePolicy requires complete valid E.164 and would add a source-incompatible policy, so its underlying metadata is reused directly. No existing country-picker/clipboard component supplied this interaction.

QR uses approved qrcode 0.14.1 and png 0.18.1, M correction, four-module margin, scale four, PNG base64 data URL. Encoder mask/compression internals and grayscale PNG bytes differ from the Node library; source URI/data/dimensions are pinned, and independent macOS Vision decoded the actual rendered PNG to the expected otpauth URI. This is a library/rendering mechanics difference, not a security change. No PNG byte-parity claim is made.

`columns::Record` and Value::Object now use stackpress-lib::DataMap instead of BTreeMap, with unchanged public names/methods. This deliberate approved public underlying-type change preserves source outer/nested exported JSON/CSV insertion order. Callers spelling BTreeMap directly must use Record. The current worktree predates API M1: its API has no sql/codegen features, as the recorded attempted check confirms. Per coordinator ruling, combined-tree API tests, Clippy and `cargo check -p stackpress-framework-api --features sql,codegen` are coordinator-owned at merge; API was not edited or represented as verified against that later surface.

Accepted developer responsibilities remain documented parity, not prevented risks: local unchecked CSRF paths/confirmed GET, optional global guard, original redirects/flags/lifetimes, partial writes, stateless JWTs and source identifier/removal behavior. Signup binds submitted passwords as the source does; enrollment secrets/QR and deliberate request/results data are intentional public view values. Private signing/database seeds, SMTP credentials and native handles are not implicitly serialized. Account method projection excludes stored hashes. No new hardening/default-deny/global gate was introduced.

## Verification and UI evidence

| Scope | Result |
| --- | --- |
| Full workspace gate | fmt, Clippy -D warnings, **914/914 nextest**, 86 skipped including explicitly excluded coordinator-owned tcp_watchdog; **961 doctests**, warning-clean workspace Rustdoc. Record 69f7e06a2d41; command 664.111864s. Features session/leptos-ssr + schema/providers + email/server, no renderer all-features. |
| Final follow-up | After the full gate, the two CSS adjustments, source-translated accessible labels and sql-only private re-export cfg received final fmt, owned Clippy (sql and all-targets leptos-ssr), **49/49 owned tests** (one externally entered child ignored), six doctests, finite host and separate browser builds. Record 2b10f3db5a58. Current production was then covered and browser-tested below; no broad-gate rerun was needed for these local changes. |
| Browser | **176 hydrate + 176 CSR cases**, plus no-JS Light for each mode; final records 46d57db59952 / 094758301bb7. Desktop text-sized Regenerate focus area and one-line Personal Information are asserted, not screenshot-only claims. No runtime/hydration errors or horizontal overflow. |
| Features | Native default/native/sql/server/auth-server/leptos/leptos-ssr and WASM default/auth-server/hydrate/CSR checks. Normal browser graphs exclude native ring/JWT/AES/CBC/SHA3, SQL/server, CSRF/email/lettre and PNG/QR. Native identity/sql/server/auth-server graphs exclude UI/phone/browser additions. The isolated sql warning found during the first matrix was fixed and final sql Clippy -D warnings passes. |
| Coverage | cargo-llvm-cov nextest, aarch64-apple-darwin, session/leptos-ssr (includes auth-server/sql/server): **5179/5662 = 91.47%** executable lines. Tests/examples excluded; no production exclusion. Browser-only cfg branches are separately proved in Chromium, not claimed as native coverage. Development floor met; publication not authorized. |
| Source/lock/license/scope | Five source oracle fixtures reproduce; 16 pinned Lucide masters/Feather attribution checked. **698 existing external package blocks byte-identical**; only approved qrcode 0.14.1 tuple added. png/phonenumber/web-sys/wasm-bindgen-futures reused existing tuples, with no external change caused by those four approvals. 125-file source manifest SHA256 `a322abd985ca7aaa4852e724ce432be37101e4cec7a8971fc7226999088ff77f`. |
| KB | Existing **146 errors / 27 warnings**, zero new diagnostics. These inherited diagnostics are not a clean KB claim or code verification. |

The SQL-backed finite account_host explicitly installs Auth/Profile, seeds all three sign-in identifiers plus a 2FA row, binds/shuts down Hooks, uses the real email-send JSON transport and closes the Engine. It emits 16 pages × Light/Dark/unset (48 documents) plus 32 long-content documents, then exits. Its OTP/link/2FA GET fixtures deliberately use invalid challenges to render the original error forms; M2 registered-route SQLite tests separately prove valid OTP/magic/2FA handoffs and session creation. This is a finite proof host, not a raw demo or preview listener.

| Design check | Evidence |
| --- | --- |
| S9-UI01 | All eight auth pages/heads and direct forms; original traces/errors/redirects; real browser pass/otp/magic keyboard selection, disabled inactive password submission, password reveal, strength, PH phone full-value submission. |
| S9-UI02 | All eight account pages/heads, actual SQL profile/identifier/password/export/enrollment/removal effects, original trace status/error/body/child-call sequences and byte-identical CSV. |
| S9-UI03 | Chromium 1280px/375px, 200% zoom, long Japanese/ASCII/error/name/identifier content. Label/input association, named buttons, decorative SVGs, native keyboard focus/actions and no horizontal overflow. Visually reviewed compact inline controls and stacked narrow/zoomed profile rows. |
| S9-UI04 | Separate SSR→hydrate and CSR, 176 cases each, G39 explicit Light/Dark/unset-dark browser and separate no-JS Light, inert escaped snapshots, self-only scripts plus wasm-unsafe-eval CSP, no runtime/hydration errors. |
| S9-UI05 | Original secret/url/profile/authId and request/stored/regenerate traces, real PNG decode; readonly+hidden secret FormData values; user-triggered copy bridge with an in-memory clipboard substitute. No system clipboard was changed. Private native sentinels absent from HTML/WASM and native packages absent from browser normal graphs. |
| S9-UI06 | Actual 29-view registration order, generic fallback, concrete precedence, source config deactivation, same-route augmentation/FIFO and higher-priority Abort. Existing direct events/guard/source unchecked branches remain in the full owned suite. |

Coordinator visual points: source session/views/2fa/detail.tsx:113 uses Verify `info xl2`, while lines 119–125 use Remove 2FA `error lg`; their different size is retained under the explicit source exception, with colors unchanged. Rust-only stretched Regenerate link is now text-sized; desktop sidebar widened to 16rem so Personal Information stays on one line at 1280px. The 375px/200% stacked profile and dark enrollment screenshots were reviewed positively by the coordinator; this is scoped feedback, not final owner acceptance.

Pinned Lucide commit 03f11e2d6adc319ea21e9a1d5820a99cd3767257 matches the existing workspace family: 16px inline controls/24px headers, currentColor, exact unmodified masters, LICENSE and copied FEATHER-LICENSE. Frui country data includes its MIT license/provenance; upstream/leptos vendor copies remain untouched. Browser artifacts/screenshots/results, finite HTML, LCOV, consumer, Swift QR scanner, source manifest and reproduction scripts are under /private/tmp/slice9a/m3. Browser/platform proof does not constitute owner visual or installed Tauri/iOS/Android/live SMTP acceptance.

The five original .stderr contracts are unchanged and pass, plus Environment/Session compile-fail doctests. Handler ownership remains Send; no lock or UI Owner is held across nested native awaits. The external watchdog has a 15s absolute real-work deadline including startup, 2s entry acknowledgment, explicit kill/reap, and a deliberately blocked 200ms negative control. Suspended hook/parent cancellation/shutdown/nested work use barriers/channels. Test-harness ignored child entry is reached only through that parent.

Three test-verified review rounds completed: (1) logic/complexity preserved source Date/object CSV formatting, exact insertion order and source partial/unchecked branches; real source/SQLite tests and visual overflow fixes passed; (2) responsibility/ownership unified M2 SQL search/update with generated-style adapters, kept native projection separate from portable DTO/UI, and initialized the old M2 route fixture's SSR executor only under SSR; all owned tests passed; (3) consistency/Rustdoc/comments added compiled direct-form docs, custom head/layout helpers, complete generic POST view inventory, source duplicate enrollment inputs, readable optional-password matching and local lint fixes. Final owned tests/doctests, workspace lint/docs and both browser modes prove the finished surface.

## Resources and benchmark records

One retained warm effort target target/slice9a, sccache (20G cache), jobs=2, debug=0; edit loops unset incremental, gates/coverage set CARGO_INCREMENTAL=0. Native/WASM/disposable consumer outputs use separate subdirectories; no concurrent Cargo builds share a directory. Rustc/Cargo 1.96.0, macOS arm64. The coordinator raised the cap **12 → 16 GiB**, with **15.4 GiB headroom guard**, in /private/tmp/slice9a/run.py. Sampled M3 peak **15,761,018,129 bytes (14.679 GiB)** is filesystem artifact size, not memory; it stays below the final guard. The earlier 12 GiB guard-aborted attempt is retained as d38dd29a0175.

Recorded cleanup ledger under /private/tmp/slice9a/m3: incremental-only trim 793,571,700 bytes; later incremental 396,320,922 plus owned old consumer-m1 137,400,973 / consumer-m2 400,173,390; then incremental 520,113,754 / disposable trybuild 545,969,437 and old workspace instrumentation cleanup. The subsequent approved owned-session cleanup removed incremental 387,088,137 plus 661 package artifacts (Cargo reported 1.0 GiB); final pre-coverage owned-session cleanup removed 119 files / 407.5 MiB with zero incremental bytes. Main dependency artifacts and browser/trybuild caches were kept by those last two package cleanups; the earlier disposable-cache removals are disclosed rather than called dependency-preserving. No full effort target reset occurred. Every Cargo/proof/clean was recorder-wrapped once at the outer script; old instrumented first-party cleanup used recorder-wrapped cargo llvm-cov clean --workspace.

First browser consumer subtarget was empty but initially mislabeled warm: treat that attempt as **cold subtarget with warm sccache**. Later separate hydrate/CSR builds were warm. Target, consumer artifacts, resource observations and exact helper commands are retained for coordinator review.

Recorder store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Attributed M3 counter delta **+65 completed attempts (43 passed, 22 failed/interrupted)**, excluding other agents' concurrent records. Failed source/edit/format/guard attempts and the intentionally stopped pre-fallback gate remain visible. **35 commands emitted compilation**; observed Cargo activity {'compiling': 826, 'checking': 544, 'documenting': 42} counts repeated output observations, not unique rebuilt crates. Command total **4985.158862s** and observed Cargo-stage subtotal **1418.09s** overlap, can be incomplete and are neither additive nor continuous elapsed time. Final receipt validation is included after a metadata-only refresh.

| Record ID | Label / result | Command s | Cargo s |
| --- | --- | ---: | ---: |
| `20261030T072804.697353Z-c7b7003f6640` | edit loop native account check; failed | 23.989478 | — |
| `20261030T073318.263886Z-884c7e819539` | edit loop approved QR native check; passed | 13.504804 | 12.41 |
| `20261030T073756.721316Z-71c5839f47f4` | trim owned incremental before SSR graph; passed | 0.133149 | — |
| `20261030T073811.535092Z-685bd5e775fa` | edit loop shared SSR check; failed | 26.902012 | — |
| `20261030T073931.356708Z-e2232c86ed44` | edit loop shared SSR followup; failed | 4.998280 | — |
| `20261030T074045.482345Z-5a54ca741978` | source account invocation oracle; passed | 0.851705 | — |
| `20261030T074157.066141Z-ae52b9837ecf` | edit loop account source trace tests; failed | 27.871738 | — |
| `20261030T074328.441095Z-50adcdd950b0` | edit loop exact CSV source traces; passed | 12.254222 | 7.70 |
| `20261030T074626.460075Z-eec4c8607090` | trim incremental and completed owned consumers near cap; passed | 0.983316 | — |
| `20261030T074858.211543Z-999e80a91a39` | edit loop finite SQL Leptos host; failed | 68.384374 | 65.00 |
| `20261030T074947.531318Z-e397ab5c0689` | source signup password strength oracle; passed | 1.932713 | — |
| `20261030T075132.807266Z-18e938a511fa` | edit loop complete finite SQL host; failed | 23.682976 | — |
| `20261030T075333.737976Z-5ac76f316153` | edit loop locked finite SQL host; failed | 13.597937 | 11.05 |
| `20261030T075526.430464Z-852c1149e0f9` | edit loop finite SQL host view handoff; passed | 11.162686 | 7.93 |
| `20261030T075744.354595Z-00e7e84167de` | trim before shared UI tests; passed | 0.429112 | — |
| `20261030T075911.578659Z-4758992d2dd5` | edit loop SSR forms and SQL traces; failed | 25.242403 | — |
| `20261030T080034.955903Z-411c8f252615` | required API sql codegen ordered Record check; failed | 2.807529 | — |
| `20261030T080209.903333Z-3a68c4d703a6` | edit loop ordered SQL and shared UI; failed | 21.641421 | — |
| `20261030T080347.280153Z-bc6f9f15462f` | edit loop formatted ordered SQL and shared UI; failed | 17.088424 | 11.52 |
| `20261030T080655.479386Z-cf493f229eae` | edit loop rich Date CSV and source parent-response guard; failed | 2.887816 | 0.56 |
| `20261030T080809.503534Z-8bca61501786` | edit loop Date export and parent-body parity correction; passed | 13.402031 | 8.40 |
| `20261030T081057.746934Z-63dc3ca918d7` | trim incremental and completed proof instrumentation before WASM; passed | 10.125925 | — |
| `20261030T081218.570020Z-0602fedfdff6` | gate hydrate consumer graph; failed | 349.517961 | — |
| `20261030T081548.601637Z-13c5e86da39e` | original account and generated removal join proof; passed | 3.394162 | — |
| `20261030T082212.395748Z-268f97186602` | gate hydrate consumer corrected labels; passed | 12.335015 | 7.41 |
| `20261030T082537.021220Z-fb55d0d2a3e4` | edit loop finite host explicit theme matrix; passed | 20.685798 | 16.84 |
| `20261030T082737.919980Z-b96b718f56e6` | gate hydration finite browser matrix; failed | 96.594052 | — |
| `20261030T083001.909985Z-a427784668af` | round 1 logic and responsive layout; passed | 25.964089 | 17.30 |
| `20261030T083433.155643Z-9111f3b1acb6` | round 2 SQL responsibility and public UI contracts; failed | 2.749573 | — |
| `20261030T083537.239194Z-16ae5aafd40e` | round 2 complete public UI contracts; failed | 202.753606 | 22.10 |
| `20261030T083935.607891Z-d38dd29a0175` | round 2 native and rendered route regressions; failed | 10.228944 | — |
| `20261030T084136.416541Z-d08f17d8290c` | cap recovery owned incremental and package variants; passed | 3.809949 | — |
| `20261030T084225.797055Z-2b0534c5f16d` | round 2 full owned regressions after SSR fixture startup; passed | 41.055879 | 22.95 |
| `20261030T084725.478412Z-6c5b08bf309d` | round 3 consistency Rustdoc and source duplicate enrollment fields; failed | 44.319163 | 24.41 |
| `20261030T084904.951934Z-5a747c9417d6` | round 3 checked long-error fixture and compiled direct-form docs; passed | 56.837139 | 29.08 |
| `20261030T085011.286494Z-9811eea5b850` | final separate hydrate consumer build and browser graph; passed | 8.336424 | 4.56 |
| `20261030T085036.722407Z-215d4f9c47fe` | independent Vision scan of rendered enrollment QR; passed | 27.480528 | — |
| `20261030T085050.979906Z-c4d24d5ba921` | final hydrate 176 finite browser cases with long content; passed | 186.386084 | — |
| `20261030T085109.296095Z-1c40acef214c` | separate CSR consumer build and browser graph; passed | 48.068777 | 46.56 |
| `20261030T085216.661913Z-99f3b010845d` | full workspace gate SSR providers email excluding tcp_watchdog; failed | 70.218229 | — |
| `20261030T085231.560929Z-25a70cc5ee50` | separate CSR 176 finite browser cases with long content; passed | 201.803637 | — |
| `20261030T085357.394884Z-12607af24c40` | final gate lint branch cleanup and narrow account layout regression; passed | 64.911198 | 28.95 |
| `20261030T085441.605221Z-c31c4df63780` | source oracles external lock tuples and scope audit; passed | 15.751718 | — |
| `20261030T085540.076796Z-32d71e836076` | final hydrate and CSR rebuild after narrow layout fix; passed | 15.607404 | 11.37 |
| `20261030T085637.182474Z-3ed4fc21a252` | final full workspace gate after lint fixes; failed | 25.924031 | — |
| `20261030T085637.360370Z-2e15c67d1abc` | final hydrate browser proof after narrow layout fix; passed | 209.066684 | — |
| `20261030T085637.535691Z-5c2ce501a0fb` | final CSR browser proof after narrow layout fix; passed | 230.687962 | — |
| `20261030T085809.229756Z-0eac5b9f06c7` | full workspace gate final example borrow lint; failed | 250.846452 | 104.00 |
| `20261030T090304.797194Z-fb856196f3c3` | source generic POST signin view inventory regression; passed | 16.178884 | 10.63 |
| `20261030T090455.500932Z-87312ee7ffca` | direct custom account head and layout helper regression; passed | 37.805281 | 16.25 |
| `20261030T090613.054878Z-13d75292e690` | final direct-helper hydrate and CSR builds; passed | 14.912773 | 11.95 |
| `20261030T090654.409573Z-ff8c5ab40816` | completed helper inventory hydrate browser matrix; passed | 209.612920 | — |
| `20261030T090703.547090Z-69f7e06a2d41` | completed inventory full workspace gate; passed | 664.111864 | 267.59 |
| `20261030T090703.560061Z-bd763e50163b` | completed helper inventory CSR browser matrix; passed | 227.443945 | — |
| `20261030T091323.716207Z-7f9fa211b1b0` | zoomed profile row geometry review; passed | 12.935991 | — |
| `20261030T091835.147445Z-cdd5e48d3e62` | approved owned-package headroom before coverage under 16 GiB cap; passed | 2.863429 | — |
| `20261030T091902.365478Z-637f015a9574` | native WASM feature and dependency boundaries; passed | 233.829829 | 227.58 |
| `20261030T092331.018994Z-3f6b8494e1e8` | coordinator visual followup and sql-only cfg warning gate; passed | 82.285381 | 55.69 |
| `20261030T092632.614806Z-8e42d311a99e` | final source-translated accessible labels regression and browser artifacts; failed | 6.433077 | — |
| `20261030T092654.741482Z-2b10f3db5a58` | source-translated accessible labels final formatted gate; passed | 94.820797 | 54.30 |
| `20261030T092900.135212Z-094758301bb7` | final coordinator visual fixes CSR browser matrix; passed | 234.769330 | — |
| `20261030T092900.138564Z-46d57db59952` | final coordinator visual fixes hydrate browser matrix; passed | 236.578643 | — |
| `20261030T092900.157808Z-85f4fe1a05ef` | session leptos SSR line coverage gate; passed | 338.265201 | 314.00 |
| `20261030T092924.263158Z-81e2c4297216` | final original oracles lock scope license and KB audit; passed | 19.696687 | — |
| `20261030T093959.122441Z-c8e9b1a6c081` | final receipt and unchanged-source validation; passed | 3.406321 | — |

## Questions for the owner and stop

Should account removal eventually remove the profile despite the existing parent response body? Exact source: packages/stackpress-session/src/session/pages/remove.ts:28 loads that body; line 69 passes it to profile-remove; packages/stackpress-sql/src/transform/events/remove.ts:79–81 skips when res.body is truthy. Both the original joined-handler proof and real SQLite preserve 1 auth removal/0 profile removals with success flash and /auth/signout redirect. No fix is selected. Prior ILIKE/Auth create-return/positive hasnt questions remain at source parity.

No M4/composite integration, other-lane write, historical Agent File edit, commit/push/install/GitHub write or listener occurred. Coordinator verification, including tcp_watchdog and the combined API sql/codegen surface, is next. Work stops at M3.

## Merge integration

Integration-only follow-up on merged HEAD `7b4995b`. Replaced these three deprecated Record/IndexMap calls with insertion-order-preserving `shift_remove` in `crates/framework/framework-api/tests/sql.rs`:

- Line 297: `input.remove("id")` → `input.shift_remove("id")`.
- Line 298: `input.remove("secret")` → `input.shift_remove("secret")`.
- Line 411: `without.remove("id")` → `without.shift_remove("id")`.

Workspace-wide Rust/source/template searches across src, crates, examples and tools found no other deprecated Record/DataMap removals; remaining remove calls use vectors, standard maps/sets, serde_json maps or nondeprecated wrapper/action APIs. The exact requested combined-feature workspace Clippy `-D warnings` passed (record `20261030T095201.079591Z-7a3249cf83aa`); API nextest with `sql,codegen --locked` passed **81 tests, five skipped** (record `20261030T095329.633702Z-97d9af85eb1b`). Warm target/sccache/gate environment and the existing 16 GiB cap/15.4 GiB guard were retained; no cleanup was needed. Only those three test lines and this appended note changed; Cargo.lock is unchanged. No commits; stop after this integration fix.

### Email/session renderer assertions

Coordinator accepted diagnosis `20261030T100115.575096Z-9ec88bdcf189`: the real session view matches upstream, leaves the email input empty, and renders before the response-only fixture. Its tuple snapshot and absent fixture `render` marker invalidated the old test assumptions. Only `framework-email/tests/cases/integration/{rendering.rs,mod.rs}` changed in this follow-up. Helpers now check the selected public snapshot/HTML, native magic-link confirmation without a form, native request-input round-trip, and source-empty email field. Existing fixture escaping, event order, status, redirect, CSRF and mail assertions remain; shared checks reject literal injected script/seed markup, SMTP/provider sentinels and leaked database seed. No production or oracle changes.

Three review rounds covered branch/escaping logic (package tests), responsibility and dependency-feature selection (six-feature workspace tests), then comments/consistency (formatting, Clippy and full combined tests). No further refactor was warranted. These test-only changes do not alter the previously measured production coverage.

| Verification | Result | Benchmark record | Command seconds | Cargo stage seconds |
| --- | --- | --- | --- | --- |
| Email/server package, fixture graph | 51 passed | `20261030T102119.453392Z-095085fc6ab4` | 35.508626 | 23.67 |
| Six-feature workspace, email selection, fixture graph | 51 passed | `20261030T102530.311710Z-37e9e9e69a69` | 42.085426 | 37.53 |
| Full combined workspace formatting + Clippy `-D warnings` | passed | `20261030T102641.859646Z-920a81c3ccf0` | 15.186045 | 9.56 |
| Full combined workspace, default parallelism | 950 passed; unrelated HTTP m1_watchdog timing assertion failed | `20261030T102724.419469Z-0573c0170f1e` | 95.483607 | 15.66 |
| Isolated m1_watchdog, same full graph | passed | `20261030T103030.787587Z-8b6e298eabeb` | 6.266025 | 0.73 |
| Full combined workspace, two test threads, native session view | 951 passed; 88 skipped | `20261030T103130.789936Z-0be9714a1198` | 73.409059 | 0.71 |

All workspace nextest commands exclude tcp_watchdog; the six-feature run selects all email tests in the workspace graph. The initial default-parallelism failure was the existing 80–300 ms elapsed-time assertion at `crates/server/http/tests/m1.rs:492`; no HTTP source/test was edited. The final full run retains every selected test, with `--test-threads 2` limiting interference. Full features are session/leptos-ssr, csrf/server, email/server, api/sql, api/codegen, session/auth-server and schema/providers (full Cargo package prefixes in recorded commands).

Warm `target/slice9a`, sccache, two build jobs and nonincremental compilation were retained for headroom. Incremental-only cleanup freed zero bytes (`20261030T101639.875385Z-57b079cdc60b`); owned session/email package cleanup freed 627.4 MiB while retaining dependencies (`20261030T101719.359067Z-dd8cda64f2a7`). The first six-feature build hit the old 15.4 GiB guard (`20261030T102227.435826Z-3b9875295dd6`; size inventory `20261030T102506.071966Z-84e3ddd5f938`). Coordinator then raised the cap from **16 to 20 GiB**, guard **15.4 to 19.4 GiB**, updated in `/private/tmp/slice9a/run.py`. Resumed checks peaked at **16.503 GiB**; no further cleanup or dependency-artifact removal occurred. Cargo.lock/external tuples remain unchanged. No commits or listeners; stop for coordinator verification.
