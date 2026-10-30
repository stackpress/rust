//! Application feature selection can omit both framework plugins entirely.

/// The default facade remains usable while optional plugin packages are absent from the graph.
fn main() {
    let _request = stackpress::Request::new();
    println!("slice9b disabled consumer passed");
}
