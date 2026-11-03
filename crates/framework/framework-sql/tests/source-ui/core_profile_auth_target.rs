//! Auth[] cannot be overridden as Application[] even though both payloads are Vec<Record>.
#[allow(dead_code)]
#[path = "native/profile_auth_target.rs"]
mod generated;
#[path = "../main/source_models/support.rs"]
mod support;
fn main() {}
