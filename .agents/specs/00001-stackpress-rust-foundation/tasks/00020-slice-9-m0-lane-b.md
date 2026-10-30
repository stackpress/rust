# Task 00020: Slice 9 M0, Lane B receipt

M0 implementation notes for [the parent task](00020-slice-9-session-csrf-email.md), on `trial/3-slice9-csrf-email`, base `97edcb0d109bb2af03b04ece2d8e98b4cbfcc857`. Coordinator verification is pending. Only M0 was released by the coordinator's 2026-10-30 brief; later milestones and refactoring rounds remain gated by the [accepted milestone plan](../slice-9-design-milestones.md).

## Task summary

Prove real CSRF generation/clear/variable-string validation, per-call Send email transport ownership, source option/default mapping, and supervised execution of non-awaited hooks. Acceptance criteria: none; these are library mechanics, not visual or live-provider acceptance.

## Implementation steps

1. Read contributor/context/workflow instructions, G42–G44, the accepted design and original CSRF/email/session-composite source.
2. Recorder-wrap both fresh-effort cleans before the first build; retain one warm `target/slice9b`, maximum 8 GiB.
3. Add only the two owned crates and root workspace members; resolve approved lettre minimally without replacing existing external lock tuples.
4. Prove the M0 contracts with fake transports, source-derived fixtures, compile-fail doctests and externally bounded concurrency tests.
5. Run the milestone gate, record evidence below and stop for independent coordinator verification.

## Implementation notes

The owned implementation is [framework-csrf](../../../../crates/framework/framework-csrf/src/lib.rs) and [framework-email](../../../../crates/framework/framework-email/src/lib.rs). Root changes are workspace membership and additive dependency resolution only. Root facade/default features, session/API/schema crates, upstream sources and historical Agent Files were not edited.

| Contract | Implementation and observable proof |
| --- | --- |
| CSRF randomness/projection | `SystemRandom` supplies 32 bytes, lowercase hex produces 64 characters; outgoing cookie and `{name,token}` form projection are staged in source order. Two independent outputs differ; configured cookie scope survives replacement. |
| CSRF valid | Incoming scalar cookie and aggregate input must be nonempty strings. UTF-8 byte length precedes subtle equality. Arbitrary one-character, whitespace, Unicode and long strings work. Missing/list/object/array/null/bool/number, empty, unequal-byte-length and equal-length mismatch cases fail with exact 419/Page Expired text. Query aggregate input is eligible. |
| Generate-before-valid / clear | Generating an outgoing token does not replace the incoming cookie used by valid. Clear removes aggregate input, outgoing cookie and response projection; incoming cookie and unrelated fields survive. Existing cookie deletion semantics are reused. |
| Email per-call Send | `Service::send` calls `TransportFactory::create` on every invocation and owns one `Transport: Send` through await. Two concurrent sends reach separate explicit barriers with different IDs. A Cell-backed fake proves Sync is unnecessary; spawned callers prove the whole future is Send. Completion/cancellation destroys the handles. Factory/send errors propagate without retry. |
| Source config/options/info | Object/string union preserves generic, SMTP, pool, JSON, stream, sendmail, SES and connection-string fields without inserting provider defaults. Missing/null/false/zero/empty-string sections are inactive; empty object is active. Original handler fixtures prove exact factory/send arguments and unfiltered provider-info results in 13 cases. This is the M0 boundary proof, not a claim that all native adapters exist. |
| Non-awaited hooks | `TaskSupervisor` schedules owned Send/static futures on an explicit Tokio Handle. Parent completion does not await hook completion. Errors and panic diagnostics remain joinable; shutdown aborts/joins pending tasks. Owner drop requests abort, with explicit destruction-barrier proof. No borrowed parent state or detached JoinHandle is allowed. |
| Negative compile contracts | Rustdoc compile-fail cases reject an Rc-backed transport, an Rc retained across send await and a hook borrowing parent state. Positive execution tests exercise corresponding valid forms. |

### Source provenance and oracle

Read-only source root: `/Users/cblanquera/server/projects/stackpress/stackpress/packages`. No upstream source was copied into the KB or modified.

