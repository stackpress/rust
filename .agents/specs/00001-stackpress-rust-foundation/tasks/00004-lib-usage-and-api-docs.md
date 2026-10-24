# Task 00004: Lib usage and API docs

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Requested by the owner on 2026-10-24 after Slice 0+1 closeout.

## Task summary

Create `crates/lib/docs/`, developer documentation for `stackpress-lib` covering how to use it and an API reference. Follow the installed `chrisai-docing` skill: guided-learning pages first, then lookup-oriented API reference pages, then formatting. Owner decisions on 2026-10-24: the implementer agent writes and the coordinator reviews; examples are plain Markdown, not wired into doctests.

## Implementation steps

1. Write the guided-learning set: `README.md` (start here and reading paths), `getting-started.md` (a quick start reaching first success), `concepts.md` (mental model), and `guides/` with one page per common task.
2. Write `api/` reference: an index plus one page per major public surface, following the skill's page structure and method standard.
3. Verify every type, method, signature, default, error, and behavioral claim against the current source in `crates/lib/src`.
4. Apply the skill's formatting rules, then its copy-editing workflow.
5. Run the skill's review gates for guided learning, API reference, and formatting.

## Verification process

The coordinator reviews every page against the `chrisai-docing` review gates, checks API accuracy by spot-comparing with the source, checks DX and reading order, and confirms links resolve. The workspace checks (fmt, tests, doctests, Clippy, Rustdoc) must still pass, since no source code should change.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on `trial/3` in `crates/lib/docs/`, using the current Rust source and contract tests as API authority. No Rust source, tests, manifests, other trial branches, commits, publishing, or listeners were changed/used.

- Preserved the proposed 22-page structure: start page, quick start, concepts, seven intent-based guides, API index, and eleven public-surface reference pages. The quick start reaches a printed `Hello, Ada` through a named async handler and plain-data resolve. Guides cross-link to lookup pages and references link back to teaching paths.
- Covered all 243 public inherent/free operations found in `crates/lib/src` with named reference entries, plus public trait adapters, fields, enum variants, constants, conversions, callback bounds, defaults, and error/attempt behavior. Private adaptation traits/helpers are omitted. Standard derived trait behavior is summarized with each owner; DataMap/DataSet remain documented aliases rather than duplicating indexmap's full inherited method catalog.
- Explained accepted design choices: Send returned futures versus static ownership, construction-time mutable registration followed by an immutable runtime table, ordinary deep Clone with explicit SharedNative identity, captures local to Invocation, nested versus top-level lifecycle scope, and one-attempt load/delivery with cancellation and partial-effect limits.
- Examples remain plain fenced Rust Markdown. Named async functions are primary; closure helpers explain Box::pin and lifetime inference. No include_str, doctest wiring, or tracked example harness was added. A disposable consumer at `/private/tmp/stackpress-lib-doc-examples` compiled 263 API blocks (256 executed checks and seven compiled Invocation handler fragments), executed all seven task-guide walkthroughs, and ran the quick start with output `Hello, Ada`. It uses an independent target directory and does not establish untested host/platform support.
- Applied content ownership first (guided learning, then API reference), a copy-edit for terminology/assumptions/transitions, and final Markdown/Rust-snippet formatting. Long reference pages have numbered sections and contents links; short guides use direct task headings. Author review checked first-success sequencing, example explanations, reference method schema, public-only scope, and formatting/link gates.
- Author self-checks passed: `cargo fmt --all -- --check`; `cargo test --workspace --all-targets --locked` (37 passed, two watchdog child entrypoints intentionally ignored in the parent suite); `cargo test --workspace --doc --locked` (263 passed); `cargo clippy --workspace --all-targets --locked -- -D warnings`; warning-clean `cargo doc --workspace --no-deps --locked` with `RUSTDOCFLAGS=-D warnings`. The docs relative-link/anchor scan passed for 196 links. Agent Workspace validation passed with existing warnings only. These are implementation self-checks; coordinator verification remains below.

DX observations, documented without code changes:

- `sent()` includes Stopped, while `result()` excludes Text/Bytes even though envelope conversion exposes those body forms as results. Precise state/representation inspection needs the enum/body APIs.
- A response can accept values under larger custom limits yet fail later envelope JSON conversion because `ResponseEnvelope::try_to_json` uses Value's default limits. Generic serialization of Value also applies its own defaults before a receiving helper's selected policy.
- Optional path reads collapse malformed syntax into absence; typed reads preserve parse errors. DataPath Display can quote keys for clarity but its output is not a general round-trip parser syntax.
- Response convenience setters retain unrelated earlier fields (including errors, body on redirect/error application, and total on set/html/xml/json). Envelope status-only input can select code 0; manually built envelope JSON conversion does not apply Response's metadata-length/count checks.

No additional product decisions were needed. Native transport/lifecycle implementations, platform parity, alternate wire codecs, and dependency method catalogs were not inferred or expanded into this documentation task.

## Verification notes

Coordinator review on 2026-10-24. Result: **verified**.

- Scope: 22 pages and 5,971 lines in `crates/lib/docs/`, in the proposed structure. `git diff` confirms no changes to `crates/lib/src`, tests, `src/`, manifests, or `Cargo.lock`.
- Workspace checks rerun: 37 integration tests and 263 doctests pass.
- Mechanical formatting and link scan across every page: one H1 per page, every fenced block has a language tag, no heading sits directly above a lower-level heading without orientation text, no double blank lines, and every relative link target exists. No issues.
- Guided-learning gates, reviewed by reading `README.md`, `getting-started.md`, `concepts.md`, and `guides/events-and-routing.md` in full:
  - Pages open with the reader's problem rather than "this page is for".
  - The quick start reaches a visible first success (`Hello, Ada`), then explains what happened.
  - The examples are small, and each is explained before the next heading.
  - Named async functions come first; the closure helper is secondary and explains the boxing.
  - Concepts follow first success; pages end with a recap and next steps; reference detail stays out of the main flow.
- API reference gates, reviewed by sampling `api/event.md`:
  - Numbered sections with a contents line, a minimal orienting example, and per-method signatures.
  - Each method has a realistic example, a parameters table, and **Returns** prose including errors.
  - Pages link back to guides.
- Accuracy spot checks against source, all matching:
  - Depth limit is inclusive 1–256 with default 64, and the previous limit is retained on error.
  - `results` stages through `results_value`, selecting `set`'s default OK status and total 1.
  - Captures are not injected into `req.data` during raw dispatch.
  - `inv.resolve` uses a fresh request/response pair.
- Example compilation: per the owner's decision, examples are not doctests. The implementer's disposable out-of-repo consumer compiled 263 API blocks and ran all seven guide walkthroughs plus the quick start. That is implementer evidence; the coordinator did not re-run the disposable crate.
- Confirmed defect from the DX observations: `ResponseEnvelope::try_to_json` converts `errors`/`results` with `Value::to_json()`, which applies `ParseLimits::default()`. A response built with larger custom limits can therefore accept values that its own envelope later rejects. This is the same class of issue Round 4 fixed for Nest and Response JSON text, and that fix missed the envelope path. The docs describe current behavior accurately; the code fix and the other DX observations are proposed as side quest 00004A. The owner approved only the envelope-limits defect as [task 00004A](00004A-envelope-json-limits.md); the other observations remain documented behavior.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow's definition, so this task finishes at `verified` after coordinator review. The owner may still review the pages.
