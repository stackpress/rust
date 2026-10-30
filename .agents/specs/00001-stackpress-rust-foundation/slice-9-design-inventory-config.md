# Slice 9 config, CSRF and email parity inventory

Status: Revised design only under G43. **Every existing capability/variant is now in scope.** Configured plugins/individual processes determine protection. Typed Rust configuration and compiled transport selection are mechanics adaptations; they must not add defaults or reject extension input merely to harden it.

## Every relevant configuration key

| Source key/default | Why / source behaviour | Rust mapping now |
| --- | --- | --- |
| session absent/falsy | Deactivate identity contribution, including me/authorize/global guard/account routes | Optional typed section; missing config is a no-op, not selected-but-missing startup failure. |
| session.key=session | Choose normalized cookie/session field for JWT | Same checked low-level cookie field; no __Host prefix or independently imposed flags/scope. |
| session.seed=abc123 | Configurable signing/verification seed; direct class default remains usable | Same fallback/native getter/configure semantics. Never compile actual deployment secrets into shared/browser artifacts, but no new mandatory secret service/minimum length. |
| session.access omitted versus {} | Presence opts into global request listener; empty map authorizes all | Preserve Option<role map> separately from effective {}; no explicit AllowAll/deny-default mode or required nonempty declaration. |
| Session.expires=0 (class setter, not config key) | No added exp by default; nonzero absolute NumericDate | Same setter/direct mechanism; no session.ttl_seconds. Session plugin does not set it. |
| auth absent/falsy | No auth routes/views, but auth Listen events are still registered by auth subplugin | Preserve exact phase gate; no global disable of reusable auth-signup/signin/signout events. |
| auth.base=/auth | Mount auth and account URLs | Same interpolation/fallback and dynamic params; source quirks where auth.base is used directly remain visible. Do not apply a new inventory ownership policy. |
| auth.redirect default / in signout | Default return target | Same caller redirect_uri precedence; no new same-origin/default-repair filter. Existing Rust header framing constraints are mechanical. |
| auth.2fa={} and auth.2fa.issuer fallback brand.name | Extension settings and enrollment issuer label | Retain object/issuer; actual enabled 2FA comes from auth rows of type 2fa, not a new required method inventory or feature config. |
| auth.captcha={} | Extension placeholder | Retain/pass through now; no provider code exists to reproduce and no mandatory CAPTCHA provider added. |
| auth.email.name,address | Sender identity for OTP/link | Same missing-sender no-send branch at its source position; no earlier startup failure. |
| auth.roles=[] | Signup default roles | Merge config first then request data; request roles can override, matching auth/events/signup.ts:17–20. |
| auth.menu[].type,target,name,icon,path; default [] | Menus/footers/targets | Same configured data and navigation; project UI/icon mechanics apply, not a new permission/location filter. |
| auth.password={} | Optional validation rules | Required nonempty secret still source behaviour; all additional checks require their original truthy config value. |
| auth.password.min,max | Ordered length checks; JS UTF-16 code-unit count | encode_utf16 count/compatible numeric comparison; no new minimum/default maximum or Unicode-scalar migration. Keep original wording, including less-than message with inclusive accepted maximum. |
| auth.password.upper,lower,number,special | Optional ASCII/composition predicates | Preserve source server rule and independently different browser checklist predicate; no unification/default complexity policy. |
| database.seed=abc123 in AuthActions.make; constructor seed='' | Deterministic field encryption codec input | Same source selection/defaults, separate from JWT seed; do not replace by per-user salt/lookup-MAC/AEAD policy. |
| database plugin/engine | Actions need an existing SQL engine when called | App-owned Engine/adapter and full Store/Actions; no new eager dependency failure for a disabled/unused process. Explicit install/upgrade etc. are available now. |
| brand.name=Stackpress | Greeting/issuer/template display | Same fallback/content, not secret or policy authority. |
| host fallback http://localhost:${server.port}; server.port=3000 | Email absolute links work with configured or development host | Same fallback and link assembly; no required canonical HTTPS origin or no-Host-fallback hardening. |
| email absent/false | No email-send integration from email plugin | Preserve has check at selection and truthy check in Listen; event itself also no-ops when config absent. |
| cookie.domain,expires,httpOnly,maxAge,path,partitioned,priority,sameSite,secure | Existing transport/global cookie policy | Reuse lib/adapter options and original configured/default behaviour; session/CSRF methods do not impose new identity-specific flags. |
| csrf.name=csrf | Shared token/cookie/data field name | Same name across req.session/req.data/res.session/res.data, not separate new cookie/header names. |
| csrf.error default source page-expiry text | Explicit public retry guidance | Preserve exact default text: “This page may have been requested from an external source. We corrected the issue. Please try again.” No shorter replacement copy. |
| api.expires and API session/application/scopes config | Separate API credential lifecycle, not JWT expiry | Task 00021 owns all API behaviours/SQL now; preserve distinction in the shared interface, no use of api.expires as session JWT TTL. |
| language/r22n/view settings referenced by auth pages | Existing locale/layout context | Reuse accepted Rust language/view ownership and G39–G41; no new identity-language coupling, nor restoration of removed framework-ui/preferences packages. |

No ttl_seconds, empty_access, auth.methods, required Origin, attempt-budget, browser-binding, credential-strength or provider-egress keys are introduced by this port. Preserve extension maps and missing/null/empty distinctions where source consumes them. Optional typed shapes replace TypeScript types, not a license for arbitrary unknown-field rejection.

