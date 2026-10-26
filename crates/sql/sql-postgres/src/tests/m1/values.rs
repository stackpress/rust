//! Native strict scalar, JSON, NULL, arity and retained-output boundaries.
use super::*;

/// Execute the supported/native-rejection matrix on the parent's owned database only.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;

    for (sql, value) in [
        ("SELECT $1::INT2", i16::MIN as i64),
        ("SELECT $1::INT2", i16::MAX as i64),
        ("SELECT $1::INT4", i32::MIN as i64),
        ("SELECT $1::INT4", i32::MAX as i64),
        ("SELECT $1::INT8", i64::MIN),
        ("SELECT $1::INT8", i64::MAX),
    ] {
        assert_eq!(
            super::run(&database, sql, [SqlValue::Integer(value)])
                .await?
                .rows[0]
                .try_get_at::<i64>(0)?,
            value
        );
    }
    for (sql, value) in [
        ("SELECT $1::INT2", i16::MAX as i64 + 1),
        ("SELECT $1::INT2", i16::MIN as i64 - 1),
        ("SELECT $1::INT4", i32::MAX as i64 + 1),
        ("SELECT $1::INT4", i32::MIN as i64 - 1),
    ] {
        assert_eq!(
            database
                .run(
                    query(sql, [SqlValue::Integer(value)]),
                    QueryLimits::default()
                )
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidQuery
        );
    }
    // Native parameter identity rejects coercion and unsupported types, including NULL.
    for (sql, value) in [
        ("SELECT $1::BOOL", SqlValue::Integer(1)),
        ("SELECT $1::INT4", SqlValue::Text("1".into())),
        ("SELECT $1::DATE", SqlValue::Null),
    ] {
        assert_eq!(
            database
                .run(query(sql, [value]), QueryLimits::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Unsupported
        );
    }
    for sql in [
        "SELECT $1::BOOL",
        "SELECT $1::INT2",
        "SELECT $1::INT4",
        "SELECT $1::INT8",
        "SELECT $1::FLOAT4",
        "SELECT $1::FLOAT8",
        "SELECT $1::TEXT",
        "SELECT $1::VARCHAR",
        "SELECT $1::BYTEA",
        "SELECT $1::JSON",
        "SELECT $1::JSONB",
    ] {
        assert_eq!(
            super::run(&database, sql, [SqlValue::Null]).await?.rows[0].cells()[0].1,
            SqlValue::Null
        );
    }
    for (sql, value) in [("SELECT $1::FLOAT4", 1.5), ("SELECT $1::FLOAT8", 1.5)] {
        assert_eq!(
            super::run(&database, sql, [SqlValue::Real(value)])
                .await?
                .rows[0]
                .cells()[0]
                .1,
            SqlValue::Real(value)
        );
    }
    assert_eq!(
        database
            .run(
                query("SELECT $1::FLOAT4", [SqlValue::Real(f64::MAX)]),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidQuery
    );
    for value in [false, true] {
        assert_eq!(
            super::run(&database, "SELECT $1::BOOL", [SqlValue::Bool(value)])
                .await?
                .rows[0]
                .cells()[0]
                .1,
            SqlValue::Bool(value)
        );
    }
    assert_eq!(
        super::run(
            &database,
            "SELECT $1::BYTEA",
            [SqlValue::Blob(vec![0, 255, 128, 1])]
        )
        .await?
        .rows[0]
            .cells()[0]
            .1,
        SqlValue::Blob(vec![0, 255, 128, 1])
    );
    for value in ["", "O'Reilly ☃", "? $1 \\"] {
        assert_eq!(
            super::run(
                &database,
                "SELECT $1::VARCHAR",
                [SqlValue::Text(value.into())]
            )
            .await?
            .rows[0]
                .try_get_at::<String>(0)?,
            value
        );
    }
    for sql in [
        "SELECT 'NaN'::FLOAT8",
        "SELECT 'Infinity'::FLOAT4",
        "SELECT '-Infinity'::FLOAT8",
        "SELECT 1::NUMERIC",
        "SELECT NULL::NUMERIC",
        "SELECT '2026-10-27'::DATE",
        "SELECT '12:00:00'::TIME",
        "SELECT '2026-10-27 00:00:00'::TIMESTAMP",
        "SELECT '00000000-0000-0000-0000-000000000000'::UUID",
        "SELECT ARRAY[1,2]",
    ] {
        assert_eq!(
            database
                .run(query(sql, []), QueryLimits::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Decode
        );
    }
    super::run(&database, "CREATE DOMAIN m1_domain AS INT4", []).await?;
    super::run(&database, "CREATE TYPE m1_enum AS ENUM ('a')", []).await?;
    // PostgreSQL RowDescription flattens domains to their supported base wire type.
    let mut lease = database.lease().await?;
    let statement = lease
        .connection()?
        .prepare(sqlx::SqlStr::from_static("SELECT 1::m1_domain"))
        .await?;
    assert_eq!(
        sqlx::Column::type_info(&sqlx::Statement::columns(&statement)[0])
            .oid()
            .unwrap()
            .0,
        23
    );
    lease.finish(Ok(()), true).await?;
    assert_eq!(integer(&database, "SELECT 1::m1_domain").await?, 1);
    assert_eq!(
        super::run(&database, "SELECT NULL::m1_domain", [])
            .await?
            .rows[0]
            .cells()[0]
            .1,
        SqlValue::Null
    );
    assert_eq!(
        database
            .run(query("SELECT 'a'::m1_enum", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    assert_eq!(
        database
            .run(
                query(r#"SELECT '{"a":1,"a":2}'::JSON"#, []),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    // Invalid UTF-8 is rejected natively, never repaired or exposed as text.
    assert!(
        database
            .run(
                query(
                    "SELECT convert_from($1::BYTEA, 'UTF8')",
                    [SqlValue::Blob(vec![255])]
                ),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    assert_eq!(
        super::run(
            &database,
            "SELECT $1::JSONB",
            [SqlValue::Text("123456789012345678901234567890".into())]
        )
        .await?
        .rows[0]
            .try_get_at::<String>(0)?,
        "123456789012345678901234567890"
    );

    // Invalid input must be rejected by unchanged core validation before native effects.
    assert!(SqlValue::utc_text("2026-02-30T00:00:00Z").is_err());
    let utc = SqlValue::utc_text("2026-10-27T00:00:00Z")?;
    assert_eq!(
        super::run(&database, "SELECT $1::VARCHAR(30)", [utc.clone()])
            .await?
            .rows[0]
            .cells()[0]
            .1,
        utc
    );
    assert!(
        Sql::literal(DIALECT, "SELECT $1::FLOAT8")
            .bind(SqlValue::Real(f64::NAN))
            .render(DIALECT)
            .is_err()
    );
    // Native MySQL cannot produce nonfinite DOUBLE under strict mode; exercise the
    // exact decoding gate as well as its native out-of-range rejection below.
    let mut budget = crate::collect::OutputBudget::new(QueryLimits::default());
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(budget.real(value).unwrap_err().kind(), ErrorKind::Decode);
    }
    // Duplicate names retain position and values; name lookup must remain ambiguous.
    let duplicate = super::run(&database, "SELECT 1 AS duplicate, 2 AS duplicate", []).await?;
    assert_eq!(duplicate.rows[0].cells()[0].0, "duplicate");
    assert_eq!(duplicate.rows[0].try_get_at::<i64>(1)?, 2);
    assert!(duplicate.rows[0].try_get::<i64>("duplicate").is_err());
    // Same native preparation checks both missing and extra arguments independently.
    let missing = database
        .run(query("SELECT $1::INT4", []), QueryLimits::default())
        .await
        .unwrap_err();
    assert!(matches!(
        missing.kind(),
        ErrorKind::Arity {
            expected: 1,
            actual: 0
        }
    ));
    let extra = database
        .run(
            query("SELECT 1", [SqlValue::Integer(1)]),
            QueryLimits::default(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        extra.kind(),
        ErrorKind::Arity {
            expected: 0,
            actual: 1
        }
    ));
    let literal = super::run(&database, "SELECT '?', $1::INT4", [SqlValue::Integer(7)]).await?;
    assert_eq!(literal.rows[0].try_get_at::<String>(0)?, "?");
    assert_eq!(literal.rows[0].try_get_at::<i64>(1)?, 7);
    // Each cap fails the whole result; a later lease remains usable and has no pending row.
    for limits in [
        QueryLimits {
            rows: 0,
            ..QueryLimits::default()
        },
        QueryLimits {
            bytes: 1,
            ..QueryLimits::default()
        },
    ] {
        assert_eq!(
            database
                .run(query("SELECT 'large' AS value", []), limits)
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Limit
        );
        assert_eq!(integer(&database, "SELECT 1").await?, 1);
    }
    assert_eq!(
        database
            .run(
                query("SELECT 1 UNION ALL SELECT 2", []),
                QueryLimits {
                    rows: 1,
                    ..QueryLimits::default()
                }
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    let bounded = database
        .run(
            query("SELECT repeat('x', 10000)", []),
            QueryLimits {
                bytes: 20,
                ..QueryLimits::default()
            },
        )
        .await
        .unwrap_err();
    assert_eq!(bounded.kind(), ErrorKind::Limit);
    // JSON SQL NULL differs from JSON null, and nested objects get deterministic key order.
    let json = super::run(
        &database,
        "SELECT $1::JSONB AS document",
        [SqlValue::Text(
            "{\"z\": [1, null], \"a\": {\"b\": true}}".into(),
        )],
    )
    .await?;
    assert_eq!(
        json.rows[0].try_get_at::<String>(0)?,
        "{\"a\":{\"b\":true},\"z\":[1,null]}"
    );
    let nulls = super::run(&database, "SELECT NULL::JSONB, 'null'::JSONB", []).await?;
    assert_eq!(nulls.rows[0].cells()[0].1, SqlValue::Null);
    assert_eq!(nulls.rows[0].cells()[1].1, SqlValue::Text("null".into()));
    assert_eq!(
        database
            .run(
                query(
                    "SELECT $1::JSONB AS document",
                    [SqlValue::Text("{\"long\":\"value\"}".into())]
                ),
                QueryLimits {
                    bytes: 5,
                    ..QueryLimits::default()
                }
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    assert!(
        database
            .run(
                query(
                    "SELECT $1::JSONB AS document",
                    [SqlValue::Text("invalid JSON".into())]
                ),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    database.close().await?;
    Ok(())
}
