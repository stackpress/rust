# Slice 9 refactoring rounds 1–4 Lane B receipt

Status: **Rounds 1–4 complete; stop for coordinator verification**. Started from 87a60ff on trial/3-slice9-csrf-email. This receipt covers only the released CSRF/email rounds. G42/G43 and the accepted lettre capability ruling remain binding. No public contract, event name, option mapping, response behavior, route inventory or dependency changed. The source auth-update/body/email-send skip remains intact pending the owner's answer. Acceptance criteria: none for this code/refactoring gate; coordinator verification and owner acceptance remain separate. The later documentation follow-up is not started.

## Round 1: complexity

Reviewed factory resolution/fallback precedence, per-message capability/metadata checks, event body/status/config skips, CSRF mutation order and validation branches. Consolidated three constructor provider-or-named-error branches into private NativeFactory::provider_transport. It forwards the complete original section and constructs one provider at the existing fallback point. Message-level fallbacks remain separate because they own native readers and their errors. CSRF's short validation flow and the exhaustive transport dispatch did not justify extra predicates.

Consumer/integration duplication: small explicit source-shaped configuration/message literals serve different import and response-oracle checks; merging their setup would obscure those checks. The repeated provider branch belonged in the existing native factory, not a shared crate: it requires email-specific configuration, transport ownership and capability errors. No foreign helper move is proposed.

Baseline and round-close tests: **58/58 passed** each. Original config/native-provider/error vectors, per-call construction/cancellation, exact event skips/order and CSRF source branches remain covered. Round 1 record: 20261030T090455.363646Z-c19e2e2346de.

## Round 2: responsibility and abstraction

Moved sendmail argv/process/exit-result mapping into private native/sendmail.rs and SES command/region/result mapping into private native/ses.rs. The native dispatcher still selects adapters and owns transport overrides, pre-IO capability checks, metadata and full-provider fallback. The moved functions take the existing mutable per-call transport; no new trait, wrapper object, public export or Sync requirement was introduced. Sendmail's leading-dash error, explicit empty args, Bcc handling and exit-code mapping remain in source order. SES's MIME/override/region/send/Message-ID order is unchanged.

Consumer/integration duplication: consumers intentionally exercise ordinary public imports and minimal normal graphs, while integration fixtures use real auth/SQL/SSR with fake external IO. Their repeated message literals and factory selection should stay visible and independent. Adapter implementations belong in feature-owned email modules; moving them or the fake host/SQL harness to a shared crate would couple direct/server-disabled graphs to unrelated test responsibilities. Existing public SmtpBackend/ProcessBackend/SesClient/TransportFactory seams already provide actual reuse.

Round-close **58/58 passed** after removing an unused parent import exposed by extraction. The first extraction run also passed 58 with that warning; final Clippy is warning-free. Final round 2 record: 20261030T091035.340774Z-86345db05f93.

## Round 3: consistency, Rustdoc and comments

Documented the private per-call transport fields and deferred MIME stream's pending/completed/cursor invariants; explained completion clearing and partial-read/EOF bounds beside polling. Added source argument/exit/SES override/region/result comments at their new module boundaries, metadata input/error/entropy documentation, MIME node variant documentation and CSRF raw-section field Rustdoc. Corrected NativeProcessBackend's broad cancellation-cleanup wording: drop requests kill; normal awaited completion reaps. This changes claims, not process behavior.

Consumer/integration duplication: kept application examples independent and avoided copying the large fixture harness into docs. Local docs describe the actual owner and lifetime of each field rather than sharing generic prose. No helper belongs in another crate: response truthiness is tied to rich Body values, while source JSON coercion and compiler capability predicates are email-specific; genericizing them would expand dependency graphs and foreign ownership.

Round-close **58/58 tests**, **8/8 doctests** (including four Send/static/parent-borrow compile-fail cases), selected all-target Clippy -D warnings and Rustdoc -D warnings passed. Records: 20261030T091141.068677Z-509ad2ed0eea and 20261030T091254.660296Z-6d794667dbd8.

## Round 4: Rust safety and soundness

Reviewed Send versus Sync boundaries, boxed-future lifetimes, native-reader one-shot transfer/poison errors, lock release before IO/await, provider fallback before content consumption, supervisor JoinSet ownership/drain/shutdown/drop, reader error timing, MIME/preencoded framing, checked port/timeout/TLS representations and byte/index invariants. Hex Message-ID slices use 32 ASCII hex bytes; percent decoding checks the pair before slicing; data-URI slicing follows the ASCII prefix check; MIME indices come from the same alternative list; deferred offsets advance only by copied bytes. Static JSON expects are versioned checked-in-data invariants exercised by source vectors. No unsafe block or Rust soundness defect was found; production logic needed no further edit. Existing source access/verification choices remain unchanged under G43.

Strengthened the existing original-source stream vector to consume three bytes per read, compare the same original byte oracle and read EOF again. The deferred-error fixture now reads again after its error and confirms EOF with exactly one resolver call. This tests cursor bounds/completed-future reuse without a new implementation path or fabricated policy. Existing finite barriers prove non-Sync Send transports, cancellation destruction, started/unpolled task shutdown, panic/error retention and parent/child isolation.

