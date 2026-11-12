# stackpress-sql-mysql

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Bounded native MySQL execution for Stackpress SQL, using SQLx 0.9.0 and the caller's
Tokio runtime.

Start with the [documentation index](docs/README.md) or
[read a bound value](docs/getting-started.md). The
[codec and lease contract](CODECS.md) is also included in crate Rustdoc.

This checkout uses Rust 1.96, edition 2024 and local path dependencies; publication is disabled.
The workspace facade exposes `stackpress::sql_mysql` through feature `sql-mysql`.
