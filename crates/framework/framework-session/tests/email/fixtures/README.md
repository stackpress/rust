# Generated email integration models

This family moved from framework-email so its SQL/schema/session/SSR integration
can compile after those crates in the publication order. The authored idea,
managed generated files and source-page vectors retain their original bytes.

`generation.rs` composes the public `SCHEMA_SOURCE` export through `PackageLoader`,
normalizes the authored graph and uses the managed contract/store emitters.
Session-only graphs retain Application/Session as external declarations.

From the repository root, run
`cargo run -p stackpress-framework-session --example regenerate_email_models --locked --offline`.
Add `-- --check` for a read-only freshness check. The `email_integration` test
performs the same check with `auth-server` or `leptos-ssr`. The generator example
can repair missing output before the importing integration tests compile.
Never edit managed output by hand.
