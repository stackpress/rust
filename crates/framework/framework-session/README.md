# Native identity mechanisms (M0)

This milestone implements the original JWT Session mechanism. It does not yet implement
Auth/Profile stores, auth/email composition, account routes or Leptos pages. Those remain
in the accepted full Slice 9 scope, behind their coordinator release gates.

Default features expose configuration and permission types. `native` enables the owned
`Service` and per-token `Session`; `server` enables the identity constituent's ordinary
`plugin::register`. Selecting it with no session config contributes no service/listeners.
Missing `session.access` omits the request guard; `{}` installs a permissive guard.
Each route/event remains responsible for its own configured process under G42/G43.

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