Consumer/integration duplication: kept ownership probes local to their distinct direct/event/lifecycle boundaries; sharing state across consumers would weaken isolation. No shared-crate helper is warranted: NativeReader requires the server's rich native carrier, DeferredMessage is a private email compiler reader, and TaskSupervisor is already the public email/session composition seam. No raw-pointer, pin projection or custom unsafe wrapper is needed.

Round-close **58/58 passed**; record 20261030T091504.305967Z-ee7e579e71ce. No new Rust-mechanics deviation, hardening, global gate, route ownership or conflict arbitration was added in any round.

## Final gate and evidence

| Final invocation | Result |
| --- | --- |
| cargo fmt --all --check; workspace all-target Clippy -D warnings, six compatible features | Passed. |
| workspace/all-target nextest, default features | 900 passed, 86 skipped, 224 binaries; excludes tcp_watchdog. |
| workspace/all-target nextest, six compatible features | 940 passed, 88 skipped, 224 binaries; excludes tcp_watchdog. |
| workspace doctests, six compatible features | 962 passed, including all four email ownership compile-fail contracts. |
| workspace Rustdoc --no-deps -D warnings, six compatible features | Passed. |
| graphs.py --check | All four default/server feature combinations and minimal SDK feature assertions passed. |
| Each crate separately with --no-default-features | CSRF 6 passed; email 31 passed. |
| consumers/run.py | Direct/event/disabled standalone graph, lock subset, format, Clippy and executable proofs passed, each with a separate absolute target. |

Compatible features: stackpress-framework-csrf/server, stackpress-framework-email/server, stackpress-framework-session/auth-server, stackpress-framework-schema/providers, stackpress-framework-api/sql and stackpress-framework-api/codegen. Renderer --all-features was not used. Selected round suites have a 300-second external process-group kill/reap deadline; final workspace nextest 1800 seconds per run, remaining gate 1800 seconds and coverage 900 seconds per crate.

Coverage: cargo llvm-cov -p <crate> --features server --locked --offline --json --output-path /private/tmp/slice9b-rounds-<name>-coverage.json --fail-under-lines 70. All implementation files in each selected crate are included without custom exclusions; doctests run separately. This exceeds the development floor; no publication gate is claimed.

| Crate/features | Covered/executable lines | Coverage | JSON SHA-256 |
| --- | ---: | ---: | --- |
| framework-csrf/server | 67/69 | 97.10% | `9d06edff36d55ca156dead58ac1c7c41368570554c3faaf5c0b6479cd24f62df` |
| framework-email/server | 2423/2739 | 88.46% | `77326e434c197fed03d79fe93195b6fb40136ffbd0459ec8ea78bc708ac77818` |

Source-page regeneration matches the existing eleven-case fixture byte for byte; SHA-256 f9099a67d8110e242b1d6cadb71a17b3b4b079f10546b61ab5e8c4071cd464b4. Original source and golden fixtures were not edited. Original option/transport/event vectors remain exercised, including provider bindings for all lettre gaps. Fakes replace SMTP/SES/sendmail delivery; the existing /bin/cat test exercises inert native process IO only. No live server/provider or listener was used.

Final audit passed: only assigned crates and this new receipt changed. Root Cargo.lock and all three consumer locks remain **byte-identical** to 87a60ff; all 698 original external tuples remain and none was added. KB diagnostics remain 146 errors/27 warnings, zero delta. Baseline lock SHA-256: `290304ae3ac6b038058ce5c2ed3c412f9f31256ac835bb560a5250bbfa5b2c67`. Source regeneration, scope, whitespace and cap checks passed; historical records were preserved.

All compilation/test round-close and final-gate commands used CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G and CARGO_BUILD_JOBS=2. The effort's warm absolute target is /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b; no fresh-effort clean was repeated. Consumer targets remain consumer-direct, consumer-event and consumer-disabled beneath it. Cargo accepts --locked --offline on builds/checks/tests/docs/trees/runs; cargo fmt has no such flags. Proof scripts are recorded only at the outer boundary. Two recorder-schema validation errors and one rejected incorrect wrapper-path request ran no Cargo and produced no run record; the corrected invocations are recorded.

The coordinator's **20 GiB cap / 19.4 GiB headroom guard** remains in force for these rounds. Largest recorded sample: **15,634,632 KiB (14.910 GiB)**; final sample: **15,634,632 KiB (14.910 GiB)**. Samples are periodic rather than continuous. No incremental or dependency artifact was removed during these rounds; all consumer targets and dependency artifacts are retained.

Remaining limits: native aarch64-apple-darwin, Rust/Cargo 1.96.0; finite in-process auth/SQL/SSR and provider seams do not establish live SMTP/TLS, SES/sendmail service behavior, pool recycling, PostgreSQL/MySQL, browser/WASM or installed Tauri acceptance. No UI/client behavior changed, so no new browser/installed proof is claimed. Keep the owner question about source auth-update/body/email-send behavior open. No session/API/schema/facade/renderer implementation, historical Agent File, commit, push, install, GitHub write or other-agent message was made. Stop after round 4.

