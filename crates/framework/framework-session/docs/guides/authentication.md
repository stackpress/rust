# Reuse authentication processes

Choose a page process when you need its complete source checks and redirects. Choose an event or exported helper when your own process supplies the surrounding checks. The distinction matters because direct reuse does not run a calling page's method or CSRF logic.

## Start with a named handler

Any auth event remains an ordinary async handler. This example exposes the source signout event on an application event name, without selecting the auth registrar or requiring SQL/config.

<!-- example: server -->
```rust
use stackpress_framework_session::auth::events;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

pub async fn finish_session(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    events::signout(req, res, inv).await
}

pub async fn direct_signout() -> Result<()> {
    let mut app = ServerBuilder::new();
    app.on("finish-session", finish_session)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let resolved = server
        .resolve("finish-session", serde_json::json!({}))
        .await?;
    assert_eq!(resolved.response.code, Some(200));
    Ok(())
}
```

Signout deletes the literal `session` cookie, including when the configured identity key differs. The page then redirects by request `redirect_uri`, auth redirect, `/`. [Real event tests](../../tests/main/auth_flows/events.rs) pin the literal key and request flags.

## Pick the matching flow

The [process implementation](../../src/auth/pages/mod.rs), [source invocation traces](../../tests/main/auth_flows/oracle.rs) and [SQLite event tests](../../tests/main/auth_flows/events.rs) establish the following flow. The [auth detail page](../auth.md#config-flags-errors-and-redirects) retains exact branch/error/redirect notes.

| Need | Public entry / source behavior |
| --- | --- |
| Signup | `pages::signup` wraps CSRF/page handling around `events::signup`; configured roles are overwritten by request roles, known secrets are stripped after hook submission; missing phone succeeds under G44 |
| Username/phone/email password | `pages::{username,phone,email}` force type/password fields then reuse `pages::password_signin`; event `signin` checks the truthiness of request `password` and only literal false skips its 2FA search |
| Email OTP | `pages::email` with auth=otp → `otp_signin`; rotate consumed, reload row, derive six-place challenge, late mail checks, same-response email-send; OTP success clears CSRF and redirects |
| Magic link | auth=magic → `magic_link_signin`; same rotation/reload, no success CSRF clear/redirect; link finalization checks consumed hash without a new CSRF/TTL/auth-type gate |
| OTP finalization | GET loads and strips secret without code/hash/TTL checks; POST uses CSRF, trim/padStart(6), UTF-16 length/hash, then shared 2FA handoff |
| 2FA handoff | Missing factor can finalize on GET after consumed-hash match; existing factor waits on GET and POST checks CSRF/TOTP with ±1 step |
| Signout | `events::signout` or GET `pages::signout`; no extra CSRF or revocation lookup |

There is no added challenge TTL, account-active check or global gate beyond source branches. Requests and redirects keep source values and precedence. Invalid-link statusCode(404) followed by setError ends at 400 under the existing response contract. Errors keep source field names, capitalization and status shapes; native causes remain diagnostics rather than fabricated V8 stack fields.

## Supply collaborators at their access points

Signup/signin consume the host Engine and native Environment. Page branches consume CSRF and Session services where the source does. Select [the composite](registration.md) or constituents, register [SQL events](../account.md) explicitly and own [hooks](hooks.md). A missing dependency need not prevent startup; its consuming branch can fail later.

**Open question — current behaviour follows upstream:** email sign-in can skip sending because auth-update leaves a truthy body on the shared response. No response clearing is added. The [real email integration oracle](../../../framework-email/tests/fixtures/source-page-boundaries.json) and [email integration test](../../../framework-email/tests/cases/integration/mod.rs) pin it; a fresh email-send resolve still sends.
