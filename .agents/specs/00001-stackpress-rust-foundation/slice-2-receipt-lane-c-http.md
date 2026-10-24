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
