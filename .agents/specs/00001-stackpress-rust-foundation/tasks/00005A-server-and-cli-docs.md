# Task 00005A: Server and CLI usage and API docs

Follow-up of [task 00005](00005-slice-2-server-and-cli.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The Slice 2 design planned it, and the owner accepted it on 2026-10-24, to run in parallel with Slice 3 ([task 00006](00006-slice-3-idea.md)).

## Task summary

Write developer documentation for `stackpress-server` in `crates/server/docs/` and for `stackpress-cli` in `tools/cli/docs/`, covering how to use them plus an API reference. Follow the installed `chrisai-docing` skill and the same conventions as [task 00004](00004-lib-usage-and-api-docs.md):

- guided-learning pages first, then lookup-oriented reference pages;
- examples as plain Markdown, compile-checked in a disposable consumer outside the repository;
- no source changes;
- API rough edges reported as DX observations.

## Implementation steps

1. Server docs:
   - `README.md` with reading paths;
   - `getting-started.md` (a server with a plugin, calling a capability);
   - `concepts.md` (plugins and dependency order, config layering, services, startup phases, the top-level lifecycle and its G12/G17 rules, raw versus top-level dispatch, reports and correlation, the HTTP adapter);
   - `guides/` (by task intent);
   - `api/` (an index plus one page per public surface).
2. CLI docs: the same structure, scaled to the crate. Cover an application-owned `stackpress` binary via `main_with`, operator flags versus capability arguments, the stdout/stderr contract and exit codes, `Terminal`, `Control`, and prompts with the native TTY driver.
3. Verify every claim against source and tests. Use `examples/capability-host` as the canonical integration example.
4. Apply copy-editing and formatting, and run the skill's review gates.

## Verification process

The coordinator reviews the pages against the `chrisai-docing` review gates, spot-checks accuracy against source, runs a mechanical format and link scan, and confirms that no source, tests, or manifests changed. Workspace checks must still pass.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Pending.

## Verification notes

Pending.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
