# Slice 9 complete route and view parity inventory

Status: Revised design only under [G43](slice-9-design-decisions.md). `B=auth.base`, default /auth. **Every original page and flow is in scope now when its source config gate is active.** Source paths below are under packages/stackpress-session/src. Reasons follow comments/call flow. No route ownership is claimed; G42 permits multiple handlers and priority/FIFO/abort cooperation.

## Auth routes: auth/plugin.ts

| Exact source registration | Why / source process | Rust mapping |
| --- | --- | --- |
| GET/POST B/signup; signup view | Render signup; generate response CSRF before checking incoming token; POST emits auth-signup, clears CSRF on success, redirects; existing identity redirects | Same order and request data/roles behaviour. No new atomic signup, Origin gate, password-value scrubbing or POST-only redirects. |
| GET B/signin; signin/index view | Method menu; existing identity redirects using redirect_uri | Leptos chooser with same base/menu and redirects. |
| GET/POST B/signin/username; username view | Page explicitly sets password=true/type=username before emitting shared signin | Preserve page-level enforcement and generic-event passwordless capability separately. |
| GET/POST B/signin/phone; phone view | Same enforcement with source normalizePhone | Do not require E.164 or infer country. Preserve request normalization and source error branches. |
| GET/POST B/signin/email; email view | pass/otp/magic branch selects shared password signin or email issuance | Same auth default pass, invalid-method error, trim in email issuance, sender/config absence no-op at original point. |
| GET/POST B/signin/otp/:auth/:challenge; otp view | Opaque hash is paired with auth id; GET fetches and strips secret for form; POST validates CSRF, trims/pads code, checks hash(consumed.toString()+padded), redirects to 2FA | Preserve exact parameter roles, six-character padded-length check, errors and URL format. No separate pending-challenge table/TTL/attempt/replay policy. |
| GET/POST B/signin/link/:auth/:challenge; link view | Link verifies hash(consumed.toString()) and redirects through same 2FA checkpoint; existing identity redirects | Preserve GET completion path, params and redirects. No browser binding, fragment transfer or mail-scanner confirmation policy. |
| GET/POST B/signin/2fa/:profile/:auth/:challenge; 2fa view | Verify selected profile/auth and consumed hash; if no 2FA, reuse auth-signin with password=false/2fa=false, even on GET; otherwise POST CSRF/TOTP then same reuse | Same data mutation, errors/flash/redirects and code padding. No sealed-proof-only path or matched-step replay table. |
| GET B/signout; page only | Emits/resolves auth-signout then redirect_uri/auth.redirect | Preserve GET and literal session-cookie deletion in event. No new POST confirmation or revocation. |
| POST view-only B/signin/:type → signin/index | Provides a generic view fallback independently of concrete signin handlers | Include this exact view contribution, even though there is no corresponding generic page handler. Do not remove it as redundant or unsafe. |
| All auth view contributions priority -100 | Data handlers precede views; independent handlers can cooperate | Register with existing view/handler lifecycle at the corresponding priority. Existing body/non-200 guards remain per source page; no blanket Abort added. |

## Email and challenge branch detail

| Source anchor | Intent / exact branch to preserve |
| --- | --- |
| auth/pages/signin/email.ts:88–106 PasswordSignin | Force password=true/type=email, emit signin, retain non-200, clear CSRF then redirect. |
| email.ts:118–144,155–196 OTPSignin | Missing email→field error; missing account→Account not found; update consumed then reload to bind issuance to persisted time; floor(Math.random()*1000000), six-character pad; hash timestamp+code. |
| email.ts:198–235 | Missing email config or sender stops at that point; build original template/mail, await email-send with response propagation; clear CSRF then redirect to OTP route. No new uniform sent acknowledgement. |
| email.ts:247–350 MagicLinkSignin | Same lookup/rotate/reload, hash timestamp only, email sender/template/send error propagation; successful first step ends without the OTP redirect/clear sequence. Do not add it for consistency. |
| otp.ts:56–121 | Missing link/row→404 original expired-link message; results strip secret only; GET returns; POST code/answer errors differ; success redirects through 2FA using auth.base as source does. |
| link.ts:49–81 | Missing/mismatched hash→404 original message; success builds profile/auth/challenge URL. No added expiration clock check. |
| 2fa.ts:61–160 | Missing profile/auth→chooser; no rows/current-auth/hash mismatch have distinct source errors/flash; no-2FA completion before POST branch; otherwise POST CSRF then verifyTOTP and rebuild normal signin data. |

