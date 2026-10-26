# Strict SQL values

`SqlValue` stores explicit native scalar data. It does not guess a type from text or coerce
invalid values to zero/NULL. Source: [value.rs](../../src/value.rs).

```rust
use stackpress_sql::{JsonScalar, JsonValue, Result, SqlValue};

fn main() -> Result<()> {
    assert_eq!(SqlValue::try_from(42_u64)?, SqlValue::Integer(42));
    assert!(SqlValue::try_from(u64::MAX).is_err());
    let utc = SqlValue::utc_text("2026-10-26T10:15:30.123Z")?;
    assert!(matches!(utc, SqlValue::Text(_)));
    let json = SqlValue::json(&JsonValue::Scalar(JsonScalar::Null))?;
    assert_eq!(json, SqlValue::Text("null".into()));
    assert_eq!(SqlValue::Blob(vec![0, 255]).retained_bytes(), 2);
    Ok(())
}
```

The date and JSON constructors return text deliberately. JSON null text stays distinct from
`SqlValue::Null`, and binary bytes do not pass through UTF-8 conversion.

## 1. Variants and conversions

SqlValue is Clone/Debug/PartialEq, with `Null`, `Bool(bool)`, `Integer(i64)`, `Real(f64)`,
`Text(String)`, and `Blob(Vec<u8>)` variants. Finite validation happens during rendering/execution,
so merely constructing `Real(f64::NAN)` succeeds but using it as a bind fails.

| Input conversion | Behavior |
| --- | --- |
| `From<i8/i16/i32/i64/u8/u16/u32>` | Lossless widening to Integer. |
| `TryFrom<u64>` | Integer or InvalidQuery outside signed i64 range. |
| `From<f64>` | Retain Real; nonfinite input fails later validation. |
| `From<bool>` | Preserve Boolean without truthiness coercion. |
| `From<String>`, `From<&str>` | Own/copy Text without parsing. |
| `From<Vec<u8>>` | Own Blob bytes. |

Other input types require an explicit checked conversion. There is no implicit Option-to-NULL,
JSON/date-string detection, unsigned wraparound, or integer-to-real precision loss policy.

## 2. Explicit constructors and accounting

These methods operate on values without opening a database:

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `json(value)` | `&JsonValue` | Encode a bounded structured document as text. |
| `utc_text(value)` | `&str` | Validate canonical UTC date/time and own its text. |
| `retained_bytes(&self)` | none | Count retained scalar bytes. |

**Returns** `json` and `utc_text` return `Result<SqlValue>`; `retained_bytes` returns usize,
excluding allocation/container overhead. JSON encoding rejects duplicate keys, nonfinite values,
and depth/node/byte overflow. Use WriteValue::Json when dialect-aware JSON validation is needed.

UTC syntax is `YYYY-MM-DDTHH:MM:SS[.1-9 digits]Z`. Validation checks the calendar, leap years,
clock components, and year greater than zero; offsets, impossible dates, and leap seconds fail.
It uses no ambient timezone or invalid-date fallback. Arbitrary Text remains arbitrary text.

PostgreSQL cannot retain NUL text; rendering rejects such binds as Unsupported. SQLite/MySQL
retain bound NUL text. A native adapter must preserve storage types rather than infer a type
from a column's declaration.

Related: [rows and FromSqlValue](rows.md), [WriteValue](insert.md#2-writevalue),
[defaults](schema.md#1-sqltype-length-and-defaultvalue), [JSON](json.md).
