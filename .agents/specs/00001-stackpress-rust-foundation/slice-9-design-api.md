# Slice 9 proposed API and API-slice caller contract

Status: Source-parity design under G43, **not implemented or compiled repository APIs**. Named public shapes below are proposed Rust mechanics around original behaviour; they do not introduce a new security model. Read [registration DX](slice-9-design-registration.md) for configuration/ordinary functions.

## Single API-facing caller and permission interface

The API package does not use one universal credential path. Keep these original paths distinct:

| Upstream source | Actual identity acquisition / check | Required Rust equivalent |
| --- | --- | --- |
| stackpress-api/src/pages/oauth.ts:48–58 | resolve('me', req), test truthiness of results.id; if absent/zero redirect to signin with original return URI | Keep the ordinary me event and response shape, including native plugin overrides/order under G42. Do not bypass it with a mandatory new repository or require a role-can check the source lacks. |
| stackpress-api/src/plugin.ts:106–134 | Parse Authorization through helpers.authorize, resolve session-detail(id), compare secret only for non-GET, require any scope intersection, set endpoint data/profileId, emit target event with same req/res | API slice owns its SQL Session record and scope policy. This is not the identity JWT. Preserve profileId mutation and same-request delegation. |
| stackpress-api/src/plugin.ts:140–184 | application-detail plus source secret/scope rules for app; public directly emits | Separate API app/public identity; do not convert all endpoints into cookie-authenticated Session.can callers. |
| stackpress-session/src/session/Session.ts:123–228 | Cookie→Session.load→verified-or-null data, roles/grants and all-of can | Native helper below supplies the same mechanism to handlers that choose it. It adds no automatic permission check to API endpoints. |

Minimal proposed interface for both implementers to reconcile:

```rust
pub type SessionData = serde_json::Map<String, serde_json::Value>;

// Serializable source result: id=0/GUEST defaults, payload fields, raw token, permits.
pub struct SessionTokenData { /* source fields plus flattened application claims */ }

// Native, request-local verified-or-Guest view of one loaded Session instance.
// Private cache/runtime fields; public data is projected through authorization().
pub struct CallerIdentity { /* token, Option<SessionData>, role-policy runtime */ }

pub async fn caller(
    req: &mut stackpress_lib::Request,
    invocation: &stackpress_lib::Invocation<'_>,
) -> stackpress_lib::Result<CallerIdentity>;

// Pure after JWT loading, so synchronous policies can use an already filled caller.
pub fn can(caller: &CallerIdentity, permits: &[SessionPermission]) -> bool;

// Borrow the explicitly filled native caller, if present; absence grants/denies nothing itself.
pub fn current(req: &stackpress_lib::Request) -> Option<&CallerIdentity>;
```

`caller` resolves the configured Session Service, reads only its normalized cookie/session key and performs original data verification/cache semantics. Missing/invalid/expired JWT gives data=None/Guest, not a new public verification error. Missing Session Service is the existing missing-capability error when a native caller directly requests it; it does not activate a service, install a guard or make every route deny. The me event remains registered only when session config activates it.

`can` is all-of over source helpers and role grants, true for an empty permit list. It adds no required-authentication, freshness, expiry-recheck, resource or CSRF policy. A loaded instance's data stays cached as in source; no live database/revocation/role fetch. API scope matching remains its own any-of logic and does not automatically invoke can. `SessionTokenData` remains a serializable source DTO; native CallerIdentity/cache is a mechanics-only companion, not a replacement restricting source response data.

### Coordinator-released request-local carrier

Current Request has cookie state and JSON data but no typed native extension slot; OperationPolicy receives only &Request. A filled caller must therefore be available on that Request if a downstream synchronous policy chooses to read it. **Owner of the minimal generic slot: stackpress-lib**, in `crates/lib/src/request.rs` plus a focused private/type-map module. No framework-session/API dependency enters lib. A narrow coordinator release is required before any shared-code edit.

Propose private typed entries `Arc<dyn Any + Send + Sync>` with native insert/get/remove methods. They are not accepted from req.data, headers or serialized snapshots. Session/API code owns concrete caller types and filling; lib owns only storage. No request/body Clone is added. The same mutable Request passed to nested emit retains entries; a new resolve/call request starts empty unless the native delegating process explicitly transfers a selected entry. No process-global current-user state or automatic cross-root inheritance.

