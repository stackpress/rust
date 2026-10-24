//! Non-Send/non-Sync services cannot enter the host registry.
use std::{rc::Rc, sync::Arc};
use stackpress_server::StartupContext;
fn insert(start: &mut StartupContext) {
    start.register(Arc::new(Rc::new(()))).unwrap();
}
fn main() {}
