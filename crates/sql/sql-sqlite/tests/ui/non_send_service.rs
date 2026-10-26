use std::rc::Rc;
use std::sync::Arc;
use stackpress_server::StartupContext;
use stackpress_sql::Engine;

struct LocalDatabase { engine: Engine, local: Rc<u8> }

fn register(start: &mut StartupContext, local: LocalDatabase) {
    start.register(Arc::new(local)).unwrap();
}

fn main() {}
