# Slice 9 M1 Lane B: direct CSRF and email transports

Status: M1 implemented; local gate complete; stopped for coordinator verification. Coordinator verification and commit remain separate. This receipt stops at the Lane B M1 row of [the milestones](../slice-9-design-milestones.md), under G43 and the coordinator's native-provider ruling. M2 registration/integration is not claimed.

## Scope and direct contracts

Base: `29b695388454b481f8d039469a763a7e46e0e1bd`, branch `trial/3-slice9-csrf-email`. Only framework-email files, Cargo.lock additions and this assigned receipt changed. Framework-csrf's full direct implementation already existed at M0 and is reverified without an artificial rewrite. No root src/lib.rs, session/API/schema, historical KB, other lane or upstream file is edited.

[CSRF](../../../../crates/framework/framework-csrf/src/lib.rs) retains real 32-byte random generation, outgoing cookie/SSR placement, exact clear scope, arbitrary nonempty matching UTF-8 strings, equal-byte-length timing-safe comparison, exact configured/default error and 419/Page Expired. Six direct tests cover random staging, query aggregation, malformed/missing/list values, byte/character distinctions, independent incoming/outgoing state and deletion. No Origin/method/expiry/format policy is added.

[Email direct API](../../../../crates/framework/framework-email/src/lib.rs) retains the per-call Send factory and adds native adapters. EmailConfig is the lossless object/string carrier; SendMailOptions retains extensions; NativeSendMailOptions additionally owns one-shot bytes/readers. SentMessageInfo retains provider JSON fields plus Buffer/Readable/raw outputs. Live reader serialization fails explicitly without draining or dropping it. SmtpError.fields preserves source command/code/response/rejection data, with the native SDK cause available through Error::source.

M1 changes the M0 JSON result alias into the serializable SentMessageInfo struct: custom providers return Value.into(); ordinary indexed JSON access remains available, while native stream/raw outputs have explicit ownership. M2 event/native-response projection remains integration work and consumes no new shared carrier interface here.

The M0 supervisor and three compile-fail contracts remain: Rc transport, non-Send state across await and parent-borrow rejection. Task drop/shutdown tests remain. Native readers have Send ownership; no detached task or global mutable transport is added.

## Transport and option inventory

The [crate inventory](../../../../crates/framework/framework-email/README.md) details the option/binding surface. [Resolution](../../../../crates/framework/framework-email/src/options.rs) preserves URL-before-variant selection; pool/sendmail/stream/JSON/SES/SMTP precedence; service aliases/domains and override order; pool defaults/rates; source truthiness/numeric/string coercion. Object url deliberately discards its siblings before selection. A full provider receives the original config and can use the public resolver for source defaults.

| Variant | Implementation and proof |
| --- | --- |
| SMTP | Lettre connection API; source host/port/secure/defaults, implicit/optional/required/ignored TLS, name/localAddress/connectionTimeout, LOGIN/PLAIN/static XOAUTH2, TLS servername/rejectUnauthorized/ca/minVersion and single client identity. MAIL/RCPT/DATA covers optional AUTH, partial/all rejection, command errors, SMTPUTF8/8BITMIME when advertised, raw framing and Bcc projection. Fake SDK vectors compare complete settings/envelope/raw/info; actual SDK protocol/parser uses an in-memory duplex, never TCP. |
| Pool | Full config/defaults resolve now. The whole pool is provider-bound, including default maxMessages=100: lettre lacks source recycling. Missing provider yields a clear named error before delivery. Fakes prove complete original options reach the provider. |
| JSON | Exact original-source object/string results: header/address precedence, mutation/property order, body and Buffer encoding, arbitrary fields and envelope removal. No delivery. |
| Stream | Buffer/readable results; exact raw bytes; generated MIME roles/headers/encoding compared semantically. Non-buffer results defer IO/errors until read and cancel by ownership on drop. Source transport newline flag does not rewrite raw bytes. |
| Sendmail | Exact default/explicit path/args (including empty args), envelope.to, leading-dash rejection and exit results. Fake processes capture invocation/raw. Native driver uses /bin/cat as an inert sink and a nonexistent path for errors, never delivery. Exit/stdin errors compete; completion is awaited/reaped. |
| SES | Native SESv2 client/command handles; complete command plus arbitrary mail.ses overrides, region fallback, id/response/raw projection. Fake SDK vectors only; no AWS SDK or live request. |
| String | Source smtp/smtps/direct fields/auth/query precedence; unknown protocol remains ordinary SMTP options as source. URL crate representation gaps/nonfinite numbers route to the full provider or a named error. |

