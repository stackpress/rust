//! An invocation's host borrow cannot be extended to static.
use stackpress_lib::Invocation;
use stackpress_server::ServerInvocationExt;
fn escape(inv: Invocation<'_>) -> &'static u64 { inv.service::<u64>().unwrap() }
fn main() {}
