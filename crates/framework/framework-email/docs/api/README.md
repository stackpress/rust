# Email API reference

Use this index for callable contracts and native data types. Start with [getting started](../getting-started.md) when you need a first successful send rather than a lookup.

| Public surface | Reference |
| --- | --- |
| `EmailConfig`, `Service`, `TransportFactory`, `Transport`, error/future aliases | [Service and factories](service.md) |
| `SendMailOptions`, native content and result types | [Messages and results](messages.md) |
| `TransportKind`, `TransportOptions`, `SmtpOptions`, URL parsing | [Resolved options](options.md) |
| `NativeFactory`, SMTP/process/SES/content providers, `SmtpError` | [Native adapters](adapters.md) |
| `compose::addresses`, `compose::metadata`, `MessageMetadata` | [Address and metadata helpers](compose.md) |
| `EmailSend`, `NativeReader`, plugin registrars and raw section | [Server events](events.md) |
| `TaskSupervisor`, `TaskOutcome` | [Owned tasks](tasks.md) |

Direct surfaces exist with default features. `events`, `plugin` and `TaskSupervisor::resolve_on` require `server`; backend/content/task primitives do not. [Source behavior](../source-behavior.md) lists library capability adaptations and native-target limits.
