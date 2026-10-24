# Lane B: CLI and prompt receipt

Spec file of [00001 Stackpress Rust foundation](index.md), recording lane B of [task 00005](tasks/00005-slice-2-server-and-cli.md). Load for the `stackpress-cli` parsing, control, prompt and native TTY baseline, its upstream Terminal/control/input disposition, rounds 1–4 and verification. Moved losslessly from `tools/cli/LANE-B-RECEIPT.md` (lane branch commit `cb09e90`, merged in `55ca02b`).

**Current status: rounds 1–4 complete; ready for coordinator re-verification.**
The final round receipt at the end of this file supersedes the baseline limitations
and pending-round statements below. Run/main_with integration remains gated on M2.

## Historical baseline snapshot

Date: 2026-10-24. Worktree: `trial3-lane-b-cli`, branch `trial/3-lane-b-cli`,
based on approved M0 `996d31f`. Scope is the coordinator's pure CLI/prompt brief.

**The baseline was verified by the coordinator before rounds 1–4 were released.**
At that earlier stop, formal rounds had not started. The coordinated workflow §5.3
and Slice 2 verification's Baseline
stop require coordinator re-verification before those rounds. This receipt does
not claim whole Slice 2 completion, acceptance, integration or publication.

## Scope and changed files

Only `tools/cli/**` changed. No manifest, dependency, feature or lockfile changes
were needed. No commits, pushes, listeners, other-trial reads or KB edits occurred.
The clean initial worktree had no unrelated dirty files to preserve.

Modified:

- `src/lib.rs`: explicit exports and a compiling parsed-command example.
- `src/tty.rs`: retain the M0 guard; add real bounded, joinable TtyInput driver.

Added:

- `src/args.rs`: Terminal, TerminalOptions, ArgumentError and supplied argv parsing.
- `src/control.rs`: Control, levels, color policy and output-environment snapshot.
- `src/prompt/mod.rs`: explicit public prompt API and compiling example.
- `src/prompt/options.rs`: options, limits, sync/async validators, transformer/errors.
- `src/prompt/state.rs`: pure editing, generation-checked validation and cancellation.
- `src/prompt/render.rs`: bounded sanitized display and explicit writer framing.
- `src/prompt/driver.rs`: supplied buffer/channel/native orchestration and cancellation.
- `tests/args_control_contracts.rs`: argv, alias, budget and exact formatting contracts.
- `tests/prompt_contracts.rs`: pure state and watchdog-backed async prompt scenarios.
- `tests/tty_prompt_contracts.rs`: production driver/worker in an actual native PTY.
- `tests/support/tty_prompt_probe.py`: bounded handshake-based PTY parent and reaper.
- `LANE-B-RECEIPT.md`: this receipt.

Existing M0 `tests/tty_contracts.rs` and its PTY harness remain unchanged and pass.
Root Cargo.toml/Cargo.lock, src/lib.rs, crates, examples and .agents are untouched.

## Implemented contracts and inventory disposition

