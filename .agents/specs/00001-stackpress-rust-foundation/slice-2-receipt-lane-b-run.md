# Lane B: CLI execution integration receipt

Spec file of [00001 Stackpress Rust foundation](index.md), recording lane B CLI execution for [task 00005](tasks/00005-slice-2-server-and-cli.md). Load for `stackpress-cli` `run`/`main_with`, stdout/stderr and exit-code policy, output authority, rounds 1–4, and verification. Moved losslessly from `tools/cli/LANE-B-RUN-RECEIPT.md` (lane branch commit `6e820d1`, merged in `fc14274`).

Date: 2026-10-24. Branch: `trial/3-lane-b-cli`. Base: `8309047`.
Scope: `tools/cli/**` only. No commits, pushes, listeners, dependencies, manifest changes or
lockfile changes. No missing server API blocked this implementation.

## Implemented behavior

- `CliSinks` exclusively borrows caller stdout/stderr as `Write + Send`; no handles enter services
  or invocation metadata. `CliContext` carries verbose, force, config path and brand separately
  from untrusted `Request.data`, including nested delegation.
- `run` consumes the existing parsed Terminal without a second argv parse, constructs the
  `terminal/arguments` request, and enters `Server::handle_with_options` exactly once. Its
  `CliRun` retains the actual host report and a final execution exit code of 0 or 1.
- An adapter-owned `OutputGuard` lives outside the replaceable Response. Open output is converted
  with `Response::try_to_json` under its own limits before one JSON/newline delivery attempt.
  Stopped output produces no stdout. Foreign Sent/Attempted/Failed output is unavailable and
  becomes an Output failure. There is no serialization fallback or lifecycle/output retry.
- `write_all` may continue after a short write within that one attempt. Any BrokenPipe or flush
  failure ends the attempt; accepted prefixes are never resent. Output failures retain their
  original typed causes and never re-enter error/response hooks.
- Default failure diagnostics are concise and sanitized. Explicit operator `-v` selects the
  developer report/source chains, including errors directly applied to a Response. Machine
  stdout never passes through branding/color formatting. A failed stderr delivery is terminal,
  even if stdout already succeeded.
- `main_with` acquires native argv/stdout/stderr once. Its reusable `main_with_args` seam accepts
  supplied argv/sinks and uses the same executable pipeline. Both consume a `FnOnce` application
  builder only after parsing one explicitly selected JSON file into ConfigLayers.
- Bootstrap reads use `File::take(default_max_bytes + 1)`: exactly 1 MiB is accepted; one extra
  byte is rejected before JSON/build. Paths are opened literally, without extension search,
  executable imports or dynamic plugin loading. Applications install the supplied layers on
  their own ServerBuilder before building.
- Exit policy: final public 2xx/3xx success is 0, including deliberate error recovery and empty
  Abort normalized to 204. Application/unknown/terminal/output failures are 1. Usage/config/build
  failures are 2. Unrecovered typed PromptError::Aborted becomes 130 only at the executable
  boundary; successful recovery wins, and output failure remains 1. Text/code 130 alone does
  not imply cancellation. Source inspection is finite for cyclic/deep application errors.

## Files changed

| File | Purpose |
| --- | --- |
| `src/args.rs` | Consume checked Terminal parts without cloning/reparsing; refresh verbose docs. |
| `src/lib.rs` | Re-export the execution/entry API; replace the stale integration deferral. |
| `src/execution.rs` | CliSinks, CliContext, CliRun, lifecycle execution and guarded output. |
| `src/entry.rs` | Native/supplied executable entry, bounded config loading and exit policy. |
| `src/diagnostics.rs` | Human sink delivery, bounded developer causes and typed cancellation. |
| `tests/run_contracts.rs` | Real ServerBuilder lifecycle/output/exit/safety and native-entry proofs. |
| `tests/entry_contracts.rs` | Real typed bootstrap layering, exact paths, bounds and usage proofs. |
| `LANE-B-RUN-RECEIPT.md` | This coordinator review receipt. |

All file names in this table are relative to `tools/cli/`.

## Baseline and review rounds

Before the first build, `cargo clean` removed 21,406 files (1.7 GiB), followed by
`cargo llvm-cov clean --workspace`. The initial working integration passed the new watchdog
tests with/without TTY, all CLI tests, 53 doctests and Clippy.

