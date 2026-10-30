# Email for direct calls and events

Send source-shaped email options through a transport owned by each call, or select the reusable `email-send` event. Start locally with JSON output, then choose a transport or bind an application provider.

[Getting started](docs/getting-started.md) produces inspectable output without delivery. [Concepts](docs/concepts.md) explains configuration, per-call ownership and native streams after that first example.

| Need | Read |
| --- | --- |
| Select SMTP, pool, JSON, stream, sendmail, SES or a connection string | [Choose a transport](docs/guides/transports.md) |
| Map config/defaults and message options | [Configure transport and content](docs/guides/configuration.md) |
| Compose events and non-awaited hooks | [Register and supervise](docs/guides/events-and-hooks.md) |
| Pass native bytes/readers | [Use native content](docs/guides/native-content.md) |
| Look up a type or method | [API reference](docs/api/README.md) |
| Compare source behavior and Rust mechanics | [Source behavior](docs/source-behavior.md) |
| Check documentation and standalone consumers | [Verification](docs/verification.md) |

Default features expose the direct native library. Optional `server` adds ordinary registrars, `EmailSend` and rich native reader transfer. Transport variants are compiled adapter paths, with explicit complete-provider bindings for capabilities lettre cannot represent. Native direct use is not a WASM support claim.

In the preserved shared auth response, the auth-update results body causes email-send to skip. Fresh email-send resolves still send; see [the source note](docs/source-behavior.md#shared-auth-response-and-hook-behavior). The framework adds no response clearing or global guard.
