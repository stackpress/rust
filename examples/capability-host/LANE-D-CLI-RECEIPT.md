# Lane D: application executable and cross-surface proof

Date: 2026-10-24. Branch: `trial/3-lane-d-app`, based on coordinator fast-forward `3ab0a80`.
This completes the application CLI brief using the merged `stackpress_cli::main_with` implementation.
Stopped for coordinator review; no commit, push, listener or socket was performed.

## Implementation and scope

- Added a real application-owned binary target named `stackpress`, with a 34-line `src/main.rs`.
  Its named `async fn build_application(ConfigLayers)` installs supplied layers, selects profiles,
  conditionally selects the existing experimental plugin, then consumes startup. The named main
  runs `main_with(build_application)` on an explicit current-thread Tokio runtime.
- Customization stays independently selected by existing tests. No new customization feature,
  runtime plugin loading, alternate operation implementation or placeholder binary was introduced.
- Added the `stackpress-cli` path dependency with its normal default TTY feature, plus explicit
  Tokio `macros`/`rt` runtime dependencies. The dev dependency retains `rt-multi-thread`. The only
  Cargo.lock change is this example's `stackpress-cli` dependency edge; no external crate or
  dependency version was added or changed.
- Set the executable target's `doc = false`: the executable and root facade share the Rust name
  `stackpress`, and generating both Rustdoc targets otherwise overwrites the same HTML directory.
  Source Rustdoc remains on every function; tests and Clippy compile the executable. Public library
  docs are generated normally. After discovering the collision, ran `cargo clean --doc`, regenerated
  docs and checked that `target/doc/stackpress/index.html` describes the facade again.
- Added `tests/cli.rs` and `tests/support/cli_watchdog.rs`. Updated only crate-overview and one
  profiles comment to remove the obsolete CLI deferral. No existing production behavior changed.
- All edits are under `examples/capability-host/`, plus the permitted lockfile change. No root
  manifest, framework, tools, facade or KB file changed. No other trial branch/worktree was read.

Run the application explicitly:

```sh
cargo run -p stackpress-example-capability-host --bin stackpress -- \
  emit profile-search --id abc123
```

## End-to-end executable evidence

Every CLI invocation uses `std::process::Command` with the actual
`env!("CARGO_BIN_EXE_stackpress")` executable. Arguments are native OS values, never joined into
a shell command. Tests observe process exit status and separate stdout/stderr bytes.

| Case | Observable result |
| --- | --- |
| `emit profile-search --id abc123` | Exits **0** with exactly one JSON object and one trailing newline on stdout, empty stderr, code 200, Ada Lovelace row and total one. Its complete decoded envelope equals both `Server::call` and a real buffered HTTP GET `/profiles/abc123` on the compiled profiles operation. HTTP explicitly matches `HttpOutput::Response` and native status 200. |
| `emit unknown-event` | Exits **1** with one public JSON 404/error envelope, no results, and separate human stderr. |
| Usage errors | Missing command, missing event and an unknown pre-emit option each exit **2**, leave stdout empty and write diagnostics to stderr. |
| `-b <file> emit profile-search --id abc` | A real JSON file containing profiles limit one changes default total three to total one. A filename containing spaces remains one argument. |
| `emit profile-search --id abc -b <file>` | Existing limit-one config and nonexistent paths both remain target arguments: exit 0, default total three, empty stderr. Neither loads operator config nor causes file-open failure. |
| Invalid pre-emit config | Malformed JSON and valid JSON with profiles limit zero both exit **2** with empty stdout; these exercise file parsing and actual application config validation respectively. |
| Compiled experimental selection | Default binary preview invocation exits 1/404. Enabled binary preview exits 0 and matches ordinary search output. Both feature modes launch the actual binary, rather than only inspecting the library builder. |

The tests pin successful data as well as envelope equality, so three matching error replies cannot
satisfy the cross-surface proof. Existing customization, route, lifecycle and feature-exclusion
tests continue passing. **Proof checklist item 5 is now demonstrated through the real app CLI and
buffered HTTP paths invoking the same profile-search source.** No real network support is inferred.

## Watchdog ownership and bounds

- Each command gets a **10-second** external deadline. Exit is polled every 10 ms, independently
  from the child's Tokio/runtime progress. Expiry kills the exact owned child and calls `wait`.
- A child guard also kills/waits on I/O error, capture-limit rejection or unwinding. It is declared
  after its capture fixture so the process is reaped before fixture cleanup runs.
- stdout/stderr go to distinct regular files, avoiding full-pipe deadlocks and preserving machine
  versus human output. Retained reads are bounded to **64 KiB per stream**, with one detection byte;
  oversized evidence is rejected rather than silently truncated. File growth is checked while
  polling; the policy is not a strict disk-write quota between polls.
- stdin is null. No reader worker, detached task, process-global argv change or inherited input
  source is introduced. Temporary directories are exclusively created under ignored target output
  with PID/counter names; cleanup removes only the directory that helper created.
- A known synchronously blocked child is explicitly selected under a **250-ms** watchdog self-test
  and returns a killed/reaped status. A separate deliberately noisy blocked child exercises the
  capture-limit early return; the guard's cleanup order was inspected in the safety round.
- These two child entrypoints are ignored by ordinary test enumeration and explicitly launched
  by passing parent tests. The fixture directory was empty after verification.

## Baseline and scoped rounds

