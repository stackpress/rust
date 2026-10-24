//! A typed context reference cannot escape its actual invocation borrow.
use stackpress_lib::Invocation;

/// Deliberately request a lifetime longer than the owner permits.
fn escape(inv: Invocation<'_>) -> &'static String {
    // Any requires a static payload type, not a static reference to that payload.
    inv.context::<String>().expect("context")
}

fn main() {
    // Rust checks the rejected function even without a call.
}
