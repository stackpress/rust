//! Independent app-owned tool entry; no listener or generated-app dependency.

/// Run one explicit tool capability through the existing CLI lifecycle.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    stackpress_cli::main_with(generated_workspace_tool::build_tool).await
}
