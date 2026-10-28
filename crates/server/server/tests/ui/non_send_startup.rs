//! A named startup function must return Send work across suspension.
use std::rc::Rc;
use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
async fn configure(_: &mut StartupContext) -> Result<Flow> {
    let value = Rc::new(());
    std::future::pending::<()>().await;
    drop(value);
    Ok(Flow::Continue)
}
fn main() { ServerBuilder::new().on_config(configure).unwrap(); }
