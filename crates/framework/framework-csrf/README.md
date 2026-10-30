# Explicit CSRF mechanism

`Service::default()` works directly with `stackpress_lib::Request` and `Response`.
Generate stages a fresh outgoing cookie and `{name, token}` form projection. Validate
compares the independent incoming cookie and aggregate form input, accepting arbitrary
equal nonempty UTF-8 strings. Failure sets source 419/Page Expired. Clear removes
submitted aggregate input and outgoing cookie/form state, retaining the incoming cookie.

With `server`, `plugin::register(&mut ServerBuilder)` publishes the same default or
configured service, even with missing/null/falsy CSRF config. `csrf.name` and
`csrf.error` are optional typed string overrides. Typed Rust configuration errors and
existing cookie-name errors propagate. Select it independently; each process chooses
whether to call `valid`. Registration installs no global guard or routes.

The [email consumers](../framework-email/consumers/) exercise direct use, public
registrars and complete compile-time omission in separate standalone workspaces.
Real Lane A auth integration and the finite Leptos SSR projection are covered by
email's integration suite. OS entropy failure and invalid typed startup configuration
remain mechanical error paths; no browser, installed host or timing certification
is inferred from native tests. Full guides/API documentation follow the released
milestones rather than changing these source behaviors.
