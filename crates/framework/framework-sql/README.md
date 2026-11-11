# stackpress-framework-sql

Support generated stores over borrowed Stackpress SQL executors. Applications own transaction completion; optional migrations, code generation, server registration and SQLite maintenance are selected separately.

```rust
use stackpress_framework_sql::CONTRACT_REVISION;
assert_eq!(CONTRACT_REVISION, 1);
use stackpress_framework_sql::DataSession;
use stackpress_sql::Engine;
fn session(engine: &Engine) -> DataSession<'_> {
    DataSession::pooled(engine)
}
```

A session borrows the supplied engine. Generated application stores use this executor without opening a backend or committing implicitly.

Licence: [Stackpress License](LICENSE).