[ContentResolver](../../../../crates/framework/framework-email/src/content.rs) supports native readers/bytes, Buffer JSON, text encodings, files, data URIs and HTTP(S). Transport access/header settings override message values, including false. Source JSON normalization bypasses access flags and ignores httpHeaders; MIME honors them. Fakes prove locations/full objects/header selection; native file/data decoding is exercised. No added size/origin/recipient/URL policy.

## Rust-mechanics deviations: library capability

The coordinator accepted native bindings for source capabilities lettre cannot honor. A recognized unsupported source option never silently disappears: the complete provider gets original config/message/native slots before content is consumed, or a named capability error is returned. Unknown extensions stay intact; source-unused fields remain unused. This choice is explicit in public docs and preflight code.

- Complete pool/maxMessages, LMTP, supplied/already-secured sockets, socket/auth callbacks; separate DNS/greeting/socket timeouts and DNS controls; proxy/logging callbacks; opportunistic TLS failure recovery/forceAuth; unknown auth mechanisms/OAuth2 refresh; unsupported TLS controls/native representations use the full binding.
- SMTP DSN, REQUIRETLS and SIZE require the full binding because lettre does not expose all server extensions. Original EHLO response lines are unavailable: the built-in result cannot supply source ehlo; a complete application TransportFactory supplies that SDK-specific result. Unknown-only AUTH advertisements and live default DNS/greeting/socket timing remain explicit SDK limitations, not proven source-equivalent negotiations.
- DKIM/compiler hooks, custom header normalization/data-image conversion, watchHtml/AMP/iCalendar/List headers, custom boundaries/newline, per-message auth, root MIME overrides, explicit encoding preferences for non-ASCII headers, raw/custom attachment MIME headers, alternative filename parameters and permissive/group address syntax outside the parser use the full binding. JSON data-image conversion also selects it before skipping the source default compile-plugin step.
- Lettre 0.11.23 Rustls Identity::from_pem stores the certificate argument as DER. The adapter converts one PEM certificate to DER; certificate-chain identities select the provider or named error. Public test-only cert/key data proves actual SDK identity construction without a handshake. No dependency/upstream source is patched.
- Native MIME folding/RFC2231/boundaries are tested semantically, not claimed byte-identical. MIME preparation buffers inputs; non-buffer output defers preparation until read. Raw bytes are exact. JavaScript Error/resource/prototype identity becomes explicit native errors/JSON/traits; timing uses a monotonic native clock. Process cancellation uses Tokio kill-on-drop without claiming explicit reaping after runtime shutdown.

These are Rust/library mechanics adaptations, not a new security model, hardening policy or removed transport scope. No original Stackpress defect is asserted; no owner product question remains open in this lane.

## Source and verification

The read-only original CSRF/email packages and session composition remain authority. M0's [source hashes/provenance](00020-slice-9-m0-lane-b.md) apply. M1 assumes original auth → email → session composition and caller-owned event/route protection; it needs no new Lane A/API interface. M2 will exercise email-send skips/results/errors and signup integration.

