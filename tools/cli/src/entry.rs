//! Application executable composition: one argv parse, one bounded config file, one build and run.

use std::ffi::OsString;
use std::fs::File;
use std::future::Future;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use stackpress_lib::{Error, ParseLimits, Result};
use stackpress_server::{BuildResult, ConfigLayers, LifecycleStage, Server};

use crate::control::{Control, Level};
use crate::{CliRun, CliSinks, Terminal, diagnostics, run};

/// Supply native argv/output once to an application's selected compiled server builder.
///
/// The builder receives data-only file layers before it runs; it must install those
/// layers on its ServerBuilder. No plugin is loaded dynamically and no prompt runs
/// automatically. Bootstrap acquisition runs on a Tokio blocking worker. Sink I/O remains synchronous.
/// An in-flight OS read is not cancellable; regular-file checks assume cooperative path writers.
///
/// Exit 2 covers usage/config/startup failures, 1 covers execution/output failure,
/// and 130 denotes unrecovered typed prompt cancellation. Successful public error
/// recovery remains exit 0. A failed diagnostic sink is itself an output failure.
///
/// # Examples
/// ```
/// use stackpress_cli::{ConfigLayers, main_with};
/// use stackpress_server::{BuildResult, Server, ServerBuilder};
/// async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
///     let mut app = ServerBuilder::new();
///     app.config_layers(layers)?;
///     // Select the application's compiled plugins here before building.
///     app.build().await
/// }
/// async fn application_entry() -> std::process::ExitCode {
///     main_with(build_application).await
/// }
/// ```
pub async fn main_with<F, Fut>(build: F) -> ExitCode
where
    F: FnOnce(ConfigLayers) -> Fut + Send,
    Fut: Future<Output = BuildResult<Server>> + Send,
{
    // Acquire each process input/output handle once, without holding a stream lock across await.
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (mut stdout, mut stderr) = (io::stdout(), io::stderr());
    main_with_args(&args, &mut CliSinks::new(&mut stdout, &mut stderr), build).await
}

/// Run the executable policy with supplied argv/sinks, excluding the executable name.
///
/// This is the same pipeline used by main_with and a reusable test/embedded entry
/// point. Invalid input/config never invokes the FnOnce builder; valid input calls
/// it exactly once. The selected bootstrap path is opened exactly as supplied and
/// read up to the default ParseLimits byte ceiling plus one detection byte.
///
/// # Examples
/// ```
/// use stackpress_cli::{CliSinks, ConfigLayers, main_with_args};
/// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// use stackpress_server::{BuildResult, Server, ServerBuilder};
/// async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.results(&"hello")?;
///     Ok(Flow::Continue)
/// }
/// async fn application(layers: ConfigLayers) -> BuildResult<Server> {
///     let mut app = ServerBuilder::new();
///     app.config_layers(layers)?;
///     app.on("hello", hello)?;
///     app.build().await
/// }
/// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
/// let args = ["emit".into(), "hello".into()];
/// let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
/// let exit = main_with_args(&args, &mut CliSinks::new(&mut stdout, &mut stderr),
///     application).await;
/// assert_eq!(exit, std::process::ExitCode::SUCCESS);
/// # });
/// # Ok::<(), std::io::Error>(())
/// ```
pub async fn main_with_args<F, Fut>(
    args: &[OsString],
    sinks: &mut CliSinks<'_>,
    build: F,
) -> ExitCode
where
    F: FnOnce(ConfigLayers) -> Fut + Send,
    Fut: Future<Output = BuildResult<Server>> + Send,
{
    // Parse supplied argv exactly once before configuration or application construction can run.
    let terminal = match Terminal::parse_os(args, ParseLimits::default()) {
        Ok(terminal) => terminal,
        Err(error) => {
            // No checked terminal exists yet, so usage never guesses verbosity by reparsing argv.
            return finish(
                sinks,
                &Control::new("", Default::default()),
                2,
                &error.to_string(),
            );
        }
    };
    // Keep presentation/operator settings separate from the target's parsed data namespace.
    let control = terminal.control();
    let verbose = terminal.options().verbose;
    let layers = match configuration_async(terminal.options().bootstrap.clone()).await {
        Ok(layers) => layers,
        Err(error) => {
            // Reject a bad explicit file before any compiled builder callback can run.
            let message = detail("bootstrap configuration failed", &error, verbose);
            return finish(sinks, &control, 2, &message);
        }
    };
    // All file parsing is complete before consuming FnOnce; cancellation drops this owned build.
    let app = match build(layers).await {
        Ok(app) => app,
        Err(error) => {
            // Startup never publishes a partial Server; typed cancellation is a distinct exit.
            let code = if diagnostics::prompt_aborted(&error) {
                130
            } else {
                2
            };
            let message = detail("application startup failed", &error, verbose);
            return finish(sinks, &control, code, &message);
        }
    };
    // Move the same parsed Terminal into one run; no event reads or reparses process argv.
    match run(&app, terminal, sinks).await {
        Ok(result) => ExitCode::from(run_exit(&result)),
        Err(error) => {
            // An outer setup/host error has no HandleReport; never fabricate or retry a lifecycle.
            let code = if diagnostics::prompt_aborted(&error) {
                130
            } else {
                1
            };
            let message = detail("command handling could not start", &error, verbose);
            finish(sinks, &control, code, &message)
        }
    }
}

