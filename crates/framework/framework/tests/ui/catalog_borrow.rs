use stackpress_framework::{CompiledClient, register_with};
use stackpress_server::ServerBuilder;

fn main() {
    let mut app = ServerBuilder::new();
    let client = CompiledClient { bindings: stackpress_framework::FrameworkBindings { session: None, api: None }, build: "local", models: &[], describe: Vec::new };
    register_with(&mut app, &client).unwrap();
}
