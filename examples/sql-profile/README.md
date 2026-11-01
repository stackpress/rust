# SQLite profile search

Run the existing profile-search capability against an app-owned SQLite file. The direct-call, CLI and buffered HTTP entry points return the same public envelopes as [capability-host](../capability-host), including case-sensitive ID-prefix matching and seed order.

## Try it

From the repository root, explicitly install and seed a new file:

```sh
cargo run -p stackpress-example-sql-profile --bin sql-profile-setup --locked --offline -- target/sql-profile-develop.sqlite
```

Setup creates the table, index and three reference rows in one transaction. It refuses an existing profile installation; it never clears or reseeds one.

Then search through the ordinary application executable:

```sh
cargo run -p stackpress-example-sql-profile --bin sql-profile --locked --offline -- \
  -b examples/sql-profile/config/develop.json emit profile-search --id abc
```

The JSON envelope contains Ada Lovelace, Grace Hopper and Margaret Hamilton in that order, with `total: 3`. Change the final ID to `abc4` to return Grace Hopper alone. `ABC` returns an empty successful result because the match is case-sensitive.

## Where configuration goes

The original Stackpress `config/common.ts` becomes shared values in [`src/config.rs`](src/config.rs).
Its `shared()` layer sets a fallback `profiles.limit` of two. Per-environment data goes in
[`config/develop.json`](config/develop.json) and [`config/build.json`](config/build.json), selected
with `-b`. Both files select the existing limit of three; their only difference is the SQLite
file path, so development and build-time reads can use separate data.

To try the build environment, explicitly set up its file and select its configuration:

```sh
cargo run -p stackpress-example-sql-profile --bin sql-profile-setup --locked --offline -- target/sql-profile-build.sqlite
cargo run -p stackpress-example-sql-profile --bin sql-profile --locked --offline -- \
  -b examples/sql-profile/config/build.json emit profile-search --id abc
```

Precedence, lowest first: plugin section defaults → shared `src/config.rs` layer → `-b` file →
explicit app code overrides appended after that file. [`build_application`](src/lib.rs) installs
the shared layer before the incoming `ConfigLayers`. JSON holds data; templates, computed values
and environment-derived values belong in `src/config.rs`, with executable values wired through
their existing typed APIs. Neither the CLI nor `ConfigLayers` discovers or executes config code.

Set `profiles.limit` to a value in `1..=100` to narrow the returned rows; request data cannot
override this policy. Paths are relative to the process working directory, so these commands run
from the repository root. The setup commands require fresh files and deliberately fail on repeat.

Startup opens an existing file read-only and verifies the required columns before publishing an `Engine` service. It never installs schema or seeds data. A missing file or missing schema is a startup error; use the separate setup command first.

## Follow the code

[`src/lib.rs`](src/lib.rs) selects a plain database plugin and a profiles plugin through the existing `ServerBuilder`. [`src/profiles.rs`](src/profiles.rs) owns the named `profile-search` handler and explicit `GET /profiles/:id` delegation. Its fixed, bound byte-prefix expression treats `%`, `_`, backslashes, quotes, Unicode and embedded NUL bytes literally, without introducing wildcard or case-folding semantics.

[`src/setup.rs`](src/setup.rs) assigns each canonical seed row an ordinal. Searches filter first, order by that ordinal and then apply the configured limit. Public DTOs contain only `id` and `name`; their `total` counts returned rows. No database path, native row or SQL metadata enters the response.

The HTTP proof uses `HttpAdapter` with buffered requests; no listener starts. Embedding applications can await `app.host().shared_service::<Engine>()?.close().await` before dropping the runtime. The CLI uses `stackpress_cli::main_with` and its existing RAII shutdown path; a universal host shutdown hook is outside this example.

## Verify the example

Run the adapter and application suites:

```sh
cargo nextest run -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline --test-threads 2
cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --doc --locked --offline -- --test-threads 2
```

The application tests compare complete serialized envelopes against the actual in-memory application, preserve CLI newline framing, invoke both real executables and use finite process watchdogs. Positive hostile-prefix fixtures prove literal matching. Independent temporary files also prove startup has no setup effects and failed setup rolls its schema changes back.

The [configuration scenario](tests/cases/configuration.rs) loads both checked-in files through the
real builder, pins the default/shared/file/code precedence with `profiles.limit`, checks the
resulting searches and rejects invalid typed file values. Only database locations are replaced
for isolated test fixtures; tests never change the process working directory.
