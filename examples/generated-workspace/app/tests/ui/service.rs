use stackpress_framework_sql::DataSession;
use stackpress_server::StartupContext;
use std::sync::Arc;
fn publish(startup: &mut StartupContext, session: DataSession<'static>) {
    startup.register(Arc::new(session)).unwrap();
}
fn main() {}
