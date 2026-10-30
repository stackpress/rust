# Slice 9 Lane A M2 implementation receipt

Status: M2 implemented; pending independent coordinator verification. Stop at M2. Base/worktree HEAD is 70917ac63b74e283bbb9f5203f9af19f4f00ea6d. G42/G43/G44 remain binding. This receipt does not release M3, integration, publication or owner acceptance.

## Delivered surface and ownership

Only framework-session production/tests/docs and its three internal Cargo.lock dependency edges changed. No new external package was added. Feature `auth-server` selects the existing native/sql/server mechanisms plus reusable auth events and native auth process routes. The independent identity registrar remains `plugin::register`; the source auth→email→identity composite is reconciled at M4 with Lane B's registrar. M3 owns account pages, shared presentation props, forms and Leptos views.

Stable public paths under `stackpress_framework_session`:

- `auth::{register, AuthConfig, AuthSection, AuthEmailConfig, AuthMenuConfig, Environment, RandomFraction, Hooks}`.
- `auth::events::{signup, signin, signout, search, update, normalize_phone, json_result}`. Search/update are explicitly selected source SQL event adapters, with no new operation/global policy.
- `auth::pages::{signin, signup, signout, username, phone, email, otp, magic_link, two_factor}`; exported helper reuse is `password_signin`, `otp_signin`, `magic_link_signin` with an Invocation borrow.
- `auth::templates::{SigninEmailTemplateInput, Template, make_otp_template, make_magic_link_template}` and `auth::totp::{base32_decode, generate_at, verify_at}`.
- Identity handlers are directly reusable as `plugin::{me, authorize_event, request_guard}`. API caller identity/can contract and M1 `profile::ProfileSchema` are unchanged.

The auth registrar owns only its auth section and Hooks service. Missing/falsy auth config omits process routes while retaining signup/signin/signout events. Configured unused processes require no eager Engine/session/CSRF/email dependency. The native host supplies Engine/CSRF and optional Environment mapping database.seed, email presence, host/server.port, brand.name and local clock/timezone. It binds Hooks to an Arc<Server> and explicit Tokio Handle after build, before accepting calls. Weak host ownership prevents a registry cycle; pre-bind payloads remain queued. Drain/shutdown are explicit lifecycle responsibilities. No stackpress-lib request carrier was needed or edited.

Lane B's frozen contracts are consumed through `email-send` and `TaskSupervisor::spawn_on`; no transport implementation or other-lane source was created/edited. Finite tests supply email-send results and do not claim live provider acceptance. Full caller/API examples and behavior notes are in the crate [auth reference](../../../../crates/framework/framework-session/docs/auth.md).

## Source parity and mechanics

Read-only source HEAD: a71d683051ba8350fdd12d6b5a33f268fdcc285f, packages/stackpress-session/src and source SQL helpers. Oracle uses the original TypeScript handlers, templates, TOTP and date column/helper functions with finite child services. Node 26.2.0 executes its installed upstream TypeScript compiler; no download/install occurred.

