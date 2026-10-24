# Lane C: HTTP refactoring rounds 1–4

Date: 2026-10-24. Worktree branch: `trial/3-lane-c-http`, starting at
`3ab0a8057dec17f63c035820ce7da18570f8f414` after coordinator integration.
This is the requested stop for independent review; it does not promote whole-Slice-2 status.

## Scope and baseline

Changed only `crates/server/src/http.rs`, `crates/server/tests/http_input.rs` and
`crates/server/tests/http_output.rs`, plus this requested receipt. No lifecycle/shared source,
manifests, dependencies, lockfile, facade, CLI, example or KB edits. No commits, pushes,
listeners or sockets. Sources were this worktree and its existing project instructions.

Preserved every public name/signature, field and output variant. The before/after snapshot of
16 public declaration lines (types, aliases, fields and entry methods) is identical; enum
variants were also unchanged. The existing example consumer compiled and passed its workspace
tests without edits. The private extraction adds no traits or generic adapter hierarchy.

Before the fresh round baseline, ran `cargo clean` and `cargo llvm-cov clean --workspace`;
Cargo removed 18,791 files / 1.9 GiB. Baseline command
`cargo test -p stackpress-server --features http --all-targets --locked` passed all 53 tests,
including 22 HTTP tests. Two ignored helper entrypoints are invoked by watchdog parents.

## Round 1: logic and complexity

Finding: `finish` mixed delivery-state eligibility, conversion/fallback and guarded publication.
Extracted `prepare_or_fallback` and `publish` while retaining the same sequence:

1. Respect Stopped/foreign-finalized state before conversion.
2. Complete conversion or retain its failure and choose the fixed sanitized fallback.
3. Enter the same private OutputGuard once and record its actual output outcome.

`finish` shrank from 67 to 39 physical lines, or 62 to 32 nonblank/non-comment code lines.
The conversion helper is 21 physical lines; publication is 30. Failure ordering, suppression,
fallback status/body and output stage observations are unchanged.

Verification: reran the complete HTTP-enabled server suite: **53 passed, zero failed**.

## Round 2: responsibility and abstraction

Finding: representation conversion still owned native framing, and ordinary header conversion
also contained session codec policy. Extracted private `frame_response` and
`append_cookie_headers`. The cookie helper borrows only Session/native header storage; framing
owns the prepared header map and byte buffer and cannot mutate framework response state.

Kept these cohesive HTTP phases together in `http.rs`. No file split or public abstraction was
warranted: there is one buffered transport owner and no alternate writer/runtime to substitute.
Prepared native output remains private until publication; structured serialization still uses
the final owning Response's limits. Cookie/header error precedence is unchanged.

Verification: **53 server tests and 3 server doctests passed**, zero failed.

## Round 3: consistency, Rustdoc and comments

Added local explanations for base/target authority, normalization discrepancy rejection,
multi-value headers, raw limits, MIME dispatch/atomic loading, output availability, fallback,
guard sequencing, final response budgets, status legality, HEAD/body-forbidden framing,
Content-Length ownership, Transfer-Encoding rejection, scoped cookies and codec diagnostics.
Tests now explain scenario setup, the invoked boundary and the observable assertion groups.

Added compiling examples for HttpOptions defaults/construction and all HttpOutput variants,
alongside the existing HttpAdapter construction/handling examples. The availability example
builds buffered native requests and shows Response, Suppressed and foreign-finalized Unavailable.
There are **4 HTTP examples / 5 server doctests**, with no ignored examples.

Counting convention: inline comment lines begin with `//` but exclude Rustdoc/module docs;
code lines are nonblank lines that do not begin with a comment. Wrapped continuation lines
count as comment lines. Comment blocks below count adjacent wrapped lines once.

| File | Baseline comment/code lines | Final comment/code lines | Final comments per 100 code lines | Final comment blocks |
| --- | --- | --- | --- | --- |
| `src/http.rs` | 31 / 344 | 92 / 366 | 25.14 | 91 |
| `tests/http_input.rs` | 6 / 464 | 68 / 464 | 14.66 | 65 |
| `tests/http_output.rs` | 12 / 486 | 102 / 576 | 17.71 | 92 |

The final output-test count also includes Round 4's explained safety regressions. A declaration
scan found no function without Rustdoc in the three owned Rust files. The width scan found
**zero lines over 100 columns** in those files. Comments describe actual rules rather than
inventing a numeric density target.

