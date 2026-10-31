# Aggregate API reference

Use these pages for signatures, arguments, return values and error/phase behaviour. The aggregate exposes qualified native modules; it does not reproduce the upstream package's flat JavaScript re-export tree.

| Surface | Lookup |
| --- | --- |
| register, register_with, Register, CompiledClient, CompiledModel, Client | [Registration and catalog](registration.md) |
| Cargo leaves, facade exports and renderer guards | [Features](features.md) |
| Environment, typed source roots and activation | [Configuration](configuration.md) |
| Bindings, server/schema event wrappers and direct scripts | [Commands](commands.md) |
| Engine/Scripts/History, SQL event and lifecycle functions | [SQL](sql.md) |
| Language Config, Service, Language and Scalar | [Language](language.md) |
| Render/build providers, source props and Packaging | [View](view.md) |
| Rust generator re-exports, Host, provider and transform registration | [Code generation](codegen.md) |

For a first execution use [getting started](../getting-started.md). For constituent account/auth/CSRF/email/API details, follow the owning [session](../../../framework-session/docs/api/README.md), [CSRF](../../../framework-csrf/docs/api/README.md), [email](../../../framework-email/docs/api/README.md) references and [API crate Rustdoc source](../../../framework-api/src/lib.rs). Selecting their types is distinct from registering their events.