### Round 1: logic and complexity

Two material regressions were first demonstrated with failing tests, then fixed:

1. Executable cancellation classification traversed retained error sources even after successful
   public recovery. A recovered source whose `source()` panics proved the unnecessary traversal.
   Success now short-circuits before classification, and Output failure takes precedence over
   typed cancellation.
2. A valid Terminal with an eight-byte argv budget failed request construction because the
   Request builder's default informational URL spent that same input budget. The CLI supplies
   the one-byte synthetic path `/`, preserving the parsed target's own limits.

Additional cases cover empty public error presence at code 200, directly applied error developer
details, target `--verbose` failing to enable diagnostics, stderr failure after stdout success,
and cancellation combined with stdout failure. The new run/entry tests passed again with both
feature selections after the fixes.

### Round 2: responsibility and abstraction

Extracted the executable-only `run_exit` policy from startup composition and removed an unneeded
fixture wrapper. Startup/config I/O stays in entry; lifecycle/output authority stays in execution;
human disclosure and typed source inspection stay in diagnostics. No new trait, dynamic callback
layer, global state or native sink service was needed. Ownership/Send bounds match the approved
application builder and borrowed-writer seams. New tests passed with/without TTY; 53 doctests passed.

### Round 3: consistency, Rustdoc and comments

Added compiling examples for all new public entry points and CliContext access, documented
error/exit/output/cancellation/ownership boundaries, and refreshed the crate's integration docs.
Every function/helper has Rustdoc. Purposeful comments explain logical phases, branch precedence,
async ownership, source bounds, serialization and test setup/action/assertion intent. Reviewed
the new tests again after the safety round and clarified scenario comments. Final fmt, tests,
doctests, Clippy, Rustdoc and source-width scans pass.

### Round 4: dedicated safety review

No production safety fix was needed after rounds 1–3. Added observable safety proofs:

- Cancel a pending run by polling then dropping its owning future, with no sleep interleaving.
- Propagate handler and writer panics inside owned tasks; a panicking writer is entered once.
  A later independent invocation succeeds, and weak host ownership confirms no hidden retention.
- Traverse an application-defined cyclic source on startup with and without verbose formatting;
  classification/formatting terminate under the watchdog and remain exit 2.
- Spawn the actual native `main_with` future, proving native handles and composition remain Send.

Reviewed preflight-before-write, typed causes, explicit finite config reads, borrowed handles,
response replacement, one-attempt output, diagnostic disclosure and feature isolation. No unsafe,
framework detached reader/task, new runtime, listener or global mutation was introduced. Relevant
run/entry tests passed with/without TTY; Clippy passed. Full final checks below followed the edits.

## Final verification

Host: `aarch64-apple-darwin`; Rust/Cargo 1.96.0; LLVM 22.1.2; cargo-llvm-cov 0.8.6.
Every command below exited 0. Logs are ignored artifacts in `target/lane-b-run-checks/`.

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Clean. |
| `git diff --check` | Clean. |
| `cargo test -p stackpress-cli --all-targets --locked` | 17 passed; 0 failed; 8 child helpers ignored. |
| `cargo test -p stackpress-cli --all-targets --no-default-features --locked` | 14 passed; 0 failed; 6 child helpers ignored. |
| `cargo test --workspace --all-targets --locked` | 92 passed; 0 failed; 13 helpers ignored. |
| `cargo test -p stackpress-cli --doc --locked` | 53 passed. |
| `cargo test -p stackpress-cli --doc --no-default-features --locked` | 48 passed. |
| `cargo test --workspace --doc --locked` | 323 passed. |
| `cargo clippy -p stackpress-cli --all-targets --locked -- -D warnings` | Clean. |
| `cargo clippy -p stackpress-cli --all-targets --no-default-features --locked -- -D warnings` | Clean. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Clean. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-deps --locked` | Clean. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-deps --no-default-features --locked` | Clean. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Clean. |

Ignored helpers are explicitly selected by their passing parent tests. New run scenarios use
individual two-second async deadlines; entry scenarios use a two-second aggregate deadline.
Their independent ten-second process watchdogs kill and reap even a synchronously blocked child.
The native argv rejection wrapper also has both deadlines. Existing PTY
and prompt watchdogs remain passing. Test-harness counts omit assertions executed inside these
explicitly invoked children.

