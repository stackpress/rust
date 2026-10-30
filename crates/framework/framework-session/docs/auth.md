# Auth events and native processes (M2)

Select `auth-server` for `stackpress_framework_session::auth::{register, events, pages,
AuthConfig, AuthSection, Environment, Hooks, templates, totp}`. This native-only feature
includes `sql` and `server`. Default/browser graphs retain portable permissions/config
without SQL, cryptography, transport or runtime services.

`plugin::register` remains the independent identity constituent. The full source
composite is coordinated at M4 in auth → email → identity order. Lane B owns email
registration and `email-send`; session never implements a second email transport.
M3 supplies shared presentation props, forms and Leptos views. These M2 routes run
native auth processes and do not claim rendered pages.

## Registration and direct handlers

```rust
use stackpress_framework_session::{auth, plugin as identity};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

fn select(app: &mut ServerBuilder) -> Result<()> {
    auth::register(app)?;
    // Call Lane B's reviewed email registrar here at composite integration.
    identity::register(app)?;
    Ok(())
}
```

Select the outer ordinary function with `app.plugin("account", select)`. Constituents
call ordinary registrars, without nested plugin selections or route ownership. Several
handlers can serve the same event/route; higher priority then registration order applies.
A normal source return continues the chain; an explicit handler Abort stops that chain.

`auth::register` owns only the `auth` typed section and `Hooks` service. It always installs
`auth-signup`, `auth-signin`, `auth-signout` in that order. Missing/null/false/zero/empty
string auth sections disable the 16 process routes, while `{}` activates them. Registration
requires no Engine/session/CSRF/email provider at startup. Processes read dependencies at
their original branch; missing dependencies fail only when consumed.

Every event/page is also an ordinary public handler. For example, `app.on("quit",
events::signout)` works without any auth registrar/config/Engine. Public identity handlers
are `plugin::{me, authorize_event, request_guard}`. `pages::{password_signin, otp_signin,
magic_link_signin}` take `&Invocation` and deliberately do not run the calling page's
method/session/CSRF checks. Applications choose their own event exposure and policies.

Optional adapters `events::{search, update}` implement the source `auth-search` and
`auth-update` data calls using the existing direct SQL Actions. Register them explicitly
or provide the application's existing handlers for those events. The auth registrar
neither selects a global SQL operation policy nor silently registers those extensions.
`search` accepts `storage::StoreQuery`; `update` uses `id` plus patch fields, returning the
first row or null. Existing source ILIKE/create-return/positive-hasnt behavior is retained.

## Native resources and task ownership

During Config the host supplies an `Arc<stackpress_sql::Engine>` when actions need it,
and a `stackpress_framework_csrf::Service` when a process reaches its CSRF calls. Table
installation remains explicit. `auth::Environment` maps keys belonging to other plugins:

| Field | Source key / default |
| --- | --- |
| database_seed | database.seed, missing → abc123; explicit empty is literal |
| email_configured | truthy email config; missing → false |
| host / port | host, otherwise http://localhost:<server.port>, port defaults 3000 |
| brand | brand.name, default Stackpress |
| offset_minutes / zone_name | Native host's local Date.toString context; default UTC |
| clock | Epoch seconds, matching the existing `Clock`; converted to Date.now milliseconds |
| random | Native fraction in [0,1), then source floor(random*1,000,000)/six-place padding |

The host maps these cross-plugin settings explicitly instead of auth claiming another
package's config section. UTC is deterministic; supply the actual local offset/zone when
matching a non-UTC source host. SQL action/schema and challenge conversion use that same
offset. `Environment` is a native service, not serializable configuration or an SSR DTO.
OS randomness substitutes for JavaScript's engine-specific Math.random implementation;
the fraction/format and injectable proof boundary retain the source behavior.

After build, retain `Arc<Server>` and bind `server.host().service::<Hooks>()?` using
`hooks.bind(&server, tokio::runtime::Handle::current())` before accepting calls. A Weak
server binding avoids a registry ownership cycle. Before binding, submitted hook payloads
remain owned in the pending queue; binding schedules them in submission order. No
completion order is promised by the runtime.

Signup submits `email-email-send`, then `auth-phone-verify`, without awaiting either.
Each has an independent owned request/response and a payload captured at the source
submission point, before known auth secrets are removed from the parent result. Lane B's
frozen `TaskSupervisor::spawn_on` owns these futures. Parent cancellation/result delivery
does not await or roll back them. The lifecycle owner drains outcomes with `hooks.drain()`
or stops calls then invokes `hooks.shutdown()` to abort/join. Canceling a drain aborts its
taken group; dropping a supervisor requests abort. A hook must never join its own group.

