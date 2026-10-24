# Task 00006: Slice 3, Idea parser, transformer, and generation

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-24 ("do both in parallel", alongside [task 00005A](00005A-server-and-cli-docs.md)). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Bring the Idea language into the Rust monorepo under the accepted structure, as three crates:

- `crates/idea/idea-parser` (`stackpress-idea-parser`): grammar, AST, compiled schema, diagnostics;
- `crates/idea/idea-transformer` (`stackpress-idea-transformer`): `use` composition and ordered transform plugins;
- `crates/idea/idea` (`stackpress-idea`): the standalone convenience wrapper.

Then prove generation before compilation with one model. Per the KB, first **assess the existing Rust parser** on Idea branch `feature/rust-parser-2` (pinned `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`, `packages/idea-rust/crates/idea_parser_core`) against the current fixtures before writing anything new. Keep its Node/N-API distribution concerns out of the Cargo crates.

This slice resolves the carried Gaps Q08 (parser parity and extraction strategy) and Q09 (generator packaging, order, output cleanup and ownership, build/runtime consistency) in [decisions](../decisions.md). It covers the Slice 1 deferrals assigned to it: host filesystem (`FileLoader`/`NodeFS` capability) and the template engine. Its target proofs from the `00006` reference are item 1 (one Idea model produces compilable public types and an operation registration) and item 10 (repeat generation is stable and the generated code compiles).

## Implementation steps

1. Design checkpoint, with no production code. Write it as spec files `slice-3-design-*.md`. It covers:
   - parser assessment evidence, built and tested from a read-only extraction outside the repository;
   - the upstream inventory (Idea parser, transformer, language catalog, Stackpress schema/generation boundaries, and the Slice 1 deferrals);
   - the API sketch;
   - the risk inventory;
   - open decisions with recommendations;
   - the milestone plan with lane ownership.
2. Coordinator review and owner decisions, then M0, the riskiest proof.
3. Milestones, a stop before rounds, rounds 1–4, and closeout.
4. Docs as a follow-up task.

## Verification process

As defined in the coordinated workflow: coordinator re-verification at every stop, including generated-code compilation, clean and repeat generation, and per-crate coverage of at least 70%.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Pending.

## Verification notes

Pending.

## Acceptance notes

Library and generator work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
