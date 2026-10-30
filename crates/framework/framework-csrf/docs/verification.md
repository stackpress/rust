# Verify CSRF documentation

CSRF examples share the CSRF/email [documentation checker](../../framework-email/docs/check.py) with email. It includes this crate's README and every Markdown page, checking local paths/anchors, page structure, source/test claim anchors and every fenced Rust example.

## Compile the plain Markdown examples

Follow [the external consumer procedure](../../framework-email/docs/verification.md#compile-every-example). Direct examples compile without the optional server integration; registration and event-handler examples compile in the server graph. Temporary consumer workspaces live outside the repository with separate target subdirectories and byte-identical existing lock snapshots. All applicable Cargo commands use --locked --offline.

The local round-trip and config examples also execute with no listener. Source Rustdoc examples remain a separate `cargo test --doc` check, including the server registrar example. No production source, tests or manifests need editing to check this documentation.

## Check behavior claims

The [claim map](../../framework-email/docs/claims.json) ties defaults, arbitrary-string comparison, mutation order, incoming-cookie retention and no-global-guard statements to [source](../src/lib.rs), [registrar](../src/plugin.rs) and [regression tests](../tests/main.rs). The source/test anchors support review rather than proving arbitrary prose automatically.

Use the existing [standalone consumers](../../framework-email/consumers/) for direct/event/disabled graph evidence. All proofs use the project's benchmark recorder and the released 20 GiB effort cap. [Source behavior](source-behavior.md) keeps OS entropy, timing, browser and installed-host limits explicit.
