# Concepts

A command serves both people and programs: people need readable diagnostics, while programs need a
stable machine result. The CLI keeps those channels explicit and gives the application control over
which compiled operations are available.

## The app owns the executable

`main_with` reads process arguments excluding the executable name, parses them once, optionally
loads one JSON bootstrap data file, calls your FnOnce builder, and runs one host lifecycle. Your
builder must install the ConfigLayers it receives and select compiled plugins. No cwd imports,
config plugin strings, stock empty binary, or runtime plugin discovery are involved.

`main_with_args` exposes that same pipeline with supplied OS argv and sinks. `run` starts later: it
accepts a built Server and parsed Terminal, executes once, and returns CliRun. It never reads a
bootstrap file or rebuilds the Server. Parsing, formatting, and prompt state can also be used alone.

## Operator settings stop at emit

The syntax is `stackpress [operator options] emit <event> [capability arguments]`. For example,
`stackpress -v emit profile-search --id abc123` requests verbose host diagnostics and supplies `id`.
In `stackpress emit profile-search --verbose false`, verbose is instead capability data.

Operator `-v/--verbose`, `-f/--force`, and `-b/--bootstrap` precede emit. Grouped short options and
attached bootstrap paths are accepted; `--` may end operator options immediately before emit.
Everything after the event goes unchanged to Lib's strict argv parser, preserving original token
boundaries. `force` is trusted operator metadata, not permission for destructive behavior.

`Terminal::expect` checks literal alias keys in order. The first present key owns the value or type
error; false, zero, empty, and null do not fall through. Decode null as `Option<T>` to distinguish
Some(None) from an absent alias. See [arguments](guides/arguments.md) for examples.

## Machine output and human output

The execution adapter reserves stdout for one public JSON envelope followed by a newline. It
prepares the whole envelope with the final Response's limits before entering the writer. It uses one
OutputGuard; partial write/flush failure cannot authorize a second stdout attempt. Stopped output is
suppressed. Foreign Sent/Attempted/Failed state counts as an adapter Output failure.

Prompts, progress, metadata, and diagnostics belong on stderr or another explicitly chosen human
sink. Successful non-verbose runs emit no automatic stderr. Verbose mode explicitly selects
developer messages and causes; normal failure diagnostics give concise public status. A diagnostic
sink failure is itself terminal Output failure, even if stdout already succeeded. Application
callbacks that print directly own those effects and can violate the channel contract.

Control formats human output only. It sanitizes control characters and bidi overrides, replaces
original-template `%s` placeholders in order, prefixes an optional brand, and owns the final
newline. It never renders a machine envelope. Auto color requires that the chosen output be a
terminal and NO_COLOR be absent or empty; Always and Never override that policy. Generic CliSinks
cannot prove terminal identity, so automatic adapter diagnostics use plain output under Auto.

## Exit codes and independent diagnostics

The executable boundary evaluates public success and output, independently of diagnostic outcome. A
host report can remain Failed after an error listener deliberately rebuilds successful output.

| Code | Meaning |
| --- | --- |
| 0 | Final public success, including deliberate recovery or empty successful Abort. |
| 1 | Operation, conversion, native output, or diagnostic output failure; outer run/start error. |
| 2 | Usage, explicit bootstrap config, or ordinary startup/build failure. |
| 130 | Unrecovered typed PromptError::Aborted in the startup/execution cause chain. |

Output failure takes precedence over prompt cancellation and returns 1. Successful public recovery
returns 0 even if a retained failure was cancellation. `run` itself returns only 0/1 in CliRun;
main_with/main_with_args add executable cancellation mapping. Usage/config failure never invokes the
builder; successful parsing/config calls it exactly once. Framework entry points return ExitCode;
they do not call process::exit.

## One lifecycle and trusted metadata

`run` moves parsed data into one Request with MIME `terminal/arguments`, a synthetic `/` URL, and
the input's selected limits. It calls `handle_with_options` once. CliContext supplies verbose,
force, config path, and brand through a trusted adapter slot, separate from Request.data and
inherited by nested dispatch. It contains no I/O handles.

The [Server lifecycle](../../../crates/server/docs/concepts.md#one-lifecycle-per-top-level-call)
owns guards, Abort, fresh Error responses, successful recovery, and reports. Nested operations use
Invocation emit/resolve/delegate, so they inherit the same lifecycle metadata. The CLI only performs
Output afterward; an output error cannot re-enter Error/Response or repeat the capability.

## Prompts are requested explicitly

No prompt runs automatically during dispatch. PromptOptions configures question/default/validation;
PromptState handles serialized edits; prompt drives supplied events and a writer; prompt_native
chooses explicit answer, non-TTY default, or native terminal input. This separation makes most
prompt behavior testable without process stdin.

Required-empty checks precede Rust-regex search and one custom validator. Validation returns Valid
or Invalid(message); callback errors terminate instead of pretending the answer was merely invalid.
Keep preserves rejected text, while Clear removes both candidate and default. Transformers affect
live/final display only; returned answers remain original. Backspace removes one Unicode scalar, not
an entire grapheme cluster. EOF never submits unfinished text implicitly.

Async validation is polled locally, with bounded pending edits. Edits replay after rejection;
cancellation wins and drops the pending validator. EOF after Enter allows that submitted validation
to finish. External input cannot forge Validated completion events. Custom PromptInput::next must be
cancellation-safe and nonblocking; synchronous callbacks/writers can still block.

## The native terminal boundary

With default feature `tty`, TtyInput owns one process-wide framework stdin lease, raw-mode guard,
32-event queue, and joined polling worker. A second framework reader is rejected. Drop requests stop
and joins; explicit close reports restoration/worker errors. Cleanup preserves preexisting raw mode,
but OS failure and competing direct crossterm calls remain outside the guarantee.

Native non-TTY stdin uses an explicit/default answer or immediately returns NonInteractive; it never
starts a background pipe reader. With tty disabled this applies even to terminal stdin. To consume
already supplied pipe text, use BufferedInput::lines. An explicit answer, including empty text,
replaces prefill and still validates/renders; rejection is InvalidAnswer, distinct from EOF.

Rendering follows the selected output, independently of stdin. Plain writers receive readable text
without cursor escapes. Terminal output wraps with Unicode non-CJK display width and captured
geometry; scalar boundaries may split a grapheme. Clear-on-done applies to terminal output only.
Resizing or external writes during a prompt require cancellation and a new output session. No broad
platform support follows from native-target PTY tests.

You now have distinct boundaries for command input, host execution, machine output, and prompts. Use
the [task guides](README.md#common-tasks) to choose a working path and the [API
index](api/README.md) for exact contracts.
