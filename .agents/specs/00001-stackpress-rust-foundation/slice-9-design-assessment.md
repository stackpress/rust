# Slice 9 pinned library assessment and disposable evidence

Status: Revised design only under G43, 2026-10-30 Asia/Manila. Argon2, lettre, totp-rs and qrcode are approved new dependencies; ring, subtle, zeroize, base64, url and reqwest may be reused with pinned minimal features. **Only jsonwebtoken remains a dependency question.** Approval does not replace source algorithms or add policy. No repository manifest/lockfile/build or installation changed.

## Pinned capability and source fit

| Purpose / exact pin | Evidence and G43 mapping | Fit still to prove before production |
| --- | --- | --- |
| JWT `jsonwebtoken =11.1.0`, default-features=false, `rust_crypto` | Recommended Q-JWT. Historical functional corpus plus the new 3-test source-default probe pass. HS256 create with alg-only header, short abc123 seed and optional exp work. Explicit validation adapter is required. | Cross-sign/verify with original jose, full header/claim/type errors, fractional NumericDate and source HS256/384/512 verification. Provider feature pulls RSA/EC/EdDSA despite this symmetric use; no custom JWT provider is proposed. |
| JWT `jsonwebtoken =9.3.1`, default=false | Retained comparison baseline using ring; 5-test corrected baseline passes. | Not recommended over the assessed 11.1.0. Defaults of 60s leeway, required exp and nbf off cannot be transferred as source semantics. |
| `ring =0.17.14`, defaults off where supported | Approved reuse for OS entropy/HMAC/SHA256. Historical HMAC/AEAD/reference-format fixtures pass. | Ring does **not** supply source SHAKE256 or AES-CBC. Historical opaque-reference/AEAD tests do not prove the source codec and are not requirements after G43. Resolve provider fit through coordinator before M0 passes; no home-grown primitives. |
| `argon2 =0.5.3`, default=false, alloc/password-hash/rand as needed | Approved capability; historical Argon2id v19 PHC m=19456,t=2,p=1, salted variation/correct/wrong/malformed tests pass. | Original schema helper uses deterministic SHAKE256 hex, not PHC. Preserve that algorithm; this approval does not select Argon2 for existing credentials/challenges or authorize migration. Avoid a gratuitous default feature/dependency if unused. |
| `subtle =2.6.1`, default=false | Fixed-width equality/every single-byte mismatch and empty/unequal public length corpus passes. | Source CSRF accepts variable-length nonempty UTF-8 strings and checks byte length before timing-safe compare. Exercise that exact adapter. No new constant-time password/link/TOTP contract or timing certification. |
| `lettre =0.11.23`, default=false, builder/smtp-transport/tokio1-rustls-tls baseline | Approved transport foundation. Builder/stub success/failure, Send+Sync SMTP construction and mailbox/subject injection fixtures pass; no SMTP connection was made. | Full source union now: per-call SMTP/pool/JSON/stream/sendmail/SES/string, attachments/content/file/URL/extension options and provider result/error mapping. Enable only needed native features or compiled adapters. No source transport ban or extra wrapper header/egress/auth policy. |
| `lettre =0.11.19`, same baseline features | Historical baseline passes after explicit executor annotation. | Comparison only; recommend assessed 0.11.23. Library mailbox rejection and encoded Subject CRLF are observations, not authorization for new framework validation. |
| `totp-rs =6.0.0`, default=false | Approved. RFC SHA1 timestamp/six-digit/window/repeated-check corpus passes; check can return matched step and does not consume it. | Source default generateSecret(16) is 80 bits; library compliant constructors require 128 bits. Use an applicable raw/noncompliant construction path to preserve source semantics. Preserve base32 handling, six digits/30s/±1 and no replay table. Probe the exact path, not strict library defaults. |
| `qrcode =0.14.1`, default=false, svg in historical probe | Approved encoder; SVG generation passes without image dependency. | Source QR.toDataURL renders PNG/data URL. SVG is probe-only evidence; prove PNG/data-URL adapter and minimal necessary encoding features at M0. Preserve source enrollment secret/result fields; no one-time provisioning replacement. |
| `zeroize =1.9.1`, default=false | Approved reuse; fixture primitive overwrite passes. | Native buffer handling only; do not delete intended JWT/secret/QR page data. No guarantee about registers/copies/third-party allocations. |
| Existing `base64 =0.22.1`, `url =2.5.8`, `reqwest =0.13.4` | Approved reuse, exact versions already resolved in repository lock (which also contains base64 0.21.7/0.23.1 for other dependencies). | Reuse matching workspace features, do not widen defaults. URL parsing is not permission for new SSRF/host policy. Source URL/transport semantics still need vectors. |

