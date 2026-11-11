# Email source fixtures

The JSON transport, event and option vectors in this directory remain crate-local
and are replayed by email's direct tests. Their source-oracle generators live at
[`tools/release/proofs/email-source/`](../../../../../tools/release/proofs/email-source/).

The generated model family, source-page vectors and email/session/SQL/SSR
integration tests moved to
[`framework-session/tests/email/`](../../../framework-session/tests/email/).
Session comes after these dependencies in the publication order. Email's package
tests use only its normal dependencies and Tokio.

Regenerate the moved family from the repository root:

```sh
cargo run -p stackpress-framework-session --example regenerate_email_models --locked --offline
```

Pass `-- --check` for a read-only freshness check. The moved integration target
calls the same checker; source or generated-output drift fails. The example
remains buildable when output is missing. Managed fixture output must not be
edited by hand.
