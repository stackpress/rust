# Proofs

Spec file of [00001 Stackpress Rust foundation](index.md). Load when checking which technical uncertainties were settled by prototypes or proof tests, and what each proved.

## P1: Send handler signature on stable Rust

- **Gap:** G06 (D1) in [decisions](decisions.md): can event dispatch use `Send` futures while keeping named async functions and closures ergonomic?
- **Hypothesis:** A lifetime-indexed helper trait whose associated future type is bounded `Send + 'a` accepts named `async fn` handlers directly; closures need one higher-ranked boxing helper.
- **Expected signal:** Named and helper handlers compile, run nested emit, run inside `tokio::spawn`, and a handler holding a non-`Send` value across an await is rejected at compile time with the `Send` diagnostic.
- **Failure signal:** Compilation requires unstable features, `unsafe`, lifetime casts, or a handler mutex.
- **Scope:** The three-argument `(&mut Request, &mut Response, Invocation)` contract on rustc 1.96 stable. Non-goals: full emitter semantics and server adapters.
- **Prototype path:** A disposable coordinator probe crate under `/tmp/sendprobe` (not retained). The retained proof lives in production tests rather than root `proofs/`: `crates/lib/tests/handler_signature.rs` and `crates/lib/tests/ui/non_send_handler.rs`.
- **Result:** Proved on 2026-10-24.
  - The coordinator probe confirmed four variants: (a) named `async fn` with the helper trait compiles; (b) `AsyncFn` with a `Send` bound on `CallRefFuture` fails (unstable `async_fn_traits`); (c) a plain closure returning a borrowing `async move` fails on lifetimes; (d) a closure through an identity boxing helper compiles.
  - [Task 00001](tasks/00001-scaffold-and-send-signature-proof.md) proved the contract with fixtures.
  - [Task 00002](tasks/00002-foundation-data-events-and-values.md) re-proved it against the production emitter, request, and response.
- **Remaining unknown:** Behavior inside real server and Tauri hosts is untested until Slice 2 and Slice 5.

Further slices add their proofs here when a Gap cannot be settled from docs, source, or existing behavior.
