# Lane C: buffered HTTP implementation stop

Spec file of [00001 Stackpress Rust foundation](index.md), recording lane C of [task 00005](tasks/00005-slice-2-server-and-cli.md). Load for the buffered `http` adapter (event selection, normalization, limits, body parsing, response conversion, output authority), its known limitations, and its verification. Moved losslessly from `crates/server/LANE-C-RECEIPT.md` (lane branch commit `96dd37c`, merged in `9d96fd7`).

Date: 2026-10-24. Branch: `trial/3-lane-c-http`, based on
`f19f37d2260150536e9c8c5448bee56726bf4c56` (accepted M1/M2).
This is the HTTP milestone stop for coordinator review, before formal refactoring rounds.
It does not establish whole-Slice-2 acceptance, publication or network/platform support.

## Changes and ownership

- Implemented `HttpOptions`, `HttpAdapter::{new, handle}`, `HttpOutput` and `HttpReply` in
  `crates/server/src/http.rs`, retaining the existing buffered type aliases.
- Added 22 in-process tests in `tests/http_input.rs` and `tests/http_output.rs`.
- Replaced the temporary `input_failure` dead-code allowance in `src/lifecycle.rs` with
  `#[cfg(any(feature = "http", test))]` and an inline explanation. The coordinator separately
  authorized this exact gate after the allowance removal exposed a default-feature warning.
- No other shared source, manifests, lockfile, facade, CLI, examples or KB files changed.
  No commits, pushes, listeners, sockets or other trial/worktree sources were used.
- Ran `cargo clean` and `cargo llvm-cov clean --workspace` before the first build in this
  fresh worktree; Cargo reported zero existing files to remove.

## Implemented contract

`HttpAdapter` is immutable and reusable. Each `handle` owns its framework request/response,
report and private `OutputGuard`; handlers never receive the native publication handle.

Input selects only the normalized method and exact native URI pathname. Host/forwarded headers,
query, post and body fields cannot select an event. Origin-form input uses the configured trusted
HTTP(S) origin (default `http://unknownhost/`); absolute HTTP(S) targets retain checked authority.
Non-path targets, credentials and a pathname changed by URL normalization are rejected. Percent
escapes, case and repeated/trailing slashes are preserved; dot/backslash normalization is never
silently accepted as another route. Query parsing stays separate.

Raw method/URI/header/body totals, repeated header count and header bytes are checked before
normalization copies or callbacks. `limits.max_bytes` also bounds the URI and body individually.
These are already-buffered limits, not bounded network reads. Every header occurrence and all
incoming Cookie lines survive foundation construction. Missing Content-Type uses `text/plain`.

JSON (object roots only, including `text/json` and `+json`), urlencoded and multipart bodies use
strict foundation `parse_body`. A successful `LoadResult` atomically installs raw `Body::Bytes`
and parsed post fields. Other MIME types preserve raw bytes without inventing POST fields.
Depth, node, multipart-header and aggregate-merge failures leave original query/data/body state
intact. Normalized failures use `Server::handle_input` and the ordinary Input error offer;
pre-normalization failures use `Server::input_failure`, return public 400 and invoke no callbacks.

Output checks foundation budgets and HTTP status 200–599, headers, cookie revisions and rich-value
conversion before its sole native publication. Structured/status-only output uses the public
envelope and the owning `Response::limits`; explicit text/bytes keep representation and MIME.
Repeated headers and all explicit/scalar Set-Cookie values append separately; deletion retains
domain/path/security scope. Content-Length is adapter-owned and repeated caller lengths are
replaced. Transfer-Encoding is rejected. HEAD runs HEAD/Any without GET fallback and retains
representation headers while suppressing bytes. 204/304 omit Content-Length; 205 uses zero;
all three have no body.

