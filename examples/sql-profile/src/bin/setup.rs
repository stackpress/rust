//! Operator-selected install/seed executable, separate from ordinary application startup.

use std::path::PathBuf;
use std::process::ExitCode;

/// Accept exactly one explicit file path, install once and report failures without raw SQL/data.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // Preserve native path spelling, and refuse extra arguments before opening any file.
    let mut args = std::env::args_os().skip(1);
    let path = match (args.next(), args.next()) {
        (Some(path), None) => PathBuf::from(path),
        _ => {
            eprintln!("usage: sql-profile-setup <database-file>");
            return ExitCode::from(2);
        }
    };
    match stackpress_example_sql_profile::setup::install_and_seed(path).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("profile setup failed: {error}");
            ExitCode::FAILURE
        }
    }
}
