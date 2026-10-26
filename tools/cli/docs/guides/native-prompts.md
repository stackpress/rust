# Using native prompts

A command may run from a terminal or automation. Supply a known answer when available and let
prompt_native decide whether actual terminal input is possible.

## Supply an answer first

Explicit answers use the same validation and rendering pipeline while bypassing stdin ownership.
This works with a pipe or with feature tty disabled:

```rust
use stackpress_cli::prompt::{
    PromptCancellation, PromptOptions, PromptResult, RenderOptions, prompt_native,
};

async fn example() -> PromptResult<()> {
    let mut output = Vec::new();
    let answer = prompt_native(
        &PromptOptions::new("Name?"),
        Some("Ada"),
        &mut output,
        &PromptCancellation::new(),
        RenderOptions::default(),
    )
    .await?;
    assert_eq!(answer, "Ada");
    Ok(())
}
```

Some("") is an explicit empty candidate and replaces any default/prefill. Rejection returns
InvalidAnswer(message); it is not treated as missing input. An already cancelled signal returns
Aborted before input acquisition, callbacks, or rendering.

## Ask on a real terminal

Pass no answer and choose stderr rendering when input should come from a user. This function
requires real terminal stdin unless you configure a default; it is compiled as an example but should
only be called when your application explicitly requests interaction:

```rust
use stackpress_cli::prompt::{
    PromptCancellation, PromptOptions, PromptResult, RenderOptions, prompt_native,
};
use std::io::{self, IsTerminal};

async fn ask_user() -> PromptResult<String> {
    let mut stderr = io::stderr();
    let render = RenderOptions {
        is_terminal: stderr.is_terminal(),
        ..Default::default()
    };
    prompt_native(
        &PromptOptions::new("Name?"),
        None,
        &mut stderr,
        &PromptCancellation::new(),
        render,
    )
    .await
}
```

The wrapper owns and closes TtyInput. Native keys include Enter, Tab, Backspace, Ctrl-C/Esc cancel,
and Ctrl-D EOF; unsupported keys are ignored. Non-TTY input uses the configured default or
immediately returns NonInteractive without a pipe reader. With tty disabled even terminal input
needs a supplied answer/default. Use BufferedInput::lines for text your application already read and
bounded.

## Respect input and output ownership

Only one framework native reader may own stdin. TtyInput uses a joined polling worker, finite queue,
and raw-mode guard. Explicit close exposes cleanup errors; Drop makes a best-effort restoration and
signals stop on cancellation/unwind; a prestarted cleanup owner retains and joins the reader.
The lease remains held until restoration completes, so immediate reacquisition can fail while
cleanup is pending. Explicit async close waits for restoration. OS failures and competing direct crossterm readers can violate timing
assumptions; no arbitrary input-worker or callback cancellation guarantee is made.

Terminal rendering captures width/height. It can reject a frame too tall or a character too wide;
clear-on-done cannot clear scrollback. Cancel and start a new output session after resize or
external writes. Plain writers always retain a readable transcript without cursor escapes.

You can now choose explicit/default/native input without an implicit pipe read. See [driver
reference](../api/prompt-driver.md) and [TTY reference](../api/tty.md) for ownership details.
