# Slice 9 M0 Lane A receipt

Status: **Implementation/proofs complete for coordinator verification; stop before M1.** Based on `97edcb0` (confirmed by git log -1). Authority: accepted Slice 9 design, G42/G43, G44, and coordinator's G45 approval of aes/cbc and exact base64 key derivation. No owner/UI/platform acceptance is claimed.

## Delivered scope and stable API paths

| Owner / files | Delivered now |
| --- | --- |
| [framework-session](../../../../crates/framework/framework-session/README.md) | Portable grant/config types; native Service/Session JWT/cache/can; source codecs/date adapter; server identity constituent; source/SQL/ownership proofs. Default features remain empty. |
| [schema providers](../../../../crates/framework/framework-schema/src/providers/mod.rs) | Built-in CUID2 and SHAKE256/AES-256-CBC helpers, under explicit **framework-schema/providers**. Only this new module plus minimal schema lib.rs/Cargo.toml lines changed. |
| Root Cargo.toml/Cargo.lock | One new workspace member and approved resolved packages. Root facade src/lib.rs, stackpress-lib, other shared code and Lane B files unchanged. |

Stable API for task 00021 (enable `stackpress-framework-schema` feature `providers`, native target):

| Public path under stackpress_framework_schema::providers | Contract |
| --- | --- |
| `cuid() -> SchemaResult<String>` | Shared native CUID2-compatible 24-character generator. |
| `CuidGenerator::new/with_length/generate` | Independent host generator, length 2–32; SHA3-512, ring entropy, native fingerprint, clock and atomic counter. |
| `BuiltinProviders` | Implements existing ValueProviders.generate("cuid", &[]); opt in through ValueContext.providers. No host validators or unknown providers silently added. |
| `cuid_definition()` (also enable codegen) | Matching ProviderDefinition/revision for explicit DefinitionRegistry::with_provider before normalization. Real parsed @default("cuid()") integration is tested. |
| `encrypt(value: &str, seed: &str) -> String` | Source UTF-8 AES-256-CBC/PKCS7, lowercase hex. Key = first32 ASCII chars of base64 SHA256(seed); IV = first16 of that key. Built in; no injected workaround. |
| `decrypt(ciphertext: &str, seed: &str) -> SchemaResult<String>` | Source hex-prefix decoding and UTF-8 replacement behaviour; block/padding failure returns Codec error. |
| `hash(value: &str) -> String` | Source SHAKE256 with 32 output bytes, lowercase hex. No Argon2 replacement/migration. |

Session public paths are `stackpress_framework_session::{Service, Session, SessionConfig, SessionPermission, SessionData, SessionTokenData}`; native feature enables JWT/direct-token methods; server additionally enables request/response helpers and `plugin::register`. The M0 registrar is explicitly the **identity constituent**, not the unfinished auth→email→session composite. It uses ordinary #14 callbacks and opens no database/listener.

## Source parity and M0 proofs

