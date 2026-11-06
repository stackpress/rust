//! Native finite command entry; no development listener is bound by this binary.

/// Use the existing CLI parser and `-b` data-layer precedence.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    // Capture OS argv once. The existing CLI pipeline parses it once; the source query adapter
    // also receives its original final token, without reparsing input inside an operation.
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let native = args.clone();
    let (mut stdout, mut stderr) = (std::io::stdout(), std::io::stderr());
    stackpress_cli::main_with_args(
        &args,
        &mut stackpress_cli::CliSinks::new(&mut stdout, &mut stderr),
        move |layers| async move {
            let seed = std::env::var("DATABASE_SEED").unwrap_or_else(|_| "abc123".into());
            let mut argv = vec!["blog".into(), "entry".into()];
            for arg in native {
                argv.push(
                    arg.into_string()
                        .map_err(|_| stackpress_lib::Error::invalid("argv requires UTF-8"))?,
                );
            }
            blog_web::build_storage_with(
                layers,
                &seed,
                stackpress_framework::config::Environment {
                    argv,
                    ..Default::default()
                },
            )
            .await
        },
    )
    .await
}