Before the first build, ran `cargo clean` (17,270 stale files / 1.8 GiB) and
`cargo llvm-cov clean --workspace`. The initial package test allowed the one required lockfile
update; all later test/build/lint/doc commands used `--locked`.

Baseline: **23 example tests passed in each feature mode**, including seven CLI parent tests
and one explicitly invoked ignored watchdog child.

1. **Logic/complexity:** reviewed executable exit classes, exact machine framing, bootstrap
   position, validation and plugin feature selection. Added malformed/invalid config failures and
   a nonexistent post-event bootstrap path, preventing accidental file loading from looking like
   successful default policy. CLI suites reran: **8 passed in each mode**, one helper ignored.
2. **Responsibility/abstraction:** reviewed main versus CLI library ownership, shared operation
   source and test-fixture ownership. Kept main's explicit standalone selection as requested; the
   library proof builder and executable selection are checked together by parity/feature tests.
   The watchdog/captures remain test-only, with no generic production runner or new app API.
   No-change round; CLI suites reran: **8 passed each**.
3. **Consistency/Rustdoc/comments:** updated stale CLI comments, improved exit assertion diagnostics,
   and reviewed function/module/state docs and purposeful logical-block comments. CLI suites
   passed **8 each**, app doctests **2 default / 1 experimental**, and scoped Clippy was clean.
   All three new Rust files have function Rustdoc and no lines above 100 columns.
4. **Safety/soundness:** reviewed subprocess custody, kill/reap ordering, bounded reads, output
   overflow, exclusive fixture paths, native args, explicit stdin and owned runtime state. Added
   the excess-output child/parent proof. CLI suites passed **9 each**, with two child helpers
   explicitly launched. Final Clippy caught an unnecessary `drop(stdout)` in that new fixture;
   removed it and reran both CLI suites successfully. No unsafe or new application concurrency
   primitive was added. The only atomic state is test-local fixture-name allocation.

Follow-up: fixed the observed Cargo Rustdoc target collision with `doc = false`, cleaned only
generated docs and regenerated both feature selections warning-free. Verified the facade HTML
contents after regeneration. No runtime or public-behavior decision was required.

## Final checks

Native target: **aarch64-apple-darwin**. rustc **1.96.0** (`ac68faa20`), LLVM **22.1.2**;
cargo-llvm-cov **0.8.6**. Logs and JSON reports live under ignored `target/lane-d-cli-*` paths.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check`; `git diff --check` | Passed. |
| `cargo test -p stackpress-example-capability-host --all-targets --locked` | **25 passed**, zero failed; two child helpers ignored by ordinary enumeration and launched by parents. |
| `cargo test -p stackpress-example-capability-host --all-targets --features experimental --locked` | **25 passed**, same helper policy. |
| `cargo test --workspace --all-targets --locked --features server-http` | **139 passed**, zero failed; 15 existing/new helper entrypoints run through passing watchdog/PTY parents. |
| `cargo test --workspace --doc --locked --features server-http` | **327 passed**: facade 1, CLI 53, app 2, lib 268, server 3. |
| `cargo test -p stackpress-example-capability-host --doc --features experimental --locked` | **1 passed**. |
| `cargo clippy --workspace --all-targets --locked --features server-http -- -D warnings` | Passed warning-free after the fixture fix. |
| `cargo clippy --workspace --all-targets --locked --features server-http,stackpress-example-capability-host/experimental -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http` | Passed warning-free after collision cleanup. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http,stackpress-example-capability-host/experimental` | Passed warning-free. |

After the final fixture correction, both CLI suites and both complete all-target instrumented
package suites were rerun. The subsequent manifest doc setting only affects generated docs.

## Coverage, including the actual executable

Each collection was independently preceded by `cargo llvm-cov clean --workspace`. The corrected
command includes `examples/` production code and excludes only dependencies, other workspace
packages, test support/source and build artifacts. **No example production file is excluded**;
`src/main.rs` appears in both reports with executed lines from the native child processes.
Ordinary doctests run separately without nightly coverage.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-cli-coverage-default.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets \
  --features experimental --locked --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-cli-coverage-experimental.json --fail-under-lines 70
```

| Package scope | Covered / executable lines | Line coverage |
| --- | --- | --- |
| Defaults, library plus native binary | **120 / 120** | **100%** |
| Experimental, library plus native binary | **143 / 145** | **98.62%** |
| Native `src/main.rs`, defaults | **12 / 12** | **100%** |
| Native `src/main.rs`, experimental | **15 / 16** | **93.75%** |

Both package scopes exceed the 70% development floor. The two experimental gaps are defensive
error returns for fixed valid experimental-plugin selection in the library and executable;
no extra failure injection or exclusion was added to manufacture coverage. This is production
line coverage, not branch, all-platform, release or network evidence.

## DX feedback and remaining boundaries

`main_with` accepted the named app callback directly. Bootstrap layering, native argv/sinks,
public envelopes and exit policy worked through the real executable without framework edits.
No missing server/CLI API blocked this task. The only integration rough edge was the genuine
binary/facade Rustdoc name collision, resolved locally with the normal Cargo target setting.

The initial profiles, customization and experimental proofs remain intact. CLI/HTTP/direct-call
proof item 5 is complete within this buffered, native-target scope. Generated models, database,
real networking/streaming, serverless, UI/Tauri, other platforms, publishing and deployment remain
outside this application brief. Ready for coordinator review; stopped.