Conversion failure retains an Output diagnostic, drops all handler state and uses one fixed empty
500 with Content-Length zero. The coordinator approved fresh `Response::new()` budgets here so
even a pathological zero-node response policy cannot prevent the fallback. No error/response
callbacks rerun. Stopped output returns `Suppressed`; Sent/Attempted/Failed without this adapter's
native result return `Unavailable`. Existing OutputGuard replacement/partial-failure proofs remain
green; HTTP tests also exercise replacement after foundation finalization and concurrent Send
handling. Arbitrary callback-owned external I/O remains outside adapter authority.

## Verification

Native toolchain: rustc 1.96.0 (`ac68faa20`, LLVM 22.1.2), `aarch64-apple-darwin`;
cargo-llvm-cov 0.8.6. Cargo build/test/lint/doc invocations used `--locked`.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check`; `git diff --check` | Passed. Owned Rust files have no lines over 100 columns. |
| `cargo test -p stackpress-server --features http --all-targets --locked` | 53 passed, zero failed; two ignored helper entrypoints are run/reaped by watchdog parents. Includes all 22 new HTTP tests. |
| `cargo test --workspace --all-targets --locked` | 77 passed, zero failed; six explicit helper entrypoints run through their watchdog/PTY parents. HTTP tests are feature-gated out of this default run. |
| `cargo test --workspace --doc --features server-http --locked` | 272 passed: facade 1, lib 268, server 3. |
| `cargo test -p stackpress-server --features http --doc --locked` | 3 passed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed warning-free after the authorized seam gate. |
| `cargo clippy --workspace --all-targets --features server-http --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --features server-http --no-deps --locked` | Passed warning-free. |
| `cargo tree -p stackpress-server --edges normal --locked` and the same with `--features http` | Default has no HTTP value dependency. HTTP adds `http` 1.5.0 and its `bytes` dependency; neither normal tree has Tokio, hyper, crossterm or mio. |

The public-envelope parity test explicitly registers `/profiles/:id` delegating to
`profile-search`, then compares decoded HTTP output with `Server::call` on that capability.
It preserves the same ID result and explicit zero total. It is in-process HTTP/call evidence;
lane D owns the complete CLI/HTTP application proof.

## Coverage and artifacts

Clean collection command:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-server --features http --all-targets --locked \
  --ignore-filename-regex '/tests/' --fail-under-lines 70 \
  --json --output-path target/lane-c-coverage.json
cargo llvm-cov report -p stackpress-server --locked --lcov \
  --ignore-filename-regex '/tests/' --output-path target/lane-c-coverage.lcov
```

| JSON report scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| `stackpress-server`, HTTP enabled | 1,791 / 1,911 | 93.72% |
| `src/http.rs` | 267 / 272 | 98.16% |

The JSON retains full file summaries, segments and functions. LCOV is also retained under ignored
`target/`. Integration-test paths, dependency code and build output are outside the measured
scope; no shipped production file is excluded. The tool excludes the standalone `core_tests.rs`
fixture, while unit fixtures inside `lifecycle.rs` remain in its file-level JSON summary.
For a separate production-only check, counting unique LCOV `DA` lines and removing only
`lifecycle.rs` lines at/after its `#[cfg(test)]` boundary (534) gives 1,680 / 1,773 = 94.75%.
LCOV unique-line aggregation differs from LLVM's JSON summary/instantiation accounting; these
are separate denominators. No branch or doctest coverage is claimed. No unsafe code was added.

The remaining HTTP JSON gaps are defensive final-dispatch failure handling and a MIME insertion
conversion path normally prevented by foundation validation. No extra output hook or retry was
introduced to manufacture coverage. Formal complexity, responsibility, consistency/documentation
and safety/soundness rounds remain gated on coordinator review.

## Upstream inventory and deliberate differences

