# M0 — scaffold and Send handler-signature proof

Status: **passed; stopped for coordinator review before M1**. Date: 2026-10-24. Branch: `trial/3`. This receipt does not mark Slice 1 complete or begin the post-M3 refactoring rounds.

Historical milestone receipt: the coordinator subsequently approved M0. M2 replaced the private adapter fixture with production API re-exports and re-ran the Send/nested/spawn/compiler proofs. See the [M1–M3 checkpoint receipt](m1-m3-receipt.md) for the current implementation and coverage; claims below describe M0 at its review stop.

## Scope and applied decisions

Updated the [design checkpoint](slice-1-design.md) in place with the owner decisions: Send dispatch futures; Rust 1.96/edition 2024/resolver 3; ordinary Value/Nest data clones deeply, with aliasing confined to the explicitly named Shared variant; Request/Response themselves remain non-Clone; corrected slice numbering; common dot-path/Serialize/priority ergonomics; bootstrap mapping; M0–M3 order and review stops.

Created exactly two workspace packages: the root `stackpress` facade and `crates/lib` (`stackpress-lib`), with one Cargo.lock. Both are `publish = false`; no placeholder backend/host/UI crates. Current production API is deliberately small: `Flow`, `DispatchOutcome`, `HandlerFuture`, and the facade's explicit exports. It has no production dependency other than facade → foundation, no executor and no unsafe code. Both crates forbid unsafe code and require public documentation.

The [signature integration test](../../crates/lib/tests/handler_signature.rs) and its [private adapter fixture](../../crates/lib/tests/support/signature.rs) prove the actual three-argument lifetime shape using private Request/Response/exact-event registry fixtures. These are **not** production Request/Response, complete event semantics, or finished plugin registration. M1–M3 must integrate and reprove the contract with the real values/matcher/diagnostic APIs.

## Compiler contract proved

The internal future-family trait is bounded for every invocation lifetime, with `Future: Send + 'a`. Its blanket implementation accepts an ordinary `Fn(&'a mut Request, &'a mut Response, Invocation<'a>) -> Fut`. Object-safe storage erases only to `Pin<Box<dyn Future<Output = Result<Flow>> + Send + 'a>>`; it never extends a borrow.

Named async functions register directly. Closures use one identity helper supplying the higher-ranked boxed-future output type:

```rust
builder.on("parent", named_async_parent);
builder.on("closure", handler(move |req, res, invocation| {
    let configuration = configuration.clone();
    Box::pin(async move {
        invocation.emit("child", req, res).await?;
        // Use configuration and mutate borrowed values after child completion.
        Ok(Flow::Continue)
    })
}));
```

This is a summary of the passing fixture; the linked source contains the complete executable examples. No macros, unstable AsyncFn associated types, callback mutex, unsafe Send assertion, lifetime casts or registry Clone were required. No design point failed compilation during the positive proof.

Passing scenarios cover:

- Named parent → named child with both mutable arguments held across a real executor yield, then parent mutation and unchanged parent metadata.
- Closure helper → named child with captured owned configuration, `?`, nested emit, post-await mutation and unchanged parent metadata.
- A compile-time Send assertion on the returned borrowed dispatch future; a Send + Sync assertion on the immutable table.
- Concurrent named and closure roots inside `tokio::spawn`, each owning its request/response and explicitly retaining the table with Arc. This proves Send plus valid `'static` ownership rather than claiming Send makes arbitrary borrowed inputs spawnable.
- Two concurrent nested children with different req/res pairs and isolated paths/results.
- An ordinary ParseIntError after an await propagating through a nested named handler via `?`, preserving prefix mutations and skipping the parent's trailing mutation.
- No-listener as an execution outcome, independent of response status.

The [negative compiler fixture](../../crates/lib/tests/ui/non_send_handler.rs) uses the **same adapter**. A named function item creates an `Rc<()>` internally and retains it across an await; registration is rejected even though the function item itself is Send + Sync. The test checks the non-Send returned-future diagnostic and the HandlerFn bound rather than accepting any arbitrary compile error. Expected compiler excerpt:

```text
future cannot be sent between threads safely
the trait `Send` is not implemented for `Rc<()>`
```

## Cleanup, deadlines and verification

Before the first build:

| Command | Result |
| --- | --- |
| `cargo clean` | Passed; reported `Removed 0 files`. No stale target artifact was used as evidence. |
| `cargo llvm-cov clean --workspace` | First invocation warned that its delegated Cargo cleanup could not resolve crates.io DNS, despite returning outer exit 0. Retried through normal network approval; completed with exit 0 and no warning. |

The initial test dependency download also failed on sandbox DNS. The authorized network retry downloaded Tokio and tokio-macros and passed. No dependency/toolchain installation beyond the authorized Cargo dependencies was performed. Lockfile resolves Tokio 1.54.1 for tests only.

The positive async scenario child has a 2-second executor deadline and a **10-second external process watchdog**. The negative rustc invocation also has a 10-second external deadline. The watchdog uses bounded diagnostic reads and kill + wait/reap cleanup on deadline/error paths. A separate deliberately stuck child is killed/reaped after 250 ms, proving the timeout mechanism does not depend on an executor timer or joining a blocked thread. That child requires an explicit test-only environment marker so manually running ignored helpers does not create an endless process.

Verified on `rustc 1.96.0 (ac68faa20 2026-05-25)`, Cargo 1.96.0, native `aarch64-apple-darwin`; cargo-llvm-cov 0.8.6 is installed. Commands after formatting:

| Check | Result |
| --- | --- |
| `cargo test --workspace --all-targets --locked` | Passed: 3 parent integration tests. Two child entrypoints are marked ignored for ordinary harness selection but are executed explicitly by those watchdog parents. No skipped behavioral proof is implied by that harness status. |
| `cargo test --workspace --doc --locked` | Passed: 5 compiling doctests (1 facade, 4 foundation). |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed without warnings. |
| `cargo +1.96.0 check --workspace --lib --locked` | Passed on the explicitly selected installed baseline. |
| `cargo tree --workspace --edges normal --locked` | Only stackpress → stackpress-lib; no third-party production dependency. |
| `cargo fmt --all -- --check` | Passed. |

Compiler/watchdog logs are under ignored `target/m0/`. They were generated in this milestone, after the required cleanup; they are not durable source evidence or output from another trial.

## Limits and next stop

No line-coverage percentage is claimed at M0. Current production items are a future type alias, control-flow enum declarations and re-exports; the executable adapter/emitter mechanism currently lives in proof fixtures. Measuring fixture execution would not establish production coverage. M1–M3 will add actual implementation and measure the per-crate 70% gate, with a current coverage report at the M3 review. Refactoring rounds 1–4 remain after that review.

Production matchers/priority/hooks/handles/composition, rich data and Shared clone behavior, request/response/session/cookie, serialization/envelopes/resolve, full diagnostics, and general lifecycle are not established by M0. Other targets and real server/Tauri integrations remain untested. The positive proof establishes a sound implementable signature; it does not prove those later contracts by resemblance.

No other trial branches, listeners, commits, pushes, publishing or deployments were used. **M1 has not started.**