[Original oracle](../../../../crates/framework/framework-email/tests/source-transports.cjs) runs Nodemailer 7.0.13 with inert SMTP/process/SES boundaries; fixtures include source hashes. Corpus: 17 constructor, 18 JSON, 12 MIME, 3 stream, 4 sendmail, 3 SES, 4 SMTP cases (61), plus M0's 13 Stackpress option vectors. MIME comparisons assert observable roles/headers/encoding; JSON/raw comparisons are exact. Fakes do not establish external delivery/HTTP/TLS acceptance.

SHA-256: source-transports.json `49a07281a6ab52bf2743df02980de839b62f6ff10850f2154f4be60795e73ab2`; services `e4581edb26e207b7d64a79371fd6343e37d9c622c323138d891486141ec9e6d0`; MIME catalog `e3ef0475c660feaac6d76f83d82499a0d56f6f822722947e44616b3d703a5658`. Copied data retains original license and [provenance](../../../../crates/framework/framework-email/data/README.md).

Environment: absolute CARGO_TARGET_DIR=/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b; CARGO_PROFILE_DEV_DEBUG=0, CARGO_BUILD_JOBS=2, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G. Edit loop leaves incremental unset; gate/coverage uses CARGO_INCREMENTAL=0. M0 performed initial cleans; M1 reuses the target. Recorder-wrapped trimming removed only its old debug/incremental directory under the original 8 GiB cap. The coordinator subsequently raised this effort cap to **12 GiB**; the final size guard uses 12 GiB. Nothing else changed. Outer watchdogs bound compiler/concurrency proofs and kill/reap process groups; ordering uses no sleeps.

### Final gate

The first workspace run caught the new pinned-SDK identity representation issue (791/792 passed); the adapter conversion corrected it. A subsequent run passed all Lane B tests but hit two existing 10-second UI finite-child deadlines (791/793 passed). Their logs were empty; fixture source/deadlines were preserved. The final full-scope rerun uses two nextest workers to reduce process contention, still excluding only the coordinator-owned tcp_watchdog. No narrowed package/test acceptance is substituted. Subsequent source MIME path/stream-encoding and spelled-out encoding regressions were corrected and the complete gate rerun on the final implementation; all failed attempts remain in the table.

| Gate command after the recorder/watchdog prefix | Result |
| --- | --- |
| cargo fmt --all -- --check | Passed. |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | Passed. |
| cargo nextest run --workspace --all-targets --locked --offline --no-fail-fast --test-threads 2 -E 'not test(tcp_watchdog)' | **793 passed, 0 failed, 82 skipped**, 219 binaries. Includes all 37 Lane B tests (6 CSRF, 31 email); no other exclusions. |
| cargo test --workspace --doc --locked --offline | **947 passed, 0 failed, 0 ignored**; includes the three negative compile contracts. |
| RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps --locked --offline | Passed. |
| cargo tree -p stackpress -p stackpress-framework-csrf -p stackpress-framework-email -e normal --depth 1 --locked --offline | Passed; root default normal graph remains stackpress-lib only. |

Nextest's skips include existing ignored/environment-gated cases and the directed tcp_watchdog exclusion. Native aarch64-apple-darwin, default compatible workspace features; no renderer --all-features. Browser/WASM/installed targets and live provider acceptance are not M1 evidence.

Coverage uses `cargo llvm-cov -p <crate> --locked --offline --json --output-path /private/tmp/slice9b-m1-<crate>-coverage.json --fail-under-lines 70`, each under a 600-second watchdog and gate environment. All first-party implementation files are measured; no custom exclusions. Native Cargo/rustc 1.96.0; default crate features.

| Crate | Covered/executable lines | Line coverage |
| --- | ---: | ---: |
| framework-csrf | 52/54 | 96.30% |
| framework-email | 2202/2527 | 87.14% |

Email file detail (covered/executable lines): backend.rs 332/416; compose/headers.rs 117/118; compose/json.rs 109/111; compose/metadata.rs 112/119; compose/mime.rs 260/281; compose/node.rs 129/159; compose/part.rs 94/112; config.rs 7/7; content.rs 167/213; message.rs 50/50; native.rs 220/236; native/capabilities.rs 266/316; options.rs 247/296; service.rs 27/27; smtp_error.rs 46/47; tasks.rs 19/19.

