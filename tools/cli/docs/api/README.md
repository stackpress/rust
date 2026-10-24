# CLI API reference

Lookup pages for application entry, parsing, human output, and explicit prompt drivers. Use [Getting
started](../getting-started.md) first when creating a binary.

| Surface | Reference |
| --- | --- |
| main_with, main_with_args, exit codes | [Executable entry](entry.md) |
| Terminal, TerminalOptions, ArgumentError | [Arguments](terminal.md) |
| Control, ColorMode, Level, OutputEnvironment | [Human output](control.md) |
| run, CliSinks, CliContext, CliRun | [Execution](execution.md) |
| PromptOptions, limits, validation, transformer, errors | [Prompt options](prompt-options.md) |
| Editing/events/actions/status and render types | [Prompt state](prompt-state.md) |
| PromptInput, buffers, cancellation, prompt/native | [Prompt drivers](prompt-driver.md) |
| Feature-gated TtySession and TtyInput | [Native TTY](tty.md) |

Imports use `stackpress_cli`. Public modules args, control, and prompt are always available; tty is
feature-gated. Root re-exports include Terminal/control/entry/execution types and Server's
[ConfigLayers](../../../../crates/server/docs/api/config.md). The facade currently has no CLI
re-export. Private driver helpers and process-policy internals are not application APIs.
