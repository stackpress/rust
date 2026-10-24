# Slice 3 open decisions and recommendations

Design checkpoint ledger for [task 00006](tasks/00006-slice-3-idea.md). Load for Q08/Q09 recommendations, scope deviations and the decisions required before M0; this file proposes answers without changing the accepted G01–G17 ledger or KB.

## Decision status

The implementer has delivered design/evidence only. The coordinator reviews breadth, ergonomics, dependencies and receipts; the owner decides public behavior/scope. [G01–G17](decisions.md) remain binding. The unchanged-parser reuse veto is evidence-backed: unbounded nesting aborts instead of returning a recoverable error. No production implementation is released by this checkpoint.

| ID / owner | Question | Recommendation and alternative |
| --- | --- | --- |
| S3-D01 / owner, Q08 | Extract, depend on, or replace the pinned parser? | **Extract and repair** core source with provenance/licenses, typed Rust AST, compatible compiled schema and the accepted workspace policy. Fix boundedness, ordering, env and spans before broad use. Alternative: upstream a repair then pin it, at the cost of cross-repo coordination. A fresh rewrite has no evidence-based benefit. Unchanged reuse is vetoed. |
| S3-D02 / owner, Q08 | Which accidental parser behaviors should Rust preserve? | Approve the explicit corrected grammar in [findings](slice-3-design-grammar.md): capital declarations/uppercase enum keys; lowercase dotted attributes; exactly T/T?/T[]/T[]?; duplicate member names error with related spans; finite numbers/exact 64-bit integer range; missing env error with explicit Empty compatibility; prior references in attributes; env in all scalar positions; malformed EOF errors. Preserve raw quoted strings for now. Alternative: retain selected TS last-wins/casing/modifier quirks explicitly. Neither ESTree AST JSON nor bug-for-bug syntax compatibility is recommended. |
| S3-D03 / owner, Q08/Q09 | Does `!` protect the local declaration or allow imported replacement? | **Keep immutable locals unchanged**, matching catalog intent. Current upstream source replaces them. Adopt the documented merge/order table, importer-relative resolution, cycle errors and no child-plugin activation. Alternative: reproduce source replacement deliberately, with tests/documentation. This is a behavior choice, not a silent bug fix. |
| S3-D04 / owner, Q09 | How does generation bootstrap and how are plugins selected? | **Independent app-owned pre-build codegen tool** via existing `emit generate`, inspectable app source, function registrations in listed order; root plugin-block order must agree. Shared selection/revision checks catch mismatched generated runtime selection; `--check` gates schema freshness. Defer build.rs/OUT_DIR mode. Alternative: OUT_DIR-only generation, with less inspectable app output and a separate adapter proof. A final app compiling its own first generator is rejected as a bootstrap cycle. |
| S3-D05 / owner, Q01/Q09 | What does the first generated proof own? | **New generated-profile example**, using Profile's id/name Strings and a deliberately example-specific in-memory operation generator. Generate public contracts, operation/route registration and readable module registry; prove CLI/direct/HTTP equality, removal→404, byte stability and clean bootstrap. No framework-schema crate now. Alternative: start that full semantic layer now, requiring a new design/scope milestone before its public APIs. |
| S3-D06 / owner, scope | What do Slice 1 filesystem/template deferrals require now? | **Implement native source resolution and owned output IO in transformer; defer general TemplateEngine/helpers/render** to a separately approved task when an actual template grammar is required. Typed emitters/loops suffice for this proof. Generic streams/runtime serving remain the native streaming follow-up. Alternative: include a real general template engine now, but first choose grammar, escaping, helper and error/limit contracts with meaningful tests. Do not fabricate APIs just to mark an inventory row complete. |
| S3-D07 / owner + implementer safety review, Q09 | What output/cleanup promises are acceptable? | **Dedicated marked root, explicit per-plugin partitions, aggregate owner, no manual-file overwrite, cooperative single writer, staged/recoverable install**. Remove only promised owned outputs; no crash-atomic/durability or uncooperative-writer guarantee. Approve this bounded contract; alternative requires platform-specific atomic/exclusive ownership tooling and a larger proof. |
| S3-D08 / coordinator within dependency authority | Is the dependency/feature budget acceptable? | Approve only the [budget](slice-3-design-generation.md): Serde/JSON/order support, tempfile host staging, SHA-256 owned-file drift checks and existing CLI/server/tokio tool adapter. No runtime default edge or new toolchain. Added dependency beyond this list returns for review. |