| Upstream Terminal/control/input family | Current implementation or explicit disposition |
| --- | --- |
| Terminal command/args/data/brand/control | Immutable Terminal with borrowed command/args/data/options and explicit Control creation. Parsing is eager, bounded and uses lib `parse_args` directly. No argv join/reparse or shallow mutable copies. |
| Terminal expect/alias/default lookup | `expect<T>(&[aliases]) -> Result<Option<T>>` selects the first present literal key, retaining false/zero/empty/null. Present wrong type returns an error rather than falling through. Defaults use `?.unwrap_or(default)`. |
| Server Terminal verbose/force/bootstrap | Operators only before `emit`; all tokens after the event remain target argv. Grouped `-vf`, `-vfb file`, `-bfile` and `--bootstrap=file` are accepted. Force is metadata only. Bootstrap is an owned explicit path, with no file read yet. |
| Invalid argv and parser bounds | Typed non-UTF-8 index, concise usage errors, nonempty/control-free event, aggregate byte/node bounds, source-retaining lib parse/decode errors. Executable name is excluded from supplied argv. |
| Terminal server/cwd/bootstrap/run; scripts emit | Deferred to the coordinator's post-M2 integration brief. No run/main_with/CliSinks/CliRun/CliContext/exit mapping or fake Server::handle implementation exists. File loading and startup integration also remain there. |
| Mutable Terminal self-registration | Intentionally absent: no mutable terminal/input owner enters shared services. |
| control output/error/info/success/system/warning | Explicit writer and level policy; original red/blue/green/magenta/yellow colors, printable brand, template `%s` substitution and I/O propagation. No global stdout or stdin mutation. |
| Color/injection policy | Auto requires selected output TTY and nonempty NO_COLOR absent; Always/Never explicitly override Auto. Caller brand/message/variables cannot inject C0/C1 controls or bidi overrides. Only fixed trusted ANSI is generated. |
| control input wiring | `Control::input(question, default)` returns real PromptOptions, required iff default is absent. The caller explicitly drives those options; creating them never acquires stdin. |
| input configuration/state/theme/context | Owned PromptOptions, non-Clone PromptState, Prefill::Tab/Editable, InvalidMode::Keep/Clear, explicit cancellation/render options and finite limits. No Node stream or global context aliases. |
| validateInput | Required then immutable Rust regex then sync/async user validator. Validation::Valid/Invalid is exact; callback failure is a distinct source-retaining error. Rust regex syntax/search semantics replace JS RegExp state/coercion. |
| formatValue/renderPrompt/clearRender | Live/final transformer, sanitized bounded prompt/error text and clear-on-done terminal framing. Answer is returned untransformed. Plain output has initial/errors/final text and no escapes. See the physical-row limitation below. |
| Typed/default/required/pattern/Keep/Clear tests | Covered by buffered fixtures; clearing invalid text clears its default too. Empty default is distinct from no default. |
| Editable/Tab/backspace/Unicode | Covered, including emoji scalar deletion and UTF-8 byte-budget boundaries. Backspace deletes one scalar, not a grapheme cluster. CRLF in supplied lines becomes one Enter. |
| Async validation/cancellation/races | Driver owns one validation future, queues bounded edits and replays only after rejection. Cancellation drops the future; pure-state generation tokens reject stale results after cancel and between submissions. Input sources cannot forge completion events. EOF after Enter lets its validation finish. |
| EOF/validator failure/output error/unwind | Explicit EOF and Aborted differ from invalid input and I/O. Callback causes survive. Partial-write/flush and transform failure terminate without retry. Panic propagates; fresh independent prompts still work. |
| Native TTY input/output/raw ownership | Optional real crossterm TtyInput wraps M0 TtySession. One exclusive lease, 32-event channel, 20-ms poll/backpressure interval, stop/close/unpark/join, prior raw state retained. No validator/writer in worker. Explicit close awaits completion then joins; Drop joins during cancellation/unwind. |
| Native cancellation/drop/send saturation | Actual PTY production proofs cover success, Unicode input, queue saturation before drop, pending prompt drop, Ctrl-C during pending validator, output error, Ctrl-D EOF, validator panic and prior raw mode. Parent verifies actual termios before reaping the child. |
| Native non-TTY input | Explicit/default answer uses real validation/rendering; no answer/default returns NonInteractive immediately. No blocked background pipe reader. Full native async pipe polling remains the approved later OS-proof boundary. Supplied buffered/line/channel sources work now. |
| serve/develop/network/Node resources | Outside this brief and remain S2-T/S3; no listener, stock binary or executable plugin loader was added. |

Deliberate formatting detail: substitutions consume placeholders only from the
original template. An inserted `%s` is literal; a later variable cannot rewrite
it. All human output uses the caller-selected sink; final machine stdout/channel
and exit behavior await run integration.

## Verification commands and results

