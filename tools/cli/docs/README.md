# Stackpress CLI

`stackpress-cli` supplies the command boundary for an application's compiled Stackpress host:
argument parsing, JSON output, human diagnostics, and explicit prompts. Each app owns its executable
and selected plugins; this crate ships no stock `stackpress` binary.

These docs cover the public surface in this checkout, using Rust 1.96 and edition 2024. Packages
currently have `publish = false`, so examples use local Cargo dependencies. Feature `tty` is on by
default; disable defaults to omit the native crossterm input driver.

## Start here

Follow [Getting started](getting-started.md) to create an app-owned binary and emit one greeting.
Then read [Concepts](concepts.md) for the operator boundary, output channels, prompt ownership, and
process exit policy. The [Server docs](../../../crates/server/docs/README.md) explain the host; [Lib
docs](../../../crates/lib/docs/README.md) own data and handler contracts.

## Common tasks

Pick a guide by the result you need. Human output and prompts are explicit operations, separate from
the machine envelope produced by capability execution.

| I want to… | Read |
| --- | --- |
| Compose a compiled application's entry point | [Application binary](guides/application-binary.md) |
| Parse operator flags and target input | [Arguments](guides/arguments.md) |
| Write readable human output with color policy | [Human output](guides/human-output.md) |
| Ask and validate a supplied answer | [Prompts](guides/prompts.md) |
| Acquire real terminal input | [Native prompts](guides/native-prompts.md) |
| Test argv and output without global streams | [Testing](guides/testing.md) |

## Returning to the API

Use the [API index](api/README.md) for signatures, field defaults, error variants, and return rules.
Examples are plain Markdown snippets, compile-checked outside the repository; they are not doctests.
Source authority is [the crate](../src/lib.rs), [contract tests](../tests/), and the canonical
[app-owned stackpress binary](../../../examples/capability-host/src/main.rs).
