# Rows, decoding, and statement outcomes

`Row` retains ordered names, values, and optional metadata. `QueryResult` keeps one statement's
outcome. Source: [value.rs](../../src/value.rs).

```rust
use stackpress_sql::{ColumnMetadata, FromSqlValue, QueryResult, Result, Row, SqlValue};

fn main() -> Result<()> {
    let row = Row::new(vec![
        ("id".into(), 7.into()),
        ("name".into(), SqlValue::Null),
    ])
    .with_metadata(vec![
        ColumnMetadata {
            name: "id".into(),
            ordinal: 0,
            declared_type: Some("INTEGER".into()),
        },
        ColumnMetadata {
            name: "name".into(),
            ordinal: 1,
            declared_type: None,
        },
    ])?;
    assert_eq!(row.try_get::<i64>("id")?, 7);
    assert_eq!(row.try_get_at::<Option<String>>(1)?, None);
    assert_eq!(row.columns().len(), row.cells().len());
    assert!(row.retained_bytes()? > 0);
    assert_eq!(bool::from_sql(&SqlValue::Integer(1))?, true);
    let result = QueryResult {
        rows: vec![row],
        affected: 0,
        insert_id: None,
    };
    assert_eq!(result.rows.len(), 1);
    Ok(())
}
```

Metadata checks count/name/ordinal consistency. Native declaration text is diagnostic and never
changes the actual cell's strict storage type.

Contents: [Row](#1-row-and-metadata) · [Decoding](#2-fromsqlvalue) · [Outcomes](#3-queryresult)

## 1. Row and metadata

Row is Clone/Debug/PartialEq with private cells. Duplicate column names remain separate.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(cells)` | `Vec<(String, SqlValue)>` | Own ordered cells, with no declared types. |
| `with_metadata(self, columns)` | `Vec<ColumnMetadata>` | Attach declarations after consistency checks. |
| `columns(&self)` | none | Iterate borrowed metadata in ordinal order. |
| `cells(&self)` | none | Inspect every ordered name/value pair. |
| `retained_bytes(&self)` | none | Account for names, values, and declarations with checked arithmetic. |
| `try_get<T: FromSqlValue>(&self, name)` | `&str` | Decode a unique named column. |
| `try_get_at<T: FromSqlValue>(&self, index)` | `usize` | Decode an exact zero-based ordinal. |

**Returns** `new` returns Self; `with_metadata` returns `Result<Row>`. `columns` returns
`impl ExactSizeIterator<Item = ColumnMetadataRef<'_>>`; `cells` returns `&[(String, SqlValue)]`;
accounting returns `Result<usize>`; lookups return `Result<T>`. Missing/ambiguous names, absent
ordinals, inconsistent metadata, and type/range mismatch return Decode. Byte-count overflow is Limit.

ColumnMetadata is Clone/Debug/Eq/PartialEq with public `name: String`, `ordinal: usize`, and
`declared_type: Option<String>`. ColumnMetadataRef is also Copy, with `name: &'a str`,
`ordinal: usize`, and `declared_type: Option<&'a str>`. Views borrow the row; decoded values own
their contents after the lease ends. Names are counted once, not duplicated for metadata.

## 2. FromSqlValue

`FromSqlValue: Sized` has one operation,
`from_sql(value: &SqlValue) -> Result<Self>`. Implement it for application-specific scalars;
combine Row lookups for application DTOs.

| Decoder | Accepted storage |
| --- | --- |
| `String` | Text only, cloned to owned text. |
| `i64` | Integer only. |
| `i8/i16/i32/u8/u16/u32/u64/usize` | Integer only, checked target range. |
| `f64` | Finite Real only; no implicit Integer conversion. |
| `bool` | Bool or Integer 0/1 only. |
| `Vec<u8>` | Blob only. |
| `Option<T: FromSqlValue>` | SQL NULL becomes None; otherwise delegate to T. |

**Returns** An owned decoded value or a typed Decode error. No lossy string/numeric coercion or
implicit JSON/date parsing occurs. Custom implementations should preserve this error discipline.

## 3. QueryResult

QueryResult is Debug/Default, with these public fields and no inherent operations:

| Field | Type | Default / meaning |
| --- | --- | --- |
| `rows` | `Vec<Row>` | Empty; ordered owned rows. |
| `affected` | `u64` | Zero; native affected count for this statement. |
| `insert_id` | `Option<i64>` | None; absent differs from zero. |

Results are statement-local; no shared last-ID slot exists. Adapter-specific insert-ID and
affected-row semantics remain native behavior rather than predicted values. SQLite observes
rowid transitions, so successful inserts with reused explicit IDs or WITHOUT ROWID tables can
still have no insert_id. Use RETURNING when every generated key is required. See the
[SQLite outcome rules](../../../sql-sqlite/docs/api/database.md).
Core makes no total-operation memory bound for a vector of many plan outcomes.

Related: [decode guide](../guides/decode-rows.md), [values](values.md), [Engine](engine.md),
[SQLite adapter](../../../sql-sqlite/docs/README.md).
