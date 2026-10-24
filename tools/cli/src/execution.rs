//! One top-level CLI invocation and adapter-owned output authority, separate from process startup.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use stackpress_lib::{DeliveryState, Error, Request, Response, Result};
use stackpress_server::{HandleOptions, HandleReport, OutputGuard, Server};

use crate::control::Level;
use crate::{Terminal, diagnostics};

/// Caller-owned machine/human sinks; no cloning, global locks or stdin ownership.
///
/// Synchronous writes can block and are not implicitly cancellable. Auto color remains
/// plain because an arbitrary Write object cannot prove native terminal identity.
pub struct CliSinks<'a> {
    /// Exactly one prepared JSON envelope/newline for eligible native output.
    pub stdout: &'a mut (dyn Write + Send),
    /// Human diagnostics only, separate from machine output.
    pub stderr: &'a mut (dyn Write + Send),
}

impl<'a> CliSinks<'a> {
    /// Borrow both sinks without acquiring stream locks or starting background work.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::CliSinks;
    /// let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    /// let sinks = CliSinks::new(&mut stdout, &mut stderr);
    /// ```
    pub fn new(stdout: &'a mut (dyn Write + Send), stderr: &'a mut (dyn Write + Send)) -> Self {
        // Mutable borrows retain exclusive writer ownership across the one execution future.
        Self { stdout, stderr }
    }
}

/// Trusted operator metadata inherited by nested calls through ServerInvocationExt.
///
/// This value contains no I/O handles and is never inserted into Request.data.
/// Force is metadata, not implicit authorization for destructive operations.
///
/// # Examples
/// ```
/// use stackpress_cli::CliContext;
/// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// use stackpress_server::ServerInvocationExt;
/// async fn operation(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
///     let verbose = inv.adapter_context::<CliContext>().is_some_and(|ctx| ctx.verbose);
///     res.results(&verbose)?;
///     Ok(Flow::Continue)
/// }
/// ```
#[derive(Debug)]
pub struct CliContext {
    /// Explicit operator request for developer diagnostics.
    pub verbose: bool,
    /// Request-local operator intent; capabilities decide its effect explicitly.
    pub force: bool,
    /// The explicitly selected bootstrap data path, if any.
    pub config: Option<PathBuf>,
    /// Application-owned label for human output only.
    pub brand: String,
}

/// Final adapter result; diagnostic history survives deliberate successful public recovery.
#[derive(Debug)]
pub struct CliRun {
    /// Zero for final success, one for application/delivery/diagnostic failure.
    /// The executable boundary additionally maps typed prompt cancellation to 130.
    pub exit_code: u8,
    /// Actual host stages, retained causes and native-output observations.
    pub report: HandleReport,
}

