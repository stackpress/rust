# Identity, permissions and server adapters

Portable types live at the crate root. `native` adds Service/Session/Clock; `server` adds cookie and event adapters. The direct [getting-started example](../getting-started.md) needs only native, JSON and a runtime.

## Service and Session

[`session.rs`](../../src/session.rs), [`jwt.rs`](../../src/jwt.rs) and [native tests](../../tests/main/native.rs) are the behavior anchors. `SessionResult<T>` reports signing/config/pattern failures; invalid verification itself becomes `None`.

| Method | Parameters / result |
| --- | --- |
| `Service::default()` / `new(config)` | Source defaults or owned SessionConfig; returns Service |
| `with_clock(config, Arc<dyn Clock>)` | Host clock is Send + Sync; `now() -> f64` epoch seconds |
| `config()` / `configure(config)` | Owned snapshot / replace live settings; no lock crosses await or callback |
| `set_expires(f64)` | Nonzero absolute NumericDate seconds, zero preserves an input exp; returns unit |
| `create(SessionData).await` | HS256 sign object claims with issued-at; returns SessionResult<String> |
| `load(impl Into<String>)` | New independently memoized Session; no verification yet |
| `Session::token()` | Borrow readonly raw `&str` |
| `data().await` / `guest().await` | `Option<&SessionData>` / bool; success or null cached once per instance |
| `authorization().await` | SessionResult<SessionTokenData>: Guest defaults → claims → raw token/permits |
| `permits().await` | SessionResult<Vec<SessionPermission>> from current access/roles |
| `can(&[SessionPermission]).await` | SessionResult<bool>, all-of; empty request true before roles parsing |

**Returns:** `SessionData` and `SessionTokenData` are JSON object maps. A Session is not Clone or Serialize; use its deliberate data/authorization projection. Service is cloneable and shares settings. New instances verify independently; cached claims do not expire again inside an existing instance.

Signing uses the configured UTF-8 seed, including the short source default. An explicitly empty direct seed fails at the source WebCrypto-equivalent boundary. Verification accepts the source HMAC algorithms/temporal rules, requires no added exp claim and maps malformed/invalid/expired tokens to cached null. [JWT vectors](../../tests/jwt-vectors.json) pin the original JOSE tokens, including padded base64url cases.

## Grants and matching

`SessionPermission` is an untagged Event string or Route(Arc<SessionRoute>). `SessionRoute` has exact `method` and `route` strings. `SessionPermissionList` maps role strings to ordered grants; parsed equal route objects remain separate identities. Roles use source truthiness/property-key coercion, with truthy non-arrays returning an error. [Matching tests](../../tests/main/native.rs) pin GUEST, empty roles, deduplication and syntax limits.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_session::{Service, SessionConfig, SessionPermission, SessionResult};

pub async fn explicit_permission() -> SessionResult<()> {
    let config: SessionConfig = serde_json::from_value(json!({
        "access": {"USER": ["profile-*", {"method": "GET", "route": "/account/:id"}]}
    }))
    .unwrap();
    let service = Service::new(config);
    let claims = serde_json::from_value(json!({"roles": ["USER"]})).unwrap();
    let token = service.create(claims).await?;
    let caller = service.load(token);
    assert!(
        caller
            .can(&[SessionPermission::Event("profile-read".into())])
            .await?
    );
    assert!(
        caller
            .can(&[SessionPermission::route("GET", "/account/one")])
            .await?
    );
    assert!(
        !caller
            .can(&[SessionPermission::Event("auth-remove".into())])
            .await?
    );
    Ok(())
}
```

The event wildcard consumes one hyphen segment; route parameters consume one slash segment. Double wildcards derive the source broader expression. `permissions::{match_event,match_route,match_any}` expose matching directly and return regex errors. Method comparison is exact except literal ALL. Source regex punctuation is not escaped; slash-delimited expressions retain supported flags, fresh global matches and start-zero sticky matches. Unsupported Rust-regex lookaround/backreferences are explicit mechanics errors, not silently rewritten patterns.

## Caller interface for API and other handlers

There is no separate universal caller type or request-local slot: the verified caller is `stackpress_framework_session::Session`, obtained from `Service::load_request(&Request)`. Its permission signature is `async fn can(&self, permits: &[SessionPermission]) -> SessionResult<bool>`. A named handler can read it directly.

<!-- example: server -->
```rust
use stackpress_framework_session::{Service, SessionPermission};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

pub async fn inspect_caller(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let caller = inv.service::<Service>()?.load_request(req);
    let allowed = caller
        .can(&[SessionPermission::Event("profile-read".into())])
        .await
        .map_err(Error::from_source)?;
    res.results(&serde_json::json!({"guest": caller.guest().await, "canRead": allowed}))?;
    Ok(Flow::Continue)
}
```

The example reports the check; the application decides what action that result permits. API OAuth uses the public `me` event with the original mutable request and a fresh response. Its application-detail/session-create events use fresh requests. API stored Session credentials and scopes remain separate from the browser JWT; a bearer JWT is not a replacement for the configured cookie. The [plugin consumer trace](../../consumers/plugin/src/main.rs) and [API OAuth source](../../../framework-api/src/pages/oauth.rs) anchor that contract. No synchronous SQL OperationPolicy carrier is required or added.

## Cookie, authorize and guard operations

[`plugin.rs`](../../src/plugin.rs) supplies the following server operations. [Host tests](../../tests/main/host.rs) and [source registration traces](../../tests/main/auth_flows/registration.rs) check mutation/response order.

| Operation | Arguments / effect |
| --- | --- |
| `Service::token(&Request)` | Option<String> from configured cookie only |
| `load_request(&Request)` | Session from cookie or empty token, one fresh cache |
| `Session::save(&mut Response)` | Stages raw token under current configured key with existing cookie options; returns lib Result<()> |
| `Service::authorize(&Request, &mut Response, &mut Vec<SessionPermission>).await` | Empty access returns true unchanged; otherwise prepends current uppercase method/path, checks all, writes authorization or source 401 Unauthorized; returns lib Result<bool> |
| `plugin::{me,authorize_event,request_guard}` | Ordinary `(&mut Request,&mut Response,Invocation) -> async Result<Flow>` |
| `plugin::register(&mut ServerBuilder)` | Identity-only ordinary registration; returns lib Result<()> |

`me` deliberately returns raw token and permits. The request guard skips source-truthy bodies and existing nonzero/non-200 status, emits authorize using the same request/fresh response, and copies/offers non-200 authorization to error. It does not invent Abort; ordinary source returns continue. [Configuration](../guides/configuration.md) describes missing versus empty access.

## Source codecs

With native, `codecs::SourceCodecs::new(seed)` retains a separate database seed. `serialize_token(&str) -> String` encrypts deterministically; `unserialize_token(&str)` returns SchemaResult<String>; `serialize_secret(&str) -> String` uses source SHAKE256. `codecs::{encrypt,decrypt,hash}` re-export the shared schema providers. `date_string(milliseconds, offset_minutes, zone_name) -> SessionResult<String>` formats source Date.toString, rejecting unsupported native ranges. [Storage reference](storage.md) gives the stable provider paths and golden evidence.
