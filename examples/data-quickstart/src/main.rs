//! Read the explicitly installed demo file; this executable starts no listener.

/// Accept one local SQLite path and print the trusted synthetic example result.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next().filter(|_| args.len() == 0) else {
        eprintln!("usage: data-quickstart <existing-sqlite-file>");
        return std::process::ExitCode::FAILURE;
    };
    // The library owns awaited transaction/pool cleanup; no install or migration is implicit.
    match data_quickstart::queries::open_and_read(std::path::Path::new(&path)).await {
        Ok(report) => {
            println!("{report}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
