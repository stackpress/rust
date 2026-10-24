# Lane D: application HTTP proof

Date: 2026-10-24. Branch: `trial/3-lane-d-app`, based on the coordinator's fast-forward
to `727e5e2`. This extends the accepted application proof using the merged buffered HTTP adapter.
The app-owned CLI binary remains deferred until the coordinator releases the `run`/`main_with`
integration brief. No CLI stub was added.

## Changes and scope

- Enabled `stackpress-server/http` in this package's dependency declaration.
- Added `tests/http.rs`: four integration tests covering six HTTP scenarios against the real
  profiles application, its service and its independently selected customization plugin.
- Updated two crate-overview Rustdoc lines to reflect the available HTTP proof and pending CLI.
- Used the adapter's exported `BufferedRequest` alias to construct native `http::Request<Vec<u8>>`
  values. No direct `http` dependency, new crate, dependency version change or lockfile edit was
  needed. No production Rust behavior changed.
- All edits are under `examples/capability-host/`. The root manifest, lockfile, framework, tools,
  facade and KB remain unchanged. Source inspection stayed in this worktree and its KB; no other
  trial branch or other lane worktree was read.

## Proved behavior

| Case | Observable result |
| --- | --- |
| GET `/profiles/abc123` and direct `profile-search` | One compiled application serves both entries. The native HTTP status is 200, and the entire decoded public envelope matches `Server::call`, including the Ada Lovelace row and total one. Report event/correlation and final Output/Sent state are checked independently from JSON. |
| Valid route with unmatched ID | Both entries return status 200, an empty result array and an explicit total zero. This remains distinct from an unknown route. |
| Unknown HTTP route | `/unknown?event=profile-search&id=abc123` produces native status 404 and a public JSON 404/error envelope with no results. The report selects only `GET /unknown` and records NoListener; query data cannot select the internal operation. |
| G17 asterisk-form input | A representable native URI of `*` cannot normalize into the adapter's pathname contract. It returns native/JSON 400 with only Input/Output records, one Input failure, no error offer and no operation. |
| G17 normalization-changing path | `/ignored/../profiles/abc123` is rejected with the same no-callback G17 contract, rather than silently selecting the valid profiles route after URL normalization. |
| Customized HTTP entry | A separate dependent plugin changes names through HTTP and direct calls identically. Captured `id` overrides query data, configured limit one overrides request `limit=100`, and delegated Abort skips Response while still producing Output/Sent HTTP output. |

The shared decoding helper explicitly matches `HttpOutput::Response`; Suppressed and Unavailable
fail the proof. It validates native status before decoding actual native response bytes. The
malformed-input host attaches a named panic-on-entry handler to Request, profile-search, Error
and Response: any fabricated callback input fails immediately, supplementing the exact report
assertions. These tests construct buffered values and open no sockets or listeners.

## Baseline and scoped rounds

Before the first build, ran `cargo clean` and `cargo llvm-cov clean --workspace`; Cargo removed
14,203 stale files / 1.5 GiB from this worktree. Every build/test/lint/doc invocation used
`--locked`, and Cargo.lock remained unchanged.

Baseline: the four new HTTP tests passed, and the complete example suite passed **16 tests in
each feature mode**. No package tests are ignored.

1. **Logic/complexity:** reviewed successful versus empty results, missing route versus malformed
   input, query/capture precedence, configured limits and delegated Abort. Added the successful
   empty-result scenario and a normalization-changing path that could otherwise enter a real
   profiles route. Both full example suites reran: **16 passed each**.
2. **Responsibility/abstraction:** reviewed native construction, output decoding and scenario
   ownership. Kept two focused local helpers and explicit per-scenario app composition. Using the
   public buffered alias avoids unnecessary dependency coupling; no production helper, generic
   wrapper or shared test-fixture extraction was justified. No-change round; HTTP tests reran in
   both modes: **4 passed each**.
3. **Consistency/Rustdoc/comments:** consolidated lib imports and updated the crate overview for
   the integrated HTTP adapter. Reviewed Rustdoc for every new function and purposeful setup,
   branch, conversion and assertion-block commentary. Both HTTP suites passed; app doctests
   passed **2 default / 1 experimental**; scoped all-feature Clippy passed with `-D warnings`.
   The final line audit also shortened an overlong comment; final suites below passed afterward.
