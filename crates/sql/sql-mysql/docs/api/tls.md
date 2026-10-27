# MysqlTls

TLS policy is an application choice made before eager open. The enum is Clone, Debug,
Default and Serialize/Deserialize. It uses an internally tagged `mode` object, snake_case
variant names and unknown-field rejection.

## Modes

Select exactly one policy:

| Rust variant | Serialized form | Behavior |
| --- | --- | --- |
| `VerifyIdentity` (default) | `{"mode":"verify_identity"}` | Verify certificate chain and server identity using bundled WebPKI roots |
| `CertificateAuthority { path: PathBuf }` | `{"mode":"certificate_authority","path":"config/db-ca.pem"}` | Verify identity using the supplied PEM CA file |
| `Plaintext` | `{"mode":"plaintext"}` | Explicit unencrypted connection; no automatic downgrade |

The server hostname must match its certificate for verified modes. CA paths come from
application configuration, not query text or a URL. Invalid path shape fails validation;
loading or certificate/authentication failure fails open.

## Choose a private CA or local plaintext

These helpers only construct options; call `MysqlDatabase::open` separately:

```rust
use stackpress_sql_mysql::{MysqlOptions, MysqlTls};
use std::path::PathBuf;

fn private_ca_options(secret: String, ca: PathBuf) -> MysqlOptions {
    let mut options = MysqlOptions::new("db.internal", "profiles", "profiles_app", secret);
    options.tls = MysqlTls::CertificateAuthority { path: ca };
    options
}

fn local_development_options(secret: String) -> MysqlOptions {
    let mut options = MysqlOptions::new("127.0.0.1", "profiles", "profiles_app", secret);
    // Select plaintext only for an explicitly authorized development server.
    options.tls = MysqlTls::Plaintext;
    options
}
```

The first keeps server identity verification. The second opts into unencrypted local
development explicitly. Neither helper creates a database or grants access.

Local native tests exercise plaintext only. They do not verify a certificate deployment,
mutual TLS or all authentication plugins. Client-certificate configuration is not a public
adapter option.
