use stackpress_framework::{CompiledClient, register_with};
use stackpress_server::ServerBuilder;

fn main() {
    let mut app = ServerBuilder::new();
    let client = CompiledClient { build: "local", models: &[], describe: Vec::new };
    register_with(&mut app, &client).unwrap();
}