S3-D02 is a grouped compatibility decision with explicit subchoices, not an assumption that silence accepts every correction. Coordinator can ask the owner one short decision at a time and record any exception verbatim. Defaults become implementation authority only after acceptance is recorded.

## Q08 answer carried for review

Evidence: own tests 3/3; fixture parse/compile 10/10 per identical corpus; exact compiled schema/final goldens pass; AST 4/9 excluding spans; current TS finalization confirms the three standalone unresolved-reference errors. Known gaps include env-in-enum, plugin-order loss, modifiers/casing/duplicate behavior, invalid metadata acceptance, ambient env, missing/invalid spans, docs/style and stack exhaustion. See [assessment](slice-3-design-assessment.md) and [grammar](slice-3-design-grammar.md) rather than treating this brief as a parity certificate.

Recommend source extraction and repair, with typed AST/source semantics rather than preserving ESTree's incidental fields. Q08 stays **unresolved in the authoritative spec ledger** until the coordinator/owner accept the policy and M0 proves it. No parser extraction has been performed in the repo.

## Q09 answer carried for review

Generation before compilation runs in a separate app codegen package using a named host event. Generator crates compile independently of final applications. Registration order, prerequisite validation, output ownership and source inputs are explicit. Selection/revision checks and a no-write regeneration check have distinct jobs; a plain app build is not a freshness check. The output writer promises cooperative ownership and recovery, not imaginary portable atomic replacement.

Public contracts and server runtime output are separate modules; neither ships generator tooling to the app/browser. Shared aggregate modules have one writer. Removed model/plugin files and registrations disappear only inside declared owned output. Framework metadata meanings, SQL/UI/admin/AI generation and revision history retain their later semantic owners.

Q09 stays **unresolved in the authoritative spec ledger** until decisions and M0 results are accepted. The full contract and limitation wording are in [generation](slice-3-design-generation.md).

## Carried issues that do not block this checkpoint

- Full framework-schema public lenses/dictionaries/generated assertions: later task, not inferred from the proof.
- General templating, streaming/native files, build.rs integration and editor/WASM parser support: separate designs/proofs, contingent on scope approval.
- First SQL backend/serverless (Q06), Leptos/Tauri UI and local execution (Q02/Q11/Q12), publication/catalog/version policy (Q13), broader target matrix (Q15): unchanged later owners.
- Rust provenance/license retention needs inspection of applicable upstream LICENSE/NOTICE when extracting; no publishing authorization follows.
- Exactly sized parser/import/output budgets are implementer technical choices within the approved boundedness contract, frozen by M0 boundary tests. A raised hard nesting ceiling requires re-proving resource safety.

## Checkpoint coverage and next gate

Required A is covered by [assessment](slice-3-design-assessment.md) and [grammar](slice-3-design-grammar.md); B by [inventory](slice-3-design-inventory.md); C by [API](slice-3-design-api.md) and [generation](slice-3-design-generation.md); D/F by [risks and milestones](slice-3-design-verification.md); E by this ledger. Every file has one H1 and load guidance from the spec index. Task notes point here; verification notes/status decisions remain coordinator-owned.

Next: coordinator review and owner decisions, then release **M0 only**. The stop is required by [coordinated crate development](../../workflows/coordinated-crate-development.md): “Brief and design checkpoint (no code)” ends with “Stop.” No production code should be added while the checkpoint is pending.
