//! An opaque host context still needs Send and Sync for movable dispatch futures.
use stackpress_lib::{EmitterBuilder, Request, Response};

fn main() {
    // Rc state cannot cross the new borrowed-context boundary.
    let state = std::rc::Rc::new(());
    let emitter = EmitterBuilder::new().build();
    let mut req = Request::new();
    let mut res = Response::new();
    let _ = emitter.emit_with_context("no-listener", &mut req, &mut res, &state);
}
