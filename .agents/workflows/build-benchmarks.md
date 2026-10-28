# Build and compile benchmarks

Use this workflow for every development Cargo build, test, check, Clippy, Rustdoc,
coverage command, and clean operation, including coordinator re-verification.
Recording begins with adoption of this workflow; it does not reconstruct history.

## Record commands

Run commands through [the benchmark recorder](../scripts/build-benchmark.py).
It streams combined stdout/stderr, saves a log and JSON record for each attempt,
and returns the command's exit code. It never cleans targets automatically.
Run from the same directory and with the same environment as the original command.

```sh
python3 .agents/scripts/build-benchmark.py run \
  --label 'SQL round 1' --kind test --scope selected --cache warm \
  -- cargo test -p stackpress-sql --locked

python3 .agents/scripts/build-benchmark.py run \
  --label 'fresh effort cleanup' --kind clean --scope workspace \
  -- cargo clean

python3 .agents/scripts/build-benchmark.py run \
  --label 'fresh workspace baseline' --kind test --scope workspace --cache cold \
  -- cargo test --workspace --all-targets --locked
```

Wrap `cargo llvm-cov clean --workspace` separately as `--kind clean`. Label each
baseline, milestone, refactoring round, feature/target combination, and retry.
Record failed attempts too. Do not add extra builds just to collect benchmarks.
Use the [coordinated workflow's cleanup rule](coordinated-crate-development.md)
at the beginning of a fresh effort; routine reruns reuse their target directories.

For an existing Python/bindgen/Tauri proof that launches Cargo internally, wrap
the outer command once. Use its actual kind and scope; do not also wrap its
nested Cargo commands, which would double-count time. Cargo stages are only
detected if the outer command forwards Cargo's normal text output. If it captures
that output, the record measures the outer command only. `--message-format=json`
and quiet output can likewise hide build-stage evidence; missing data stays null.
Separately launched consumers not under that outer command need their own records.

## Labels and comparisons

| Option | Required value |
| --- | --- |
| `--label` | Task/milestone/round and relevant feature/target intent |
| `--kind` | `build`, `test`, `check`, `clippy`, `doc`, `coverage`, `clean`, or `other` |
| `--scope` | `workspace`, `selected` crates, external `consumer`, or `other` |
| `--cache` | `cold`, `warm`, or `unknown` (default) |

`cold` means the effective target output for this command was empty or cleaned
immediately beforehand. `warm` means that output was retained from a prior run;
it does not promise that all artifacts can be reused. Use `unknown` when uncertain.
Dependency downloads and OS disk caches are separate from these labels.
Workspace scope means the command selects the entire applicable workspace;
feature-limited selections and exclusions must remain visible in command/label.
Use `selected` for partial selections. A clean operation is not itself a rebuild.

## Persistent storage and counters

Default storage is `.agents/resources/build-benchmarks/` in the main checkout,
resolved through Git's common directory. Linked worktrees share that store and
retain their own working-directory, branch, and commit metadata in each record.
Outside Git the default is relative to the current directory. For a nonstandard
Git layout, or an isolated proof/test, pass `--store /absolute/path` **before**
`run` or `summary`. Use the same store for all runs intended for one counter.

Each attempt owns `runs/<UTC timestamp>-<unique ID>.json` and a matching `.log`.
The start record is written before execution and replaced atomically on finish.
Unique files allow parallel agents to record without sharing a mutable counter.
Keep these records when cleaning targets or worktrees. Include them in the
owner's normal repository commit when requested; recording does not commit them.
Do not put credentials in command arguments or labels. Only selected build
environment settings are recorded; output logs contain whatever the command emits.

```sh
python3 .agents/scripts/build-benchmark.py summary
python3 .agents/scripts/build-benchmark.py summary --json
```

The summary lists **each attempt and its duration**, plus counts by status,
kind, scope, and cache state. It separately counts attempts emitting `Compiling`
and successful declared cold workspace builds/tests/coverage runs that emitted
compilation. These are invocation counts, not numbers of crates or rustc calls.
The latter is the operational counter for full workspace rebuilds; scope/cache
labels are declarations, not independently proven by the recorder.

Two different durations are retained:

- `command_seconds`: measured wall time, including tests, downloads, lock waits,
  nested fixtures, and other command work. Metadata collection is outside the timer.
- `cargo_build_seconds`: sum of observed Cargo `Finished ... profile ... in ...`
  stages. This includes build overhead/linking/cache checks; it is not compiler
  CPU time. A warm no-op can report a stage without emitting `Compiling`.
  It is null when no stage is reported, including compilation failures before
  the first finished stage. Failed multi-stage commands may have partial timings.

Cargo's displayed stage durations are rounded. Doctest compilation and nested
fixture compilation can occur after the main stage, so command time and Cargo
stage time are not interchangeable. Concurrent nested Cargo stages can overlap;
their sum is not elapsed wall time. Concurrent invocations also overlap, so totals
represent accumulated recorded durations, not elapsed calendar time.

A hard-killed recorder retains a `running` attempt with unknown duration. Treat
it as unfinished evidence, not a pass or zero-second completed run. Ctrl-C and
SIGTERM record an interruption and stop the child process group. Preserve corrupt records and
repair from evidence: summary errors instead of silently skipping them.

## Verification and closeout

Include the store path, relevant record IDs, attempt/rebuild counter changes,
command time, Cargo build time (or unavailable), and failures in task receipts.
Compare equivalent commands, targets, features, profiles, cache states, toolchains,
and load. Host and toolchain metadata aid comparison; command-line toolchain
overrides, effective config, and contention still require receipt explanation.
This is operational timing evidence, not a controlled performance guarantee.

Verify changes to the recorder with:

```sh
python3 -m unittest discover -s .agents/scripts/tests -p 'test_build_benchmark.py'
python3 .agents/scripts/validate-agent-workspace.py
```
