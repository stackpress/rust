# Server API reference

Lookup pages for the public host surface. Start with [Getting started](../getting-started.md) when
trying the crate for the first time; these pages assume you know the task or type to look up.

| Surface | Reference |
| --- | --- |
| Selection, composition, config claims, build | [ServerBuilder](builder.md) |
| Dependency declarations and provenance | [Plugins](plugin.md) |
| Explicit data layering and budgets | [ConfigLayers](config.md) |
| Async phases, context mutation, build errors | [Startup](startup.md) |
| Shared builder/context listener and route methods | [Registration and RouteSpec](registration.md) |
| Frozen services/config and handler shortcuts | [HostContext and ServerInvocationExt](context.md) |
| Raw and top-level entry points | [Server](server.md) |
| Outcomes, failures, stage history, correlation | [Reports](reports.md) |
| Adapter-owned one-attempt output | [OutputGuard](output.md) |
| Feature-gated buffered HTTP values and conversion | [HTTP](http.md) |
| Owned one-shot HTTP producer, body and publication | [Streaming HTTP](streaming-http.md) |

Imports use `stackpress_server`; the facade's `server` feature exposes `stackpress::server`. Lib
data, handler, request, response, and matcher contracts live in the [Lib
API](../../../../lib/docs/api/README.md). Only public surfaces are documented here; private runners and
registries confer no application extension API.
