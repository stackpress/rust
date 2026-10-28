# Stackpress HTTP host

`stackpress-server-http` serves a built Stackpress application over HTTP/1.1 using
Hyper and Tokio. It owns finite admission, deadlines, sequential connection reuse,
cancellation, optional Rustls HTTPS, and bounded shutdown. Runtime creation and
process signals belong to the application.

Start with the [crate documentation](docs/README.md) and
[production main walkthrough](docs/getting-started.md). The compiled
[deployment example](examples/deployment.rs) serves health, a verified asset, and
an owned response stream. Use the [managed development recipe](docs/guides/development.md)
to run it through devmetrics.

The `tls` feature is optional and off by default. In the workspace facade,
`server-http-host` exposes this crate as `stackpress::server_http_host`;
`server-http-host-tls` also enables TLS. The existing `server-http` feature keeps
its buffered-only meaning. Default and browser consumers gain no network host.

See [limits and statuses](docs/api/limits.md), the accepted
[G31 framing policy](docs/concepts.md#framing-and-connection-reuse), and
[limitations](docs/concepts.md#supported-boundaries) before deployment. These
crates currently use Rust 1.96, edition 2024, and `publish = false`; dependency
examples therefore use local paths.
