# Stackpress Lib

`stackpress-lib` supplies owned data, buffered requests and responses, and ordered async event
dispatch. Use it when an operation should run against the same values from a command, a service, or
another operation, without selecting a transport or executor.

These docs cover the public API currently implemented in this checkout. They are for Rust developers
trying the crate, adding handlers, or integrating a host. The workspace declares Rust 1.96 and
edition 2024; both packages currently have `publish = false`, so the quick start uses a local Cargo
dependency.

## Start here

Begin with [Getting started](getting-started.md): register a named handler, resolve it with plain
data, and print its result. Then read [Concepts](concepts.md) to understand listener order, nested
calls, ownership, and execution outcomes.

The foundation performs no network or filesystem I/O for you. HTTP/CLI adapters, top-level
request/response/error lifecycle orchestration, services, Idea generation, SQL, and UI are outside
this crate's current surface. Route patterns match event strings; registering one does not open an
endpoint.

## Common tasks

Choose the guide that matches the work in front of you. Each guide introduces one useful case and
links to exact API details.

| I want to… | Read |
| --- | --- |
| Store values and update nested fields | [Data and paths](guides/data-and-paths.md) |
| Register routes and call another operation | [Events and routing](guides/events-and-routing.md) |
| Construct input and prepare output | [Requests and responses](guides/requests-and-responses.md) |
| Track cookies and preserve deletion scope | [Sessions and cookies](guides/sessions-and-cookies.md) |
| Parse query, command, JSON, or multipart input | [Parsing input](guides/parsing-input.md) |
| Return public errors and inspect diagnostics | [Errors and diagnostics](guides/errors-and-diagnostics.md) |
| Register plugins through ordered startup work | [Startup with task queues](guides/startup-with-task-queues.md) |

## Returning to the API

Use the [API index](api/README.md) for signatures, parameters, returns, defaults, and errors.
Imports in these pages use the direct crate name `stackpress_lib`; the workspace facade also exposes
the crate as `stackpress::lib` and selects common root re-exports.

Examples are plain Markdown snippets, with omitted surrounding code stated locally. They are not
included in the crate's doctests. Production code and Rustdoc live in [the source](../src/lib.rs);
executable usage contracts live in [the crate tests](../tests/) and
[the facade integration test](../../../tests/foundation.rs).