| Requirement | Evidence / observable result |
| --- | --- |
| Source defaults/claims/expiry | key=session, seed=abc123, no added exp by default; nonzero expires is absolute; input exp retained at zero; alg-only HS256 signing/iat. Original jose verifies an actual Rust-produced token. |
| Null-on-invalid and header/algorithm boundaries | 21 original-jose vectors cover HS256/384/512, arbitrary object claims, optional/fractional exp/nbf, exact boundaries, invalid iat/exp/array payload, critical/noncritical headers and padded base64url. Wrong seed/forged/empty malformed tokens become null. |
| Source provider boundary | Original jose accepts the short default but rejects a zero-length HMAC key. Separate original-provider fixture and failing-before-fix test prove identical native create failure/null verification, without a stronger minimum. |
| Memoization/ownership | Each owned token has one OnceLock cache, including null; immutable token, no implicit Clone. Expired/seed-changed new instances reverify while old data stays cached. 16 concurrent readers, real Send tasks and reentrant constructor-clock proof pass. **No request-local slot is needed or changed.** |
| Missing versus empty access | No session section contributes no service/listeners. Missing access contributes me/authorize but no request guard; present {} installs permissive guard. Empty authorize skips token load, Vec mutation and response projection. |
| me/can | Guest defaults merged with claims then raw token/permits; empty can=true, all-of requests/any-grant; roles=[] differs from missing/falsey; truthy nonarray roles error. Strings dedup by value and route objects by Arc identity. |
| Guard/G42 | authorize prepends current route, returns original 401 Unauthorized or authorization data. Guard uses fresh child response, copies failures and emits error; successful child results stay separate. Normal guard returns Continue; priority/abort cooperation retained. |
| Matching/coercion fit | Exact methods/ALL, unescaped wildcard regex derivation, case/sticky flags, params/trailing slash and source role truthiness tested. Unsupported JS lookaround/backreference syntax is an explicit Rust-backend error, not a new denial policy. |
| SQL codec fit | Original Node crypto vectors: empty/Unicode/full-block values and seeds. Real SQLite store→encrypted equality lookup→decode, global token uniqueness, deterministic secret bytes and consumed field roundtrip. This is a codec fit proof, not the M1 Auth/Profile adapter. |
| Date/challenge fit | Three original UTC Date.toString/hash/OTP-hash vectors plus positive/negative offset day boundaries. Native adapter takes actual host offset/zone label explicitly; unsupported native calendar/offset ranges error. No ISO substitution or new TTL. |
| CUID2 fit | Nine direct @paralleldrive/cuid2 3.3.0 init vectors (length 2/24/32 and three entropy streams), native shape/length cases, concurrent 1,024-ID sample and real ValueProviders/normalization integration. No universal collision guarantee is inferred. |
| Native/browser boundary | Default session/schema WASM builds pass; normal dependency trees exclude ring/jsonwebtoken/aes/cbc/sha3/server/SQL. Service/cache/resources have no Serialize implementation; source authorization DTO deliberately retains token data. No SSR/page acceptance is claimed at M0. |

Compile-fail cases: non-Send/non-Sync Clock, readonly token, non-Clone Session, nonserializable native Service; separate non-Clone doctest. Native clock/caches retain explicit ownership and no held settings lock across await/host callbacks.

Deadlock-class test runs in an external parent process: 2s entry acknowledgement, absolute 15s deadline including startup, kill/wait/reap on failure. An intentional permanently blocked child is detected/killed at 200ms; real barrier-controlled cache/reentry children pass. Children create threads, not helper processes/listeners. The outer command supervisor uses process-group TERM/KILL/reap and a 30-minute bound; Cargo temporary-file races are tolerated by its target-size walk.

## Verification and limits

| Gate component | Final evidence |
| --- | --- |
| Full workspace gate | fmt and workspace Clippy -D warnings passed; **776/776 nextest tests passed**, 83 skipped; **944 workspace doctests passed**. Per coordinator, `-E 'not test(tcp_watchdog)'`; coordinator owns that port-binding case. Other ignored live backends/child helpers remain explicit. |
| Final affected-crate rerun | **41/41 tests passed**, one intentionally ignored watchdog child executed by its parent; 4/4 doctests, fmt, Clippy and warning-clean Rustdoc pass. Includes final empty-key correction and tightened watchdog/outcome assertions. |
| Coverage | cargo-llvm-cov nextest, native aarch64-apple-darwin, session/server + schema/providers/codegen. Production source only; test/example source excluded, no production-file exclusions. |
| framework-session | **520/556 = 93.53%** executable lines. |
| framework-schema | **2680/3815 = 70.25%** executable lines, including existing codegen/runtime. Meets 70%; this is not 90% publication coverage. |
| Reproduction/scope | Stored crypto/CUID/date/JWT/empty-key fixture JSON reproduced from read-only original packages; git diff --check and exact write-scope audit pass. Source/test/proof manifest digest `f5b8eadea831c86f0e61e3370d8e27f5662d75359ebf550f7cd306a12c9a2440` (temporary manifest outside repo). |
| KB validation | Baseline 146 errors / 27 warnings (existing upstream path/layout links). Final validation preserves that baseline: zero new diagnostics; receipt links and line cap pass. KB checks are not code tests. |

Coverage LCOV/summary are `/private/tmp/slice9a/m0-coverage.lcov` and coverage-summary.json. Fixtures and reproduction scripts are retained in framework-session/proofs and schema/providers; private-tmp files may be cleaned externally. Probes use installed Node 26.2.0, jose 6.2.0 and CUID2 3.3.0; no installation/download of Node tooling occurred.

