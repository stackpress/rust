# Session identity, Auth/Profile primitives and account processes and views (M4)

This milestone implements the original JWT Session mechanism and complete direct
Auth/Profile data primitives. `auth-server` adds reusable auth events and native
signup/signin/password/OTP/link/2FA and account processes. Select `leptos-ssr`,
`leptos-hydrate` or `leptos-csr` separately for the shared views. With `auth-server`,
`composite::register` registers auth, email and identity in the source order.
The constituent registrars remain usable on their own. Nothing opens a database, starts a
listener or installs tables automatically.

Default features expose configuration and permission types. `native` enables the owned
`Service`, per-token `Session` and source schemas; `sql` adds direct Stores/Actions. `server` enables the identity constituent's ordinary
`plugin::register`. Selecting it with no session config contributes no service/listeners.
Missing `session.access` omits the request guard; `{}` installs a permissive guard.
Each route/event remains responsible for its own configured process under G42/G43.
See [auth events and native processes](docs/auth.md) for config gates, direct handlers,
source flags/errors/redirects and the explicit owned hook lifecycle.

```rust
use stackpress_framework_session::{Service, SessionData};

async fn identity() -> stackpress_framework_session::SessionResult<()> {
    let service = Service::default(); // original key=session, seed=abc123
    let claims: SessionData = serde_json::from_value(serde_json::json!({
        "id": "person-1", "roles": ["USER"]
    })).unwrap();
    let token = service.create(claims).await?;
    let identity = service.load(token);
    assert!(!identity.guest().await);
    assert!(identity.can(&[]).await?);
    Ok(())
}
```

`Session::data` caches verified data or null once per instance. Load a new object to verify
again. `authorization` returns source Guest defaults, flattened claims, raw token and grants.
No database lookup, revocation generation, automatic refresh or required-expiry policy is
added. The source WebCrypto provider rejects an explicitly empty JWT seed; the short
original default remains accepted. Nonzero `set_expires` is an absolute NumericDate; zero preserves an input exp claim.
Native Result errors cover source-invalid roles and unsupported expression syntax; invalid
JWT verification itself becomes null. A request-local carrier is unnecessary for this cache.

Source regex punctuation, wildcard derivation, exact methods, ALL and fresh/sticky cursor
behaviour are retained. Expressions use the existing Rust regex backend: unsupported JS
lookaround/backreferences are explicit errors, not silently rewritten or denied. JSON
claims use the monorepo's finite serde_json object representation. Date challenge adaptation
requires the host's actual local UTC offset and zone label; it does not substitute ISO text.

The shared native schema feature is **`stackpress-framework-schema/providers`**. Stable paths:

- `stackpress_framework_schema::providers::cuid() -> SchemaResult<String>`
- `providers::CuidGenerator::{new, with_length, generate}`
- With `codegen` enabled too, `providers::cuid_definition()` supplies the explicit build-host
  `DefinitionRegistry::with_provider` entry and matching algorithm revision.
- `providers::BuiltinProviders` implements the existing `ValueProviders` trait for `cuid()`.
- `providers::encrypt(value, seed) -> String`
- `providers::decrypt(ciphertext, seed) -> SchemaResult<String>`
- `providers::hash(value) -> String`

CUID2 uses SHA3-512 and ring OS randomness, a native random fingerprint, clock and atomic
counter. Schema codecs preserve SHAKE256(32-byte hex) and deterministic AES-256-CBC/PKCS7:
key=first32 chars of base64 SHA256(seed), IV=first16 chars of that key, ciphertext=hex.
They do not add an integrity envelope or Argon2 format migration. Source golden vectors
and bounded concurrency/SQL proofs live in the crate tests; no listeners are required.


Auth/Profile schemas and SQL
---------------------------

Stable API composition path: `stackpress_framework_session::profile::ProfileSchema`.
`ProfileSchema::model()` returns an owned `stackpress_framework_schema::Model`, including
Auth/Application/Session relation metadata. `AuthSchema` lives under `auth`. Both dereference
to `columns::RecordSchema`, exposing source-order `columns`, `column`, `defaults`,
`defaults_at`, `assert`, `filter`, `populate`, `serialize` and `unserialize`.
Descriptors describe data shape; the native source schema's codecs/assertions remain
responsible for source coercion and UTF-16 string bounds. Shared presentation now lives under `views`, with native projection under `presentation`.

Select `sql` for `profile::{ProfileStore, ProfileActions}` and
`auth::{AuthStore, AuthActions}`. These also live under `storage`. Constructors retain an
`Arc<stackpress_sql::Engine>` supplied by the application. SQL is rendered/executed by the
existing SQL crates; a session crate does not create another pool or ORM. Direct Auth
options use seed `""`; `AuthActions::make` applies the source `database.seed` fallback `abc123` after the host
resolves its engine/config at invocation time.
See [direct storage reference](docs/storage.md) for examples and observable source edges.

See [account processes and shared views](docs/account.md) for all account routes,
ordered records, source write behavior and the listener-free finite SQLite host.

The root facade exposes the same crate as `stackpress::framework_session` with the
opt-in `framework-session` feature. Forwarding features such as
`framework-session-native`, `framework-session-auth-server` and
`framework-session-leptos-ssr` select the corresponding crate features. Default
facade builds retain only `stackpress-lib`. Register the session composite after
views and before API and application admin registration; register CSRF earlier
when the application's source processes require it. Registration does not select
or open a database, install schemas, bind hooks or start a listener.

The standalone [consumers](consumers/run.py) exercise direct imports, facade
imports, the SQL-backed plugin composition and the disabled facade. Their locks
contain only workspace registry tuples and their targets are separate. The
browser consumer builds hydrate and CSR independently through the facade.

These checks cover native SQLite and headless Chromium with requests fulfilled
in memory. They do not establish live SMTP, SES, sendmail, PostgreSQL or MySQL
acceptance, or installed Tauri desktop/mobile behavior. SQL engines, hook binding
and shutdown, challenge clock/zone information, email provider configuration and
route/event access rules remain application responsibilities. No global gate is
added when `session.access` is missing.
