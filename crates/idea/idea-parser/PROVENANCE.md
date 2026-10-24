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
