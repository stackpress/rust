# Configure transport and message options

Keep transport configuration separate from a message. The factory reads the original config on each send; option resolution explains which built-in adapter and defaults that call will use.

## Inspect selection before IO

`TransportOptions::resolve` reads a source object or connection string without opening connections. This example also demonstrates port, newline and pool-rate accessors.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::options::{TransportKind, TransportOptions, parse_connection_url};
use stackpress_framework_email::{EmailConfig, EmailResult};

pub fn inspect_options() -> EmailResult<()> {
    let config: EmailConfig = serde_json::from_value(json!({
        "pool": true, "host": "relay.example.test", "port": 465,
        "rateDelta": 2000, "rateLimit": 8, "newline": "dos"
    }))?;
    let options = TransportOptions::resolve(&config)?;
    assert_eq!(options.kind, TransportKind::Pool);
    assert_eq!(options.fields["maxMessages"], 100);
    assert!(options.smtp().secure);
    assert_eq!(options.pool_rate(), (2000.0, 8.0));
    assert!(options.windows_newlines());
    let parsed = parse_connection_url("smtp://user:pass@relay.example.test/?pool=true")?;
    assert_eq!(parsed["auth"]["user"], "user");
    Ok(())
}
```

Omitted `secure` with port 465 implies implicit TLS. Explicit false/null does not receive that omitted-field rule. Inspecting pool options succeeds here; constructing a native pool transport still selects the complete-provider binding.

## Preserve selection and defaults

URL resolution happens before variant selection. A truthy object `url` replaces its sibling option bag; the selection order is pool, sendmail, stream, JSON, SES, then SMTP. SMTP service catalog values override supplied fields. Unknown fields remain available to providers; fields ignored by the source stay unused.

| Options | Source mapping |
| --- | --- |
| `host`, `port`, `secure`, `secured` | Host falls back to localhost; zero/falsy numeric port falls back to 465 when secure is truthy, otherwise 587; omitted secure with effective port 465 becomes true; already-secured sockets require a binding |
| `service` | Versioned source service catalog, including aliases/domains/email keys; catalog settings override ordinary SMTP fields |
| `maxConnections`, `maxMessages`, `rateDelta`, `rateLimit` | Pool defaults 5/100; numeric rate defaults 1000/0; complete pool uses a provider |
| `auth`, `authMethod` | LOGIN/PLAIN/static XOAUTH2 mapping; empty auth object does not activate authentication; token refresh/provisioning/custom mechanisms require a binding |
| `ignoreTLS`, `requireTLS`, `name`, `localAddress`, `connectionTimeout` | Native TLS selection, EHLO name, IP bind and milliseconds; supported connection timeout defaults to 120000 |
| `tls.servername`, top-level `servername`, `tls.rejectUnauthorized`, `ca`, `minVersion`, `cert`, `key` | Native TLS server name, verification setting, root certificates, TLS 1.2/1.3 and one PEM identity; chain/other TLS capabilities select provider or error |
| `skipEncoding`, `buffer`, `newline` | JSON object/string; stream bytes/reader; source win/windows/dos/CRLF aliases are retained without rewriting raw bytes |
| `path`, `args`, `SES` | Sendmail executable/arguments or SES client selection; actual native SDK handles are provider bindings rather than JSON callbacks |

The full original config reaches a complete provider, rather than this resolved inspection projection. See [adapter reference](../api/adapters.md) for each binding's request/result shape.

## Configure native SMTP authentication and TLS

Keep credentials and native certificate material in application-owned configuration. This function constructs password/static-token and TLS settings without connecting; the example strings are illustrative values, and actual sending uses the selected driver.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{EmailConfig, EmailResult};

pub fn smtp_auth_configs() -> EmailResult<Vec<EmailConfig>> {
    let values = [
        json!({"host": "relay.example.test", "port": 587, "secure": false,
            "auth": {"user": "application-user", "pass": "example-password", "method": "LOGIN"},
            "requireTLS": true, "connectionTimeout": 4000, "name": "application.example.test",
            "tls": {"servername": "relay.example.test", "rejectUnauthorized": true, "minVersion": "TLSv1.3"}}),
        json!({"host": "relay.example.test", "port": 465, "secure": true,
            "auth": {"type": "OAuth2", "user": "reader@example.test", "accessToken": "example-token"}}),
    ];
    values
        .into_iter()
        .map(|value| Ok(serde_json::from_value(value)?))
        .collect()
}

pub fn client_identity(cert: String, key: String, ca: String) -> EmailResult<EmailConfig> {
    Ok(serde_json::from_value(
        json!({"host": "relay.example.test", "secure": true,
        "tls": {"cert": cert, "key": key, "ca": ca}}),
    )?)
}
```

LOGIN and static XOAUTH2 are supported mappings; the server's advertised AUTH controls whether native authentication runs. Refresh/provisioning callbacks need a complete provider. The identity function accepts the caller's PEM material; the built-in adapter converts one certificate to the SDK's DER representation, while chains require the complete provider. No certificate or handshake is fabricated by these examples.

## Keep message fields source-shaped

Message fields are an open map. This example constructs content and access settings without performing file or URL IO.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{EmailResult, SendMailOptions};

pub fn message_options() -> EmailResult<SendMailOptions> {
    Ok(serde_json::from_value(json!({
        "from": {"name": "Sender", "address": "sender@example.test"},
        "to": ["reader@example.test"], "cc": [], "bcc": [],
        "replyTo": "reply@example.test", "subject": "Report", "text": "Attached",
        "html": "<p>Attached</p>", "headers": {"X-Example": "yes"},
        "attachments": [{"filename": "report.txt", "content": "report"}],
        "alternatives": [{"contentType": "text/plain", "content": "alternative"}],
        "disableFileAccess": true, "disableUrlAccess": true,
        "ses": {"ConfigurationSetName": "application"},
        "applicationExtension": {"retained": true}
    }))?)
}
```

Address/header/envelope/messageId/date/priority/content options follow source selection and normalization. Native Buffer/Readable slots use [the owned carrier](native-content.md). Group/permissive addresses, custom compiler/header controls, DKIM, watchHtml/AMP/iCalendar/List/custom MIME options and other library gaps route to a complete provider or return a named error.

## File and URL resolution

Body/raw/attachment/alternative source nodes accept content, path or href forms. `ContentResolver` supplies native file/data URI/HTTP(S) reads and can be replaced by a fake. Native fetch retains source UA, gzip/deflate, a fresh cookie jar and five redirects. Its HTTP(S) client accepts invalid TLS certificates, matching the source fetch setting; it adds no origin/size/timeout policy.

MIME honors `disableFileAccess` and `disableUrlAccess`; transport values override per-message values, including false. Source JSON normalization bypasses those flags and does not use httpHeaders. For MIME href reads, httpHeaders are retained. These distinctions and deferred stream errors are preserved source behavior, not new global policy.