Rust mechanics only: per-host Arc configuration and synchronized per-instance cache replace JS statics; Result/Send/owned lifetimes replace exceptions/promises; explicit timezone input and existing Rust regex grammar are stated above. JSON claims use the monorepo's finite serde_json representation. Source flags/roles, errors, lifetimes, unchecked paths and future account flows are not redesigned.

M0 reviews: (1) logic/branch/lock reentry and CUID substring bounds, tested; (2) separated JWT wire/claim adapter from Session ownership, tested; (3) consistent exports/comments/Rustdoc/native serialization boundary, tested; (4) Rust safety/Send/cache/task/feature review, no unsafe or additional material finding. Subsequent source padding/empty-key regressions failed before their fixes and passed after. These are M0 reviews, not acceptance of later full-slice rounds.

## Dependency and lock audit

Approved direct additions: jsonwebtoken =11.1.0 (defaults off, rust_crypto), sha3 =0.10.8 (defaults off), aes =0.8.4 (defaults off), cbc =0.1.2 (defaults off, alloc/block-padding). Reuse ring =0.17.14, base64 =0.22.1, time =0.3.55 and existing serde/regex/lib/server/SQL/Tokio. Native schema dependencies are feature/target gated; default graphs unchanged. No Cargo update/generate-lockfile command ran.

All **616 original external (name,version,source,checksum) tuples are unchanged**; 46 external packages added. Original tuple digest: `ff7003fee2586f93fde428ea4bed71f5f125a8d5f8b6a3bbc88551fa103bcf4f`. Existing resolved edges are retained. Cargo necessarily changed text in nine existing blocks: version qualifiers in five; approved feature unification adds const-oid/subtle to digest 0.10.7, zeroize to generic-array 0.14.7, libm to num-traits 0.2.19, zeroize_derive to zeroize 1.9.1. No original version/checksum/source was replaced. The strict initial edge-equality audit failed on those additions; final audit reports them explicitly rather than hiding them.

## Benchmark records and resources

Shared store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Every executed Cargo/proof/clean command is recorder-wrapped; outer multi-command proofs are wrapped once. Full commands/environment/output live in runs/<id>.json/.log; nested gate commands are printed in logs. One target root `target/slice9a`, sccache, jobs=2, dev debug=0; edit loop incremental unset, gates incremental=0. Required cargo clean and llvm-cov clean --workspace ran before first build.

Attributable completed records: **33 (24 passed, 9 failed)**. Command sum **1278.595926s**; observed Cargo Finished-stage subtotal **472.07s**. These overlap and are not session elapsed time; captured nested/failed builds can leave the Cargo subtotal incomplete. First coverage record labelled warm used a cold instrumented subtarget within the warm effort; the final instrumented rerun is warm.

Observed target peak: **5,504,620,267 bytes (5.127 GiB)**, below 8 GiB; sampled, not a continuous process-memory peak. Warm artifacts retained for coordinator reruns. Native test children/SQLite engines are reaped/closed; no listener remains. Raw coverage, helper scripts and baseline copies are outside the repository.