Before the first build, `cargo clean` and `cargo llvm-cov clean --workspace` both
passed in this worktree; cargo clean removed zero files. No tool/dependency fetch
or install was needed. Compiler is Rust 1.96.0, native aarch64-apple-darwin;
cargo-llvm-cov is 0.8.6.

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. Source scan found zero Rust source lines over 100 columns. |
| `git diff --check` | Passed. |
| `cargo test -p stackpress-cli --all-targets --locked` | 13 ordinary parent/state/format tests passed; 4 ignored helper entrypoints are explicitly run/reaped by their parents, including the intentionally blocked self-test. Zero failures. |
| `cargo test -p stackpress-cli --no-default-features --all-targets --locked` | 10 ordinary tests passed; two ignored child entrypoints are explicitly exercised by the watchdog parents. TTY-only targets have zero cases in this feature scope. |
| `cargo test -p stackpress-cli --doc --locked` | 2 passed. |
| `cargo test -p stackpress-cli --no-default-features --doc --locked` | 2 passed. |
| `cargo clippy -p stackpress-cli --all-targets --locked -- -D warnings` | Passed without warnings. |
| `cargo clippy -p stackpress-cli --no-default-features --all-targets --locked -- -D warnings` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-deps --locked` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-default-features --no-deps --locked` | Passed without warnings. |
| `cargo tree -p stackpress-cli --edges normal --locked` | Existing approved graph; retained at target/coverage/lane-b-default.tree. |
| `cargo tree -p stackpress-cli --no-default-features --edges normal --locked` | Crossterm/native polling dependencies absent; retained at target/coverage/lane-b-no-tty.tree. |

Prompt async scenarios have individual two-second timeouts and an external
ten-second watchdog. New PTY prompts have two-second timeouts, Python an eight-
second child bound, Rust a ten-second harness bound. The unchanged M0 harness has
its existing twelve-second outer bound. Every watchdog kills and reaps the exact
child; the deliberately blocked self-test uses 250 ms. Handshakes, Notify receipts
and actual queue-length observations establish new interleavings without sleeps.

Early development checks caught a missing borrow in typed JSON conversion, a
fixture escape typo, and Clippy's intentional-invalid-regex fixture diagnostic.
They were corrected; final checks above pass. A retained-queue accounting edge
found during baseline work now has a deterministic regression: text remaining
behind a queued Enter counts against the next validation's aggregate byte budget.

## Coverage

