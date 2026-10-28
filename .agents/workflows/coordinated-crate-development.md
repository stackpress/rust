# Coordinated crate development: owner, coordinator, implementer

Use this workflow to build or extend a crate or slice with three roles: the project owner, a coordinator agent, and an implementer agent run through Paseo. It is the owner's preferred process for most crates. It layers role, gating, review, documentation, and fix discipline over the [Rust programming workflow](rust-programming.md), which still governs how code is implemented, tested, and refactored.

## 1. Roles and ownership

Each role owns different decisions. Keep them separate so safety, breadth, and DX each have an accountable owner.

| Role | Default | Owns |
| --- | --- | --- |
| Owner | Project owner | Product and public-behavior decisions, deviations from upstream, scope, toolchain policy, commits and pushes, and promotion into `.agents/context/`. |
| Coordinator | Claude Opus | Direction drawn from the KB; briefs; review of DX/API ergonomics, breadth against upstream, KB conformance, and docs/comments; independent re-verification; spec tracking; in-scope permission approvals. Does not write implementation code. |
| Implementer | Codex `gpt-6.1-sol`, thinking `xhigh`, mode `auto`, via Paseo | All code, tests, refactor rounds, and documentation drafts. Owns safety, soundness, and Rust best practice with an open-ended mandate and a soundness veto backed by evidence. |

- Owner examples, such as named past failures, are a minimum and never the boundary of a concern. Define responsibilities by mandate, and let the implementer build its own risk inventory.
- When the coordinator disagrees with the implementer, settle it with evidence such as compiler output, a probe, or a test. Escalate to the owner only for genuine product or public-behavior choices, each with a recommendation.

## 2. Ground rules for every session

These apply to every role and every stage.

- Start with `AGENTS.md`, [the context index](../context/index.md), [Rust code style](../context/rust-code-style.md), [testing and quality](../context/testing-and-quality.md), and the relevant upstream references. When the owner asks, review all of `.agents/` first.
- Allowed sources are the current branch, the KB, and upstream sibling checkouts. On a trial branch, never read, diff, or build on other `trial/*` branches.
- Treat existing `target/` output as stale. Run `cargo clean` and `cargo llvm-cov clean --workspace` before the first build of a fresh effort.
- Record both cleanup commands and subsequent Cargo/proof commands through the [build benchmark workflow](build-benchmarks.md). Implementer and coordinator runs use the shared persistent resource store; include record IDs and counter changes in receipts, and never delete that store with target cleanup.
- Commit, push, or publish only on the owner's explicit request. Start listeners only through devmetrics.

## 3. Paseo mechanics

The coordinator runs the implementer through Paseo tools.

