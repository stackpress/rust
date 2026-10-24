//! Application-owned CLI entry; no listener starts here.

use std::process::ExitCode;

/// Execute one CLI command through the existing host lifecycle and await worker shutdown.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    stackpress_cli::main_with(stackpress_example_profile_codegen::build_tool).await
}
