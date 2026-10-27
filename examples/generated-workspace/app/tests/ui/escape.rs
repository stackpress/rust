use stackpress_framework_sql::DataSession;
use stackpress_sql::Engine;
fn escape(engine: Engine) -> DataSession<'static> { DataSession::pooled(&engine) }
fn main() {}