## Account routes: session/plugin.ts

Account contributions exist only when the session subplugin is active and auth config exists. Each row includes all its paired GET/POST view registrations at -100.

| Exact source registration | Why / source process | Rust mapping |
| --- | --- | --- |
| GET B/account → account | Validate JWT data/guest, load profile plus linked auth display information, missing profile→401 | Same loadAccountProfile projection (id,type,token,verified), no live-session repository. |
| GET/POST B/account/update → update | GET loads profile; POST normalizes fields, requires current password only when adding a missing sign-in method, updates profile then each auth field sequentially | Existing identifier edits do not gain a new password requirement; no new CSRF call, transaction, verification reset or cookie refresh. |
| GET B/account/security → account loader/security view | Reuse loaded profile/auth details for security navigation | Same shared loader and public projection. |
| GET B/account/security/export → export | Warning unless download flag; profile-only CSV download with source formatting/formula guard | Same selected data/order, object/list flattening, leading = + - @ prefix and account-timestamp.csv attachment. Do not import stronger admin G36 transfer rules into this separate process. |
| GET/POST B/account/security/remove → remove | Session/profile check, POST+confirmed and current password; remove linked auths sequentially then profile, flash and signout redirect | Preserve scoped lookups, first failing removal stop and partial effects. No new CSRF/atomic revocation or generic reauthentication protocol. |
| GET/POST B/account/security/password → password | Current/new password/rules and inequality checks; search auth rows, compare current hash, update all auth rows, success flash/redirect | Preserve errors, sequential updates and handling at the original points; no new CSRF call or JWT invalidation. |
| GET/POST B/account/security/2fa → 2fa/detail | Choose secret from request, existing record unless regenerate, or generated value; produce QR; POST CSRF/code then update/create auth; page exposes url/secret/profile/authId | Same enrollment and repeat-display data. No private provisioning endpoint, server-owned pending secret or enforced current-password gate. |
| GET/POST B/account/security/2fa/remove → 2fa/remove | Session/profile check, truthy confirmed branch, auth id lookup scoped to profile/type; remove, flash, redirect; generate CSRF on remaining/error branches | Retain the confirmed branch without adding POST-only or csrf.valid checks; each source line remains intentional unless owner answers a concrete defect question. |

## Every exported view surface

Every default Page and Head alias is covered below. All source UI behaviour is **Now**; React hooks/types, Frui/FontAwesome imports and ESM/CJS module layout become Leptos/current Rust UI mechanics, not a reason to change data or process policy. G39–G41 remain binding for theme/ownership.