Each feature measurement was preceded by `cargo llvm-cov clean --workspace`.
The ignored PTY/watchdog children inherit coverage settings and their profiles
participate in the report. No production file or branch was excluded.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-cli --all-targets --locked \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-baseline.json --fail-under-lines 70
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-cli --no-default-features --all-targets --locked \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-no-tty-baseline.json --fail-under-lines 70
```

Default tty scope: **873/905 executable production lines = 96.46%**, passed.
No-default-features scope: **708/729 lines = 97.12%**, passed. Its JSON report is
retained independently. Tests, dependencies and build output are outside production
coverage; doctests passed separately. This is line coverage on one native target,
not branch coverage, cross-platform acceptance or publication readiness.

| Production file, default tty | Covered/executable |
| --- | --- |
| src/args.rs | 155/157 |
| src/control.rs | 132/132 |
| src/prompt/driver.rs | 173/182 |
| src/prompt/options.rs | 84/89 |
| src/prompt/render.rs | 65/68 |
| src/prompt/state.rs | 108/108 |
| src/tty.rs | 156/169 |

No Cargo implementation outside tools/cli changed, so this receipt does not claim
a new workspace-wide lifecycle/HTTP/application integration result.

## API rough edges and remaining review work

- Terminal rendering currently redraws one physical row. Long/wrapping text can
  leave prior wrapped rows behind and clear-on-done cannot erase them reliably.
  RenderOptions documents plain rendering for such prompts. Width/grapheme-aware
  layout remains a concrete review finding, not a claimed full-terminal proof.
- A supplied native answer that fails validation renders its error, then returns
  EOF because that one-answer source is exhausted. There is no InvalidAnswer
  termination variant. Empty supplied text follows Enter/default semantics.
- `Control::input` is an options builder rather than a hidden async stdin reader.
  Applications explicitly call prompt/prompt_native and select their human sink.
- Callback setters are currently public optional boxed values; async validators
  own a String and use Validator::asynchronous. Regex construction uses regex's
  concrete Regex type. These are usable but merit the coordinator's DX review.
- TtyInput::buffered_events is a read-only diagnostic snapshot, added to observe
  actual queue saturation before drop in the native proof. It grants no second
  input reader or readiness guarantee.
- OS query/poll/read/thread-spawn/raw-restoration error injection and other OS
  targets are unmeasured. Drop attempts cleanup; persistent restoration failure
  retains the lease. Competing external crossterm readers/locks or OS stalls are
  outside the exclusive-framework-reader cleanup bound.
- Synchronous caller writers/validators/transformers can block. Async callbacks
  must yield/cooperate for cancellation. No arbitrary callback liveness promise
  is made, and no background task is spawned for user validation.
- Terminal.parse_os rejects non-UTF-8 paths with the rest of supplied argv; no
  lossy fallback. Parse/bootstrap file execution/error exits are not yet wired.
- Formal complexity, responsibility, consistency/Rustdoc/comments and safety
  rounds are pending coordinator baseline review/release, per G05. Rustdoc is on
  every source function, and detailed comment review still belongs to that gate.
- Full crate guide/reference documentation is the later dedicated docs task.

At the baseline stop, the next release was coordinator review and then rounds
1–4. Those rounds are now complete as recorded below. Execution integration
remains separately gated on lane A's accepted M2 and the extended CLI brief.

## Rounds 1–4 closeout (current, 2026-10-24)

The coordinator independently verified the baseline and explicitly released all
four rounds. Only tools/cli and the now-approved Cargo.lock were changed. No
root manifest, lib/server implementation, examples, KB, other trial branch,
commit, push or listener was touched. Existing M0 test changes are comments only.

### Round 1: logic, complexity and wrapped rendering

Finding: clearing one physical row left stale question/answer/error rows after
wrap. Added display-width layout with unicode-width 0.2.2, explicit CRLF row
breaks and committed previous-row tracking. Clear/redraw erases from the last
owned row back to the first; CR cancels a full last cell's pending auto-wrap.
Only successful write/flush commits cursor bookkeeping. Whole-frame preparation
and bounds precede output, including row-framing/erasure overhead.

RenderOptions accepts an injected width for buffers; otherwise terminal mode
queries crossterm's real terminal dimensions once for that prompt. Plain mode
never queries them. Invalid widths fail before writing. Native frames exceeding
screen height fail before paint, keeping their rows out of unerasable scrollback.

The dependency is coordinator-approved with default features disabled and no
normal dependency children. Cargo's first sandboxed index request failed DNS;
the approved network rerun fetched it. Cargo.lock changes only seven added lines:
the CLI edge and the one new package; no existing version changed. The sole
unlocked command was `cargo check -p stackpress-cli --all-targets` to resolve
that authorized lock update. All later code verification uses --locked.

Proofs added in tests/render_contracts.rs cover widths 1/2/3/4/6/8, exact right
margins, ASCII, CJK, emoji, ZWJ/variation sequences, combining marks, a four-row
rejection followed by a shorter correction, clear-on-done, invalid widths and
framing limits. The real PTY is set to eight columns and its transcript proves
the production prompt used native dimensions, wrapped wide/emoji text and erased
both rows. Its termios restoration remains independently verified by Python.

Verification after the round: `cargo test -p stackpress-cli --all-targets
--locked` and its `--no-default-features` counterpart passed: 14 and 11 ordinary
tests respectively, plus their watchdog-owned helper processes. The full final
commands/results appear below.

### Round 2: responsibility, abstraction and callback DX

Finding: exposed optional boxed fields made callers manage erasure and compile
regexes separately. Pattern/validator/transformer storage is now private.
PromptOptions supplies consuming pattern/with_pattern/without_pattern,
validator/async_validator/without_validator and transform/without_transform
builders. Named async functions register directly without caller-side boxing.
Display callback invocation belongs to options; rendering retains bounds and
sanitization. The state, I/O and native-worker responsibilities remain separate.

Supplied/default-answer rejection now returns InvalidAnswer(String), carrying
the checked validation message. Display names that failure without dumping its
message/answer. Interactive buffers still return EOF when exhausted without an
accepted answer; their invalid input can be corrected normally.

Verification after the round: both all-target feature commands passed (14/11).
Existing fixtures were migrated to the public builders; no test reaches into
private callback storage. Pattern-construction examples also prove invalid regex
is rejected before terminal ownership is involved.

### Round 3: consistency, Rustdoc and purposeful comments

Expanded explanations for every function body and logical phase, especially
operator cursor/value consumption, alias presence, limit-before-mutation checks,
state/token transitions, serialized validation/edit queues, cancellation ordering,
row layout/erasure and raw-mode/worker ownership. Tests explain their fixtures,
handshakes, actions and observable assertions. Existing M0 tests only gained
scenario explanations; their accepted proof behavior is retained.

Added compiling Examples on parsing/lookup, color policy, Control construction
and every output helper, prompt configuration/builders, cancellation, buffers,
the prompt/state/render entry points and native ownership. Named sync/async
validators are shown directly. Real-terminal operations alone use no_run; a
supplied native answer is demonstrated by an executed doctest with no terminal.

Round verification: both feature test suites passed; 47/42 doctests initially
passed, then the additional real native-prompt example brought final totals to
48/43. Clippy and Rustdoc are warning-clean in both configurations. The final
mechanical source scan found zero functions without Rustdoc and zero Rust lines
over 100 columns.

Per-file inline-comment counts (first non-whitespace characters are `// `;
excludes Rustdoc, not a target or substitute for the actual review):