| Record ID | Label / result | Command s | Cargo s |
| --- | --- | ---: | ---: |
| `20261030T025415.789023Z-6b9f414bc8e5` | gate initial cargo clean; passed | 2.050654 | — |
| `20261030T025803.446764Z-5b92bb0f82c0` | gate initial llvm-cov clean workspace; passed | 2.054229 | — |
| `20261030T030237.890435Z-cf363ca3f7e0` | source crypto CUID2 date vectors; passed | 2.041289 | — |
| `20261030T030612.735504Z-144c1d3b48ef` | edit loop first native check approved dependencies; failed | 2.045698 | — |
| `20261030T030747.086387Z-720f9e191ee0` | edit loop native check fetch approved packages; failed | 101.513433 | — |
| `20261030T030836.780123Z-0fcf644cbda3` | source jose JWT interoperability vectors; passed | 2.063280 | — |
| `20261030T031246.057311Z-ef4154f3985e` | edit loop native tests and compile contracts; failed | 2.084018 | — |
| `20261030T031325.675406Z-871cad2a7e6e` | edit loop test compile and lock local dev dependencies; failed | 49.888255 | — |
| `20261030T031532.475275Z-dc64d93d39a0` | source jose critical header acceptance vectors; passed | 2.159079 | — |
| `20261030T031617.944169Z-a301d0474985` | edit loop corrected integration layout and JWT adapter; passed | 97.503914 | 58.47 |
| `20261030T031926.982581Z-ddb085eca303` | edit loop round1 lock reentry and source bounds; passed | 14.124757 | 4.58 |
| `20261030T032001.888206Z-39396b092992` | gate original external lock identity and edge audit; failed | 2.297516 | — |
| `20261030T032127.639252Z-a4ec6a880fd1` | edit loop round2 JWT responsibility extraction; passed | 13.569107 | 4.10 |
| `20261030T032338.451162Z-fd57bf2779f6` | edit loop round3 formatting; passed | 4.562032 | — |
| `20261030T032345.116784Z-252579a195f7` | edit loop round3 tests docs and native serialization contract; passed | 23.144723 | 9.71 |
| `20261030T032534.522568Z-2977347335a0` | gate final provider formatting; passed | 7.945874 | — |
| `20261030T032857.965911Z-9830cb3a59bf` | lock tuple audit with explicit optional-edge report; passed | 2.537391 | — |
| `20261030T032927.391777Z-6cc799b98e78` | gate workspace excluding coordinator tcp_watchdog; failed | 193.002389 | — |
| `20261030T033405.812237Z-11e5df0b1550` | source jose base64 acceptance boundary; passed | 2.347013 | — |
| `20261030T033449.789444Z-9fe84287cdbe` | source jose base64url padding fixture; passed | 2.273994 | — |
| `20261030T033526.215089Z-06d22c954e24` | source padding regression before fix; failed | 6.860322 | 3.43 |
| `20261030T033656.933632Z-e92b1bc0d68b` | gate source-padding formatting; passed | 4.651617 | — |
| `20261030T033734.376063Z-02f958847fa4` | gate workspace corrected codegen and source padding; failed | 449.723305 | 184.03 |
| `20261030T034701.093489Z-fc90ac594ea6` | gate final assertions and Rustdoc formatting; passed | 4.839634 | — |
| `20261030T034716.013964Z-666f8f9606a2` | gate native and WASM boundaries with original jose consumer; passed | 56.332891 | 44.75 |
| `20261030T034825.312113Z-75723c8624ff` | gate final fmt clippy and warning-clean Rustdoc; passed | 40.329224 | 36.27 |
| `20261030T034937.295223Z-8d575afed58a` | gate per-crate line coverage native providers and codegen; passed | 130.742042 | 107.00 |
| `20261030T035608.164383Z-d1503fe33623` | source zero-length JWT provider boundary; passed | 2.577790 | — |
| `20261030T035708.555548Z-e2c0f5cff3e4` | zero-length source regression before adapter fix; failed | 7.152723 | 4.21 |
| `20261030T035833.010901Z-7c6447eb9f26` | gate final source provider boundary and owned checks; passed | 23.943604 | 10.17 |
| `20261030T040049.429797Z-9d40dfb5438c` | gate final per-crate coverage after source boundary fix; passed | 17.045652 | 5.35 |
| `20261030T040354.602345Z-b4a073040e7f` | gate final source fixture lock and write-scope audit; passed | 2.498469 | — |
| `20261030T040903.009521Z-410246cc1e21` | gate receipt links line cap and unchanged KB diagnostics; passed | 2.690008 | — |

Retained failed attempts: initial offline missing package, response cookie accessor compile error, locked local-dev dependency refresh, integration-module layout, overly strict edge-equality audit, initial codegen argument/monitor race, bare Rustdoc URL, and two expected source-parity regression failures. Each relevant fix has passing evidence above. One invalid recorder kind was rejected before Cargo ran. The first full gate approval was rejected because tcp_watchdog was not excluded; renewed coordinator approval supplied the explicit exclusion. It was never executed here.

No production work beyond M0 is released. Auth/Profile full storage/actions, signup (phone optional under G44), other account/auth flows/pages and the auth→email→session composite remain for M1+. Lane B API assumptions remain the accepted ordinary registrars/mechanism interfaces; no CSRF/email stub or crate was created. No stackpress-lib request carrier/facade edit, historical Agent File edit, commit/push, install, GitHub write, upstream write or development listener occurred. Next step is coordinator verification (including tcp_watchdog), then an explicit next milestone release.
