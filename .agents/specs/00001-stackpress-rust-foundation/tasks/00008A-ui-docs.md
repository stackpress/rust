# Task 00008A: UI usage and API docs

Follow-up of [task 00008](00008-slice-5-ui.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner requested it on 2026-10-26, as option A after the Slice 5 rounds.

## Task summary

Write developer documentation for the two UI framework crates, covering how to use them plus an API reference:

- `stackpress-framework-view` in `crates/framework/framework-view/docs/`;
- `stackpress-framework-ui` in `crates/framework/framework-ui/docs/`.

Follow the installed `chrisai-docing` skill and the conventions of tasks [00006A](00006A-idea-docs.md) and [00007A](00007A-sql-docs.md):

- guided-learning pages first, then lookup-oriented reference pages;
- examples as plain Markdown, compile-checked in a disposable consumer outside the repository;
- no source changes;
- API rough edges reported as DX observations.

Use `examples/profile-ui` as the canonical integration example and link to its README rather than copying it.

## Implementation steps

1. framework-view: page IDs, page loads and projected snapshots, script-safe JSON and limits, revision and build checks, public status, routes and the route catalog, the native page extension and its lifecycle (error, Abort, finalization), and assets.
2. framework-ui:
   - the SSR, hydrate and CSR modes and their feature selection;
   - `initialize_ssr` and the owned-Head requirement (reactive SSR Head is deferred);
   - render ownership and cleanup;
   - routing context and base paths;
   - components, state and notifications;
   - language and plural phrases;
   - the single-owner signal-root boundary.
3. Guides:
   - adding a page to a ServerBuilder app;
   - hydrating in the browser;
   - packaging with Tauri (pointing to the example);
   - handling contract mismatch and recovery;
   - translating and pluralizing.
4. Verify every claim against the source, tests, the Slice 5 receipts, and decision G25.

## Verification process

The coordinator:

- reviews the pages against the `chrisai-docing` review gates;
- spot-checks accuracy, especially the feature and mode matrix, the Head and executor requirements, snapshot escaping and limits, and the lifecycle table;
- runs a mechanical format and link scan;
- confirms no source, tests or manifests changed.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Pending.

## Verification notes

Pending.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