Cookie helpers fill their session caller only when invoked; API Session/app adapters may fill their own typed record facts after their original checks, alongside original profileId data. They must not manufacture cookie roles from an API credential. A native code plugin can replace/omit these facts just as it can replace handlers. SQL OperationPolicy and preference Admission remain app-owned choices; they can read the slot synchronously without awaiting storage, and must not gain an unconditional new requirement for the slot to exist. JWT loading itself needs no SQL lookup; source API credential lookup remains async before delegation.

Carrier tests must prove same-request visibility, separate concurrent roots, explicit child transfer, absence on fresh requests, nonserialization, current-token replacement when explicitly reloaded, and no deadlock/lock across await. The slot is not a new authorization policy. It is a coordinator-owned mechanics change to make native facts readable through the current synchronous signature.

## Direct Session API without plugins

```rust
pub struct Service { /* per-host configured runtime */ }
pub struct Session { /* owned token, runtime, data cache */ }

impl Service {
    pub fn configure(&self, key: String, seed: String, access: SessionPermissionList);
    pub fn set_expires(&self, absolute_numeric_date: f64);
    pub async fn create(&self, data: SessionData) -> SessionResult<String>;
    pub fn token(&self, req: &stackpress_lib::Request) -> Option<String>;
    pub fn load(&self, token: String) -> Session;
    pub fn load_request(&self, req: &stackpress_lib::Request) -> Session;
    pub async fn authorize(
        &self,
        req: &stackpress_lib::Request,
        res: &mut stackpress_lib::Response,
        permits: &mut Vec<SessionPermission>,
    ) -> stackpress_lib::Result<bool>;
}
impl Session {
    pub async fn data(&self) -> Option<SessionData>;
    pub async fn authorization(&self) -> SessionTokenData;
    pub async fn guest(&self) -> bool;
    pub async fn can(&self, permits: &[SessionPermission]) -> bool;
    pub async fn permits(&self) -> Vec<SessionPermission>;
    pub fn save(&self, res: &mut stackpress_lib::Response) -> stackpress_lib::Result<()>;
}
```

Synchronous JWT libraries can sit behind the async source-shaped methods without IO. Configure/seed/key/access accessors remain available natively; per-host shared runtime and synchronized cache replace JS class-global mechanics. Do not add an implicit signing-secret strength check or revocation store. Helpers retain source-compatible regex/coercion requirements detailed in the inventories.

```rust
use stackpress_framework_session::{Service, SessionPermission};
use stackpress_lib::{Request, Response, Result};

async fn direct_authorize(
    sessions: &Service,
    req: &Request,
    res: &mut Response,
) -> Result<bool> {
    let mut permits = vec![SessionPermission::Event("profile-search".into())];
    // Source authorize mutates this Vec by prepending the current route unless access is empty.
    sessions.authorize(req, res, &mut permits).await
}
```

## Auth, CSRF and email direct APIs

| Surface | Source-shaped direct use / effects |
| --- | --- |
| AuthActions::make / new(engine,ActionOptions), ProfileActions | Complete generic CRUD/batch/install/upgrade/etc. and signin(type,input,password), signup(input). Preserve Engine/resource use, serialization, filters, source errors and partial effects. |
| auth process functions | Mutable Request/Response + Invocation; same nested resolve/emit operations, password/2fa flags, roles spread, original redirects and result/secret-stripping points. No sealed proof or private-only event replacement. |
| Csrf::generate(res), clear(req,res), valid(req,res) | Same names/config and outgoing projection; valid returns false after original 419 on invalid incoming scalar/equality. No session-specific binding parameter. |
| Email::send(options,config).await | Per-invocation native transport factory, source options and provider result/error map; no mandatory SendReceipt filter, producer role check or limits. |
| SessionTokenData/Auth/Profile DTOs | Preserve source exported data contracts and deliberate page results. Rust resource handles/services/caches stay out of serialization by mechanics, without removing source-intended token/QR/secret fields. |

```rust
fn direct_csrf(
    csrf: &stackpress_framework_csrf::Service,
    req: &stackpress_lib::Request,
    res: &mut stackpress_lib::Response,
) -> stackpress_lib::Result<bool> {
    csrf.valid(req, res)
}

async fn direct_email(
    mail: &stackpress_framework_email::Service,
    options: stackpress_framework_email::SendMailOptions,
) -> stackpress_framework_email::EmailResult<stackpress_framework_email::SentMessageInfo> {
    mail.send(options).await
}
```

These are proposed signatures, not claimed working imports. Implementation docs will compile-check complete examples in disposable consumers. `Result` and error adapters express Rust fallibility/Send/lifetimes, while original statuses, messages, optional data and event ordering remain the behavioural oracle.
