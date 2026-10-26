//! Deliberate compiler rejection: render-local Rc cannot survive a native await.
use std::rc::Rc;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};

async fn bad_page(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let local = Rc::new(String::from("render-local"));
    tokio::task::yield_now().await;
    drop(local);
    Ok(Flow::Continue)
}

fn main() {
    stackpress_server::ServerBuilder::new().get("/bad", bad_page).unwrap();
}
