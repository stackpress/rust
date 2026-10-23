# Rustdoc and detailed comment examples

Owner: [Rust code style](../context/rust-code-style.md). Load when judging documentation density or writing stateful Rust logic. Examples below are standalone illustrative Rust, not proposed released Stackpress APIs.

## Explain public behavior and inner sequence

```rust
/// Describes whether dispatch should continue to the next listener.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dispatch {
    /// Run the next listener after the current one has completed.
    Continue,
    /// Stop the remaining listeners without undoing completed mutations.
    Abort,
}

/// Returns whether another listener may run after this dispatch result.
///
/// Aborting is cooperative: the caller must honor this result before starting
/// another listener. Work already performed by the current listener remains.
pub fn should_continue(result: Dispatch) -> bool {
    // Match the semantic result explicitly so adding a state requires review.
    match result {
        // Continue only when the completed listener permits the next step.
        Dispatch::Continue => true,
        // Preserve the abort signal rather than treating it as an error/retry.
        Dispatch::Abort => false,
    }
}
```

This example explains declaration purpose, mutation semantics, condition, and consequence. It does not require commenting every brace or repeating every Rust keyword.

## Explain conditions and consequences separately

```rust
/// Tracks how many attempts may still be made for a fallible operation.
pub struct RetryBudget {
    /// Number of future attempts, reduced before an attempt is started.
    remaining: usize,
}

impl RetryBudget {
    /// Creates a budget with the specified number of permitted attempts.
    pub fn new(attempts: usize) -> Self {
        // Store the limit as remaining attempts so exhaustion is explicit.
        Self { remaining: attempts }
    }

    /// Reserves one attempt, returning false when the budget is exhausted.
    ///
    /// The reservation is consumed immediately; this method does not refund
    /// attempts after success, failure, cancellation, or a caller abandoning work.
    pub fn reserve(&mut self) -> bool {
        // Check exhaustion before subtraction to prevent unsigned underflow.
        if self.remaining == 0 {
            // Leave state unchanged because no attempt can be reserved.
            return false;
        }

        // Reserve before the caller starts work, including work that may fail.
        self.remaining -= 1;

        // Signal that one attempt has been consumed and may now begin.
        true
    }
}
```

The comments identify the state invariant and the ordering decision. If a later implementation refunds cancellation, the Rustdoc and tests must change with that behavior.

## Documentation checklist

- Crate/module: responsibility, entrypoints, callers, feature/target boundaries.
- Type/field/variant: meaning, invariants, ownership, when state changes.
- Function/method: purpose, inputs/output, mutation/side effects, error/panic contract, order/cancellation, example where useful.
- Logical block: why this phase exists and what it establishes for the next phase.
- Branch: condition and resulting action/returned state, especially denial/fallback/default behavior.
- Loop/iterator: traversal order, accumulation/state, early termination, and why repeated work is needed.
- Async step: dependency on completion, lock/borrow lifetime, error mapping, cancellation, task context.
- Serialization: public field selection, representation, missing/null handling, target visibility.
- Unsafe: local safety obligations and why callers uphold them.
- Test: setup/action/observable outcome when the scenario is not immediately obvious.

## Keep comments useful

Good explanations answer why a particular order, mutation, conversion, or boundary matters. Short local narration is welcome, especially when it helps a human read the function methodically. Avoid replacing several meaningful block comments with one generic summary.

Do not add tautological comments such as "increment i" above an obvious increment without explaining the counted state. Do not delete explanatory density merely because a reviewer prefers sparse comments. The project owner deliberately requested detailed comments and Rustdoc.

Comments and Rustdoc are part of the maintained contract. Remove obsolete commented-out code, inaccurate guarantees, and references to old APIs. Keep examples compiling or explicitly label pseudocode.

## Relationship to chrisai-coding

The installed [commenting reference](../skills/chrisai-coding/references/typescript-commenting-style.md) requires comments by default, one per logical block, denser around branches/state, and separate condition/consequence explanations. The [declaration reference](../skills/chrisai-coding/references/typescript-jsdoc-and-declaration-comments.md) requires function/method documentation and state/property explanation.

For Rust use `///`, `//!`, conventional `// ` spacing, normal Rust naming, and relevant Rustdoc sections. Do not port JS comment-spacing, class-per-file, JSDoc syntax, or import formatting rules literally.
