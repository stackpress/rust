# Idea parser source provenance

Extracted from stackpress/idea commit `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`,
`packages/idea-rust/crates/idea_parser_core`, on 2026-10-24.
Upstream author: Chris <chris@incept.asia>; upstream license: Apache-2.0.
The complete applicable root LICENSE is retained. The pinned tree has no NOTICE file.

The AST, parser, compiler and config implementation derive from that core. Changes:
private bounded AST; explicit environment inputs; ordered config; corrected grammar;
valid byte spans and named-source diagnostics; checked budgets; Rustdoc and comments.
The independent debug tokenizer and Node/N-API packaging are not included in M0.
They do not participate in this generation proof. This is extraction and repair,
not a claim that the upstream core can be used unchanged.

M1 (2026-10-25) adds immutable public syntax ownership, typed environment/reference
provenance, a tokenizer sharing the extracted scanner, duplicate-related spans,
validated ordered JSON and checked export/finalization. The M0 compatibility API
is retained. No Node/N-API distribution source or dependency is introduced.

`tests/fixtures/compiled/*.json` records complete non-final/final outputs from a
disposable copy of the current TypeScript parser at
`cf8e922ba6e57746e15a05c26df0546dda2cc5c9`, with only `DATABASE_URL=test` supplied.
All 21 original pinned/current fixture files were compared byte-for-byte and match.
The three standalone fieldset/model fixtures retain their expected unresolved Text
reference failure. The TypeScript oracle was run outside the upstream checkout;
no upstream build, installation or modification was performed.