Reuse serde 1.0.228/serde_json 1.0.150 as assessed and existing lib/server/SQL/Leptos/Tokio contracts. Rust mechanics require owned/Send futures, typed option unions and compiled factories in place of Node objects/dynamic imports. Those adaptations must retain source behaviour; they do not authorize omitted transport variants. License/MSRV/resolved-feature review remains a release check; probes used Rust/Cargo 1.96.0 on aarch64-apple-darwin, not every supported target.

## New JWT source-default probe

`/private/tmp/slice9-g43/jwt-probe` is a separate disposable workspace pinned to jsonwebtoken 11.1.0 with rust_crypto, serde 1.0.228 and serde_json 1.0.150. It reuses the approved disposable Cargo home offline and a fresh external target. **3 tests passed, 0 failed; 0 doctests.**

The tests cover alg-only header, short source seed, no required exp, wrong-key rejection, HS256/384/512 and arbitrary object claims/empty roles. A source NumericDate adapter checks optional exp/nbf types including fractions: exp<=now rejects and nbf>now rejects, without adding issuer/audience/iat rules. Library validation disables its default temporal/audience checks before the explicit source checks. This matters because library exp boundary/integer decoding differs from jose. Signature verification remains enabled. Full original-jose cross-runtime/error interoperability has not yet been run; M0 must prove it.

| New disposable file | SHA-256 |
| --- | --- |
| jwt-probe/Cargo.toml | `9cf847711af87fe59ba11cbd5c69be68ac0c83157448f7c6014af2e632a5094b` |
| jwt-probe/Cargo.lock | `b6553ddf65f17a125d8bd5db3f73bd50151224782f7cfcf4fb64115a7af06759` |
| jwt-probe/src/lib.rs | `14994b53b782f40e95d4695eea2c70a7ced7bef012891078d3eff8b38b978091` |

## Retained historical probe evidence

The earlier draft's opaque-reference/AEAD, Argon2 replacement, strict TOTP/replay and header-policy recommendations are superseded by G43. Their executed results remain historical capability evidence. They do not become source requirements. All historical manifests/locks/tests/Cargo home/target remain outside the repository under `/private/tmp/slice9-probes`.

`baseline-Cargo.toml`, `baseline-Cargo.lock`, `baseline-lib.rs` retain the corrected 5-test baseline. That root's current Cargo.toml/Cargo.lock/src/lib.rs retain the final 9-test comparison (167 locked packages including the probe). run.py owns a 240s process-group watchdog with TERM/KILL/reap. No listener, SMTP send or remote provider operation was attempted. No third-party source tree was copied into .agents/resources.

| Disposable file | SHA-256 at checkpoint |
| --- | --- |
| current Cargo.toml | `d819491b80484a7fd50619c100a6fbd248ea97e57e84d20a0743ec4490c954ce` |
| current Cargo.lock | `105a4782d9fb2ee8816d8f3ae00cd4b45ab2353794da42309c5f3a8a1101d30e` |
| current src/lib.rs | `d7a6df29b70af339c8a718d0f1544cfcd5f25597d4f22f0e0579ef6b87b70ea1` |
| baseline Cargo.toml | `f655f35e39c0f48a5308ca60381a00b91543a58b02a0c34bcad44a9e68a6f79e` |
| baseline Cargo.lock | `dd46f0485496640da6f1fab31c93973d2a612e20548a4e52188c37dd46bcdea8` |
| baseline test source | `9df8c3bf1eb2a90c87c46f191ec76d491322220b5bdcf70d4f6962117968499a` |

The disposable files may be cleaned externally. Persistent benchmark logs retain commands/output, not an archive of disposable source. The following versioned primary sources explain assessed APIs; their semantics are subordinate to the source-parity adapter, not default framework policy.

## Primary sources and bounded advisory observations