## CSRF exports/mechanisms

Source packages/stackpress-csrf exports CsrfConfig, CsrfPlugin, ./types and default ./plugin. All are Now. The concrete plugin registers its default mechanism in Config even without a csrf section (plugin.ts:8–11); do not infer a config gate that source lacks.

| Source surface / lines | Why / behaviour | Rust mapping |
| --- | --- | --- |
| CsrfConfig {name?,error?}; CsrfPlugin clear/generate/valid | Small reusable mechanism; pages decide invocation | Native Service and direct methods, ordinary register function, no global mutation guard, session crate dependency or new protocol. |
| generate(res,ctx), plugin.ts:20–30 | Creates random form token and makes it available in outgoing cookie and SSR data | 32 random bytes→64-character hex; res.session[name]=token; res.data[name]={name,token}; return token. Reuse approved OS randomness/ring; do not sign, bind or expire it. |
| valid(req,res), plugin.ts:32–65 | Compare submitted token to incoming cookie/session value | Read req.session[name] and aggregate req.data[name]. Both must be nonempty strings; compare UTF-8 byte lengths then timing-safe equal. Matching arbitrary nonempty strings are valid; no hex/64-byte-only format gate. |
| timingSafeEqual, plugin.ts:52–60 | Existing source intentionally uses timing-safe comparison only after equal length | subtle/ring comparison for equal-byte slices; public shape/length checks preserve source order. Do not extend constant-time policy to password/TOTP branches that use ordinary source equality. |
| Invalid shape/length/value, plugin.ts:43–64 | Communicate page-expiry failure | Same setError(configured text), status 419/Page Expired, false. Caller returns/continues as source; no implicit global Abort. |
| clear(req,res,ctx), plugin.ts:12–18 | Remove this form token and outgoing cookie/projection | Delete req.data[name], res.session[name], res.data[name], using existing response cookie semantics. No clearing unrelated fields or new issuing-scope policy. |
| No source Origin/method/expiry/binding check | Such policy belongs to each process/plugin | Preserve absence. Query/aggregate values remain eligible through req.data; duplicate parsing follows existing parser, not a new strict duplicate rejection. |

Node/ESM/CJS packaging is not ported; functionality and optional direct use are. Generate-before-validate in the pages compares **incoming** req.session against submitted input; newly staged res.session does not replace that incoming state.

## Email exports and complete transport scope

Source packages/stackpress-email root/./types export EmailConfig; ./plugin exports registrar; ./events default emitter contains email-send. Imported NodeMailer SendMailOptions/SentMessageInfo are mapped to a native typed/extensible transport contract. Static compiled adapters replace Node transport objects; all existing variants remain in this slice.

| Source item | Why / source behaviour | Rust mapping now |
| --- | --- | --- |
| EmailConfig TransportOptions | Allows application-selected transport behaviour | Native Transport trait/callback plus source option extension map. No mandatory authorization/recipient policy beyond the caller's process. |
| SMTPOptions | Configurable SMTP relay/auth/TLS behaviour | lettre SMTP adapter with explicit minimal provider features; map source defaults/options, including configured cleartext/TLS choices. Do not force TLS, forbid developer TLS options or introduce new timeouts as security policy. |
| PoolOptions | Configured reusable SMTP pool semantics | Pool adapter now, selected only for this config; preserve configured pool/concurrency settings, no forced per-Service pooling change. |
| JSONOptions | Capture message as JSON | JSON/recording adapter now with source-compatible result envelope. |
| StreamOptions | Produce encoded message/stream rather than deliver | Native bytes/stream MIME adapter now; source options drive formatting/result. |
| MailOptions | Sendmail process transport | Compiled lettre/native sendmail adapter now, optional feature; no framework ban on process transport. Safe Rust process argument APIs are mechanics, not a recipient/egress policy. |
| SESOptions | Provider transport | Compiled SES adapter/factory with approved reqwest/ring/url as needed, now; no extra AWS SDK is approved. Provider binding/credential callbacks are native mechanics and need parity fixtures, not a deferred scope item. |
| string transport config | Source NodeMailer connection/transport string form | Parse source-supported forms through native/url adapter; do not replace with a credentials-forbidden field-only policy. |
| SendMailOptions fields/attachments/content/headers/envelope/plugins | Caller chooses mail content/options | Preserve from/to/cc/bcc/replyTo/subject/text/html/attachments and source URL/file content options via compiled adapters. Dynamic JS callback/plugin code becomes native callback signatures; do not prohibit capabilities or add size/recipient/SSRF/header caps. Library protocol encoding remains in the MIME/transport implementation. |
| plugin.ts:9–16 config absence/Listen gate | Optional email integration | Missing/falsy email config disables contribution, not a startup error; retain event-level no-config return. |
| events.ts:10–15 skip body/non-200 | Earlier handlers can already own response or error | Preserve exact predicate. Do not add an independent result-presence gate unless the source Response.body semantics already make it equivalent. |
| events.ts:18–29 createTransport/sendMail/result | One configured transport per source invocation, await completion, return provider info or throw | Per-call transport factory/native Send future. Preserve info fields via a transport result DTO/extension map, not a new stripped SendReceipt. No automatic retries/outbox or new success/unknown state machine. |

Authenticated producer restrictions, header/content limits, recipient allowlists, file/URL access policy, transport selection and external delivery credentials remain application/plugin responsibilities, as in source. Rust adapters must remain memory-safe, preserve callback/error ordering and not claim SMTP acceptance proves inbox delivery.
