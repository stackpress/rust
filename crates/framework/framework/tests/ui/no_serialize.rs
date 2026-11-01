//! Native state cannot enter the public SSR/browser seed through serde.
fn serializable<T: serde::Serialize>() {}
fn main() {
    serializable::<stackpress_framework::client::Client>();
    serializable::<stackpress_framework::CompiledClient>();
    serializable::<stackpress_framework::commands::Bindings>();
}