/// Handle one parsed event exactly once, delivering only a public envelope to stdout.
///
/// Open output is fully serialized with its owning response's limits before the
/// writer is entered. Stopped output is suppressed. Foreign Sent/Attempted/Failed
/// states are never retried and count as an adapter output failure. Handler-owned
/// external writers remain outside the adapter's private output authority.
///
/// Callback panics propagate; borrowed sinks are never detached. User callbacks
/// and synchronous writers retain their own liveness/resource responsibilities.
///
/// # Errors
/// Returns setup/outer host failures when no report could be obtained. Ordinary
/// lifecycle, serialization and sink failures return CliRun with exit 1 and report.
/// No output failure re-enters lifecycle hooks or starts another stdout attempt.
///
/// # Examples
/// ```
/// use stackpress_cli::{CliSinks, Terminal, run};
/// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// use stackpress_server::ServerBuilder;
/// async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.results(&"hello")?;
///     Ok(Flow::Continue)
/// }
/// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
/// let mut builder = ServerBuilder::new();
/// builder.on("hello", hello)?;
/// let app = builder.build().await?;
/// let args = ["emit", "hello"].map(String::from);
/// let terminal = Terminal::parse(&args, Default::default())?;
/// let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
/// let result = run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr)).await?;
/// assert_eq!(result.exit_code, 0);
/// assert_eq!(serde_json::from_slice::<serde_json::Value>(&stdout)?["results"], "hello");
/// # Ok::<(), Box<dyn std::error::Error>>(()) })?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub async fn run(app: &Server, terminal: Terminal, sinks: &mut CliSinks<'_>) -> Result<CliRun> {
    // Keep formatting and native authority outside the mutable Response handed to capabilities.
    let control = terminal.control();
    let mut guard = OutputGuard::default();
    let (event, data, options) = terminal.into_parts();
    let verbose = options.verbose;
    let context = CliContext {
        verbose,
        force: options.force,
        config: options.bootstrap,
        brand: options.brand,
    };
    // Move the original parsed Nest under its own limits; no argv reread, clone or second parse.
    let mut request = Request::builder()
        // A one-byte synthetic path keeps constant URL scaffolding out of the argv byte budget.
        .url("/")
        .limits(data.limits())
        .data(data)
        .mimetype("terminal/arguments")
        .build()?;
    let mut response = Response::new();
    // Enter once with trusted metadata; no I/O handles or output authority enter the host context.
    let mut report = app
        .handle_with_options(
            &event,
            &mut request,
            &mut response,
            HandleOptions {
                adapter_context: Some(Arc::new(context)),
            },
        )
        .await?;

    // The sole native delivery happens after the completed lifecycle chose its final response.
    deliver(&mut response, &mut report, &mut guard, sinks.stdout).await;
    if verbose || !report.is_success(&response) {
        // Default diagnostics omit payloads; verbose explicitly selects the developer view.
        let message = if verbose {
            let mut detail = report.developer().to_string();
            if report.failures.is_empty()
                && let Some(error) = response.diagnostic()
            {
                // A capability may apply an error value directly without returning a stage failure.
                detail.push_str("; ");
                detail.push_str(&diagnostics::developer(error));
            }
            detail
        } else {
            // Public status gives concise diagnostics without dumping causes, sessions or input.
            match response.status() {
                Some(status) => format!("command failed (status {})", status.code),
                None => "command failed".into(),
            }
        };
        let level = if report.is_success(&response) {
            Level::Info
        } else {
            Level::Error
        };
        if let Err(error) = diagnostics::write(&control, sinks.stderr, level, &message) {
            // A failed diagnostic is terminal even after stdout was sent; stdout is never retried.
            report.record_output_failure(error, &response);
        }
    }
    Ok(CliRun {
        exit_code: u8::from(!report.is_success(&response)),
        report,
    })
}

/// Preflight conversion before one guard-controlled writer; failures retain causes in the report.
async fn deliver(
    response: &mut Response,
    report: &mut HandleReport,
    guard: &mut OutputGuard,
    stdout: &mut (dyn Write + Send),
) {
    let bytes = if response.delivery_state() == DeliveryState::Open {
        match envelope(response) {
            Ok(bytes) => bytes,
            Err(error) => {
                // Conversion failure causes no partial JSON, fallback write or lifecycle retry.
                report.record_output_failure(error, response);
                return;
            }
        }
    } else {
        // Stopped/foreign finalization is handled by the guard without touching the writer.
        Vec::new()
    };
    let result = guard
        .dispatch(response, move |_| async move {
            // write_all may accept a prefix before failing; this one attempt is never restarted.
            stdout.write_all(&bytes).map_err(Error::from_source)?;
            stdout.flush().map_err(Error::from_source)
        })
        .await;
    match result {
        // Guard observations remain attached to the original report; hooks are never re-entered.
        Ok(()) => report.record_output(response),
        Err(error) => report.record_output_failure(error, response),
    }
}

/// Convert public values with the response's limits, then append exactly one machine delimiter.
fn envelope(response: &Response) -> Result<Vec<u8>> {
    // The detached envelope's default limits must not replace the owning response's policy.
    let json = response.try_to_json()?;
    let mut bytes = serde_json::to_vec(&json).map_err(Error::from)?;
    // Escape handling is finished before the sole raw line delimiter is appended.
    bytes.push(b'\n');
    Ok(bytes)
}
