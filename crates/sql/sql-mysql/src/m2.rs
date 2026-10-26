//! Custom opt-in M2 executable, compiling the same private production modules.
//! Only this test target enables phase gates; no production escape API is added.
#[path = "tests/m2.rs"]
mod cases;
mod collect;
mod database;
mod execute;
mod json;
mod lease;
mod options;
mod probe;
mod reuse;
mod session;
use database::MysqlDatabase;
use options::{MysqlOptions, MysqlTls};

/// Delegate fixture ownership and watchdog policy to the native scenario runner.
fn main() {
    cases::main();
}
