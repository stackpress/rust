# stackpress-framework-schema

Share owned schema values, validation and public UI descriptors across Stackpress targets. Optional code generation consumes finalized Idea configuration; native providers are explicitly selected.

```rust
use stackpress_framework_schema::{DateValue, CONTRACT_REVISION};
assert_eq!(CONTRACT_REVISION, 3);
let date = DateValue::parse("2026-01-01")?;
let _ = date;
# Ok::<(), stackpress_framework_schema::SchemaError>(())
```

The schema contract is independent of a host or SQL driver.

Licence: [Stackpress License](LICENSE).
