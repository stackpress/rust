# Walk through the SQLite profiles application

Replacing an in-memory store should preserve the operation's public behavior. The
[sql-profile example README](../../../../../examples/sql-profile/README.md) is the runnable
command guide; this page follows its setup, startup and query boundaries and shows a
temporary-file embedding. No listener is needed.

## Install explicitly, then start read-only

Begin with [setup.rs](../../../../../examples/sql-profile/src/setup.rs). Its separate
`install_and_seed` function creates the table, index and canonical rows in one transaction.
It refuses an existing installation instead of clearing or reseeding it. Failed setup rolls
back its schema/data changes, though file creation may already have left an empty file.

Then follow [database.rs](../../../../../examples/sql-profile/src/database.rs). Startup opens
the existing file read-only, prepares a qualified `ordinal`, `id`, `name` projection with
`LIMIT 0`, and publishes the Engine only after verification succeeds. Missing files or schema
fail startup. Configuration supplies the path; request data cannot change it.

This embedding runs those two phases against a temporary file. Add path dependencies on
`stackpress-example-sql-profile` and `stackpress-example-capability-host` for the two examples,
plus `stackpress-server` and `serde_json`, to the [quick-start dependencies](../getting-started.md#prepare-a-consumer).

```rust
use serde_json::json;
use stackpress_server::ConfigLayers;
use stackpress_sql::Engine;

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path).await?;
    let mut layers = ConfigLayers::new();
    layers.value("temporary file", json!({"sqlite": {"path": path}}))?;
    let app = stackpress_example_sql_profile::build_application(layers).await?;
    let reference =
        stackpress_example_capability_host::build_application(ConfigLayers::new()).await?;

    for (id, total) in [("abc", 3), ("abc4", 1), ("ABC", 0)] {
        let actual = app.call("profile-search", json!({"id": id})).await?;
        let expected = reference.call("profile-search", json!({"id": id})).await?;
        let actual = actual.response.try_to_json()?;
        assert_eq!(actual["total"], total);
        assert_eq!(actual, expected.response.try_to_json()?);
        if id == "abc4" {
            assert_eq!(actual["results"][0]["name"], "Grace Hopper");
        }
    }
    app.host().shared_service::<Engine>()?.close().await?;
    Ok(())
}
```

The assertions compare complete response envelopes with the actual in-memory application,
rather than inventing a SQL-specific response format. `abc` returns all three reference
profiles, `abc4` returns Grace Hopper, and uppercase `ABC` returns an empty successful result.
The application closes its SQLite service before the temporary directory is removed.

## Inspect the audited prefix query

Open the `search` helper in [profiles.rs](../../../../../examples/sql-profile/src/profiles.rs).
It wraps one fixed expression in SQLite-tagged `Expr::Trusted` with `TrustedSql::audited`:

```sql
substr(CAST("id" AS BLOB), 1, length(CAST(? AS BLOB))) = CAST(? AS BLOB)
```

Both placeholders bind the same input independently. BLOB length and substring compare the
UTF-8 bytes, matching Rust `str::starts_with` even with embedded NUL bytes. `%`, `_`, quotes,
backslashes and Unicode remain literal data, and matching is case-sensitive.
`TrustedSql::audited` asserts application authorship; it does not sanitize supplied SQL.

The query filters first, orders by the explicit seed ordinal, then applies `profiles.limit`
in `1..=100`. Request data cannot override that limit. It decodes owned public DTOs containing
only `id` and `name`; `total` counts returned rows. The
[SQL core query guide](../../../sql/docs/guides/build-queries.md) and
[value concepts](../../../sql/docs/concepts.md#3-values-and-identifiers) own the shared APIs.

## Trace entry-point parity

The named `profile-search` handler owns business work. The explicit `GET /profiles/:id`
route delegates to it inside the existing lifecycle. The CLI uses the shared parser and
output pipeline. HTTP tests use buffered requests through HttpAdapter without a socket.

The [parity cases](../../../../../examples/sql-profile/tests/cases/parity.rs) compare direct,
CLI and buffered HTTP envelopes against capability-host, including limits, errors and hostile
prefixes. Positive literal-match fixtures prevent wildcard bugs from hiding behind empty
results. [Executable tests](../../../../../examples/sql-profile/tests/executables.rs) exercise
the setup and application binaries. Follow the
[README verification commands](../../../../../examples/sql-profile/README.md#verify-the-example)
when running the complete suite.

You have followed one explicit setup phase, a read-only startup and a shared read operation.
The example demonstrates native persistence with the existing capability contract; it does
not add migration tooling, a generic query endpoint, an async host shutdown hook or a new
production backend. Review [cancellation and shutdown](../concepts.md#cancellation-loses-observation)
before embedding it in a longer-lived application.