| Behavior / exact upstream location | Rust mapping / evidence |
| --- | --- |
| auth/plugin.ts:9–100 | Three Listen events always present; nine GET then seven POST process routes only for active auth. Empty base uses the source missing-only route default. Actual registration order, equal-priority FIFO and higher-priority Abort are tested. |
| auth/events/signup.ts:16–63 | Config roles spread before request data; normalize phone; sequential SQL; email-email-send then auth-phone-verify without awaiting; strip only the three named secrets. Optional missing phone succeeds under G44 even though the preserved source oracle still records the old trim failure. |
| auth/events/signin.ts:15–34 | Truthy password flag; only literal false skips 2FA search. Successful Actions consumption/reload happens before late config/session work. 404 User Not Found and 401 Invalid Password retain code, capitalization and field keys. |
| auth/events/signin.ts:35–85 | Existing auth-search handlers supply factor lookup; source consumed challenge/encoded redirect; no added Profile-active check. Successful response strips secret, creates only original JWT claims and configured cookie. |
| auth/events/signout.ts:5–8; auth/pages/signout.ts:16–28 | Literal session cookie deletion/status 200; GET signout and original redirect precedence; configured identity cookie can remain. No CSRF/global gate added. |
| auth/pages/signin/{username,phone,email}.ts | Generate before incoming POST CSRF validation; force password=true/type, preserve submitted 2fa flag, normalize phone; child redirects/errors pass through. |
| auth/pages/signin/email.ts:116–235,247–355 | Search → rotate consumed using parent response → reload persisted value → derive challenge → late transport/sender checks → email-send using parent response. OTP success clears/redirects; magic success leaves send result. Missing config can retain rotation/result effects. |
| auth/pages/signin/otp.ts:80–121 | Strip secret before GET payload; GET performs no code/hash/age check. POST uses String/ECMAScript trim/UTF-16 padStart length, without digits-only requirement, then consumed+code hash and literal auth.base handoff. |
| auth/pages/signin/{otp.ts:76–77,link.ts:68–69} | Source statusCode(404) followed by setError's default produces final 400. Golden traces preserve it rather than silently fixing it. |
| auth/pages/signin/2fa.ts:80–214 | Selected auth consumed hash, flash/error/redirects, factor search and source TOTP. No factor finalizes even on GET without POST/CSRF/code checks; enabled factor GET waits. Shared event receives password=false, type/token and 2fa=false. |
| session/pages/authorize.ts:19–25; Session.ts:63–65 | Empty access returns true without results/code. The source guard still offers error on code zero/unset without creating a denial/status. Source-falsy null/false/zero/empty-string bodies do not skip; arrays/objects do. This corrects the earlier M0 adapter's skipped zero-status offer. |
| auth/templates.ts; session/helpers.ts:133–178 | Original escaped text/HTML and HMAC-SHA1/base32 behavior, lowercase/trailing-padding/leftover-bit handling, ±1 verification steps and ordinary comparison. Reuse ring; no new hashing/token/security format. |

Rust mechanics: compiled handlers replace dynamic imports; typed active config/resources, Send futures, owned hook payloads and explicit runtime/timezone replace JS runtime ownership/globals. Existing lib rich DateTime retains event instants; json_result explicitly projects ISO JSON. Native IO/resource values fail wire projection and Environment cannot serialize (compile-fail proof). Native Action causes remain inspectable; Rust does not fabricate V8 stack/start/end fields absent from the core typed error contract. Source accepted risks stay behavior/parity tests: request flags, roles override, literal logout cookie, unchecked direct helpers/GET paths, lack of extra TTL/Profile guard, and late missing-mail partial results. No default-deny, new global policy, route ownership or automatic hardening was introduced.

## Verification and refactoring

| Component | Final result |
| --- | --- |
| Workspace gate | fmt; Clippy -D warnings; **870/870 nextest tests**, 86 skipped (including explicitly excluded coordinator-owned tcp_watchdog); **957 doctests**; warning-clean workspace Rustdoc. Gate record 142bd9cc4a74: command 686.126791s, observed Cargo stages 321.72s. |
| Owned tests/docs | **38/38 tests**, one ignored child entered by the external watchdog; five session doctests pass, including native Environment serialization rejection. Five original .stderr expectations are unchanged. |
| Feature boundaries | Native default/native/sql/server/auth-server checks; WASM default/auth-server checks and normal dependency graphs exclude native session, SQL, CSRF/email, ring/JWT/AES/CBC/SHA3 and Tokio/lettre. No browser execution/installed-platform claim. |
| Independent consumer | Cold dedicated target with warm sccache: direct Session/can/Profile schema/template/TOTP plus ordinary auth/identity/manual handler registration, Hooks bind/drain/shutdown, no listener. Offline run passed: command 107.578216s, observed Cargo 104.00s. |
| Coverage | cargo-llvm-cov nextest, aarch64-apple-darwin, session/auth-server (includes native/sql/server): **3408/3774 = 90.30%** executable lines. Tests/examples excluded; no production exclusion. Development floor met; no publication claim. New auth bridge 128/130, events 189/212, hooks 66/71, templates 51/51, TOTP 50/50. |
| Source/lock/scope | Both original-source oracles reproduce; **691 existing external package blocks byte-identical, zero new/removed packages**. Only three session package edges (ring/CSRF/email) added to Cargo.lock. Final 71-file source manifest SHA256 `aac1c15f5be46ff0211ae77cb7528eb4661fd6c8eb952e40f124ced76f2cd2cb`. |
| KB | Existing **146 errors / 27 warnings** retained; receipt validation must add zero diagnostics. These inherited upstream layout/link diagnostics are not a clean KB claim or code verification. |


