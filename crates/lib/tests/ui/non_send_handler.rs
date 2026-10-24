//! Reject a non-Send returned future against the real production registration API.
use stackpress_lib::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
/// The function item is Send/Sync, but its future holds an Rc across suspension.
async fn non_send(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Retain Rc across a suspension so the returned future, despite its Send function item, must be
    // rejected.

    let retained = std::rc::Rc::new(());
    std::future::pending::<()>().await;
    drop(retained);
    Ok(Flow::Continue)
}
/// The production Handler bound must reject this callback.
fn main() {
    // Register the actual named handler against production bounds; an unrelated compiler error is
    // insufficient proof.

    let mut builder = EmitterBuilder::new();
    let _ = builder.on("invalid", non_send);
}
