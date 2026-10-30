# Resolved transport options

`stackpress_framework_email::options` exposes non-IO constructor inspection. The [configuration example](../guides/configuration.md#inspect-selection-before-io) calls every public operation below.

## Types and fields

`TransportKind` is an exhaustive enum with Smtp, Pool, Json, Stream, Sendmail and Ses. `TransportOptions` has `kind` and a complete resolved JSON `fields` map; both types implement serde serialization/deserialization, Clone, Debug and PartialEq.

`SmtpOptions` carries `host: String`, `port: f64`, `secure: bool`, `secured: bool` and the complete `fields` map. It implements Clone, Debug and PartialEq. Numeric ports stay source-shaped here; representability checks occur at the native adapter/provider boundary.

## Operations

Read [configuration](../guides/configuration.md#preserve-selection-and-defaults) for exact precedence and defaults rather than inferring them from the type carrier.

| Operation | Parameters | Meaning/errors |
| --- | --- | --- |
| `TransportOptions::resolve` | `config: &EmailConfig` | URL-before-variant selection, SMTP service merge, pool defaults; malformed strings or legacy SES shape can error |
| `smtp` | `&self` | Effective SMTP defaults without mutating fields |
| `windows_newlines` | `&self` | win/windows/dos/CRLF aliases, case-insensitive; unknown values use Unix behavior |
| `pool_rate` | `&self` | Number(value) or source defaults for delta/limit |
| `parse_connection_url` | `input: &str` | Public source URL parser; malformed/coercion representations return EmailResult errors |

**Returns:** `resolve` returns `EmailResult<TransportOptions>`; `smtp` returns SmtpOptions; `windows_newlines` returns bool; `pool_rate` returns `(f64, f64)`; URL parsing returns `EmailResult<Map<String, Value>>`.

URL parsing preserves auth percent decoding before first-colon splitting, existing-property precedence, duplicate query grouping, numeric/bool coercion and tls nesting. smtp/smtps/direct set their source fields; a direct scheme does not introduce a separate transport type. Legacy syntax outside the native URL representation can be routed by NativeFactory to a complete provider or a named error. The original config still reaches complete providers intact.

These helpers do not open connections or enforce an application policy. [Native adapters](adapters.md) describe which resolved capabilities are representable; [source vectors](../../tests/fixtures/source-transports.json) pin source option/default behavior.