Oracle evidence: 57 original invocation traces, of which all 48 native page/process traces match status/error/field errors/redirect/body/input mutations/flash and nested child-call order. Child responses come from the original captures; actual auth event actions are separately proved with SQLite. Two original template vectors, four TOTP vectors and three timezone/date→SQL vectors pass. The M1 380 column cases/17 email cases/scalars/returned-row oracle reproduces unchanged. Real registered-route SQLite proof exercises OTP issuance, persisted reload, code validation, GET final handoff, session creation, magic issuance and link handoff. No source helper's extension/no-op point became a new mandatory gate.

All five existing .stderr compiler expectations remain unchanged and pass for their original contracts. New Environment compile-fail Rustdoc rejects serialization. Actual handlers/hook futures satisfy the existing Send contracts; Lane B's owned/static supervisor contract is reused. The external watchdog now also runs real signup with two suspended hooks: parent returns, pre-bind queue remains owned, bind schedules, independent payloads retain their source submission data, drain observes completion, and shutdown aborts/joins a blocked hook with no retained server. Absolute parent deadline is 15s including startup, entry ack 2s, kill/reap on failure; the blocked negative control is killed at 200ms. Barriers/channels, not scheduling sleeps, order the work.

Three distinct test-verified rounds completed:

1. Logic/complexity: preserve String(object) versus column JSON coercion, strict false/truthiness branches and parent-response ordering; retain native Action causes and checked TOTP counter/window mechanics. Source object-code regression and full owned suite passed.
2. Responsibility/ownership: named source template entrypoints/input, cohesive oracle/SQLite/registration proof modules, explicit weak-bound hook owner/queued payloads/drain/shutdown. Owned tests plus external watchdog passed; unchanged compiler fixtures were rerun at the final gate.
3. Consistency/Rustdoc/comments: public paths, native/direct registrar examples, detailed branch/async comments, config defaults, shared timezone propagation into Auth/Profile collaborators, native value projection, and real SQL-backed OTP/magic routes. Clippy, 38 owned tests, five doctests and warning-clean Rustdoc passed. Rustdoc markup correction required no behavioral change; full workspace gate then passed.

## Resources, benchmark records and stop

One retained warm target target/slice9a, with sccache, jobs=2 and debug=0. Edit loops unset incremental; gates set CARGO_INCREMENTAL=0. The coordinator's **12 GiB** cap remains enforced by /private/tmp/slice9a/run.py; observed M2 peak **11,166,516,244 bytes (10.400 GiB)** is sampled filesystem size, not process memory. Before consumer/coverage, recorded cleanup removed only debug/incremental: **406,508,426 bytes**, retaining dependency builds. M0 initial clean/llvm-cov clean were not repeated in the same effort. Consumer uses target/slice9a/consumer-m2; no concurrent Cargo builds share the target. Raw LCOV/coverage summary, source manifest, resource observations and helper scripts remain under /private/tmp/slice9a/m2 (monitor observations in its parent). The worktree's warm target is retained for coordinator verification.


Recorder store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Attributed M2 snapshot: **29 completed attempts (17 passed, 12 failed)**; counter delta +29 excludes other agents' concurrent runs. **16 commands emitted compilation**; observed activity totals {'compiling': 368, 'checking': 184, 'documenting': 42} count repeated output observations, not unique rebuilt crates. Failed edit-loop/oracle/docs attempts remain recorded; all final verification commands pass. Command-time total **1132.245794s** and observed Cargo-stage subtotal **527.19s** are overlapping, incomplete measures and are not additive or continuous elapsed time. Every Cargo/proof/clean command is recorder-wrapped once at the outer script. The table includes the completed final receipt-validation record after the metadata-only refresh.

