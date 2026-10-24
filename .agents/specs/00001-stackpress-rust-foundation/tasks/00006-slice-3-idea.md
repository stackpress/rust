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

Design checkpoint prepared on 2026-10-24; start with [Slice 3 decisions and recommendations](../slice-3-design-decisions.md), which routes assessment, inventory, API/generation contracts and risks/milestones. The pinned core was assessed only in `/tmp`; no production code or manifests changed. Unchanged reuse is vetoed on bounded-resource evidence. Q08/Q09 recommendations and disclosed compatibility/template scope choices await coordinator/owner review. Workspace validation passed with existing warnings only; all seven new files are below 250 lines. Verification notes remain coordinator-owned.

## Verification notes

**Design checkpoint review (coordinator, 2026-10-24): approved; M0 released.**

- **Reviewed:** `slice-3-design-decisions.md`, `slice-3-design-api.md`, and the M0 plan in `slice-3-design-verification.md`.
- **Parser evidence:** own tests 3/3; all fixtures parse; compiled goldens match; AST parity 4/9 excluding spans. The unchanged-reuse veto stands on the stack-overflow abort, lost plugin order, env-in-enum rejection, and span defects.
- **DX:**
  - the parser is usable standalone with no IO;
  - plugins are function registrations;
  - output is inspectable;
  - diagnostics carry file, line, and column;
  - the separate codegen binary is justified by the bootstrap cycle and reuses the G14 CLI grammar.
- **Decisions:** owner decisions recorded as G18–G21 and the coordinator dependency approval as G22 in [decisions](../decisions.md). G19 deliberately follows upstream code over the reference 00022 catalog wording.

## Acceptance notes

Library and generator work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
