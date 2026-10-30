//! An unselected session facade feature contributes no active session package or native host.

/// The default facade remains foundation-only even though an opt-in session feature exists.
fn main() {
    let _request = stackpress::Request::new();
    #[cfg(feature = "session")]
    let _config = stackpress::framework_session::SessionConfig::default();
    println!("slice9a disabled consumer passed");
}