| Record ID | Label / result | Command s | Cargo s |
| --- | --- | ---: | ---: |
| `20261030T055551.627790Z-8739b46a3ecc` | edit loop first compile; failed | 41.415920 | — |
| `20261030T055812.054541Z-077dc47776d8` | source invocation oracle; passed | 0.675750 | — |
| `20261030T055940.201580Z-659fc901ef3f` | baseline and source traces; passed | 0.510856 | — |
| `20261030T055958.351188Z-45209b53c77e` | source nested invocation oracle; passed | 0.606421 | — |
| `20261030T060122.130462Z-9f80d7319bb5` | edit loop process traces; failed | 23.023613 | 19.23 |
| `20261030T060351.771231Z-47f8509ba777` | source child traces refreshed; passed | 0.426238 | — |
| `20261030T060417.221467Z-929e3e2515d9` | edit loop auth and supervised SQL; failed | 2.901392 | — |
| `20261030T060604.135939Z-5333d9b92358` | edit loop parity modes; failed | 9.957412 | 5.40 |
| `20261030T060707.604599Z-ef25120df175` | edit loop frozen dates and config defaults; failed | 7.578914 | 4.80 |
| `20261030T061024.180242Z-3458e3204420` | edit loop expanded source failure traces; failed | 5.137188 | — |
| `20261030T061144.455108Z-c9bf291a6c1b` | edit loop registration and guard traces; failed | 5.160484 | — |
| `20261030T061232.379154Z-9bf7e1eaaa78` | edit loop full auth branch matrix; failed | 5.217078 | — |
| `20261030T061351.931199Z-f73368cff419` | edit loop source matrix compiled callbacks; passed | 7.859437 | 3.33 |
| `20261030T061602.469873Z-bbfe59a96f54` | round 1 logic coercion and native causes; passed | 72.224962 | 5.92 |
| `20261030T061854.934383Z-243389f2abfc` | round 2 source helpers and owned hook boundaries; failed | 5.360348 | — |
| `20261030T062035.858641Z-1eac6601cb3c` | round 2 shared proof boundary corrected; passed | 7.615578 | 4.21 |
| `20261030T062700.349475Z-b0f98f863c14` | round 3 consistency Rustdoc and source timezone; failed | 3.079961 | — |
| `20261030T062912.465746Z-07cdb7215058` | round 3 source helper binding corrected; failed | 13.841655 | — |
| `20261030T063122.855457Z-168552cfb59d` | round 3 native auth API and full route flow; failed | 47.338148 | 13.07 |
| `20261030T063323.697931Z-341c4e8571d0` | round 3 warning-clean Rustdoc close; passed | 14.507271 | 5.18 |
| `20261030T063359.963063Z-142bd9cc4a74` | full workspace gate excluding coordinator tcp_watchdog; passed | 686.126791 | 321.72 |
| `20261030T063910.384121Z-bc460b11f145` | source lock and scope audit; passed | 2.007844 | — |
| `20261030T064623.277255Z-f01405059116` | native and WASM feature boundaries gate; passed | 9.451730 | 6.89 |
| `20261030T064806.418680Z-8bf149a61bf8` | 12 GiB guard trim owned incremental first; passed | 0.432368 | — |
| `20261030T064832.926926Z-a6f0c6056f6e` | standalone auth consumer cold target warm sccache; passed | 107.578216 | 104.00 |
| `20261030T065210.308951Z-5b7e4d063941` | session auth-server line coverage gate; passed | 37.494156 | 29.49 |
| `20261030T065501.198932Z-43826de2db24` | final hook lifecycle Rustdoc wording; passed | 12.198964 | 3.95 |
| `20261030T065605.595466Z-634eaedd9f3b` | final source lock scope and oracle audit; passed | 1.726668 | — |
| `20261030T065858.940515Z-f7963e9bc90c` | final receipt validation; passed | 0.790431 | — |

The three already-open source questions remain at parity: literal ILIKE, Auth create-return schema.unserialize behavior, and positive hasnt. No new owner/product/dependency decision is requested. No M3 implementation, root facade/shared-lib/schema change, other-lane write, historical Agent File edit, commit/push/install/GitHub write or listener occurred. Coordinator verification (including tcp_watchdog) is the next gate; work stops at M2.
