//! Parallel dispatch requires two distinct mutable request/response pairs.

use stackpress_lib::{EventEmitter, Request, Response};

async fn simultaneous(emitter: &EventEmitter, req: &mut Request, res: &mut Response) {
    // Keeping the first future alive retains both exclusive borrows until await; the second borrow
    // must fail.

    let first = emitter.emit("first", req, res);
    let second = emitter.emit("second", req, res);
    let _ = first.await;
    let _ = second.await;
}

fn main() {
    // Compile the invalid borrow family without a runtime executor or listener.
}
