# Render and mount the shared account UI

Keep native account work in loaders/processes and render their explicit public data through the shared Leptos pages. SSR, hydrate and CSR consume the same `Page`/`PageProps`; each artifact selects one renderer mode.

## Start with the finite SSR host

The compiled [account_host example](../../examples/account_host.rs) is the complete native example. Run its `account_host` target with feature `leptos-ssr` and an output-directory argument through the project's recorder. It explicitly initializes the framework-view SSR executor, opens SQLite, installs and seeds tables, registers CSRF/SQL/auth/email/identity, supplies Presentation/Environment and binds Hooks. It writes all 16 pages in three themes plus long-input cases, shuts down hooks, closes Engine and exits. It opens no listener.

Use `presentation::render(Page, PageProps, &Presentation)` for direct finite documents; `presentation::load(&Request, &Response, &Invocation)` projects a native process result. [Presentation source](../../src/presentation.rs) and [UI registration tests](../../tests/main/account_ui/registration.rs) pin -100 view contributions after priority-zero processes, preservation of downloads/raw HTML/redirects and the request `json` opt-out. The last view-only POST `/signin/:type` fallback remains part of source routing. Multiple contributions and Abort still apply.

## Compose a browser page

Use the [browser consumer manifest](../../consumers/browser/Cargo.toml) for this example and select hydrate or csr separately. Provide a Leptos Owner and meta context at the application root; return the shared page as an ordinary view.

<!-- example: browser -->
```rust
use leptos::prelude::*;
use stackpress::framework_session::views::{self, Page, PageProps};

pub fn sign_in_page() -> AnyView {
    leptos_meta::provide_meta_context();
    let props = PageProps {
        path: "/auth/signin/email".into(),
        ..Default::default()
    };
    views::page(Page::Email, props)
}
```

This component creates no native service and performs no network IO. It uses the source email form and head under one root-local theme. The [browser entry](../../consumers/browser/src/lib.rs) shows the complete snapshot decode, checked build/revision, `hydrate_from`/`mount_to` selection and explicit root disposal. Its root factory calls the same `views::page` after decoding `(Page, PageProps)`.

## Choose public props and assets

`PageProps` carries explicit config, input/results/errors, method/path/code, optional theme and checked language. `Presentation` supplies checked module/stylesheet/bootstrap assets, build identity, locale and public brand defaults. It never serializes Engine, Environment, signing/database seeds or SMTP services. Generic input/results are caller-selected; enrollment intentionally displays its secret and QR, and CSRF is intentionally a public hidden input. [Page projection tests](../../tests/main/account_ui.rs) and [email escaping test](../../../framework-email/tests/cases/integration/rendering.rs) check these boundaries without banning source form values.

Light, Dark and unset remain distinct. Unset without JavaScript stays Light; hydration starts with matching Light/pending attributes and adopts the host's frozen preference afterward. Language and theme state are root-local, with no automatic preference persistence. Copying an enrollment secret requires a user action. [Browser matrix](../../proofs/browser.cjs) covers keyboard controls, clipboard seams, translations, theme, narrow/zoomed layouts and hydrate/CSR execution.

## Reuse forms and helpers

Call `signup_form`, `username_form`, `phone_form` or `email_form` inside your existing layout. Supply the shared UI contexts your body/form uses; `page` installs its own root state. Named page modules each export body/head/page, and custom account pages can reuse `account_layout`, `account_head`, back header, CSRF fields, base-path and password helpers. [Views reference](../api/views.md) lists every path and argument.

Phone controls retain the source PH default and formatted full-value submission, followed by source `normalize_phone`; no E.164 requirement is inserted. The UI password checklist deliberately has a different whitespace rule from server assertions. [Strength vectors and forms tests](../../tests/main/account_ui.rs) pin both. Live-provider delivery and installed Tauri desktop/mobile acceptance are separate from these finite native/browser proofs.
