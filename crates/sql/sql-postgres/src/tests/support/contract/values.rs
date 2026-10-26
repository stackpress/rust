//! Shared public-trait values assertions; native syntax differences stay in the suite inputs.
use super::{Database, DialectId, Result, SqlValue, run, scalar};

/// Check all strict value variants, native bool representation and complete owned row shape.
pub(super) async fn check(database: &dyn Database) -> Result<()> {
    let dialect = database.dialect();
    for (index, value) in [
        SqlValue::Null,
        SqlValue::Bool(false),
        SqlValue::Bool(true),
        SqlValue::Integer(i64::MIN),
        SqlValue::Integer(i64::MAX),
        SqlValue::Real(1.5),
        SqlValue::Text("O'Reilly ☃ ?".into()),
        SqlValue::Text(String::new()),
        SqlValue::Blob(vec![0, 255, 128]),
        SqlValue::Blob(vec![]),
    ]
    .into_iter()
    .enumerate()
    {
        println!("contract {dialect:?} scalar case {index}");
        let result = run(database, scalar(dialect, &value), [value.clone()]).await?;
        let expected = match (&value, dialect) {
            (SqlValue::Bool(flag), DialectId::Sqlite | DialectId::Mysql) => {
                SqlValue::Integer(i64::from(*flag))
            }
            _ => value,
        };
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].cells()[0], ("value".into(), expected));
    }
    // JSON is validated text at the common boundary; SQL NULL and JSON null remain distinct.
    let json_sql = match dialect {
        DialectId::Postgres => "SELECT $1::JSONB",
        DialectId::Mysql => "SELECT CAST(? AS JSON)",
        DialectId::Sqlite => "SELECT json(?)",
    };
    for value in [
        SqlValue::Null,
        SqlValue::Text("null".into()),
        SqlValue::Text(r#"{"a":true,"z":[1,null]}"#.into()),
    ] {
        assert_eq!(
            run(database, json_sql, [value.clone()]).await?.rows[0].cells()[0].1,
            value
        );
    }
    let duplicate = run(database, "SELECT 1 AS duplicate, 2 AS duplicate", []).await?;
    assert_eq!(duplicate.rows[0].cells().len(), 2);
    assert_eq!(duplicate.rows[0].try_get_at::<i64>(1)?, 2);
    assert!(duplicate.rows[0].try_get::<i64>("duplicate").is_err());
    let ordered = run(database, "SELECT 1 AS item UNION ALL SELECT 2", []).await?;
    assert_eq!(
        ordered
            .rows
            .iter()
            .map(|row| row.try_get_at::<i64>(0).unwrap())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    Ok(())
}