## Benchmark records

Attributable rounds delta: **21 recorded attempts**, 20 passed and 1 failed; **10 compilation-emitting** and **2 checking-emitting** commands. Command wall total **681.300331 s**; observed Cargo-stage subtotal **278.58 s**, incomplete and overlapping command time rather than additional time. The sole expected failed record is the unchanged KB validation baseline; final code and scope gates pass. Recorded start-to-end span **1235.988 s**, including review/tool gaps between commands. Package-cache waits are included in command wall time, not inferred as Cargo stage time.

Store: /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks. Each runs/<id>.json and matching log retains exact argv, environment, cache/result and command/Cargo-stage timings. The initial 'baseline source KB target' label ran only the target-size proof; the separate baseline KB record contains its actual diagnostics. Records from other lanes are excluded.

| Record ID | Invocation/result | Command s | Cargo-stage s |
| --- | --- | ---: | ---: |
| `20261030T090413.699340Z-68766501e766` | baseline source KB target; passed (0) | 1.545706 | none |
| `20261030T090420.836090Z-79431b3738c5` | baseline tests; passed (0) | 3.583849 | 0.50 |
| `20261030T090440.951398Z-dde54ab6392a` | R1 format; passed (0) | 0.238745 | none |
| `20261030T090455.363646Z-c19e2e2346de` | R1 verified complexity; passed (0) | 15.872267 | 11.79 |
| `20261030T090836.550184Z-275f07cdbc5c` | R2 format; passed (0) | 0.514178 | none |
| `20261030T090921.735144Z-1d72f75ec135` | R2 verified responsibility; passed (0) | 44.317989 | 29.40 |
| `20261030T090943.558372Z-4cd850120df8` | baseline KB diagnostics; failed (1) | 4.252764 | none |
| `20261030T091035.340774Z-86345db05f93` | R2 final responsibility tests; passed (0) | 17.925791 | 7.69 |
| `20261030T091126.394854Z-85a57bfb701a` | R3 format; passed (0) | 0.351568 | none |
| `20261030T091141.068677Z-509ad2ed0eea` | R3 verified documentation; passed (0) | 33.836182 | 19.85 |
| `20261030T091254.660296Z-6d794667dbd8` | R3 doctests Clippy Rustdoc; passed (0) | 104.007309 | 88.61 |
| `20261030T091456.406157Z-1f6970f2a522` | R4 format; passed (0) | 0.199761 | none |
| `20261030T091504.305967Z-ee7e579e71ce` | R4 verified safety and soundness; passed (0) | 9.939077 | 4.96 |
| `20261030T091524.214449Z-3d6e72efdb3a` | gate size guard before workspace; passed (0) | 2.487809 | none |
| `20261030T091540.801878Z-42bbfd6bb39d` | gate workspace default nextest; passed (0) | 141.487080 | 24.69 |
| `20261030T091814.334789Z-01d7787b0dbb` | gate workspace six-feature nextest; passed (0) | 98.177652 | 12.66 |
| `20261030T092008.493435Z-4e46d4aab2ce` | gate remaining checks consumers; passed (0) | 182.244350 | 67.93 |
| `20261030T092322.394662Z-368581b0e5b8` | gate csrf coverage; passed (0) | 4.370401 | 1.84 |
| `20261030T092351.814022Z-15c49872d5ec` | gate email coverage; passed (0) | 12.209543 | 8.66 |
| `20261030T092423.901850Z-81ba291d0b9b` | gate final size guard; passed (0) | 0.945261 | none |
| `20261030T092446.644943Z-3f6640708e9e` | gate final scope source lock KB audit; passed (0) | 2.793049 | none |

## Coordinator verification

Committed as `b84e871` on top of `87a60ff` (trial/3 unchanged since, so no merge was needed). The lock is byte-identical; the three email consumer locks and both API checkpoint locks use only workspace registry tuples. Reviewed the provider-fallback consolidation and the sendmail/SES module moves: private, with unchanged messages, order and results. Reran at gate tier (`sccache`, non-incremental, empty target), recorded as `s9brounds-verify gate *`. All passed:
- fmt; workspace Clippy `-D warnings` and Rustdoc `-D warnings` with the six features.
- `cargo nextest run --workspace --all-targets --locked --no-fail-fast` with no exclusions: default 901 passed; six features 941 passed. Both include `tcp_watchdog`.
- Workspace doctests with the six features: 962 passed.
- `--no-default-features`: framework-csrf 6 passed, framework-email 31 passed.
- `consumers/graphs.py --check` and `consumers/run.py`.
- Coverage: framework-csrf with `server` 97.10% lines; framework-email with `server` 88.46% lines.

Agreed with the round 2 decision to keep the three consumers independent: they prove distinct dependency graphs. Merged into trial/3 on 2026-10-30. Lane B rounds are complete; the documentation follow-up remains.
