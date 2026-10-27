# MysqlOptions

`MysqlOptions::new(host, database, user, password) -> Self` accepts four values implementing
`Into<String>`. It stores explicit fields with the defaults below; it does not connect.
`validate(&self) -> stackpress_sql::Result<()>` returns `Configuration` for invalid fields or
budgets without echoing connection identity or secrets. Open calls validate again.

## Fields and defaults

All fields are public. Serialized options reject unknown fields; connection strings are not
a supported production configuration format.

| Field | Type | Constructor / omitted-field default | Accepted values |
| --- | --- | --- | --- |
| `host` | `String` | Required | Nonempty TCP host; no leading slash, URL syntax or control characters |
| `port` | `u16` | `3306` | `1..=65535` |
| `database` | `String` | Required | Nonempty, no control characters; must already exist |
| `user` | `String` | Required | Nonempty, no control characters |
| `password` | `String` | Required, even when empty | No NUL; application supplies the secret |
| `tls` | `MysqlTls` | `VerifyIdentity` | [Explicit TLS policy](tls.md) |
| `max_connections` | `u32` | `4` | `1..=128`; pool minimum is zero |
| `max_in_flight` | `usize` | `128` | `1..=4096`; includes pool waiters and held transactions |
| `acquire_timeout_ms` | `u64` | `5000` | `1..=300000` milliseconds |
| `statement_timeout_ms` | `u64` | `30000` | `1..=300000` milliseconds |
| `cleanup_timeout_ms` | `u64` | `5000` | `1..=300000` milliseconds per cleanup phase |

The four constructor fields are also mandatory during direct deserialization.
Omitted TLS, port and budget fields use these defaults; zero never means unlimited.
A custom CA path must be nonempty and contain no control characters.

## Configure before sharing

Build and validate application options before opening:

```rust
use stackpress_sql_mysql::MysqlOptions;

fn application_options(secret: String) -> stackpress_sql::Result<MysqlOptions> {
    let mut options = MysqlOptions::new("db.internal", "profiles", "profiles_app", secret);
    options.max_connections = 4;
    options.max_in_flight = 32;
    options.acquire_timeout_ms = 2_000;
    options.statement_timeout_ms = 10_000;
    options.cleanup_timeout_ms = 2_000;
    options.validate()?;
    Ok(options)
}
```

This function does no IO. Opening may still fail on authentication, TLS, CA-file loading,
connection failure or its acquisition deadline. Fields can be changed after validation;
open validates the final value.

## Secret handling

Options are Clone and implement Serialize/Deserialize. Debug redacts all connection fields;
it only shows selected pool limits. Serialization includes the password and is an explicit
secret-handling operation. Do not serialize options into logs or return them in responses.
The adapter does not resolve secrets, read a passfile or accept request-supplied URLs.

## Fixed connection policy

The adapter selects utf8mb4 / utf8mb4_unicode_ci and disables statement caching. SQLx adds
NO_ENGINE_SUBSTITUTION and PIPES_AS_CONCAT to the server's existing SQL mode; the adapter
does not replace that mode or force STRICT_TRANS_TABLES / NO_BACKSLASH_ESCAPES. A supplied
empty password explicitly selects passwordless authentication.

There is no automatic TLS downgrade. Plaintext authentication that requires an RSA
feature is outside this dependency configuration. Server-side authentication plugins and
certificate setups beyond the local fixture need separate verification.