| Production source | Inline-comment lines |
| --- | ---: |
| src/args.rs | 38 |
| src/control.rs | 23 |
| src/prompt/driver.rs | 58 |
| src/prompt/options.rs | 26 |
| src/prompt/render.rs | 39 |
| src/prompt/state.rs | 36 |
| src/tty.rs | 47 |
| src/lib.rs | 0 (exports and crate Rustdoc; no function bodies) |
| src/prompt/mod.rs | 0 (exports and module Rustdoc; no function bodies) |

### Round 4: safety, soundness and boundary regressions

Reviewed safe UTF-8 slicing, width/budget arithmetic, mutable state ownership,
callback lifetimes/Send, cancellation-safe input receives, explicit signal
sharing, absence of framework locks across await, no detached user work, worker
stop/full-queue/close/drop/unwind, prior raw-mode restoration and single-attempt
output. No unsafe code, alternate lifecycle or second native reader was added.
TtyInput::buffered_events remains explicitly diagnostic-only and makes no read-
readiness promise; it observes queue saturation in the existing production proof.

Two concrete defects were proved with failing regressions before fixing them:

- A validator could cancel its shared signal during its own ready poll, after
  select had polled cancellation but before completion was committed. The driver
  now rechecks cancellation immediately after validation and before state/final
  output. The regression proves Aborted and no accepted final answer text.
- A supplied answer was previously a Text edit, so Editable defaults produced
  `AdaGrace` for explicit `Grace`, or duplicated a fallback default. Supplied
  candidate construction now replaces preview/prefill, sharing checked text rules
  with editing. Explicit empty answers remain empty; native default fallback is
  applied exactly once. Interactive empty Enter still uses its remaining default.

The follow-up native height proof initially selected the fixture's plain output
configuration and timed out as expected for an unanswered prompt. It was corrected
to select actual terminal rendering, then proved the oversized frame produces
Limit, no output and restored raw state. This was a fixture selection error.

Verification after fixes: both full feature suites passed, including the actual
PTY/drop/queue/cancellation/panic cases. Final doctests, lint/docs and fresh
per-feature coverage below include these fixes. No further material in-scope
findings remain from the targeted follow-up review.

### Final command matrix