| File under source root | SHA-256 |
| --- | --- |
| stackpress-csrf/src/plugin.ts | `55f589f6d9d9c63d37e751ca1e550b932c4b893de3f65f263a2590286e5792e2` |
| stackpress-email/src/events.ts | `4061c0714fc3fff61e65cd05d916332629bcede7f39df0fd55aac8cf7dddc65d` |
| stackpress-email/src/plugin.ts | `4ce2b3cc8a1ad5348309415ab715d278cb1d4882c1b9cf956cb5ace527410759` |
| stackpress-session/src/plugin.ts | `aade8d295a5910f7ccc4f9ac56a7c7a6ac7a8da44bd8b544c8259859bbc4a84f` |

[The no-delivery oracle](../../../../crates/framework/framework-email/tests/source-options.cjs) removes only imports/export and the three TypeScript generic annotations, then executes the actual email handler with inert createTransport/sendMail callbacks. [The fixture](../../../../crates/framework/framework-email/tests/fixtures/source-options.json) records the source hash, config, options, invocation trace and provider result. Rust replays every case against the native factory boundary. It does not exercise NodeMailer delivery, SMTP protocol defaults, MIME generation or provider acceptance.

### Rust mechanics adaptations and cross-lane assumptions

- Existing Rust Request/Response cookie/data methods and response mutation are fallible. CSRF propagates those errors; typed PublicError represents the source string error plus 419 status. Invalid cookie names retain the existing lib rejection rather than being renamed. No new cookie policy is imposed.
- Rust strings use UTF-8. Nonempty checks plus byte-length/equality reproduce the source string check followed by Buffer conversion for representable Rust strings. Lone JavaScript UTF-16 surrogates have no direct Rust String representation.
- Typed source configuration replaces the TypeScript union. Truthy values outside its declared object/string union produce a decoding error. Native callback/provider handles belong in the factory rather than serializable JSON. Object extension fields remain intact; native stream/attachment/callback adapter types are still required by M1–M3.
- Rust futures need polling to execute. Owned Send/static tasks, explicit runtime handles and lifecycle joining replace source non-awaited promises. Explicit shutdown is the reap barrier; owner drop alone requests abort. No remote rollback/exactly-once guarantee is introduced.
- Lane A must compose auth → email → session, retain the exact `email-email-send`/`auth-phone-verify` hook names and submit fresh owned resolve payloads. The runtime owner must drain or shut down its task group. No Lane A file was edited and no integrated signup/event proof is claimed at M0.
- CSRF selected-plugin registration with defaults, email plugin selection/Listen truthiness and body/non-200 event skip rules remain M1/M2 integration work. All source transport adapters remain required by the accepted slice; M0 does not mark them deferred or complete. No global route ownership, guard, retry/outbox, recipient policy or hardening was added.

## Verification process

All Cargo/proof/clean invocations use `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label 'slice9b ...' --kind ... --scope ... --cache ... -- <command>`. The outer concurrency watchdog is recorded once; nested Cargo is not double-recorded.

Environment: `CARGO_TARGET_DIR=/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice9b/target/slice9b`, `RUSTC_WRAPPER=sccache`, `SCCACHE_CACHE_SIZE=20G`, `CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0`. Edit-loop runs leave CARGO_INCREMENTAL unset; gate/cleans/coverage use `CARGO_INCREMENTAL=0`. The initial commands through workspace nextest used the equivalent relative target path; subsequent commands use this absolute path to prevent child-cwd resolution. One canonical target is retained warm after the initial cleans; no stale target was consulted.

[The watchdog](../../../../crates/framework/framework-email/tests/run-proof.py) starts a separate process group, enforces an absolute deadline and kills/reaps on timeout or interruption. Focused tests use 240 seconds; the workspace test gate uses 1800 seconds. Ordering proofs use channels, never sleeps. The coordinator expressly excludes `tcp_watchdog`, which binds a local port and remains coordinator-owned.

### Gate results

