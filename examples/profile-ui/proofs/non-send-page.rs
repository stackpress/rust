//! Deliberate rejection at the real PageSpec loader boundary.
use std::rc::Rc;
use stackpress_framework_view::{PageLoad, PageSpec};
use stackpress_lib::{Invocation, Request, Response, Result};

/// A render-local Rc cannot survive a native loader suspension.
async fn bad_page(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<PageLoad<String>> {
    let local = Rc::new(String::from("render-local"));
    tokio::task::yield_now().await;
    Ok(PageLoad::Ready((*local).clone()))
}

/// Registration checks the actual returned future without an unsafe Send wrapper.
fn main() {
    let _ = PageSpec::get("/bad", "bad", bad_page);
}