Verification: **53 server tests and 5 server doctests passed**. HTTP-enabled workspace Clippy
and Rustdoc passed with `-D warnings`; formatting and comment-width fixes passed their checks.

## Round 4: safety and soundness

Reviewed owned buffer/response lifetimes, private publication authority, Send futures,
Response replacement, suppression/foreign finalization, cancellation, immutable adapter reuse,
normalization trust, finite parsing limits, atomic loading, framing and public disclosure.
No production defect or additional public behavior decision was found. No unsafe code,
locks, native resource handles, background tasks or dependencies were introduced.

Added three meaningful regression tests for previously uncombined safety paths:

- Cancel a genuinely pending HTTP operation after partial response mutation, then reuse the
  same adapter/host. The next call has a fresh correlation/output and no leaked header/state.
  A direct poll observes Pending; the test never waits on the intentionally pending operation.
- HEAD suppresses both a G17 pre-normalization error envelope and an Output conversion fallback.
  The former retains representation length; the fixed fallback uses length zero.
- 204/205/304 with rich in-process bodies skip unsupported JSON representation conversion
  entirely, for both GET and HEAD, and discard stale framing.

Existing tests still prove one fallback publication, no error-hook recursion on Output,
Stopped/Sent/Attempted/Failed distinctions, whole-value replacement, cookie scope/codec failure,
owning-response budgets, route authority, input atomicity and call/HTTP public-envelope parity.

Verification: **56 server tests passed**, including **25 HTTP tests**, zero failed.

## Final verification

All commands below passed on rustc 1.96.0 (`ac68faa20`, LLVM 22.1.2), native
`aarch64-apple-darwin`. Cargo build/test/lint/doc commands used `--locked`.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check`; `git diff --check` | Passed. |
| `cargo test -p stackpress-server --features http --all-targets --locked` | 56 passed, zero failed; 2 watchdog helper entrypoints ignored by ordinary discovery. |
| `cargo test --workspace --all-targets --locked` | 133 passed, zero failed; 13 helper entrypoints run by watchdog/PTY parents. Includes the existing CLI/example contributions at the starting commit. |
| `cargo test -p stackpress-server --features http --doc --locked` | 5 passed, zero failed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed warning-free. |
| `cargo clippy --workspace --all-targets --features server-http --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --features server-http --no-deps --locked` | Passed warning-free. |
| Width and Rustdoc declaration scans | Zero owned Rust lines over 100 columns; zero undocumented functions. |

Workspace default selections now include the existing example's HTTP dependency; Cargo feature
unification therefore includes the HTTP tests in that workspace run. The direct server command
above is the explicitly selected HTTP measurement, not a claim about a bare no-HTTP server graph.

## Clean coverage

Tool: cargo-llvm-cov 0.8.6, same compiler/target as above.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-server --features http --all-targets --locked \
  --ignore-filename-regex '/tests/' --fail-under-lines 70 \
  --json --output-path target/lane-c-rounds-coverage.json
cargo llvm-cov report -p stackpress-server --locked --lcov \
  --ignore-filename-regex '/tests/' --output-path target/lane-c-rounds-coverage.lcov
```

| JSON scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| `stackpress-server`, HTTP enabled | 1,815 / 1,935 | 93.80% |
| `src/http.rs` | 291 / 296 | 98.31% |

Both artifacts are retained in ignored `target/`; JSON includes file summaries, segments and
functions. Integration fixtures/dependencies/build output are outside the measured scope;
no shipped production file is excluded. As in the accepted baseline, the standalone core test
fixture is excluded by the tool, while in-file lifecycle unit fixtures remain in that file's
summary. This is stable line coverage; no branch or doctest coverage claim is made. The >=70%
development gate passed without extra exclusions or shallow threshold-driven tests.

## Remaining boundaries

No unresolved in-scope finding or API decision remains. Scalar session-cookie policy is unchanged:
lists require an explicit codec and otherwise yield a named private Output diagnostic plus the
sanitized empty 500. Real network transport, stream/backpressure, hop-by-hop stripping, remote
trust, auth and other native targets remain outside this buffered adapter milestone. Coverage
and checks establish the stated native feature scope, not publication or all-platform acceptance.

Rounds are complete; stopped for coordinator re-verification. No further implementation started.
