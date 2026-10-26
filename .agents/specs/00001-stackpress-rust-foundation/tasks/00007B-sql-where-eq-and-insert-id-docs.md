# Task 00007B: Consistent where_eq and current insert_id Rustdoc

Side quest of [task 00007A](00007A-sql-docs.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner approved it on 2026-10-26 ("please do side tasks"), on the coordinator's recommendation that `where_eq` append everywhere.

## Task summary

The SQL docs review found two defects in `stackpress-sql`:

1. **`Select::where_eq` replaces its equality slot**, while `Update::where_eq` and `Delete::where_eq` append a condition. A second `where_eq` on a Select silently drops the first filter. Make Select append, like Update and Delete. Multiple equalities combine with AND, in call order. This is a deliberate, owner-approved public behavior change.
2. **`QueryResult::insert_id` Rustdoc is stale.** It says the adapter never exposes IDs, but the SQLite adapter now returns `Some(id)` for observed rowid transitions, while some successful inserts (for example reused explicit IDs or WITHOUT ROWID tables) still return `None`. Update the Rustdoc to describe the current contract and point callers who need every generated key to RETURNING.

## Implementation steps

1. Write the failing regression first:
   - two `where_eq` calls on a Select render both predicates joined with AND, with binds in call order;
   - Select, Update and Delete behave the same.
2. Change `Select::where_eq` to append. Audit every caller, example, doctest and doc page that relied on replacement, including `crates/sql/sql/docs` (which documents the current difference), `examples/sql-profile`, and the SQLite tests.
3. Update the `QueryResult::insert_id` Rustdoc, and any doc page that repeats the stale claim.
4. Run the full checks.

## Verification process

The coordinator:

- reruns the full workspace checks;
- confirms the regression failed before the fix;
- confirms both doc sets no longer describe the old behavior;
- confirms the sql-profile parity is still byte-identical.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Pending.

## Verification notes

Pending.

## Acceptance notes

Library work has no visual acceptance artifact; this task finishes at `verified` after coordinator review.
