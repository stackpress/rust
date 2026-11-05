# Stackpress session, authentication and account UI

Use `stackpress-framework-session` to read a cookie JWT, check configured permissions, manage Auth/Profile records, or reuse the account and sign-in pages. Start with a direct identity check, then select the server and UI features your application needs.

## Start here

[Create and read your first session](docs/getting-started.md) gives a finite example with no server or database. [Concepts](docs/concepts.md) explains how identity, stored credentials and page processes fit together. For an existing application, choose a task below.

| Task | Read |
| --- | --- |
| Register identity or the auth/email/identity group | [Registration](docs/guides/registration.md) |
| Set configuration and native resources | [Configuration](docs/guides/configuration.md) |
| Reuse signup, signin, password, OTP, link and 2FA processes | [Authentication](docs/guides/authentication.md) |
| Own signup hooks and shutdown | [Hooks](docs/guides/hooks.md) |
| Extend the generated Profile/Auth family | [Generated models](docs/source-models.md) |
| Store Auth/Profile records and run maintenance | [Storage](docs/storage.md) |
| Load, update, export or remove an account | [Account processes](docs/account.md) |
| Render SSR and hydrate or mount the shared UI | [Shared UI](docs/guides/shared-ui.md) |
| Import through the facade or verify a disabled build | [Five consumers](docs/guides/consumers.md) |

## Select the surface

The [feature manifest](Cargo.toml) keeps `default = []`: portable configuration and permission matching are available first. `native` adds JWT Session and schemas, `sql` adds Stores/Actions, `server` adds the identity registrar, and `auth-server` adds auth/account processes and the composite. Select `leptos-ssr`, `leptos-hydrate` or `leptos-csr` for each UI artifact separately.

The facade is opt-in: `stackpress::framework_session` is exported by `framework-session`, with forwarding features such as `framework-session-native` and `framework-session-leptos-ssr`. The default facade graph contains only `stackpress-lib` as a first-party dependency. The [consumer graph checks](consumers/graphs.py) pin these boundaries.

## Configuration controls protection

As in the original source, plugins secure the application only as configured. Missing `session` contributes no identity service/events; missing `session.access` contributes no global request guard; missing `auth` removes auth process routes. Each route or event secures its own process. An empty access map remains permissive. See the [activation table](docs/guides/configuration.md#activation) and [source behavior](docs/source-behavior.md) for the exact guard, flags, errors and ten resolved owner decisions under G46.

[`plugin::register`](src/plugin.rs) selects identity only. [`composite::register`](src/composite.rs) calls auth, email, then identity. Both are ordinary registration functions. Multiple handlers may contribute to one route/event in priority then registration order, and any handler may abort the chain; no registrar owns a route. [Composition tests](tests/main/composite.rs) cover both forms.

## Reference and evidence

Use the [API index](docs/api/README.md) for signatures, returns, defaults and errors. [Verification](docs/verification.md) explains the compiled Markdown examples, source oracles and platform limits. Native SQLite and in-memory Chromium proofs do not establish live email, PostgreSQL/MySQL or installed Tauri acceptance.
