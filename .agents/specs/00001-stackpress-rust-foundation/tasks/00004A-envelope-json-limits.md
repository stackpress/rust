# Task 00004A: Envelope JSON conversion respects custom limits

Side quest of [task 00004](00004-lib-usage-and-api-docs.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Discovered while documenting the response API, and approved by the owner on 2026-10-24, scoped to this defect only.

## Task summary

`ResponseEnvelope::try_to_json` converts its `errors` and `results` values with `Value::to_json()`, which applies `ParseLimits::default()`. A `Response` created with larger custom limits (`Response::with_limits`) can therefore accept values that its own envelope later rejects during JSON conversion. Round 4 of [task 00003](00003-review-rounds-and-closeout.md) fixed this class of issue for Nest and Response JSON text but missed the envelope path. Make envelope JSON conversion honor the originating response's selected limits, and keep an explicit way to choose a wire budget.

Out of scope, per the owner: the other DX observations recorded in task 00004 (`sent()`/`result()` semantics, setters retaining earlier fields, status-only envelope code 0, optional path reads hiding malformed syntax). Those stay documented current behavior.

## Implementation steps

1. Write a failing regression test first. It builds a response under larger custom limits with a result above the default budget, then shows that the envelope JSON conversion succeeds. It also shows the default budget still rejects the same value, and that a hand-built envelope keeps defined behavior.
2. Fix with a non-breaking API shape by preference. For example, add an explicit-limits conversion (`try_to_json_with(&ParseLimits)`) and make the response-derived paths (`to_envelope`/`into_envelope`/`Resolution` usage) able to convert with the originating limits. Do not add required fields that break struct-literal construction of `ResponseEnvelope` unless no sound non-breaking option exists. In that case, stop and report it as an owner decision.
3. Keep the existing error-path context (`results.profile.native`) and the field-presence semantics.
4. Update the affected Rustdoc and `crates/lib/docs` pages (at least `api/response.md` and any guide or concept text that mentions envelope JSON limits) so the docs match the new behavior.
5. Run scoped review rounds over the changed code: logic, responsibility, consistency/docs/comments, and safety.

## Verification process

`cargo fmt --all -- --check`; `cargo test --workspace --all-targets --locked`; `cargo test --workspace --doc --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked`; per-crate `cargo llvm-cov` at or above 70%; and a link check of `crates/lib/docs`. The coordinator re-runs the checks and reviews the diff.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Pending.

## Verification notes

Pending.

## Acceptance notes

No visual acceptance artifact applies; this task finishes at `verified` after coordinator review.
