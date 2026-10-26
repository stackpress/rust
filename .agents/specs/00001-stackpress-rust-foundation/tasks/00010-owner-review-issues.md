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

The SQL and framework parts of #4, #6 and #7 follow in a short third pass after both lanes merge, so they don't collide with tasks 00008A and 00009.

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

Pending.

## Verification notes

Pending.

## Acceptance notes

Library work has no visual acceptance artifact; this task finishes at `verified` after coordinator review.