- [JWT 9.3.1 validation](https://docs.rs/jsonwebtoken/9.3.1/jsonwebtoken/struct.Validation.html), [11.1.0 manifest](https://docs.rs/crate/jsonwebtoken/11.1.0/source/Cargo.toml.orig) and [changelog](https://docs.rs/crate/jsonwebtoken/11.1.0/source/CHANGELOG.md): explicit provider/claim defaults; pinned source/probes supplement the docs.
- [Argon2 0.5.3](https://docs.rs/argon2/0.5.3/argon2/), [subtle 2.6.1](https://docs.rs/subtle/2.6.1/subtle/trait.ConstantTimeEq.html), [lettre features](https://docs.rs/crate/lettre/0.11.23/features), [TOTP 6.0.0](https://docs.rs/totp-rs/6.0.0/totp_rs/) and [qrcode 0.14.1](https://docs.rs/crate/qrcode/0.14.1): approved capabilities, not proof of all source adapters.
- [Ring RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009.html): patched at >=0.17.12; assessed pin 0.17.14. This is not an assertion of no other advisories.
- [Lettre RUSTSEC-2026-0141](https://rustsec.org/advisories/RUSTSEC-2026-0141.html): Boring TLS hostname-verification issue patched at >=0.11.22; the tested Rustls baseline is unaffected by that advisory. Assessed pin is 0.11.23; configured transport/TLS variants need feature-specific evidence, not a new source restriction.
- [Rustls-webpki RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382.html): patched at 0.103.16 in that line; historical disposable lock resolves 0.103.16. Recheck the eventual production lock, including JWT transitives.

No cargo-audit installation, full supply-chain audit, provider delivery or security certification is claimed. Approved crates still require appropriate feature/target review. Previously fetched baseline versions are not automatically selected for production.

## Cargo receipts and attribution

All eight attempts used the required recorder from this worktree: `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label 'slice9 design probe ...' --kind <other|test> --scope other --cache <cold|warm> -- python3 /private/tmp/slice9-probes/run.py cargo ...`.

Historical manifest path was /private/tmp/slice9-probes/Cargo.toml; test runs used --locked --offline. New G43 run used `cargo test --manifest-path /private/tmp/slice9-g43/jwt-probe/Cargo.toml --offline`, locking cached packages in the disposable directory only. No download occurred in this revision. Isolated CARGO_HOME, fresh external CARGO_TARGET_DIR, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_BUILD_JOBS=2; fetches needed no compiler wrapper. No repository target was inspected or reused.

| Benchmark record ID | Command/result | Command seconds | Observed Cargo-stage seconds |
| --- | --- | ---: | ---: |
| `20261029T164345.960151Z-dc77e8418152` | Approved baseline fetch, passed | 6.386734 | none |
| `20261029T164410.567808Z-e2573b524859` | Baseline test compile, failed: lettre build needs explicit executor type | 33.642426 | none |
| `20261029T164523.605256Z-bdecdf98e51f` | Corrected baseline: 5 passed, 0 failed; 0 doctests | 3.081074 | 1.45 |
| `20261029T164841.644984Z-17920173aa51` | Approved current/TOTP/QR fetch, passed | 2.921973 | none |
| `20261029T164937.124806Z-772f1a32328d` | Current test compile, failed: QR svg feature must be explicit | 17.514255 | none |
| `20261029T165133.389139Z-5d10a8fb5b4f` | Corrected current: 8 passed, 0 failed; 0 doctests | 3.963202 | 2.18 |
| `20261029T171525.890498Z-e05e10c71bc0` | Final token-format + candidate corpus: 9 passed, 0 failed; 0 doctests | 4.094904 | 2.35 |

| `20261030T014314.244033Z-629ffc4bad55` | G43 source JWT defaults/temporal adapter: 3 passed, 0 failed; 0 doctests | 20.907426 | 20.14 |

This revision adds **1 passed test attempt**. Cumulative evidence: **8 completed attempts: 6 passed (2 fetches + 4 test runs), 2 retained compile failures**. Historical command sum 71.604568s plus new 20.907426s = 92.511994s; observed Cargo Finished-stage subtotal 26.12s (historical 5.98s + new 20.14s). Failed builds lack a Finished subtotal. Command time and Cargo subtotal overlap; neither sum is session elapsed time.

Observed historical target size was 415,500,951 bytes; new target size 150,729,438 bytes. Neither is a continuous peak measurement. Logs/JSON are `<shared-store>/runs/<record-id>.log` and `.json`; failed attempts/fixes affected disposable files only. No further repeat solely to rerun these primitive fixtures is needed. Source interoperability, storage, options, concurrency and browser/provider proofs belong to released milestones.
