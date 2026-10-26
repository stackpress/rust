//! One app-owned CLI entry using the shared Stackpress argv/config/output pipeline.

use std::process::ExitCode;

/// Run one named capability. Shutdown uses RAII; embedding callers can await Engine::close.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    stackpress_cli::main_with(stackpress_example_sql_profile::build_application).await
}
