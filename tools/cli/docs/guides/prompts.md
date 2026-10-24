# Asking for a validated answer

A prompt should be testable before it owns a real terminal. Drive it with supplied events, validate
the submitted value, and return the original answer separately from display formatting.

## Validate supplied text

This example uses a named async validator and a newline-terminated buffer. Validation runs after
required-empty and optional regex checks:

```rust
use stackpress_cli::prompt::{
    BufferedInput, PromptCancellation, PromptOptions, PromptResult, RenderOptions, Validation,
    prompt,
};

async fn check_name(answer: String) -> PromptResult<Validation> {
    Ok(if answer == "Ada" {
        Validation::Valid
    } else {
        Validation::Invalid("Use Ada".into())
    })
}

async fn example() -> PromptResult<()> {
    let mut options = PromptOptions::new("Name?").async_validator(check_name);
    options.required = true;
    let mut input = BufferedInput::lines("Ada\n");
    let mut output = Vec::new();
    let answer = prompt(
        &options,
        &mut input,
        &mut output,
        &PromptCancellation::new(),
        RenderOptions::default(),
    )
    .await?;
    assert_eq!(answer, "Ada");
    assert!(String::from_utf8_lossy(&output).contains("Ada"));
    Ok(())
}
```

The newline produces Enter; EOF alone does not submit. BufferedInput never reads stdin. Default
RenderOptions select a plain transcript, so this example needs no terminal or cursor control.

## Choose correction and cancellation

Keep preserves rejected text for editing; Clear removes both current and default. Prefill::Tab shows
a preview until Tab/Enter copies it; Editable starts with real editable text. Backspace removes one
Unicode scalar, or clears the default preview when editing text is empty. A transformer changes
display only, including its final render; it never changes validation or the returned answer.

PromptCancellation is explicitly shareable and permanent. cancel() wakes idle input/validation;
cancellation drops pending async validation. While validation waits, bounded edits queue and replay
after rejection. User callbacks must cooperate with polling, and synchronous callbacks/writers can
block. Callback failure is distinct from Validation::Invalid and terminates the prompt.

You have obtained a validated answer without global input. Read [native prompts](native-prompts.md)
when a real terminal is needed, or use [options](../api/prompt-options.md),
[state](../api/prompt-state.md), and [drivers](../api/prompt-driver.md) for precise limits/errors.