| Exact command | Final result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `git diff --check` | Passed. |
| `cargo test -p stackpress-cli --all-targets --locked` | 14 ordinary tests passed, 0 failures; 5 ignored helper entries explicitly run/reaped by their watchdog parents. |
| `cargo test -p stackpress-cli --no-default-features --all-targets --locked` | 11 ordinary tests passed, 0 failures; 3 ignored helper entries explicitly run/reaped. TTY-only targets contain no cases in this scope. |
| `cargo test -p stackpress-cli --doc --locked` | 48 passed; includes 6 real-terminal no_run compilation proofs. |
| `cargo test -p stackpress-cli --no-default-features --doc --locked` | 43 passed; includes the 1 real-native-prompt no_run compilation proof. |
| `cargo clippy -p stackpress-cli --all-targets --locked -- -D warnings` | Passed. |
| `cargo clippy -p stackpress-cli --no-default-features --all-targets --locked -- -D warnings` | Passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-deps --locked` | Passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-cli --no-default-features --no-deps --locked` | Passed. |
| `cargo tree -p stackpress-cli --edges normal --locked` | Passed; retained at target/coverage/lane-b-rounds-tty.tree. |
| `cargo tree -p stackpress-cli --no-default-features --edges normal --locked` | Passed; retained at target/coverage/lane-b-rounds-no-tty.tree. |
| `cargo tree -p unicode-width --edges normal --locked` | Only unicode-width 0.2.2; no normal dependency children. |

The 100-column source scan used all `Path('tools/cli').rglob('*.rs')`, enumerating
`len(line) > 100`; result: empty. The function/Rustdoc scan and per-file comment
counts were checked alongside it. Timing, kill/reap and handshake rules from the
baseline still apply; the new renderer scenarios also have a two-second async
and ten-second external watchdog.

### Final clean coverage

Rust 1.96.0, cargo-llvm-cov 0.8.6, aarch64-apple-darwin. Each feature collection was
preceded by its own workspace coverage clean. Exact commands:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-cli --all-targets --locked \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-rounds-tty.json --fail-under-lines 70
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-cli --no-default-features --all-targets --locked \
  --ignore-filename-regex '/tests/' --json \
  --output-path target/coverage/lane-b-rounds-no-tty.json --fail-under-lines 70
```

| Scope | Covered/executable lines | Coverage |
| --- | ---: | ---: |
| Default tty | 1,013/1,054 | **96.11%**, passed |
| No default features | 842/877 | **96.01%**, passed |

| Default-tty production file | Covered/executable lines |
| --- | ---: |
| src/args.rs | 155/157 |
| src/control.rs | 132/132 |
| src/prompt/driver.rs | 194/206 |
| src/prompt/options.rs | 127/136 |
| src/prompt/render.rs | 125/130 |
| src/prompt/state.rs | 122/122 |
| src/tty.rs | 158/171 |

Only tests/dependencies/build output are outside production coverage. No shipped
source is excluded. Native PTY/watchdog children inherit instrumentation and
contribute profiles. Stable branch/doctest coverage is not claimed; doctests have
their separate passing matrix. This meets the 70% development gate, not release
or cross-platform acceptance.

### Remaining limits and next integration boundary

- Unicode-width uses its normal, non-CJK-ambiguous policy; terminal-specific font
  differences remain possible. Editing and row splits are scalar-based, not full
  grapheme segmentation, as allowed by the brief. Common emoji/ZWJ/variation and
  combining cases are explicitly tested.
- Width/native height are captured for one output session. Resizing or concurrent
  external writes requires cancelling/restarting that session; live reflow is
  not claimed. Injected widths model buffers and have no native height policy.
  Native frames too tall for the screen return Limit before writing.
- Native OS failure injection, competing external crossterm users, OS stalls and
  other operating systems remain outside measured evidence. Persistent raw-mode
  restoration failure retains the lease, as in M0. Synchronous writers/callbacks
  can block; cooperative async cancellation does not cancel user-spawned work.
- Run/main_with, bootstrap-file I/O, CliSinks/CliRun/CliContext, stdout envelopes,
  execution exit mapping and application integration remain deferred to the
  coordinator's post-M2 brief. No stubs or implicit prompts were introduced.

Stopped after all four rounds and final verification. No new owner decision is
needed for the delivered scope; coordinator re-verification/integration is next.
