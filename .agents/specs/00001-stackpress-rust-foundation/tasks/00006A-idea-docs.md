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

Pending.

## Verification notes

Pending.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
