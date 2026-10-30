# Configure session and auth processes

Configuration selects source mechanisms; it is not a request to add an application-wide protection policy. Supply portable session settings through the typed section and map cross-plugin native resources explicitly through `auth::Environment`.

## Activation

The [identity registrar](../../src/plugin.rs), [auth registrar](../../src/auth/plugin.rs) and [missing-config tests](../../tests/main/auth_flows/events.rs) define these modes. [View registration tests](../../tests/main/account_ui/registration.rs) cover SSR contributions.

| Selection and section | Contribution |
| --- | --- |
| Identity selected, `session` missing | No Service, me, authorize or identity guard |
| `session: {}` | Source key/seed defaults; me and authorize; no request guard |
| `session.access` missing | No global session request guard; routes remain allowed by this constituent |
| `session.access: {}` | Guard installed; authorize returns true without loading JWT, mutating permits or setting results |
| Nonempty access map | Authorize checks current route plus supplied permits against cookie-derived grants |
| Auth selected, `auth` missing/null/false/0/empty string | Three reusable auth Listen events remain; no auth process routes |
| `auth: {}` | Auth routes active; account routes also require identity selection and session config |
| Email selected, `email` missing | No email-send listener; see [email activation](../../../framework-email/docs/api/service.md) |

The empty-map authorize shortcut leaves the nested response unset. The request guard consequently offers that zero-status response to `error`, retaining source behavior without inventing a denial. [Registration traces](../../tests/main/auth_flows/registration.rs) pin the effect. Each route/event secures its own process.

## Session fields

`SessionConfig` is portable and Serde-compatible. Direct construction keeps an explicitly empty key/seed; the plugin applies source `||` fallback for empty strings. Missing section and an object with missing fields remain distinct. [Defaults tests](../../tests/main/native.rs) cover the direct mechanism.

| Field | Default and meaning |
| --- | --- |
| `key` | `session`, normalized request/response cookie field |
| `seed` | `abc123`, UTF-8 JWT signing/verification key |
| `access` | `None`; role → ordered event strings or method/route objects |

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_session::{Service, SessionConfig};

pub fn configured_session() -> Service {
    let config: SessionConfig = serde_json::from_value(json!({
        "key": "identity",
        "seed": "example-seed",
        "access": {"USER": ["profile-read", {"method": "GET", "route": "/account"}]}
    }))
    .unwrap();
    let service = Service::new(config);
    service.set_expires(2_000_000_000.0);
    service
}
```

`set_expires` is a service setting, not a config-section field. A nonzero value is an absolute JWT NumericDate in seconds; zero leaves an input `exp` unchanged. It does not set a duration or cookie lifetime. [Identity reference](../api/identity.md) explains verification-null and caching.

## Auth fields

[`AuthConfig`](../../src/auth/config.rs) owns only the auth section. Typed active objects are the Rust configuration boundary; arbitrary JavaScript runtime functions are not executable config.

| Field | Default / source effect |
| --- | --- |
| `base` | Missing route base `/auth`; most processes also fall back for empty strings, but route registration does not |
| `redirect` | Optional signout-page fallback after request `redirect_uri`, then `/` |
| `roles` | Empty vector; signup spreads request data over configured roles |
| `menu` | Empty; chooser entries have name/path and optional type/target/icon |
| `password` | All checks disabled; optional min/max UTF-16 lengths, upper/lower/number/special booleans |
| `email` | Optional sender `{name,address}`; both nonempty fields checked late during issuance |
| `2fa`, `captcha` | Optional source extension objects; no automatic global checks/providers |

The final OTP/link handoff uses literal `auth.base`, including `undefined` when missing. [Auth traces](../../tests/main/auth_flows/oracle.rs) and [detailed auth reference](../auth.md) retain these branch-specific defaults.

## Native resources

Register Engine, CSRF and optional Environment during Config, install SQL tables explicitly, then bind Hooks after build. [Auth resource reference](../auth.md#native-resources-and-task-ownership) gives every Environment field/default. `database_seed` missing uses `abc123` for Actions.make, while direct Actions and generated SQL event adapters use empty seed; `email_configured` reflects the host's transport configuration.

Supply the host's local UTC offset/zone label for source Date.toString challenges. UTC is the deterministic default, not an inferred operating-system timezone. Brand/host/port, clock and random are native resources; [presentation](../api/views.md) maps public brand/assets separately. No Environment or service is automatically serialized into a browser snapshot.
