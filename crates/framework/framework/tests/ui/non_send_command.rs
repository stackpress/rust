use std::rc::Rc;
use stackpress_framework::commands::Bindings;
use stackpress_lib::{Flow, Invocation, Request, Response, handler};

fn main() {
    let captured = Rc::new(1);
    Bindings::default().bind("serve", handler(move |_: &mut Request, _: &mut Response, _: Invocation<'_>| {
        let captured = captured.clone();
        Box::pin(async move { let _ = captured; Ok(Flow::Continue) })
    }));
}
