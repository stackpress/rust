# Stackpress Server

`stackpress-server` builds a capability host from compiled plugin functions, typed configuration,
and shared services. Use it when the same operation should run in process, through a command, or
through an HTTP adapter with one request/operation/response/error lifecycle.

These docs cover the public API in this checkout. The workspace uses Rust 1.96 and edition 2024; the
crates currently have `publish = false`, so examples use local Cargo dependencies.

## Start here

Begin with [Getting started](getting-started.md): build a plugin, call a capability, and print
`Hello, Ada`. Read [Concepts](concepts.md) next for startup order, lifecycle decisions, captures,
and the difference between public output and retained diagnostics.

The host opens no listener and chooses no executor. Feature `http` enables conversion of already
buffered HTTP values; it adds no socket, stream, TLS, or serverless runtime. The optional `http-stream` feature adds owned one-shot
response output while remaining IO-agnostic; the separate
[HTTP host](../../http/docs/README.md) owns the network boundary.

## Common tasks

Choose a guide by the result you need. Each links to the corresponding lookup page.

| I want to… | Read |
| --- | --- |
| Package operations and order prerequisites | [Writing a plugin](guides/writing-a-plugin.md) |
| Layer defaults and validate settings | [Typed configuration](guides/typed-config.md) |
| Initialize and share an application resource | [Services](guides/services.md) |
| Expose a capability under a method and path | [Operation routes](guides/operation-routes.md) |
| Recover public output and inspect failures | [Errors and recovery](guides/errors-and-recovery.md) |
| Prove behavior without a transport | [Testing in process](guides/testing-in-process.md) |
| Convert buffered HTTP input and output | [Using HTTP](guides/using-http.md) |
| Publish owned one-shot HTTP output | [Streaming HTTP](guides/streaming-http.md) |
| Serve real HTTP/HTTPS connections | [HTTP host](../../http/docs/getting-started.md) |

## Returning to the API

Use the [API index](api/README.md) for signatures, parameters, returns, defaults, and errors. The
[Lib docs](../../../lib/docs/README.md) own data, handlers, patterns, requests, and responses; these
pages explain the host behavior added around them. The optional workspace facade exposes this crate
as `stackpress::server` with feature `server` (`server-http` also enables HTTP).

Examples are plain fenced Markdown, compile-checked separately from doctests. Source authority is
[the crate](../src/lib.rs), [contract tests](../tests/), and the canonical [capability-host
application](../../../../examples/capability-host/src/lib.rs). The matching [CLI
docs](../../../../tools/cli/docs/README.md) show an application-owned executable.