4. **Safety/soundness:** reviewed owned native buffers, partial move of HttpReply output while
   retaining its report, explicit publication variants, malformed-input callback denial, immutable
   service use and absence of native I/O. The new tests add no unsafe, locks, globals, workers,
   blocking callbacks or cancellation machinery. Panic-on-entry is an intentional test failure
   oracle, not recovery logic. No-change round; both HTTP suites reran: **4 passed each**.

No in-scope material findings remain. All new test functions/helpers have Rustdoc; new test source
and the edited crate source have no lines over 100 columns.

## Final verification

Toolchain: rustc **1.96.0** (`ac68faa20`, LLVM **22.1.2**), native target
**aarch64-apple-darwin**. Coverage tool: cargo-llvm-cov **0.8.6**.
Logs and coverage JSON are retained under ignored `target/lane-d-http-*` paths.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test -p stackpress-example-capability-host --all-targets --locked` | **16 passed**, including four HTTP tests. |
| `cargo test -p stackpress-example-capability-host --all-targets --features experimental --locked` | **16 passed**, including four HTTP tests. |
| `cargo test --workspace --all-targets --locked --features server-http` | **127 passed**, zero failed. Ten existing helper child entrypoints are ignored in ordinary enumeration; their watchdog/PTY parent tests passed. |
| `cargo test --workspace --doc --locked --features server-http` | **322 passed**: facade 1, CLI 48, app 2, lib 268, server 3. |
| `cargo test -p stackpress-example-capability-host --doc --features experimental --locked` | **1 passed**. |
| `cargo clippy --workspace --all-targets --locked --features server-http -- -D warnings` | Passed warning-free. |
| `cargo clippy --workspace --all-targets --locked --features server-http,stackpress-example-capability-host/experimental -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --features server-http,stackpress-example-capability-host/experimental` | Passed warning-free. |
| `cargo tree -p stackpress-example-capability-host --edges normal --locked --depth 2` | Existing approved dependencies; server adds the already locked `http` 1.5.0 edge. No CLI, Tokio or network runtime normal edge. |
| `git diff --check`; function-doc and 100-column scan | Passed. |

Enabling HTTP in this example is unconditional; the two measured feature modes differ only in
the `experimental` plugin. The server crate's own optional feature declaration was not changed.

## Coverage

Each final collection was independently preceded by `cargo llvm-cov clean --workspace`.
The corrected command disables the tool's default `examples/` exclusion, then excludes only
dependencies, other workspace packages, test source and build output. All production source in
this example is measured; no shipped example implementation is excluded. Doctests run separately.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets --locked \
  --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-http-coverage-default.json --fail-under-lines 70

cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-example-capability-host --all-targets \
  --features experimental --locked --no-default-ignore-filename-regex \
  --ignore-filename-regex '/rustc/|/\.cargo/(registry|git)/|/\.rustup/toolchains/|/(crates|tools|tests|target)/' \
  --json --output-path target/lane-d-http-coverage-experimental.json --fail-under-lines 70
```

| Example scope | Covered / executable lines | Line coverage |
| --- | --- | --- |
| Defaults, HTTP enabled, experiment excluded | **108 / 108** | **100%** |
| Experimental plugin plus HTTP | **128 / 129** | **99.22%** |

Both exceed the 70% development floor. The remaining enabled-build line is the defensive error
return for the app's fixed valid experimental-plugin selection at `src/lib.rs:64`, as in the
accepted initial proof. These are example-package production metrics, not measurements of the
unchanged HTTP adapter or assertions about branch/platform coverage.

## Disposition and DX feedback

- Proof checklist item 5 now has the real application's **HTTP-to-capability** path in addition to
  direct-call/route evidence. Its **CLI-to-capability** portion still awaits lane B and the next
  coordinator brief; the complete CLI/HTTP claim remains pending.
- Items 7, 9 and 14 gain native HTTP error, customization and Abort/output evidence. Existing
  compile-time experimental exclusion and the original profiles implementation remain intact.
- The HTTP API supplied every required seam. Exported buffered aliases allow dependency-light
  construction; HttpOutput and retained reports express native availability and diagnostics
  clearly. No missing API, dependency expansion or framework change is requested.
- Verification is in process on the native target above. It establishes no listener, network
  buffering, streaming, deployment, other platform or publication support.

No commits, pushes, listeners, sockets, KB edits or publishing were performed. Stopped for
coordinator review; the next work item is the separately released application CLI proof.