| Gate command (after the recorder prefix) | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `python3 crates/framework/framework-email/tests/run-proof.py --timeout 1800 -- cargo nextest run --workspace --all-targets --locked --offline --no-fail-fast -E 'not test(tcp_watchdog)'` | **773 passed, 0 failed, 82 skipped**, across 219 binaries. Includes all **17 Lane B tests** (6 CSRF, 11 email). The skip count includes the directed tcp_watchdog exclusion plus existing ignored/environment-gated cases. |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed with warnings denied. |
| `python3 crates/framework/framework-email/tests/run-proof.py --timeout 900 -- cargo test --workspace --doc --locked --offline` | **946 passed, 0 failed, 0 ignored**. Lane B contributes one running CSRF example and three negative compile contracts. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline` | Passed with warnings denied. |
| `cargo tree -p stackpress -p stackpress-framework-csrf -p stackpress-framework-email -e normal --depth 1 --locked --offline` | Passed. Root default normal graph remains stackpress-lib only; neither new crate is a root default dependency. |

Coverage commands each use the same 600-second outer watchdog and gate environment:

```sh
cargo llvm-cov -p stackpress-framework-csrf --locked --offline --json --output-path /private/tmp/slice9b-csrf-coverage.json --fail-under-lines 70
cargo llvm-cov -p stackpress-framework-email --locked --offline --json --output-path /private/tmp/slice9b-email-coverage.json --fail-under-lines 70
```

| Crate / measured files | Executable lines | Result |
| --- | --- | --- |
| framework-csrf, src/lib.rs | **52/54** | **96.30%**, 6 tests passed. |
| framework-email, src/config.rs + service.rs + tasks.rs | **34/34** (7/7 + 8/8 + 19/19) | **100%**, 11 tests passed. |

Tool: cargo-llvm-cov 0.8.6; Cargo/rustc 1.96.0; native aarch64-apple-darwin; both crates' default features only. Reports include all existing first-party implementation files in each selected crate; no custom exclusions were added. Raw JSON is at the two `/private/tmp` paths above (ephemeral). SHA-256: CSRF `55351cdf84d68b984c4b4744a53fe087bc964ee9c8a1e4a7888f012edf50140d`; email `c8d4a0654576ebae20414857114ee1376d216d98e94481011405b10fd34f752a`.

Remaining branch limits: OS entropy failure and every possible existing lib limit/finalized-response propagation branch were not injected. No SMTP/MIME/pool/JSON/stream/sendmail/SES native-adapter coverage is claimed before those adapters exist. Browser/WASM/installed targets, source event registration/body skip integration and signup cross-crate composition are not M0 proofs. No behavior was changed during the gate, so passing checks were not repeated solely to collect more timing records.

### Dependency and resource evidence

All **616 pre-existing external package tuples** (name/version/source/checksum) remain unchanged against HEAD. Five external packages were added: lettre 0.11.23, email-encoding 0.4.2, email_address 0.2.9, nom 8.0.0 and quoted_printable 0.5.2. Existing nom references are version-qualified for the second major version; the selected lettre Rustls feature adds the existing log dependency edge. No pre-existing external version/source/checksum changed, and no `cargo update` or `cargo generate-lockfile` was run.

Lettre has defaults disabled, with only builder/smtp-transport/tokio1-rustls-tls selected for the M0 native Send builder proof. Existing serde/serde_json/Tokio and exact ring 0.17.14/subtle 2.6.1 are reused. Approved dependencies were fetched after the first offline attempt found lettre absent. No tool installation, live SMTP/SES/sendmail, listener, commit, push or GitHub write occurred.

Largest sampled canonical target size and final retained size: **4,162,780 KiB (3.970 GiB)**, under the 8 GiB cap. Continuous peak sampling was not performed. Workspace subprocesses resolved the original relative target against child working directories, creating four additional task targets: `crates/server/server/target/slice9b`, `crates/lib/target/slice9b`, `crates/sql/sql-sqlite/target/slice9b`, `examples/generated-workspace/app/target/slice9b`. An earlier simultaneous allocated-size sample across those four plus the canonical target was 3,988,844 KiB (3.804 GiB).

All four child targets were proven newly created after this gate began, then removed by recorder-wrapped `cargo clean --target-dir <absolute-child-path>` calls. Their final Cargo-reported removed logical sizes were 107.1, 102.6, 209.9 and 241.6 MiB respectively; these are not the same metric as allocated-size samples. The outer cleanup script was `/private/tmp/slice9b-clean-relative-targets.py`, whose path/timestamp guards confined the four operations. Final inventory finds only canonical `target/slice9b` retained warm. No unrelated target/store was deleted. The shared benchmark store is preserved.

### Benchmark records

All rows are in `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/<id>.json` with matching `.log`. Times are seconds. `none` means no observed Cargo Finished-stage timing, not zero build time.

| Record ID | Invocation / result | Command seconds | Cargo-stage seconds |
| --- | --- | ---: | ---: |
| `20261030T025324.054288Z-5454fd9783d7` | initial cargo clean; passed | 0.645819 | none |
| `20261030T025407.898241Z-a74bd9fb6cba` | initial llvm-cov clean; passed | 1.366249 | none |
| `20261030T025803.447001Z-890d90ae0613` | edit-loop offline check; failed, lettre absent | 0.506917 | none |
| `20261030T025839.319888Z-2d9eafd678ea` | edit-loop approved lettre fetch/check; passed | 31.326517 | 31.24 |
| `20261030T030029.985418Z-0fbebc574caf` | edit-loop format; passed | 0.215014 | none |
| `20261030T030108.484684Z-5201190cffae` | edit-loop tests; failed, executor type inference | 23.071592 | none |
| `20261030T030156.519556Z-2327364f8909` | explicit lettre executor retry; 16 passed | 5.367916 | 2.34 |
| `20261030T030237.890445Z-02b9d5b6080a` | original email option/default oracle; 13 vectors passed | 0.187508 | none |
| `20261030T030318.418394Z-b7088716ba29` | final scoped format; passed | 0.156901 | none |
| `20261030T030409.320804Z-87c511a39e12` | workspace nextest with directed exclusion; passed | 447.385475 | 210.00 |
| `20261030T030551.877240Z-7eefec4a17af` | workspace fmt check; passed | 3.747796 | none |
| `20261030T030853.102015Z-eb82b3e20523` | normal dependency edges; passed | 0.465929 | none |
| `20261030T031151.881282Z-d7d5ee313054` | workspace Clippy; passed | 90.401442 | 90.00 |
| `20261030T031346.967636Z-06ea7953133b` | workspace doctests; passed | 46.604214 | 3.96 |
| `20261030T031454.194471Z-60626c729e51` | CSRF coverage; passed | 25.118215 | 23.37 |
| `20261030T031550.840493Z-1d2fa559a2d5` | email coverage; passed | 23.469466 | 21.26 |
| `20261030T031639.189036Z-a2b99580db6c` | workspace Rustdoc; passed | 57.421267 | 57.35 |
| `20261030T031841.338494Z-66419323ae87` | clean only four task-created relative child targets; passed | 2.405241 | none |

Attributable counter delta: **+18 attempts, +16 passed, +2 failed; +8 compilation-emitting invocations; +0 declared cold workspace rebuilds**. Sum of recorded command durations: **759.863478 seconds**. Separately, observed Cargo Finished-stage subtotal: **439.52 seconds**. These overlap and must not be added together; neither is session elapsed time or compiler CPU time. Other lanes share the store, so global store-count growth is not attributed to this lane. Both failed attempts are retained: offline resolution was retried with the approved fetch; lettre's builder was corrected to `build::<Tokio1Executor>()`.

### KB validation

Before this receipt, the validator reports 146 existing errors (worktree-relative external-source links) and 27 existing warnings. After receipt creation the diagnostic list is byte-for-byte unchanged: **0 new errors/warnings**. Historical records are outside this lane's write scope and were preserved. Final `git diff --check` passes; no context promotion or parent-task/status edit was made.

## Verification notes

Coordinator-owned; independent rerun and M0 release decision pending. No milestone/task status or accepted context was changed by this receipt. The later three refactoring rounds and dedicated safety round remain at their explicitly gated stage after implementation milestones.

## Acceptance notes

Acceptance criteria: none. No UI, browser target, installed platform or live-provider acceptance is claimed.

## Coordinator verification

Independently reran on the committed tree at gate tier (`sccache`, non-incremental, empty target), recorded as `s9b-verify gate *`. All passed:
- Lock: every original external package tuple kept; 5 new external packages for lettre; none removed.
- fmt; workspace Clippy `-D warnings`; Rustdoc `-D warnings` for both crates.
- `cargo nextest run --workspace --all-targets --locked --no-fail-fast` with no exclusions: 774 passed, including `tcp_watchdog`.
- Workspace doctests: 946, including the three Lane B compile-fail contracts.
- Coverage: framework-csrf 96.30% lines, framework-email 100% lines.

Source review: CSRF generate/clear/valid match `stackpress-csrf/src/plugin.ts` line for line (32 random bytes as hex, cookie then SSR data, nonempty equal-length timing-safe comparison, 419 "Page Expired" with the original message), and add no automatic route guard (G43). Email builds a transport per call as the source does; option/default parity is pinned by 13 fixtures from the original handler with an inert transport. Four empty stray `target/` folders the gate created were removed by the coordinator. Merged into trial/3 on 2026-10-30; M1 released.