/// Apply executable cancellation policy only after final public and output success are known.
fn run_exit(result: &CliRun) -> u8 {
    // Recovery succeeds without traversing application error sources; output failures stay exit 1.
    if result.exit_code == 0
        || result
            .report
            .failures
            .iter()
            .any(|failure| failure.stage == LifecycleStage::Output)
    {
        return result.exit_code;
    }
    // Typed cancellation is inspected only for an otherwise unrecovered execution failure.
    if result
        .report
        .failures
        .iter()
        .any(|failure| diagnostics::prompt_aborted(failure.error.as_ref()))
    {
        130
    } else {
        result.exit_code
    }
}

/// Move native acquisition off executor threads; cancellation cannot abort an in-flight OS read.
async fn configuration_async(path: Option<PathBuf>) -> Result<ConfigLayers> {
    // Await the blocking pool before consuming the builder; preserve both join and file failures.
    tokio::task::spawn_blocking(move || configuration(path.as_deref()))
        .await
        .map_err(Error::from_source)?
}

/// Load at most one explicitly selected JSON object, retaining normal I/O/UTF-8/parser causes.
fn configuration(path: Option<&Path>) -> Result<ConfigLayers> {
    // No path means a genuinely empty external layer list, leaving application defaults in charge.
    let mut layers = ConfigLayers::new();
    if let Some(path) = path {
        // Take bounds the read itself rather than trusting a racy pre-read file length.
        let limit = ParseLimits::default().max_bytes;
        let mut bytes = Vec::new();
        // Follow input symlinks, but reject devices/FIFOs before open can wait for a writer.
        // Pre/post checks assume cooperative paths; concurrent replacement is not race-free.
        if !std::fs::metadata(path)
            .map_err(Error::from_source)?
            .is_file()
        {
            return Err(Error::invalid("bootstrap source is not a regular file"));
        }
        let file = File::open(path).map_err(Error::from_source)?;
        if !file.metadata().map_err(Error::from_source)?.is_file() {
            return Err(Error::invalid(
                "opened bootstrap source is not a regular file",
            ));
        }
        file.take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::from_source)?;
        if bytes.len() > limit {
            // The extra detection byte rejects excess before UTF-8/JSON allocation or build entry.
            return Err(Error::limit("bootstrap configuration exceeds byte limit"));
        }
        // Invalid encodings are never replaced with another configuration document.
        let text = std::str::from_utf8(&bytes).map_err(Error::from_source)?;
        // A constant printable source label never exposes the path or file contents in diagnostics.
        layers.json("cli-bootstrap", text)?;
    }
    Ok(layers)
}

/// Select concise public startup text or an explicit developer-only source-chain disclosure.
fn detail(message: &str, error: &(dyn std::error::Error + 'static), verbose: bool) -> String {
    // Never format an underlying build/config cause unless the checked operator selected -v.
    if verbose {
        format!("{message}: {}", diagnostics::developer(error))
    } else {
        message.into()
    }
}

/// Report one pre-run error, returning output failure if even stderr cannot be delivered.
fn finish(sinks: &mut CliSinks<'_>, control: &Control, code: u8, message: &str) -> ExitCode {
    // stdout is untouched for usage/config/build failures; diagnostics are attempted exactly once.
    let code = if diagnostics::write(control, sinks.stderr, Level::Error, message).is_ok() {
        code
    } else {
        1
    };
    ExitCode::from(code)
}
