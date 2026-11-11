use std::rc::Rc;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt};
use stackpress_sql::Engine;

async fn local(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let local = Rc::new(7);
    let _engine = inv.service::<Engine>()?;
    tokio::task::yield_now().await;
    assert_eq!(*local, 7);
    Ok(Flow::Continue)
}

fn main() { ServerBuilder::new().on("sql", local).unwrap(); }
