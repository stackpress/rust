# Task 00010: Owner review issues #1–#13

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner filed GitHub issues stackpress/rust #1–#13 on 2026-10-26 from a review of `trial/3` at `a094418`. Each issue said to defer until `trial/3` merges into `main`. On 2026-10-26 the owner allowed bypassing that guard, and the coordinator chose to do them now: they set error, IO and limit policies that every later crate inherits.

## Task summary

Each issue's acceptance criteria are authoritative. Read the issue with `gh issue view <n>` and recheck its source locations against current `trial/3`; the code has moved since `a094418`. The issues' code sketches are suggestions, not accepted designs.

The work is split into two lanes with disjoint write sets.

**Lane 1: `crates/lib`**

- [ ] #11: error-wrapping policy. Outer errors add distinct context, and the source carries the underlying explanation without duplicating it. Settle this first; lane 2 adopts it.
- [ ] #1: the priority queue does more work than its contract requires.
- [ ] #2: serialization builds an intermediate owned JSON tree.
- [ ] #5: several related booleans represent richer state.
- [ ] Lib parts of #4 (`#[must_use]` on consuming builders) and #7 (`# Errors` docs for fallible APIs).

**Lane 2: `crates/server`, `tools/cli`, `crates/idea/idea-transformer`**

- [ ] #10: configuration errors discard their original causes. Adopt lane 1's #11 policy once it merges; until then use the existing `with_cause`.
- [ ] #3: transformer failures lack a machine-readable category.
- [ ] #8: blocking IO inside an async function (CLI bootstrap read).
- [ ] #9: inconsistent validation across file-reading APIs (`read_source`).
- [ ] #13: resource limits checked after collecting the resource (formatter subprocess).
- [ ] #12: blocking cleanup hidden inside Drop (TTY input).
- [ ] Server, CLI and transformer parts of #4, #6 and #7.

The SQL and framework parts of #4, #6 and #7 follow in a short third pass after both lanes merge, so they don't collide with tasks 00008A and 00009. The third pass also renames the generic `List` component's Profile-specific `data-profile-id` attribute, found in the 00008A docs review. That's a coordinator-approved markup change, and the docs and proofs get updated with it. It also covers two items from task 00009: apply the #11 policy to `examples/profile-ui/web/src/backend.rs` startup errors, and make the profile-ui proof wrapper rebuild or detect a binary built for the wrong backend.

## Rules

- Every behavior fix needs a regression test that fails first. Concurrency, IO and process tests run under the existing watchdog patterns.
- Public API changes are limited to what an issue asks for: #3's additive kind, #4's attributes, and #11's message-composition policy. Anything else comes to the coordinator first.
- G23 sequencing applies: lane 1 owns lib, lane 2 owns server, CLI and the transformer.
- Implementers stay read-only on GitHub. The owner allowed the coordinator to comment on the issues (2026-10-26). After verifying a fix, the coordinator posts a note with the commit, what changed and how it was tested. An issue is closed only once its fix is on `main`, unless the owner says otherwise. Commit messages reference the issue numbers.

## Verification process

The coordinator reruns:

- each issue's acceptance checks;
- workspace fmt, Clippy, tests, doctests, Rustdoc and the 1.96 check;
- per-crate coverage for touched crates.

The coordinator also confirms that the #11 policy is applied consistently across both lanes.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

All 13 issues are implemented in three passes. Each pass has its own receipt:

- [lane 1, lib](00010-receipt-lib.md): #11 (error-wrapping policy), #1, #2, #5, and the lib parts of #4 and #7. Merged as `64a5b70`.
- [lane 2, server, CLI and transformer](00010-receipt-host.md): #10, #3, #8, #9, #13, #12, and these crates' parts of #4, #6 and #7, plus the #11 consistency pass. Merged as `ff7a251`.
- [final pass, SQL and framework](00010-receipt-final.md): the remaining parts of #4, #6 and #7; the `List` `data-profile-id` → `data-item-id` rename; #11 applied to the SQLite profile-UI startup error; proof wrappers that rebuild hosts per backend. Merged as `7031282`.

The one approved observable change is #11's Display text: generic `Error::from_source` now shows `external operation failed` instead of copying the cause. The `data-item-id` rename is the approved markup change.

## Verification notes

Coordinator review on 2026-10-26. Result: **verified**.

- **Regression-first evidence:** each lane recorded every behavior fix failing before the change. Examples:
  - #11: the wrapper Display duplicated its cause;
  - #8: bootstrap blocked a sibling future;
  - #9: a FIFO hung until the watchdog killed it;
  - #4: discarded builder chains compiled without warning.
- **Checks rerun by the coordinator on each lane and on the integrated `trial/3`:**
  - fmt passed; Clippy and Rustdoc `-D warnings` clean; `cargo +1.96.0 check` passed;
  - final integrated tree: **470 workspace tests and 867 doctests** passed;
  - the profile-UI check passed for both backends, with all 16 browser proofs. The SQLite build ran deliberately before the default browser proofs, which pass now that the wrapper rebuilds per backend.
- **Coverage, all above 70%:** lib 93.64%, server 91.61%, CLI 93.15%, transformer 93.62%. In the final-pass crates, the web example reads 75.85% because that run measured cargo tests only; task 00009's 93.24% also counted instrumented browser runs of the web executable.
- **GitHub:** the coordinator posted a verified note on every issue (#1–#13) with commits, changes and evidence. The issues stay open until the fixes reach `main`.

## Acceptance notes

Library work has no visual acceptance artifact; this task finishes at `verified` after coordinator review.