Raw coverage JSON is ephemeral at the two /private/tmp paths above. SHA-256: csrf f6aca13251c0308e9013c06c1fb04148c53ffa9b1bcf3704395ef66dbfae7dd8; email 0f3b2e29ad412756f4127cef295b04e24b34c616a7a1e2b1ed099f587b6820e1. Remaining gaps include live TCP/TLS/HTTP SDK paths, OS entropy errors and unexercised coercion/data-size branches; fake transport success is not delivery or inbox acceptance.


### Dependencies, resources and KB

All 621 baseline external tuples (name/version/source/checksum) are retained; 20 packages added with feature-driven dependency edges. No cargo update/generate-lockfile. Exact dependencies: lettre 0.11.23 defaults off, builder/smtp-transport/tokio1-rustls-tls/hostname; reused ring 0.17.14, base64 0.22.1, url 2.5.8, reqwest 0.13.4 defaults off, rustls-no-provider/gzip/deflate/cookies. Tokio native IO/process/macros are explicit. Root Cargo.toml/src/lib.rs are unchanged.

Added: async-compression 0.4.42; compression-codecs 0.4.38; compression-core 0.4.32; cookie_store 0.22.1; hostname 0.4.2; hyper-rustls 0.27.10; jni/jni-macros 0.22.4; openssl-probe 0.2.2; psl-types 2.0.11; publicsuffix 2.3.0; rustls-native-certs 0.8.5; rustls-platform-verifier 0.7.1; rustls-platform-verifier-android 0.2.0; schannel 0.1.30; security-framework 3.7.0; security-framework-sys 2.17.0; simd_cesu8 1.2.0; simdutf8 0.1.5; webpki-root-certs 1.0.11.

Final target sample: **7,272,880 KiB (6.936 GiB)**; largest sampled allocation **7,272,880 KiB (6.936 GiB)**, below the revised **12 GiB** cap. Continuous peak sampling was not performed. One absolute warm target is retained; the only mid-effort deletions were its old incremental cache under the original 8 GiB cap. Final comparative audit checks the same 146 existing KB errors/27 warnings against the captured baseline, all 621 original lock tuples, the +20 external packages, exact lane write scope and diff whitespace. Historical diagnostics are not repaired by this lane. Audit passed: **0 new KB diagnostics**, baseline tuples retained, write scope confined and diff check clean; target guard is 12 GiB.

No listener, live SMTP/SES/sendmail, tool install, commit/push/GitHub write or other-agent message occurred. Formal refactoring rounds remain at the post-M4 design gate; ordinary cohesive module separation does not claim those rounds are complete. Stop after M1 for coordinator verification.

### Benchmark records

Attributable M1 records: **59** (44 passed, 15 failed). Recorded command wall total **1647.973017 s**; observed Cargo-stage subtotal **381.17 s**. The stage subtotal is incomplete for nested compiler proofs and is not additional time. Shared store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/<id>.json with matching .log.

