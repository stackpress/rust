//! Construction, runtime and native output owners do not silently alias via Clone.
use stackpress_server::{Server, ServerBuilder, OutputGuard};
fn duplicate(server: Server) {
    let _ = server.clone();
    let _ = ServerBuilder::new().clone();
    let _ = OutputGuard::default().clone();
}
fn main() {}