| Source module / every exported name | Behaviour and Rust responsibility |
| --- | --- |
| auth/views/signup: PasswordStrengthProps, PasswordCheck, AuthSignupFormProps, passwordChecklist, evaluatePassword, PasswordStrength, AuthSignupForm, AuthSignupBody, AuthSignupHead, AuthSignupPage, Head | Preserve configured weighted min/max/composition checklist and score, source form input/error/state binding and conditional fields. Server special-character regex includes whitespace; UI regex excludes it. Do not unify those predicates or add a new default strength policy. |
| auth/views/signin/index: AuthSigninBody, AuthSigninHead, AuthSigninPage, Head | Method/menu/footer navigation from config. |
| auth/views/signin/username: UsernameSigninForm, UsernameSigninBody, UsernameSigninHead, UsernameSigninPage, Head | Username/secret/CSRF/redirect form with source input and errors. |
| auth/views/signin/phone: PhoneSigninForm, PhoneSigninBody, PhoneSigninHead, PhoneSigninPage, Head | Phone/secret form and source normalization, not an SMS provider. |
| auth/views/signin/email: EmailSigninForm, EmailSigninBody, EmailSigninHead, EmailSigninPage, Head | Original pass/otp/magic selections and source form/result handling. |
| auth/views/signin/otp: OTPSigninBody, OTPSigninHead, OTPSigninPage, Head | Original code form and auth result after specific secret stripping. |
| auth/views/signin/link: MagicLinkSigninBody, MagicLinkSigninHead, MagicLinkSigninPage, Head | Original email/link flow state, no new generic acknowledgement/confirmation experience. |
| auth/views/signin/2fa: Auth2FASigninBody, Auth2FASigninHead, Auth2FASigninPage, Head | Original second-factor form/data and redirects. |
| session/views/account: AccountProfileBody, AccountProfileHead, AccountProfilePage, Head | Name/image/linked method display from original account projection. |
| session/views/update: AccountUpdateBody, AccountUpdateHead, AccountUpdatePage, Head | Original fields, optional current password and source field issues. |
| session/views/security: AccountSecurityBody, AccountSecurityHead, AccountSecurityPage, Head | Original account security links. |
| session/views/password: AccountPasswordBody, AccountPasswordHead, AccountPasswordPage, Head | Current/new password form and original validation feedback. |
| session/views/remove: AccountRemoveBody, AccountRemoveHead, AccountRemovePage, Head | Confirmed/account-password removal interaction. |
| session/views/export: AccountExportBody, AccountExportHead, AccountExportPage, Head | Source warning/download interaction and profile CSV result. |
| session/views/2fa/detail: AccountTwoFactorBody, AccountTwoFactorHead, AccountTwoFactorPage, Head | Render original selected secret/QR, authId and regenerate/confirm form. |
| session/views/2fa/remove: AccountTwoFactorRemoveBody, AccountTwoFactorRemoveHead, AccountTwoFactorRemovePage, Head | Original authId/confirmed/redirect form. |
| session/views/helpers: AccountPageProps, AccountBackHeaderProps, AccountHeadProps, AccountLayoutProps, useAccountBase, AccountCsrfFields, AccountBackHeader, AccountHead, AccountLayout | Same base fallback, hidden CSRF fields, title/back/navigation/meta/favicon/style selection and account layout. Source source-auth data is not replaced by private-only caller models. |

## Remaining page exports/helpers

| Source export / internal helper group | Mapping now / reason |
| --- | --- |
| session/pages/account: AccountProfile, loadAccountProfile | Resolve profile then auth records; project linked methods to id/type/token/verified and preserve returned status/empty-row branches. |
| session/pages/update: updateAuth; getAuthFields,getUpdateInput,getPasswordError,hasToken,matchesCurrentPassword,shouldRequirePassword,syncAuthFields | Preserve optional empty-as-no-update, existing-id update versus creation, current password only for adding missing methods, sequential first-error stop. Rust helpers may group naturally without altering branch order. |
| session/pages/password: assert | Same ordered rule errors and current-password requirements, separate source UI guidance retained. |
| auth/pages/signin/email: PasswordSignin, OTPSignin, MagicLinkSignin | Direct reusable process functions with same mutable req/res and nested event calls. |
| All default page actions and local template/layout branches | Preserve prior-body/non-200 skips, guest/non-guest paths, generated/cleared CSRF timing, exact errors/flash/redirect/result choices. No mandatory protection added because another page uses it. |

Proposed visual verification S9-UI01–06 in the execution contract covers every page with source-intended data, SSR/hydration/CSR and current accessible UI mechanics. It is not a product redesign or acceptance claim.
