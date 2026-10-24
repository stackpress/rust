//! Dispatch metadata cannot outlive its immutable table/capture borrow.

use stackpress_lib::Invocation;

fn escape(invocation: Invocation<'_>) -> Invocation<'static> {
    // Returning a static context would outlive borrowed table/captures and must fail lifetime
    // checking.

    invocation
}

fn main() {
    // The intentionally rejected function is type-checked even without runtime invocation.
}
