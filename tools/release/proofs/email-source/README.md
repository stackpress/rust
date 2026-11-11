# Email source-oracle generators

These four unchanged generators execute the original TypeScript/Nodemailer source
with inert transport, process and host seams. They require an explicit upstream
source/package root and an explicit output path. They belong to the repository
integration proofs because those inputs live outside the published Rust crate.

The Rust tests replay the checked-in transport/event/option JSON under
[`framework-email/tests/fixtures/`](../../../../crates/framework/framework-email/tests/fixtures/)
and page vectors under
[`framework-session/tests/email/fixtures/`](../../../../crates/framework/framework-session/tests/email/fixtures/).
Those fixtures remain byte-identical; ordinary Rust tests do not run these
generators or read the upstream source. See the
[source behavior guide](../../../../crates/framework/framework-email/docs/source-behavior.md)
for the scope of each oracle and the native/provider evidence limits.

Run a generator with `node <script> <upstream-root> <owned-output>` through the
repository benchmark recorder. `source-options.cjs` accepts the upstream package's
`src` directory; `source-events.cjs` and `source-page-boundaries.cjs` accept the
upstream `packages` directory; `source-transports.cjs` accepts the installed
Nodemailer package directory. Page-oracle reproduction uses `TZ=UTC`.
