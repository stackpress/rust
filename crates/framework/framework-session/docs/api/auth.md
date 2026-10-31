# Auth events, pages and hooks

Select native `auth-server` for `auth::{register,events,pages,AuthConfig,AuthSection,Environment,Hooks,templates,totp}`. `auth::AuthSchema` needs only native; [storage reference](storage.md) describes it independently of server processes.

## Registration and handler shapes

[`auth::register`](../../src/auth/plugin.rs) installs auth-signup, auth-signin and auth-signout Listen events in that order regardless of the auth route gate. Active auth config adds the source GET routes followed by POST routes. [Registration tests](../../tests/main/auth_flows/registration.rs) verify missing modes, priorities, ties and Abort. Use [registration guide](../guides/registration.md) for complete examples.

| Surface | Arguments / return |
| --- | --- |
| `events::{signup,signin,signout}` | Ordinary async `(&mut Request,&mut Response,Invocation) -> lib::Result<Flow>` |
| `events::{search,update}` | Same shape; optional generated-style SQL collaborators, registered explicitly |
| `events::normalize_phone(Option<&str>)` | Source trim/format cleanup → Option<String> |
| `events::json_result(&Response)` | Fallible explicit ISO JSON projection; rejects unrepresentable IO/native values |
| `pages::{signup,signin,username,phone,email,signout,otp,magic_link,two_factor}` | Same ordinary async handler shape |
| `pages::{password_signin,otp_signin,magic_link_signin}` | Async `(&mut Request,&mut Response,&Invocation) -> lib::Result<()>`; caller supplies surrounding policy |

**Returns:** handlers mutate the supplied response and normally continue. Field errors, public status, redirects and partial results remain in that response; native host/SQL/conversion errors use Result with diagnostic causes. No missing-listener status is fabricated. [Detailed auth behavior](../auth.md) preserves each flag/error/redirect branch; [source traces](../../tests/main/auth_flows/oracle.rs) and [real SQL tests](../../tests/main/auth_flows/events.rs) anchor them.

## Event input and result shapes

The [event source](../../src/auth/events.rs) and [real event tests](../../tests/main/auth_flows/events.rs) retain the data contract below. The [authentication guide](../guides/authentication.md) distinguishes page checks from direct reuse.

| Event | Request data / observable result |
| --- | --- |
| auth-signup | name, secret and at least one of email/phone/username; optional type and roles. Configured roles are spread first, request wins. Missing phone means no phone under G44; supplied nonstring phone retains trim failure. Returns Profile with auth map after stripping only known credential secrets; sequential failures can retain prior writes. |
| auth-signin | type defaults username; identifier field has that type's name; secret compared when password is source-truthy (missing defaults true). Only literal 2fa=false skips factor search. Success writes JWT claims/cookie and the source result; factor branch may redirect instead. |
| auth-signout | No input required; deletes literal session cookie and sets code 200. |
| auth-search / auth-update | StoreQuery / ID plus patch; same explicit generated-style adapters as account SQL events. |

The source field errors include signup's Invalid Parameters, signin's 404 User Not Found and 401 Invalid Password. Error field keys and code/message precedence are pinned by the tests; they are not replaced with a common account-enumeration response. Wire callers can use `events::json_result` to preserve source Dates as ISO JSON; it rejects native IO rather than guessing a representation.

## Configuration and resources

`AuthSection(Option<AuthConfig>)` retains source-falsy deactivation. AuthConfig holds base/redirect/roles/menu/password/email/2fa/captcha; [configuration guide](../guides/configuration.md) gives defaults and the complete activation table. `Environment` is a nonserializable native resource for database seed, email presence, host/port/brand, local offset/zone, Clock and RandomFraction; it does not claim other plugins' config sections. [Config source](../../src/auth/config.rs) and [timezone tests](../../tests/main/auth_flows/events.rs) pin the mapping.

## Hooks

[`Hooks`](../../src/auth/hooks.rs) uses the email package's frozen TaskSupervisor and TaskOutcome. [The lifecycle guide](../guides/hooks.md) has a finite bind/drain/shutdown example and the source hook names.

| Method | Arguments / behavior |
| --- | --- |
| `default()` | Unbound owned queue/group |
| `bind(&Arc<Server>, tokio::runtime::Handle)` | Weak host binding; schedules queued jobs in submission order |
| `pending() -> usize` | Unscheduled queued jobs only |
| `drain().await -> Vec<TaskOutcome>` | Takes/joins current group; concurrent submissions form the next group |
| `shutdown().await` | Drops pending jobs and aborts/joins current group after host stops calls |

**Returns:** drain retains independent success/error/panic/cancellation outcomes as defined by [email task API](../../../framework-email/docs/api/tasks.md). Parent signup neither awaits nor rolls back hook jobs. Cancellation/drop semantics are covered by [bounded supervision tests](../../tests/main/supervision.rs); never join the group from one of its own jobs.

## Email templates and TOTP

`templates::SigninEmailTemplateInput` borrows brand/email/link/name/pin; missing brand uses Stackpress, blank trimmed name falls back to email, missing pin is empty. Both template builders return `Template {subject,text,html}` with source escaping in text as well as HTML. `totp::{base32_decode,generate_at,verify_at}` accept explicit native time/window/step and return lib Result values. [Template/TOTP vectors](../../tests/main/auth_flows/oracle.rs) pin defaults, escaping, base32 and HMAC-SHA1 behavior.

<!-- example: server -->
```rust
use stackpress_framework_session::auth::templates::{SigninEmailTemplateInput, make_otp_template};

pub fn otp_template() {
    let mail = make_otp_template(SigninEmailTemplateInput {
        brand: Some("Example"),
        email: "reader@example.test",
        link: "https://example.test/auth/signin/otp/id/proof",
        name: Some("Reader"),
        pin: Some("123456"),
    });
    assert_eq!(mail.subject, "Stackpress Sign In OTP");
    assert!(mail.text.contains("123456"));
}
```

The native TOTP helpers are explicit about time and window; they do not choose a new account flow or challenge lifetime.

| Helper | Parameters / returns |
| --- | --- |
| `base32_decode` | `secret: &str` → Result<Vec<u8>> |
| `generate_at` | `secret: &str, seconds: u64, window: i64, step: u64` → Result<String> |
| `verify_at` | `token: &str, secret: &str, seconds: u64, window: i64` → Result<bool>, 30-second step |
| `generate_secret` | `length: usize` → Result<String>, source base32 alphabet with native entropy |

This renders strings only; the email-send event owns delivery. The [email sign-in skip decision](../source-behavior.md#owner-decisions-g46) is resolved under G46: follow the original source, unchanged.
