# Session API reference

Use these pages to look up public arguments, return values and feature requirements. Start with [getting started](../getting-started.md) for a first round trip or [concepts](../concepts.md) for the identity/storage/process boundary.

| Surface | Feature / reference |
| --- | --- |
| Portable config, claims, grants and matching | [Identity](identity.md); default |
| Service, Session, Clock, source codecs | [Identity](identity.md); native |
| me/authorize/request guard and registrars | [Identity](identity.md), [auth](auth.md); server / auth-server |
| Auth/Profile columns, schema and model descriptors | [Storage](storage.md); native |
| Store/Actions, queries, batches and maintenance | [Storage](storage.md); sql |
| Auth events, processes, templates, TOTP and Hooks | [Auth](auth.md); auth-server |
| Account loaders/actions, CSV, enrollment and SQL events | [Account](account.md); auth-server |
| Shared forms/heads/pages and public props | [Views](views.md); leptos |
| Native public projection/document rendering | [Views](views.md); leptos-ssr |
| Configuration keys/resources | [Configuration](../guides/configuration.md) |

Native surfaces are also gated to non-WASM targets in [lib.rs](../../src/lib.rs). `auth-server` selected on WASM does not expose native implementations; choose hydrate or CSR for browser views. The [graph checks](../../consumers/graphs.py) verify that boundary.
