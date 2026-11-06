//! M0 source parity and ownership proofs, one integration binary.
#[cfg(feature = "auth-server")]
#[path = "main/account_registration.rs"]
mod account_registration;
#[cfg(feature = "auth-server")]
#[path = "main/generated_family.rs"]
mod generated_family;
#[cfg(feature = "server")]
#[path = "main/host.rs"]
mod host;
#[cfg(feature = "native")]
#[path = "main/native.rs"]
mod native;
#[cfg(feature = "auth-server")]
#[path = "main/native_http.rs"]
mod native_http;
#[cfg(feature = "native")]
#[path = "main/storage.rs"]
mod storage;
#[cfg(feature = "native")]
#[path = "main/supervision.rs"]
mod supervision;
#[cfg(feature = "native")]
#[test]
fn compile_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    #[cfg(feature = "sql")]
    tests.compile_fail("tests/ui-sql/*.rs");
}

#[cfg(feature = "sql")]
#[path = "main/actions.rs"]
mod actions;
#[cfg(feature = "native")]
#[path = "main/records.rs"]
mod records;

#[cfg(feature = "auth-server")]
#[path = "main/auth_flows.rs"]
mod auth_flows;

#[cfg(feature = "auth-server")]
#[path = "main/account_flows.rs"]
mod account_flows;

#[cfg(feature = "leptos-ssr")]
#[path = "main/account_ui.rs"]
mod account_ui;

#[cfg(all(feature = "auth-server", not(target_arch = "wasm32")))]
#[path = "main/composite.rs"]
mod composite;
