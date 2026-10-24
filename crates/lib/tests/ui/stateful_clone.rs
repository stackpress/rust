//! Stateful invocations cannot duplicate native attempt ownership through Clone.

use stackpress_lib::{Request, Response};

fn main() {
    // Both owners contain unique attempt state; neither may acquire an implicit second owner
    // through Clone.

    let _ = Request::new().clone();
    let _ = Response::new().clone();
}
