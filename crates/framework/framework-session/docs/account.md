# Account processes and shared views (M3)

`auth-server` exposes native `account` handlers and explicit generated-SQL-style events.
`leptos-ssr` adds ordinary view handlers at priority **-100**. Select `leptos-hydrate` or
`leptos-csr` separately for the browser; renderer modes must never be combined.
The default and native-identity-only features do not select a renderer or phone metadata.

## Registration and direct use

```rust
use stackpress_framework_session::{account, auth, plugin as identity};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

fn account_application(app: &mut ServerBuilder) -> Result<()> {
    account::events::register(app)?; // optional explicit source SQL event adapters
    auth::register(app)?;
    // Select stackpress_framework_email::plugin::register here when email is wanted.
    identity::register(app)?;
    Ok(())
}
```

Select this ordinary function with `app.plugin("account", account_application)`. The host
provides Engine, CSRF, `auth::Environment`, configuration, explicit table installation and
hook binding as described in [auth](auth.md). No registration opens a listener or database.
Several handlers may augment the same event or route, ordered by priority then registration;
any handler can Abort. These contributions neither claim a route nor arbitrate conflicts.

Account routes require **both** selected/configured identity and an active `auth` section.
Their source base is `auth.base`, missing → `/auth`.
The auth view registrar also retains the last, view-only POST `/signin/:type` chooser fallback. Missing `session.access` adds no global
request guard. Direct functions retain the same local checks independently of registrars.

| Native handler (`account::`) | Source path below `<base>/account` | Behavior |
| --- | --- | --- |
| account | GET empty and `/security` | Cookie JWT → profile-detail → auth-search; projects id/type/token/verified |
| update | GET/POST `/update` | Profile then username/email/phone writes; password only for a missing sign-in method |
| password | GET/POST `/security/password` | Current/new assertions, one matching hash, sequential updates of every auth row |
| export | GET `/security/export` | Warning unless `download` is truthy; ordered profile-only CSV attachment |
| remove | GET/POST `/security/remove` | POST+confirmed/current password; sequential credential removes then profile-remove |
| enroll | GET/POST `/security/2fa` | Request/stored/generated secret, PNG QR; POST source CSRF/code checks |
| remove_factor | GET/POST `/security/2fa/remove` | Truthy confirmed branch on either method; no csrf.valid call |

The reusable loader is `account::load_account_profile(&Invocation, Value) -> Result<Response>`.
`update_auth` skips blank tokens, updates an existing ID or creates a missing credential.
`assert_password` shares the original ordered password assertions with direct Actions.
`provisioning_uri` retains the double-encoded issuer-in-label; `qr_data_url` emits a PNG
base64 data URL. PNG compression/QR encoder internals are supplied by the approved Rust
libraries; URI, correction level, margin, scale and displayed enrollment data follow source.

`account::events::register` selects auth-search/update/create/remove and
profile-detail/update/remove. These generated-style events pass through an existing body,
non-200 response or missing Engine. Their database.seed fallback is empty, as in the
original generated SQL events; auth signup/signin's Actions.make fallback remains abc123.
Each ID event rejects missing/null/empty IDs before IO. Remove uses ID alone, including
when a calling page also supplies profileId/type. Generic events add no authorization.

## Owned presentation data

With `leptos-ssr`, `presentation::load` explicitly projects request/results/errors and
public auth/CSRF/brand/style settings into `views::PageProps`. The host optionally registers
`presentation::Presentation` with checked assets, explicit locale/theme and public defaults.
Map public brand settings there separately from native `auth::Environment` resource settings.
The service itself, database/signing seeds, SMTP configuration and handles are not serialized.

`presentation::render(Page, PageProps, &Presentation)` produces a finite document through
the existing framework-view adapter. Initialize that adapter's SSR executor explicitly.
Source structured results become view data; raw bodies, downloads, redirects and the source
`json` opt-out pass through. Initial Light/Dark/unset and external theme bootstrap remain
host choices; unset without JavaScript stays Light. Browser theme actions are root-local.

`views::{body, head, page}` accept an exhaustive `Page` variant and explicit `PageProps`.
Named modules expose their own body/head/page entry points: signup, signin, username,
email, phone, otp, link, two_factor, profile, update, security, password, export, remove,
enroll and remove_factor. Direct forms are `signup_form`, `username_form`, `phone_form` and
`email_form`; direct helpers include `use_account_base`, `account_csrf_fields`,
`account_back_header`, `account_head`, `account_layout`, `password_strength`, `password_checklist` and `evaluate_password`.
These functions need no plugin. Body/form-only callers provide their desired shared UI
contexts; page installs its root theme and optional existing LanguageContext.

The source signup spread binds results then request input. Its weighted checklist uses
UTF-16 lengths and excludes whitespace from the UI special-character rule, while server
password assertions retain the original different punctuation/whitespace class. Phone
controls retain the source country/dial choices (PH default) and formatted full-value
submission using approved pinned Rust metadata. Authoritative cleanup remains the source
`normalize_phone`, with no new E.164/country requirement. The existing Leptos PhonePolicy
requires complete, valid E.164 values and rejects extensions; it would change this source
form contract. Its underlying pinned metadata is reused directly instead. No existing
country picker or copy-to-clipboard component provides the required source interaction.

Enrollment intentionally includes secret, QR URL, profile and authId. Its readonly native
secret control and hidden secret control both submit the same value, as in the source,
with a named keyboard copy action. Browser
clipboard access occurs only on user action. Keep intentional source form values distinct
from accidental native-resource exposure when testing snapshots or browser graphs.

## Ordered records and observable source behavior

`columns::Record` and `Value::Object` now use the foundation's insertion-ordered DataMap.
Public names and methods remain; callers naming BTreeMap directly should construct Record
instead. This deliberate underlying public type change preserves stored/exported nested
JSON order and complete CSV bytes. Native Date remains distinct through CSV formatting.

Writes retain source partial effects. Password change ignores returned per-row update
errors, then flashes/redirects. Enrollment likewise retains its source post-write redirect.
Removal's original generated profile-remove sees the already-loaded parent profile body
and passes through: auth rows are removed but the profile remains active, while success is
flashed. The original joined-handler proof records this interaction as a question for the
owner; no fix is selected. No session revocation or new CSRF/reauth/global policy is added.

Run the finite SQL-backed host with the recorder as required by the project workflow:
`cargo run -p stackpress-framework-session --features leptos-ssr --example account_host -- <output>`.
It renders all pages in three themes, invokes the real email-send JSON transport and exits.
Browser verification fulfills every request in memory and starts no listener. Live email,
installed-platform behavior and owner visual acceptance remain separate evidence.