| HTTP/WHATWG inventory family | Disposition |
| --- | --- |
| Adapter construction/plug/request/response and handler/server/router/action exports | Covered by one optional `HttpAdapter` entry and the existing host. No arbitrary event override or duplicate transport value model. |
| Method, MIME, URL/query, repeated headers, cookies | Covered, with explicit trusted origin and exact pathname preservation. Blind forwarded trust and slash rewriting are deliberately not ported. |
| Loader; JSON/urlencoded/multipart; binary and empty buffers | Covered through atomic bounded lib loading, including binary multipart files. Unknown MIME remains raw bytes; source empty-object fallback is intentionally replaced. |
| Dispatcher; status-only, scalar/object/array envelopes; text/bytes | Covered, including false/null/zero and owning-response budgets. No automatic developer diagnostics on wire. |
| Multiple Set-Cookie writes and scoped removals | Covered; append fixes upstream repeated-set replacement loss. |
| List-valued Session revisions | Known limitation, confirmed by coordinator: requires an explicit codec. Output diagnostic names the cookie and says `session list value requires an explicit codec`; public output is empty 500. A future explicit cookie codec on `HttpOptions` is noted, not implemented. |
| Native resource passthrough, Web/Node request-response aliases | Intentionally not ported into foundation values. Buffered typed HTTP values and explicit availability variants own the boundary. |
| Gateway/listener/TLS, forwarded transport policy, streaming conversion/backpressure/cancellation, native writer ownership, hop-by-hop stripping, serverless | Deferred to S2-T. This milestone opens no ports and proves none of those runtime/platform contracts. |

No unresolved seam or export changes remain. The receipt is ready for coordinator review and
lossless movement into the spec on integration.

## Rounds 1–4 receipt

The following lane C rounds receipt was moved losslessly from `crates/server/LANE-C-ROUNDS-RECEIPT.md` (lane branch commit `cd5a518`, merged in `94be99b`); headings are demoted one level.

## Lane C: HTTP refactoring rounds 1–4

Date: 2026-10-24. Worktree branch: `trial/3-lane-c-http`, starting at
`3ab0a8057dec17f63c035820ce7da18570f8f414` after coordinator integration.
This is the requested stop for independent review; it does not promote whole-Slice-2 status.

### Scope and baseline

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

### Round 1: logic and complexity

Finding: `finish` mixed delivery-state eligibility, conversion/fallback and guarded publication.
Extracted `prepare_or_fallback` and `publish` while retaining the same sequence:

1. Respect Stopped/foreign-finalized state before conversion.
2. Complete conversion or retain its failure and choose the fixed sanitized fallback.
3. Enter the same private OutputGuard once and record its actual output outcome.

`finish` shrank from 67 to 39 physical lines, or 62 to 32 nonblank/non-comment code lines.
The conversion helper is 21 physical lines; publication is 30. Failure ordering, suppression,
fallback status/body and output stage observations are unchanged.

Verification: reran the complete HTTP-enabled server suite: **53 passed, zero failed**.

### Round 2: responsibility and abstraction

Finding: representation conversion still owned native framing, and ordinary header conversion
also contained session codec policy. Extracted private `frame_response` and
`append_cookie_headers`. The cookie helper borrows only Session/native header storage; framing
owns the prepared header map and byte buffer and cannot mutate framework response state.

Kept these cohesive HTTP phases together in `http.rs`. No file split or public abstraction was
warranted: there is one buffered transport owner and no alternate writer/runtime to substitute.
Prepared native output remains private until publication; structured serialization still uses
the final owning Response's limits. Cookie/header error precedence is unchanged.

Verification: **53 server tests and 3 server doctests passed**, zero failed.

### Round 3: consistency, Rustdoc and comments

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

### Round 4: safety and soundness

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

### Final verification

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

### Clean coverage

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

### Remaining boundaries

No unresolved in-scope finding or API decision remains. Scalar session-cookie policy is unchanged:
lists require an explicit codec and otherwise yield a named private Output diagnostic plus the
sanitized empty 500. Real network transport, stream/backpressure, hop-by-hop stripping, remote
trust, auth and other native targets remain outside this buffered adapter milestone. Coverage
and checks establish the stated native feature scope, not publication or all-platform acceptance.

Rounds are complete; stopped for coordinator re-verification. No further implementation started.

