# Task 00006A: Idea usage and API docs

Follow-up of [task 00006](00006-slice-3-idea.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner requested it on 2026-10-25, after the Slice 3 closeout.

## Task summary

Write developer documentation for the three Idea crates, covering how to use them plus an API reference:

- `stackpress-idea-parser` in `crates/idea/idea-parser/docs/`;
- `stackpress-idea-transformer` in `crates/idea/idea-transformer/docs/`;
- `stackpress-idea` in `crates/idea/idea/docs/`.

Follow the installed `chrisai-docing` skill and the conventions of [task 00004](00004-lib-usage-and-api-docs.md) and [task 00005A](00005A-server-and-cli-docs.md):

- guided-learning pages first, then lookup-oriented reference pages;
- examples as plain Markdown, compile-checked in a disposable consumer outside the repository;
- no source changes;
- API rough edges reported as DX observations.

## Implementation steps

1. Parser docs:
   - `README.md` with reading paths;
   - `getting-started.md`: parse and compile a schema, inspect a model, no IO;
   - `concepts.md`: the Idea language and its stages, then SourceFile, AST, compiled config, typed references versus literal strings, finalization, limits, diagnostics, and JSON import/export;
   - `guides/` by task intent;
   - `api/`: an index plus one page per public surface.
2. Transformer docs, with the same structure:
   - source loaders: native and in-memory;
   - composition: imports, the G19 merge and replacement rules, provenance, limits;
   - function-plugin registration: specs, dependencies, contracts, selection and the G20 plugin-selection table;
   - output plans: file claims, module contributions, aggregates;
   - the owned output workspace: check versus install, the ownership marker, cooperative locking and recovery, and the stated limits.
3. Wrapper docs, scaled to the crate: `generate`, `GenerateRequest`, `GenerationMode`, and how an app-owned codegen tool uses them.
   - Use `examples/generated-profile` as the canonical integration example.
   - Link to its README rather than copying it.
4. Verify every claim against the current source, tests, the Slice 3 receipts and decisions G18–G22.
5. Apply copy-editing and formatting, and run the skill's review gates.

## Verification process

The coordinator:

- reviews the pages against the `chrisai-docing` review gates;
- spot-checks accuracy against the source, especially the G19 merge rules, the G20 selection table, reference-versus-literal behavior, and the writer's recovery and limits;
- runs a mechanical format and link scan;
- confirms that no source, tests or manifests changed.

Workspace checks must still pass.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on 2026-10-25 by two implementers working in parallel in disjoint directories. Their full receipts:

- [parser receipt](00006A-receipt-parser.md): `crates/idea/idea-parser/docs`, 18 pages, 1,767 lines. All 33 Rust and 3 Idea examples were checked in a disposable consumer; the audit covers 34 exported types, 21 functions and methods, 98 fields, 31 variants, and `HARD_DEPTH`.
- [transformer and wrapper receipt](00006A-receipt-transformer.md): `crates/idea/idea-transformer/docs` (21 pages) and `crates/idea/idea/docs` (4 pages). 68 Rust blocks compile and six walkthroughs pass; the audit covers 49 transformer functions and methods, 31 fields, and the wrapper's surface.

**DX observations, no code changes:**

- `finalize` discards `use` directives for compatibility, while `finalize_ast` and typed finalization reject them.
- Plain `SchemaConfig` JSON loses reference intent; `CompiledSchema` has no named-import counterpart.
- `FinalSchema` has no checked JSON method family.
- `install` and `generate(.., Install)` return the pre-install differences, so a first install reports `is_current() == false`.
- `LoadError` has no `From` conversion into `TransformError`.
- The wrapper's aggregate renderer runs before registry preflight.
- Automatic module contributions and an app-owned root `mod.rs` are mutually exclusive.
- The `TransformSpec::owns` Rustdoc calls an empty owner list "contribution-only", but a module contribution requires an owned partition. This is a Rustdoc inaccuracy, kept as a candidate side quest.


## Verification notes

Coordinator review on 2026-10-25. Result: **verified**.

- **Scope:** 43 pages across the three `docs/` directories, plus the two receipts. `git status` shows no tracked changes, so no source, tests, manifests or lockfile changed.
- **Mechanical scan** of all 43 pages and 275 relative links/anchors: one H1 per page, tagged fences, no heading directly above a lower heading, no double blank lines, and every target and anchor resolves. 0 issues. The coordinator's scanner was itself re-validated against the verified lib and server docs.
- **Accuracy spot checks against source:**
  - **Parser:** default limits (1 MiB, depth 64, `HARD_DEPTH`, 100,000 nodes, 10,000 entries); the export list, with no public `compile`; `finalize` dropping `use`.
  - **Reference versus literal:** the coordinator ran the concepts example in a probe. Plain config holds `["${Labels}","${Labels}"]`, typed compilation finds 1 reference, and finalization gives `[{"title":"Profiles"},"${Labels}"]`.
  - **Transformer:**
    - the G19 table matches `compose/merge.rs` (whole-local props/enums; immutable accumulated types and models replaced);
    - the G20 table matches decision G20;
    - the path, plan and contribution limits match `plan/path.rs`, `plan/mod.rs` and `registry.rs`: 240 bytes, 16 components, 1,024 files, 1 MiB, 1,024 entries;
    - the writer limits and recovery order match the lane C receipt.
- **Guided learning:** the parser quick start prints `Profile: id String` with no IO. The wrapper quick start composes an import through `ProjectLoader`, installs, then checks current in a temp directory.
- **Example evidence** comes from the implementers and was not rerun by the coordinator: 101 Rust blocks were compile-checked in disposable consumers outside the repo.


## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