### Fresh executable line coverage

Ran `cargo llvm-cov clean --workspace` separately before each measurement:

```sh
cargo llvm-cov -p stackpress-cli --all-targets --locked \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-run-tty.json --fail-under-lines 70
cargo llvm-cov -p stackpress-cli --all-targets --locked --no-default-features \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-run-no-tty.json --fail-under-lines 70
```

| Scope | Covered/executable lines | Coverage |
| --- | --- | --- |
| stackpress-cli default TTY | 1,232 / 1,278 | 96.40% |
| stackpress-cli without default features | 1,061 / 1,101 | 96.37% |
| New execution module, either selection | 92 / 92 | 100% |
| New diagnostics module, either selection | 32 / 32 | 100% |
| New entry module, either selection | 90 / 97 | 92.78% |

Only test/fixture support paths are excluded; no production exclusion was added. Dependencies and
stable-toolchain doctests are outside this production line measurement. The seven uncovered entry
lines are the outer run-error mapping after an already parsed Terminal/built host: ordinary host
failures return reports and exercise the normal mapping. No fake host/stub or private counter
mutation was introduced to force that exceptional branch. Other uncovered lines remain in the
accepted parser/prompt/TTY implementation; the existing contract and PTY tests remain enabled.
These results exceed the 70% development gate and do not claim branch or all-platform coverage.

### Source width and comment audit

Scanned every `tools/cli/**/*.rs` line with Python `len(line) > 100`: zero violations.
Counts below include standalone `//` explanation lines, excluding Rustdoc/module docs. Code-line
denominators are nonblank, noncomment source lines; they are not executable coverage counts.

| Source file | Inline comments / code lines |
| --- | --- |
| `src/args.rs` | 39 / 209 |
| `src/control.rs` | 23 / 172 |
| `src/diagnostics.rs` | 9 / 41 |
| `src/entry.rs` | 18 / 127 |
| `src/execution.rs` | 16 / 121 |
| `src/lib.rs` | 0 / 13; declarations only |
| `src/prompt/driver.rs` | 58 / 253 |
| `src/prompt/mod.rs` | 0 / 10; declarations only |
| `src/prompt/options.rs` | 26 / 201 |
| `src/prompt/render.rs` | 39 / 185 |
| `src/prompt/state.rs` | 36 / 200 |
| `src/tty.rs` | 47 / 205 |
| New `tests/run_contracts.rs` | 57 / 720 |
| New `tests/entry_contracts.rs` | 26 / 251 |

## API rough edges and evidence limits

- `main_with_args` is an additional public composition/test seam. It excludes the executable name,
  shares native policy, and supports a named async function or an owning FnOnce builder without
  boxing. Applications remain responsible for installing ConfigLayers on their builder.
- `CliRun.exit_code` deliberately stays 0/1; the executable helper applies prompt exit 130.
  Typed cancellation recognition inspects a bounded source chain. Recovery/output precedence is
  explicit; application source/Display callbacks may themselves panic or block.
- Borrowed arbitrary `Write` sinks have no terminal identity, so automatic diagnostic color remains
  plain; direct run callers can request Always. `main_with` uses standard Terminal defaults.
- Synchronous file open/read and caller writes may block. The byte bound is a read-size guarantee,
  not a wall-clock guarantee; no blocked background reader is hidden by this API.
- Serialization failure has no valid public envelope and leaves stdout untouched with exit 1.
  A foreign finalized Response similarly produces no adapter output; arbitrary handler-owned
  external I/O remains outside the adapter's guarantee.
- Full executable success is proven through supplied argv/buffer sinks against real ServerBuilder.
  The actual native wrapper is additionally exercised for real argv/sink acquisition, Send and
  usage rejection. No stock binary was added. Lane D's application binary still owns its native
  successful-launch and cross-surface capability proof.
- Verification covers the local macOS target, default TTY and no-default-feature CLI, plus the
  workspace's selected default features. It does not claim Windows/Linux terminal proofs, native
  networking, HTTP features, publication, coordinator acceptance or deployment.

No product decision is pending from this implementation. Ready for coordinator review; stopped.
