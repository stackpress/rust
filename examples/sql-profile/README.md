# SQLite profile search

Run the existing profile-search capability against an app-owned SQLite file. The direct-call, CLI and buffered HTTP entry points return the same public envelopes as [capability-host](../capability-host), including case-sensitive ID-prefix matching and seed order.

## Try it

From the repository root, explicitly install and seed a new file:

```sh
cargo run -p stackpress-example-sql-profile --bin sql-profile-setup -- target/sql-profile.sqlite
```

Setup creates the table, index and three reference rows in one transaction. It refuses an existing profile installation; it never clears or reseeds one.

Then search through the ordinary application executable:

```sh
cargo run -p stackpress-example-sql-profile --bin sql-profile -- \
  -b examples/sql-profile/config.example.json emit profile-search --id abc
```

The JSON envelope contains Ada Lovelace, Grace Hopper and Margaret Hamilton in that order, with `total: 3`. Change the final ID to `abc4` to return Grace Hopper alone. `ABC` returns an empty successful result because the match is case-sensitive.

## Change the application policy

The example configuration chooses `target/sql-profile.sqlite` and a result limit of three. Set `profiles.limit` to a value in `1..=100` to narrow the returned rows; request data cannot override this policy. Paths are relative to the process working directory, so these commands run from the repository root.

Startup opens an existing file read-only and verifies the required columns before publishing an `Engine` service. It never installs schema or seeds data. A missing file or missing schema is a startup error; use the separate setup command first.

## Follow the code

[`src/lib.rs`](src/lib.rs) selects a plain database plugin and a profiles plugin through the existing `ServerBuilder`. [`src/profiles.rs`](src/profiles.rs) owns the named `profile-search` handler and explicit `GET /profiles/:id` delegation. Its fixed, bound byte-prefix expression treats `%`, `_`, backslashes, quotes, Unicode and embedded NUL bytes literally, without introducing wildcard or case-folding semantics.

[`src/setup.rs`](src/setup.rs) assigns each canonical seed row an ordinal. Searches filter first, order by that ordinal and then apply the configured limit. Public DTOs contain only `id` and `name`; their `total` counts returned rows. No database path, native row or SQL metadata enters the response.

The HTTP proof uses `HttpAdapter` with buffered requests; no listener starts. Embedding applications can await `app.host().shared_service::<Engine>()?.close().await` before dropping the runtime. The CLI uses `stackpress_cli::main_with` and its existing RAII shutdown path; a universal host shutdown hook is outside this example.

## Verify the example

Run the adapter and application suites:

```sh
cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked
cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --doc --locked
```

The application tests compare complete serialized envelopes against the actual in-memory application, preserve CLI newline framing, invoke both real executables and use finite process watchdogs. Positive hostile-prefix fixtures prove literal matching. Independent temporary files also prove startup has no setup effects and failed setup rolls its schema changes back.
