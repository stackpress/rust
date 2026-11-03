//! An application cannot change Session.secret from String to Integer.
#[allow(dead_code)]
#[path = "native/session_secret.rs"]
mod generated;
#[path = "../main/source_models/support.rs"]
mod support;
fn main() {}
