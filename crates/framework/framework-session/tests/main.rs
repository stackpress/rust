//! M0 source parity and ownership proofs, one integration binary.
#[cfg(feature = "server")]
#[path = "main/host.rs"]
mod host;
#[cfg(feature = "native")]
#[path = "main/native.rs"]
mod native;
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