| Record ID | Invocation/result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T034420.410272Z-9731eeb5da4a` | edit loop types and options check; passed (0) | 13.722022 | 13.51 |
| `20261030T035805.490408Z-98465f715712` | edit loop adapter format; passed (0) | 0.202751 | none |
| `20261030T035813.251111Z-6de4bea78eab` | edit loop native adapter check; failed (101) | 2.219378 | none |
| `20261030T040023.250285Z-b5b1cd01f39a` | edit loop native adapter compile retry; failed (101) | 2.266940 | none |
| `20261030T040220.920106Z-c9d87a046dd1` | source transport oracle all delivery boundaries fake; passed (0) | 0.334299 | none |
| `20261030T040454.026927Z-9e8ae46dac0d` | edit loop differential adapter tests; failed (100) | 15.859195 | 12.70 |
| `20261030T040732.019120Z-997eb90d2747` | edit loop source identity numeric JSON ordering fixes; failed (100) | 4.863391 | 2.73 |
| `20261030T041133.366035Z-6f614dd9daca` | edit loop preserve source Address objects; failed (100) | 6.028825 | 3.84 |
| `20261030T041431.498045Z-27e810cc2109` | edit loop data URI source normalization; passed (0) | 5.253882 | 3.14 |
| `20261030T041735.996595Z-48b0f91a50a5` | edit loop full JSON source vectors; passed (0) | 1.399813 | 0.22 |
| `20261030T042417.506081Z-b59ac9236159` | edit loop low level lettre source SMTP behavior; failed (101) | 0.482905 | none |
| `20261030T042633.769983Z-b6932c37639d` | edit loop approved lettre hostname feature fetch; failed (101) | 3.068957 | none |
| `20261030T043352.293219Z-e1c989d15db8` | edit loop native protocol and mapping; failed (100) | 5.913350 | 3.66 |
| `20261030T043500.245756Z-179dcb4c2983` | edit loop source fetch features; passed (0) | 8.817516 | 8.76 |
| `20261030T043710.549838Z-7fc54ff8b38b` | edit loop MIME transfer and protocol parity; failed (101) | 17.832351 | none |
| `20261030T043823.532931Z-2041710a297e` | expanded source transport and MIME oracle fake delivery; passed (0) | 0.224082 | none |
| `20261030T043936.884200Z-83dfeda168c4` | edit loop expanded MIME content and SDK tests; failed (100) | 8.544300 | 4.94 |
| `20261030T044124.583502Z-db2f11ab1812` | edit loop full native option error paths; failed (100) | 15.194294 | 4.27 |
| `20261030T044222.332378Z-9b3054be2ab7` | edit loop final direct parity; passed (0) | 11.929563 | 3.46 |
| `20261030T044357.502047Z-c232a9c7b1d4` | edit loop final formatting; passed (0) | 0.244499 | none |
| `20261030T044446.572653Z-46b6ad47d88a` | edit loop cohesive adapter modules format; passed (0) | 0.152649 | none |
| `20261030T044550.906382Z-89ba3c119bc6` | gate email coverage; passed (0) | 28.659943 | 25.63 |
| `20261030T044731.607314Z-2d5b18ce3c67` | edit loop gate preparation format; passed (0) | 0.234160 | none |
| `20261030T044758.580834Z-551c969d212d` | gate workspace Clippy; failed (101) | 30.566724 | none |
| `20261030T045040.579842Z-ba7c356d2cbb` | native TLS builder public test fixtures; passed (0) | 0.757238 | none |
| `20261030T045204.859058Z-eb8ac03ab961` | edit loop final SDK mapping format; passed (0) | 0.237594 | none |
| `20261030T045242.474800Z-6164289a6213` | gate workspace nextest directed tcp exclusion; failed (100) | 340.760361 | 127.00 |
| `20261030T045948.764978Z-edf68a5274eb` | gate trim owned incremental cache below cap; passed (0) | 0.420890 | none |
| `20261030T050053.446043Z-a066e088b58f` | edit loop SMTP error fields and TLS mechanics format; passed (0) | 0.216832 | none |
| `20261030T050147.424770Z-49d004759b70` | edit loop SDK identity and SMTP errors corrected; passed (0) | 7.155461 | 4.62 |
| `20261030T050340.864722Z-e45c43247709` | gate workspace Clippy final; passed (0) | 44.412340 | 44.12 |
| `20261030T050453.832009Z-299798c16dd0` | gate trim final edit cache; passed (0) | 0.115517 | none |
| `20261030T050500.957812Z-ace9c7f50c4d` | gate workspace fmt check; passed (0) | 2.531056 | none |
| `20261030T050553.614908Z-9fef65870bf9` | gate workspace nextest final directed tcp exclusion; failed (100) | 136.717190 | 3.77 |
| `20261030T050849.948030Z-3d82f9cde641` | gate full workspace nextest two workers after fixture deadlines; passed (0) | 167.599652 | 1.06 |
| `20261030T051203.923792Z-761ba37664fe` | gate workspace doctests; passed (0) | 55.898240 | 4.33 |
| `20261030T051430.013505Z-af4cb199e5a8` | source MIME alternative path oracle; passed (0) | 0.274184 | none |
| `20261030T051438.894185Z-4b6b518b7d3a` | final MIME inference format; passed (0) | 0.173504 | none |
| `20261030T051525.702810Z-81f5e44f409d` | gate workspace nextest after MIME path inference; failed (100) | 110.746927 | 3.57 |
| `20261030T051815.311619Z-2bc9d4876cb7` | stream source encoding format; passed (0) | 0.152397 | none |
| `20261030T051835.559826Z-709ee6ebb5eb` | gate Lane B final source MIME regression; passed (0) | 7.017942 | 4.07 |
| `20261030T051848.104304Z-36b5957d70db` | gate workspace tests final source MIME; passed (0) | 70.841938 | 2.99 |
| `20261030T052005.208474Z-6037d1cf123d` | gate workspace Clippy final source MIME; passed (0) | 2.972973 | 2.80 |
| `20261030T052015.406034Z-d8bf97283661` | gate workspace fmt final source MIME; passed (0) | 3.301715 | none |
| `20261030T052025.235104Z-89ea316f0d00` | gate workspace doctests final source MIME; passed (0) | 48.958657 | 1.30 |
| `20261030T052125.595828Z-f75aff678018` | gate workspace Rustdoc final source MIME; passed (0) | 65.557496 | 65.00 |
| `20261030T052236.578904Z-0c476f371f56` | gate CSRF coverage final; passed (0) | 3.994092 | 1.78 |
| `20261030T052245.741068Z-983898e6ab87` | gate email coverage final; passed (0) | 5.530232 | 3.28 |
| `20261030T052258.058287Z-1e242839de62` | gate normal dependency graphs final; passed (0) | 0.549442 | none |
| `20261030T052408.515482Z-e34521691773` | source full encoding names and stream preference oracle; passed (0) | 1.747608 | none |
| `20261030T052450.920475Z-a56d9c20d907` | encoding alias normalization format; passed (0) | 0.284953 | none |
| `20261030T052518.594010Z-323d5a96c8c8` | gate encoding forms Lane B; passed (0) | 7.652599 | 4.74 |
| `20261030T052545.578448Z-030b6649a8b0` | gate encoding forms workspace tests; passed (0) | 222.874997 | 7.35 |
| `20261030T052939.803167Z-fff4d3c8626a` | gate encoding forms workspace Clippy; passed (0) | 3.754705 | 3.57 |
| `20261030T052956.983699Z-7fb953981edc` | gate encoding forms workspace fmt; passed (0) | 3.432821 | none |
| `20261030T053010.481201Z-b68351a7fff3` | gate encoding forms workspace doctests; passed (0) | 134.862950 | 1.22 |
| `20261030T053238.601857Z-c3c17bda24a1` | gate encoding forms workspace Rustdoc; passed (0) | 4.503018 | 4.43 |
| `20261030T053255.768349Z-d96153a45c4f` | gate encoding forms email coverage; passed (0) | 6.373298 | 3.31 |
| `20261030T053555.045952Z-3277b1216542` | final scope lock KB and 12 GiB resource audit; passed (0) | 1.576409 | none |

All records above match this cwd and the slice9b M1 prefix; no shared global count delta is attributed to this lane. Command wall time and observed Cargo-stage subtotal are separate overlapping measures; none means no observed Finished-stage timing. One formatting invocation used an invalid recorder kind before Cargo ran; it produced no record and is excluded from the recorded-command count.
