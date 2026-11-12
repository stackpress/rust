# stackpress-framework-schema

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

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
