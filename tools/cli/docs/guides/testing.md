# Testing commands with injected sinks

Test the same executable pipeline with supplied OS arguments and buffers. This keeps process streams
untouched while proving JSON output, startup, and exit status together.

## Run the entry pipeline

The application builder below registers a real operation. main_with_args accepts argv without the
executable name, builds once, and returns ExitCode:

```rust
use stackpress_cli::{CliSinks, main_with_args};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}

async fn application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("hello", hello)?;
    app.build().await
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let argv = ["emit".into(), "hello".into()];
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let exit = main_with_args(
        &argv,
        &mut CliSinks::new(&mut stdout, &mut stderr),
        application,
    )
    .await;
    assert_eq!(exit, std::process::ExitCode::SUCCESS);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&stdout)?["results"],
        "hello"
    );
    assert!(stderr.is_empty());
    Ok(())
}
```

The assertion checks the actual machine envelope rather than a handler return alone. The empty
stderr assertion also catches accidental progress output from this non-verbose success path.

## Narrow the seam when useful

Use run with a parsed Terminal and existing Server to inspect CliRun.report and 0/1 policy. Use
main_with_args when usage/bootstrap/startup and cancellation-to-130 mapping matter. In failure
tests, include a writer that accepts a prefix then fails to prove no retry; output failure takes
precedence over prompt cancellation. A foreign response finalization must not masquerade as native
CLI output.

For prompts, inject BufferedInput or a bounded Tokio receiver and PromptCancellation. Test
validation, EOF, and cancellation separately. Real TTY restoration needs an actual PTY process
fixture, as in [the crate tests](../../tests/tty_prompt_contracts.rs); buffer tests cannot prove OS
cleanup.

You have tested a complete command without process-global streams. Use [entry](../api/entry.md) and
[execution](../api/execution.md) for failure contracts, or inspect [canonical application
tests](../../../../examples/capability-host/tests/cli.rs).
