use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;
async fn bad(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let local = std::rc::Rc::new(1);
    tokio::task::yield_now().await;
    drop(local);
    Ok(Flow::Continue)
}
fn main() { ServerBuilder::new().on("bad", bad).unwrap(); }
