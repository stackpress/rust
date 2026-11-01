# Identity, credentials and processes

A browser identity and a database credential answer different questions. The cookie Session verifies claims and configured grants; an Auth row stores a sign-in method; a Profile describes a person, company or another entity. Account processes connect them through ordinary events, with application-owned configuration and resources.

## One token, one verification cache

`Service::load` returns an owned Session, with one cache for verified claims or null. Configuration remains shared: new verification and permission collection read current settings, while already-cached claims stay cached even after expiry or a seed change. There is no SQL lookup, refresh or revocation generation in this mechanism. [Session source](../src/session.rs) and [memoization tests](../tests/main/native.rs) define this lifetime.

`guest()` means verification returned null; it does not assert a nonzero ID. `authorization()` spreads Guest defaults, then claims, then the raw token and effective permits. `can` requires all requested permissions; an empty request returns true before parsing roles. Empty role arrays remain empty, while source-falsy roles fall back to GUEST. Event grants deduplicate by string; route grants deduplicate by shared object identity. See [identity lookup](api/identity.md) and its linked tests.

Use module-qualified names when importing multiple Session types: the foundation Session is a cookie bag, this crate's Session is a JWT identity, and the API model Session is a stored credential. [The plugin consumer](../consumers/plugin/src/main.rs) uses those distinct paths in one application.

## Selection and configuration are separate

A feature selects compiled capabilities. A registrar contributes startup callbacks. Configuration decides which source listeners/routes those callbacks add. Missing `session.access` means there is no global request guard. Empty access installs the source permissive guard, including its zero-status error-offer interaction. No framework default-deny policy is added. [Activation and guard tests](../tests/main/host.rs) and [source response traces](../tests/main/auth_flows/registration.rs) pin that distinction.

Ordinary handlers run by priority, then registration order; any handler may return Abort. `plugin::register` is identity-only; `composite::register` calls auth → email → identity. Auth and account handlers may also be reused directly. Their calling page's method, CSRF or permission checks do not appear automatically on a direct event. [Composition tests](../tests/main/composite.rs) and [direct helper tests](../tests/main/auth_flows/oracle.rs) prove those paths.

## Storage and effects have explicit owners

Auth/Profile Actions retain the application's `Arc<Engine>` and source codecs. Constructors do no I/O; installation and maintenance are explicit. `_in` calls borrow one exclusive `DataSession`; batch owns a transaction. Signup and account processes retain sequential partial effects rather than acquiring an outer transaction. [SQL tests](../tests/main/actions.rs) cover these differences.

Signup hook jobs own separate request/response values and run without the parent awaiting completion. The application binds Hooks after build and drains or shuts them down. The email package owns `email-send` and transport creation. A shared auth-update result can make that event skip; [G46 resolves this behaviour](source-behavior.md#owner-decisions-g46) by following the original source, unchanged.

## Native work and public presentation

The native loader constructs explicit `PageProps`; the Leptos page consumes that public data on SSR and browser targets. Engine, signing/database seeds and SMTP handles are not serialized as services. Source enrollment secret/QR and CSRF form values are intentionally public page data. The caller still chooses what to put into the generic input/results payload. [Projection and page tests](../tests/main/account_ui.rs) exercise both intentional values and native-resource exclusion.

SSR, hydrate and CSR are separate artifacts. Root-local theme/language state and explicit assets belong to presentation; no listener, persistence service or installed host is created by a page. Continue with [shared UI](guides/shared-ui.md) or use the [API index](api/README.md) for a specific surface.