1. Discover with `list_profiles`, then `list_providers` and `list_models codex`. Launch with `create_agent` (provider `codex/gpt-6.1-sol`, `settings.thinkingOptionId: "xhigh"`, `modeId: "auto"`, the repository's `workspaceId`).
2. Rely on finish notifications. Don't poll. Use `send_agent_prompt` with `steer` for addenda while the implementer is running.
3. Before approving a `permission-required` request, inspect it. Check new dependency lists against the approved design, and check `git status` and diffs for file changes. Approve crates.io fetches for approved dependencies and file edits inside the brief's scope. Escalate anything else to the owner.
4. When the owner suspects a stall, check `get_agent_status`, `get_agent_activity`, and recent file timestamps before cancelling. Long turns and context compaction are normal.
5. Start a fresh implementer session for each slice, or when the current session's context is heavily used. Hand off through the spec rather than chat history.
6. The owner allows **up to twelve implementer agents at a time**, at the coordinator's discretion (raised from four on 2026-10-27). Run them in parallel only for independent work, such as separate crates, separate docs sets, or a side quest that touches disjoint files. More agents never means less review: start only as many as the coordinator can brief, approve and re-verify promptly.
   - Idle agents that are waiting for a dependency don't count against the limit; running ones do.
   - Watch disk space. Each worktree build can use several GB; cap each agent's target directories and delete them when its stage ends.
   - Give each agent its own brief and spec task.
   - Make file ownership explicit in each brief so no two agents edit the same file.
   - Use Paseo worktree isolation (`create_workspace` with `isolation: "worktree"`) when write sets could overlap. The coordinator integrates the results.
   - Shared files such as the root `Cargo.toml`, `Cargo.lock`, and `src/lib.rs` have a single owner at a time.
   - Each agent's output goes through the same stops and coordinator re-verification. Parallelism never skips a review.

## 4. Spec tracking

Planning records and receipts live in the program spec, `.agents/specs/00001-stackpress-rust-foundation/`, separate from `.agents/context/` until the owner approves promotion. Follow the [spec task implementation workflow](spec-task-implementation.md) status model.

- One task file per milestone or deliverable, with the template sections. The implementer fills in **Implementation notes**. The coordinator fills in **Verification notes** and sets `verified`.
- Library work normally has `Acceptance criteria: none` and finishes at `verified`. UI work defines visual acceptance criteria.
- Work discovered inside a task becomes a side quest numbered with the parent plus a letter (`00004A`). It stays `proposed` until the owner approves its scope.
- Renumber only tasks that are still `proposed`. Record the reason in the sprint plan.
- Design checkpoints and receipts are moved into spec files losslessly, with link rebasing and a line-coverage check, within the Agent File line caps.

## 5. Stage gates

Each crate or slice moves through these gates in order. The coordinator stops the implementer at each marked stop and re-verifies before releasing the next stage.

### 5.1. Brief and design checkpoint (no code)

The coordinator writes a brief containing scope, KB pointers, recommended directions the implementer may challenge, explicit DX goals, and rules. The first deliverable is a design document, not code. It must contain:

- an upstream inventory mapping every relevant export, type, and test behavior to implement now, a named later slice, or intentionally not ported, each with a reason;
- a public API sketch with usage examples;
- the implementer's own risk inventory, with the prevention and the proving test for each risk;
- open decisions, each with a recommendation;
- a test and verification plan, including how deadlock-class tests fail fast.

**Stop.** The coordinator reviews it against the KB and DX goals. Verify risky claims with a disposable probe outside the repository (for example under `/tmp`). Bring owner decisions as structured questions with a recommended option. Return approved changes as a milestone plan.

### 5.2. M0: scaffold and riskiest proof

Create only crates that will hold real code. Prove the riskiest contract first, such as a signature, `Send` bound, or ownership model, including compile-fail cases. **Stop** for coordinator re-verification.

### 5.3. M1–Mn: implementation milestones

Implement in order. Each milestone's tests pass before the next begins. Replace any proof fixtures with production types and re-run their proofs. **Stop before refactor rounds** with test results and per-crate coverage.

### 5.4. Refactor rounds and closeout

Run the rounds in the [Rust programming workflow](rust-programming.md), plus a dedicated safety and soundness round, carrying the coordinator's review findings into them. Close with the [receipt template](../references/00037-refactoring-round-checklists.md) extended for the safety round. The coordinator then re-verifies and may request targeted follow-up rounds while material findings remain.

## 6. Coordinator review checklist

At each stop, the coordinator re-runs the evidence rather than trusting the report, then reviews what checks can't see.

1. **Re-run:** `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --locked`, `cargo test --workspace --doc --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked`, the per-crate `cargo llvm-cov` command from the receipt (at least 70% for development), and a normal-edge `cargo tree`.
2. **Measure with quick scripts** (private review signals, not targets):
   - inline-comment density per file;
   - functions without Rustdoc;
   - functions over about 45 lines;
   - lines over 100 columns;
   - compressed doc-example formatting such as missing comma spacing.
3. **Read:**
   - the crate-level example and the bootstrap or integration example, for DX;
   - the public method list, for naming consistency;
   - hot paths, for hidden costs such as whole-structure clones per write;
   - breadth against the design's upstream inventory.
4. **Send findings** split into material items (must fix in the rounds) and DX polish (implementer's judgment). Name each file and function with the reason.

## 7. Crate documentation

After a crate's closeout, document it in `crates/<crate>/docs/` with the installed `chrisai-docing` skill. The implementer writes; the coordinator reviews.

- Structure:
  - `README.md` (start here and reading paths);
  - `getting-started.md` (quick start to a visible first success, then what happened);
  - `concepts.md` (mental model);
  - `guides/` (one page per task, by reader intent);
  - `api/` (an index plus one lookup page per public surface).
  - Keep guides and reference pages separate.
- Examples are plain fenced Markdown (owner decision), with named `async fn` handlers first. The implementer compile-checks them in a disposable crate outside the repository and documents only claims verifiable from source and tests.
- The implementer reports API rough edges as DX observations without changing code.
- The coordinator review covers:
  - a mechanical scan: one H1 per page, tagged fences, no heading directly above a lower-level heading, no double blank lines, and all relative links resolving;
  - a full read of the entry and concept pages plus a sample guide;
  - an API-page structure sample;
  - spot checks of concrete claims against source;
  - confirmation that no source, tests, or manifests changed.

## 8. Discovery and fixes

Findings from reviews and documentation flow back as tracked, scoped work.

1. The coordinator classifies each observation as a **confirmed defect** (contradicts an accepted decision or an earlier fix, with no product choice involved) or a **behavior choice** (changes public semantics).
2. The owner decides scope, with a recommendation. A typical recommendation fixes confirmed defects now and leaves behavior choices as documented current behavior.
3. Approved work becomes a side quest task with an explicit out-of-scope list.
4. The implementer writes a failing regression test first and prefers non-breaking fixes. If only a breaking change is sound, it stops and reports for an owner decision.
5. The implementer updates the affected Rustdoc and crate docs, then runs scoped review rounds and the full checks.
6. The coordinator re-verifies the diff and checks, then records the verification.

## 9. Communicating with the owner

Report at each stop, saying what passed (with numbers), what is being fixed, and what needs a decision. Lead with the decision when one is needed. Present it with options and a recommendation, and treat the owner's answer as final for that stage. State plainly when something was not verified, such as a disposable example crate the coordinator did not rerun.