## Config, flags, errors and redirects

AuthConfig retains base, redirect, roles, menu, password, email, 2fa and captcha. Config
objects alone install no additional check; 2fa/captcha remain source extension objects.
Password policy fields are optional and retain the original assertion order. Rust typed
configuration rejects values outside declared active-object types. Unknown arbitrary
runtime JavaScript objects/functions are not dynamically executable Rust configuration.

| Process | Source behavior |
| --- | --- |
| auth-signup | Config roles first, request overrides; optional phone G44; sequential SQL; original field errors; ordered nonawaited hooks; strip known email/phone/username secrets only |
| auth-signin | Default type=username; truthy password flag; only literal false skips 2FA search; Actions updates consumed before config/search/session steps; no added CSRF/Profile-active check |
| Session cookie | Claims id/name/truthy image and nonempty array roles or GUEST; current configured key, original cookie options/lifetime |
| auth-signout | Delete literal session cookie and set 200; GET process redirects by redirect_uri then auth.redirect then /; no CSRF check |
| Password pages | Generate before POST validation; force password=true and type; preserve submitted 2fa flag; normalize phone; clear/redirect only after code 200 |
| OTP/link issuance | Trim email, search → rotate consumed in parent response → reload persisted row → challenge; missing transport/sender stops only after rotation/reload |
| OTP issuance | Source template/mail → email-send using parent response; success clears CSRF and redirects; no new send acknowledgement |
| Magic issuance | Source template/mail → email-send using parent response; success leaves send result, without CSRF clear/redirect |
| OTP GET | Load row, strip secret, render payload; no code/hash/TTL check |
| OTP POST | CSRF, String/trim/padStart(6), exact UTF-16 length; no digits-only requirement; consumed+code hash, then shared 2FA handoff |
| Link GET/POST | Signed-in shortcut; otherwise consumed hash; no CSRF, TTL or auth-type check; shared 2FA handoff |
| 2FA without factor | Selected auth consumed hash, then password=false/type/token/2fa=false and auth-signin, including on GET without POST/CSRF/code checks |
| 2FA with factor | GET waits; POST validates CSRF/code, source base32/HMAC-SHA1 with ±1 step, then the same signin event |

Most processes use `auth.base || '/auth'`. Route registration and signup fallback use the
source missing-only config default. OTP/link final handoff interpolates literal auth.base,
including `undefined` when missing. Redirects retain encodeURIComponent's ASCII set and
source precedence; no allowlist is added. Invalid links call statusCode(404) then setError,
whose default replaces the final status with 400. A mismatch at 2FA stores the original
flash JSON and redirects to that selected auth type's signin page.

Errors preserve source codes, message capitalization, field keys and partial results.
Native Action causes remain inspectable through `Response::diagnostic`; Rust does not
fabricate V8 stacks/start/end fields absent from the foundation's typed public error shape.
Raw missing-listener responses keep code zero/unset. The empty-access authorize shortcut
therefore still reaches the request guard's error offer, without inventing a denial/status.
False/null/zero/empty-string bodies are source-falsy; empty arrays/objects remain truthy.

## Native values and helper reference

Event adapters preserve Date instants as existing lib `Value::DateTime`, recursively.
Wire/SSR adapters explicitly use `events::json_result(&response)` for ISO JSON projection;
raw bytes/files/native resources fail that conversion. Ordinary JSON objects are never
interpreted as Date markers. `templates::{make_otp_template, make_magic_link_template}`
accept `SigninEmailTemplateInput`; both text and HTML substitutions use original escaping.
`totp::{base32_decode, generate_at, verify_at}` retain source lowercase/padding/leftover-bit,
HMAC-SHA1, six-place code and ordinary comparison behavior. Time/window/step are explicit
native arguments; invalid base32/counter values return native errors.

Source traces and golden helpers are in `tests/auth-flows.json`, reproduced by
`proofs/auth-flows.mjs` against the read-only original source. SQLite tests separately
exercise actual auth events. The external watchdog covers suspended hooks, nonawaiting
parents, bind/queue, cancellation/reaping and nested owned work. No listener or live mail
provider is needed for these finite proofs. See [storage](storage.md) for direct actions.
