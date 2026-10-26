# Slice 7 library assessment and disposable evidence

Companion to [decisions](slice-7-design-decisions.md). Assessed 2026-10-26.
Candidate versions are deliberately pinned assessment inputs, not claims about
the latest available releases. Source/manifests in this checkout were read only.

## Candidates and exact fit

| Candidate | MSRV / license evidence | Fit and first-party obligations |
| --- | --- | --- |
| Hyper 1.10.1 + hyper-util 0.1.20 | Cached package manifests: Rust 1.63 / 1.64; both MIT. Compiled with Rust 1.96.0. | HTTP/1 codec, incremental bodies, connection graceful shutdown, header timer. Supplies neither Stackpress lifecycle nor admission/draining/static/proxy policy. Use low-level `serve_connection` over supplied IO. |
| Axum 0.8.9 | Cached manifest: Rust 1.80, MIT; compiled in the same probe. | Can wrap Stackpress as one fallback service. Its router rejects a duplicate GET path; directly replacing Stackpress registration loses all-matching semantics. `axum::serve` alone does not establish the required byte/time/admission limits. Recommend omit. |
| First-party layer over Hyper | Our MSRV 1.96; project license policy applies. | Recommended: connection/resource supervisor plus lifecycle adapter. This is first-party policy and ownership code, not an HTTP parser. Explicitly test every gap rather than attributing it to Hyper. |
| SQLx 0.9.0, postgres/mysql | Cached sqlx/core/postgres/mysql manifests: Rust 1.94.0, MIT OR Apache-2.0. Both backends and Rustls feature compiled under 1.96. | Already selected for SQLite. Pools, prepared parameter metadata, owned connections, bounded iteration and close APIs align. We still implement admission, strict codecs, rollback-only state, statement-local outcomes, uncertain lease retirement and no retry. |
| Rustls 0.23.46 + tokio-rustls 0.26.6 | Cached Rustls manifest: Rust 1.71, Apache-2.0 OR ISC OR MIT. [Published Tokio integration manifest](https://docs.rs/crate/tokio-rustls/latest/source/Cargo.toml): Rust 1.71, MIT OR Apache-2.0. | Explicit ring provider and TLS 1.2/1.3 support. SQLx Rustls dependency compiled, but tokio-rustls is not cached and direct HTTP TLS was not built/probed. M0 must verify the pinned integration and handshake fixtures. |
| tokio-postgres 0.7.18 | [Published manifest](https://docs.rs/crate/tokio-postgres/0.7.18/source/Cargo.toml): Rust 1.85, MIT OR Apache-2.0. Not cached; no install/build. | Client plus separately driven Connection future; no built-in pool equivalent to SQLx. Arbitrary IO and explicit cancellation token are useful, but connection task lifetime and pooling need additional design. Cancellation delivery is not guaranteed; cannot promise write rollback. |
| mysql_async 0.37.1 | [Published manifest](https://docs.rs/crate/mysql_async/0.37.1/source/Cargo.toml): MIT OR Apache-2.0, no `rust-version` declaration found. Compatibility with 1.96 is unproved here. | Own pool and transaction support; Rustls/native TLS choices. Drop/dirty connection cleanup still needs an adapter proof. Not cached; no install/build. No reason found to introduce a second driver family now. |

## Source checks

- [Hyper HTTP/1 builder](https://docs.rs/hyper/1.10.1/hyper/server/conn/http1/struct.Builder.html)
  and cached `hyper-1.10.1/src/server/conn/http1.rs`: configure timer,
  `header_read_timeout`, `max_headers`, `max_buf_size`, keep-alive and
  `pipeline_flush`. The latter is a flushing optimization, not a pipelining ban.
- [SQLx package/features](https://docs.rs/crate/sqlx/0.9.0) and cached
  `sqlx-core-0.9.0/src/pool/connection.rs`: `close_on_drop`, consuming async close
  and pool ownership. An ordinary uncertain drop must not recycle a connection.
- [PostgreSQL TLS modes](https://docs.rs/sqlx/0.9.0/sqlx/postgres/enum.PgSslMode.html):
  production policy overrides opportunistic defaults with VerifyFull. MySQL
  equivalent is VerifyIdentity; plaintext is an explicit local/test selection.
- [tokio-postgres cancellation](https://docs.rs/tokio-postgres/0.7.18/tokio_postgres/struct.CancelToken.html)
  and [runtime/TLS](https://docs.rs/tokio-postgres/0.7.18/tokio_postgres/):
  separate cancellation connection and pluggable TLS do not ensure interruption.
- [mysql_async options](https://docs.rs/mysql_async/0.37.1/mysql_async/struct.OptsBuilder.html)
  and [crate contract](https://docs.rs/mysql_async/0.37.1/mysql_async/): finite
  pool policy, TLS feature selection, explicit pool disconnect. Driver tracing can
  include SQL/parameters; it must not become automatic application logging.

These are primary documentation/source observations. Recommendations about fit
are inferences from those contracts plus the executed probes, not upstream claims.

## Disposable probe receipt

Location: `/private/tmp/stackpress-s7-probe/`. Cargo manifest, lock, source,
`build.log`, `http.log`, `http-default-buffer.log` and `database.log` remain there for coordinator replay.
No listening socket was opened. HTTP used `tokio::io::duplex(128)` only; the
database client connected to already-running local services. No install/fetch.

Compiler: rustc 1.96.0 `ac68faa20`, Cargo 1.96.0, aarch64-apple-darwin.
Independent empty probe target was used; repository target output was not used.
Initial `cargo clean` removed zero files. `cargo llvm-cov clean --workspace`
attempted index access and warned on sandbox DNS; it was not a coverage run.
The actual build resolved cached dependencies with `--offline`, then was repeated
with `--offline --locked`, `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.

Probe dependency selection:

```toml
hyper = { version = "=1.10.1", default-features = false,
  features = ["server", "client", "http1"] }
hyper-util = { version = "=0.1.20", default-features = false,
  features = ["tokio"] }
sqlx = { version = "=0.9.0", default-features = false,
  features = ["runtime-tokio", "postgres", "mysql", "tls-rustls-ring"] }
```

The probe also used cached Axum, bytes, http-body-util and futures-util, and
read-only path dependencies on the real lib/server. Axum is assessment-only.
Production selection excludes Hyper client features and SQLx macros/migrate/Any.
SQLx `json` will be added only for actual JSON decoding in M0; it was not probed.

| Executed experiment | Result and exact boundary |
| --- | --- |
| Real `ServerBuilder` / `Server::call` returning explicit SharedNative body | Host recognizes supplied body and status 200; synchronized slot can be taken exactly once. This probes the existing opaque-body seam with a String payload, not the final stream wrapper/output implementation. |
| Hyper stream body with two chunks, two sequential requests on one duplex | Both bodies equal `onetwo`; connection future is Send; no TCP listener. |
| Axum duplicate GET `/` registration | Panics at registration, caught by probe. Confirms it cannot directly model duplicate Stackpress listeners. |
| Hyper incomplete header with 30ms header timeout | Completes with error within 1s watchdog. A timer is configured explicitly. |
| Graceful shutdown during pending handler | Passed: pending handler response `drained` was delivered, connection completed within 2s. Not a production supervisor proof. |
| Stalled peer with 128-byte duplex and repeated 64KiB frames | Default Hyper buffering polled 7 frames and failed the probe ceiling. Explicit `max_buf_size(8192)` polled 1 frame and passed. Both logs retained; no general end-to-end heap guarantee. |
| SQLx local backend checks | PostgreSQL 12.2 and MySQL 8.0.27: prepared arity, basic bind/row values, uncommitted close rollback, timeout then explicit close and one-slot pool reacquisition all passed. MySQL insert ID is statement-local and UPDATE returns zero. |

The native database probe created only
`stackpress_s7_2039_18e21ce5a334d5a0` on each server; both were dropped. It did
not inspect other databases/roles/settings or change server configuration. The
first sandbox attempt failed PermissionDenied before connection; approved replay
passed in 0.8s, under a 45s process watchdog. Per-operation production deadlines
and error/unwind fixture cleanup remain required implementation work.

## Limits and implementation blockers

- Direct TLS handshakes, invalid certificates, HTTP/2, binary framing attacks,
  full proxy policies, real socket disconnect timing, production limits and
  supported-platform matrix were not proved. TLS feature compilation is not TLS
  verification. Use duplex TLS fixtures in M0/M1, no listener necessary.
- Network cancellation probe explicitly closes the uncertain lease. It does not
  prove a cancelled write never ran, immediate server interruption, or every
  Drop/runtime-shutdown interleaving. Preserve SQLite's loss-of-observation rule.
- SQLx may allocate a native row/packet before first-party output budgets run.
  Retained-value caps do not bound driver/native server memory. Large-cell and
  cleanup timeout risks must remain visible.
- Optional JSON codecs, typed NULLs, native unsupported types, MySQL DDL commit,
  full stream disposition/reporting and foreign Response replacement await M0.
- No repository Cargo test/coverage/publication evidence is claimed by this
  checkpoint. Noncached alternatives stay documentation-assessed, not failed.

## Replay and final source identity

```bash
cd /private/tmp/stackpress-s7-probe
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo build --offline --locked
./target/debug/stackpress-s7-probe
# Database replay needs the same G26 authorization and outer 45s watchdog.
./target/debug/stackpress-s7-probe --database
```

Final probe SHA-256:

- Cargo.toml: `5426819958c61c498b6e445f79cf63e438872d3eef453ac90b0c5d3f5ecbd735`.
- Cargo.lock: `d6adc5ce5afc514c168fdf77ba880ca376c351de7f40a70ebc7750f03694f04a`.
- src/main.rs: `92dab9ed2895f58ab1b43147eed31d6b887e1d51ee4312e37476e4ca0689ba49`.

The parent Python runner used `subprocess.run(..., timeout=15)` for HTTP and
`timeout=45` for the approved database replay. Probe source has no bind/listen
operation; database server listeners were pre-existing. A dead process can leave
a fixture database; replay must retain the exact name and report cleanup failure.
