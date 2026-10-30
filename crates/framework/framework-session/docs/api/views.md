# Shared Leptos pages and presentation

`leptos` exposes portable `views`; select `leptos-ssr`, `leptos-hydrate` or `leptos-csr` separately for an execution artifact. `leptos-ssr` adds native `presentation` and auth/account process dependencies. The [shared UI guide](../guides/shared-ui.md) links a compiled SQL-backed SSR host and a compiled browser component.

## Public data

[`views/types.rs`](../../src/views/types.rs) defines serializable `PageProps`, Config, Menu and PasswordRules. [Page tests](../../tests/main/account_ui.rs) pin source data bindings, helpers, heads and intentional public payloads.

| Type / operation | Fields or return |
| --- | --- |
| `PageProps` | config, path, method, input, results, code, error, errors, theme, language; Default is an empty GET page |
| `Config` | base, menu, password, brand, logo, favicon, csrf_name/token, styles; source defaults include /auth, /favicon.ico, csrf |
| `Menu` | type/target/name/icon/path; footer type selects lower links |
| `PasswordRules` | optional min/max and upper/lower/number/special booleans; no native seed/hash |
| `PageProps::input(key,fallback)` | String, missing-only fallback |
| `bound(key)` | String, signup results spread followed by request input |
| `identifier(key)` | String from results.auth[key].token |

**Returns:** forms accept deliberately selected public JSON, with native Date conversion performed by native adapters. Generic payloads do not automatically filter caller-supplied secrets. Services are not serialized by the projection; source CSRF/enrollment values remain intentionally public.

## Pages, forms and heads

[`views/mod.rs`](../../src/views/mod.rs) supplies `body(Page, PageProps)`, `head(Page, &PageProps)` and `page(Page, PageProps)`, each returning Leptos AnyView. `page` installs root-local theme/notifications and optional LanguageContext; body/form callers supply their shared contexts. Every named module below exposes the same body/head/page shape with its Page fixed.

| Page | Named module |
| --- | --- |
| Signup / Signin | signup / signin |
| Username / Email / Phone | username / email / phone |
| Otp / Link / TwoFactor | otp / link / two_factor |
| Account / Update / Security | profile / update / security |
| Password / Export / Remove | password / export / remove |
| Enroll / RemoveFactor | enroll / remove_factor |

Direct `signup_form`, `username_form`, `phone_form`, `email_form` accept `&PageProps` and return AnyView. Aliases `AccountPageProps` and `AuthSignupFormProps` are PageProps. Helpers are `use_account_base(&PageProps) -> &str`, `account_csrf_fields(&PageProps)`, `account_back_header(&str, Option<String>, Option<&str>)`, `account_head(&PageProps,&str,&str)` and `account_layout(&PageProps,AnyView)`, with view helpers returning AnyView. See [account layout](../../src/views/account.rs) and [controls](../../src/views/controls.rs) for exact composition.

`password_checklist(&PasswordRules) -> Vec<PasswordCheck>` and `evaluate_password(&str,&[PasswordCheck]) -> PasswordEvaluation` retain source weights/UTF-16 lengths. Checks expose id/label/weight plus pass/score. Evaluation exposes failed/score. `password_strength(RwSignal<String>, PasswordRules)` returns AnyView. [Strength vectors](../../tests/password-strength.json) and [strength tests](../../tests/main/account_ui.rs) pin whitespace and scoring differences from server assertions.

## Native Presentation

`presentation::Presentation` is a cloneable native service with public config/theme/locale/language and checked framework-view PageAssets. Default assets identify session-v1 and /assets/session.js; no files are served. [`presentation.rs`](../../src/presentation.rs) defines the complete implementation.

| Function | Parameters / result |
| --- | --- |
| `load` | `(&Request,&Response,&Invocation) -> lib::Result<PageProps>`; explicit auth/CSRF/public-host projection |
| `render` | Async `(Page,PageProps,&Presentation) -> lib::Result<String>`; finite checked document/snapshot, requires initialized SSR executor and Tokio runtime |

**Returns:** native resource/Date conversion, asset or renderer errors propagate. Views preserve downloads/raw bodies/redirects and the source json opt-out. Contributions use priority -100 after source processes and retain equal-priority registration order/Abort. [UI registration tests](../../tests/main/account_ui/registration.rs) cover these boundaries.

Unset theme stays Light without JavaScript; hydration begins with matching pending state, then adopts the host's frozen preference. Public locale/language/assets remain host choices. [Finite browser proof](../../proofs/browser.cjs) covers both hydration and CSR; it starts no listener. The seven [open source questions](../source-behavior.md#open-owner-questions), including the inherited Moment admin pattern, are not resolved by these shared account pages.
