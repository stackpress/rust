# Stackpress PostgreSQL

`stackpress-sql-postgres` supplies an eagerly opened native database owner, bounded admission,
exclusive transactions and connection cleanup. The [SQL core](../../sql/docs/README.md)
owns builders, strict values, rows and Engine.

## Start here

The [getting started walkthrough](getting-started.md) reads a bound name from an existing
database and prints `profile-name: Ada`. It requires a reachable server and explicit
credentials; importing the crate performs no IO.

## Reading paths

Choose the page that matches your next task:

| Goal | Read |
| --- | --- |
| Open an Engine and read one value | [Getting started](getting-started.md) |
| Understand ownership, reuse and shutdown | [Concepts](concepts.md) |
| Choose a compiled backend using configuration | [Switch backends](guides/switch-backends.md) |
| Complete or cancel a transaction | [Transactions and cancellation](guides/transactions.md) |
| Run isolated native verification | [Test the adapter](guides/testing.md) |
| Look up connection fields, TLS, methods, codecs or limits | [API reference](api/README.md) |

## Runtime and evidence

Async operations require a live Tokio runtime. The owner is not Clone; share `Arc<Engine>`
with application operations. Schema creation, migrations, secrets and retry decisions belong
to the application. There are no exposed SQLx connections or pools.

Native `aarch64-apple-darwin` with PostgreSQL 12.2 is the characterized local
environment. TLS modes are implemented, but the local fixture uses explicit plaintext:
it is not evidence of certificate/authentication interoperability. Other server versions,
external poolers, browser/WASM and other operating systems need separate verification.
The [test guide](guides/testing.md) records the generated-store capabilities and refusals.

Source: [public Rustdoc](../src/lib.rs), [options](../src/options.rs),
[database ownership](../src/database.rs), [codec contract](../CODECS.md) and
[native test runners](../Cargo.toml).
