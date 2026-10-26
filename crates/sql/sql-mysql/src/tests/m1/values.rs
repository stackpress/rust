//! Native strict scalar, JSON, NULL, arity and retained-output boundaries.
use super::*;

/// Execute the supported/native-rejection matrix on the parent's owned database only.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;

    for value in [i64::MIN, i64::MAX] {
        assert_eq!(
            super::run(
                &database,
                "SELECT CAST(? AS SIGNED)",
                [SqlValue::Integer(value)]
            )
            .await?
            .rows[0]
                .try_get_at::<i64>(0)?,
            value
        );
    }
    assert_eq!(
        integer(&database, "SELECT CAST(9223372036854775807 AS UNSIGNED)").await?,
        i64::MAX
    );
    assert_eq!(
        database
            .run(
                query("SELECT CAST(18446744073709551615 AS UNSIGNED)", []),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    // Native metadata distinguishes each integer width, unsigned flag, and text/binary family.
    super::run(&database, "CREATE TABLE m1_families (s1 TINYINT,s2 SMALLINT,s3 MEDIUMINT,s4 INT,s5 BIGINT,u1 TINYINT UNSIGNED,u2 SMALLINT UNSIGNED,u3 MEDIUMINT UNSIGNED,u4 INT UNSIGNED,u5 BIGINT UNSIGNED,t CHAR(1),tt TINYTEXT,mt MEDIUMTEXT,lt LONGTEXT,vb VARBINARY(3),tb TINYBLOB,mb MEDIUMBLOB,lb LONGBLOB) ENGINE=InnoDB", []).await?;
    super::run(&database, "INSERT INTO m1_families VALUES (-128,-32768,-8388608,-2147483648,-9223372036854775808,255,65535,16777215,4294967295,9223372036854775807,'x','','m','l',X'',X'FF',X'80',X'00')", []).await?;
    let families = super::run(&database, "SELECT * FROM m1_families", [])
        .await?
        .rows
        .remove(0);
    for (index, expected) in [
        -128,
        -32768,
        -8388608,
        -2147483648,
        i64::MIN,
        255,
        65535,
        16777215,
        4294967295,
        i64::MAX,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(families.try_get_at::<i64>(index)?, expected);
    }
    for (index, expected) in [(10, "x"), (11, ""), (12, "m"), (13, "l")] {
        assert_eq!(families.try_get_at::<String>(index)?, expected);
    }
    for (index, expected) in [
        (14, vec![]),
        (15, vec![255]),
        (16, vec![128]),
        (17, vec![0]),
    ] {
        assert_eq!(families.cells()[index].1, SqlValue::Blob(expected));
    }
    super::run(&database, "INSERT INTO m1_families(s1) VALUES (NULL)", []).await?;
    let nulls = super::run(&database, "SELECT * FROM m1_families WHERE s1 IS NULL", []).await?;
    assert!(
        nulls.rows[0]
            .cells()
            .iter()
            .all(|(_, value)| *value == SqlValue::Null)
    );
    super::run(&database, "CREATE TABLE m1_values (id BIGINT PRIMARY KEY, flag TINYINT(1), u BIGINT UNSIGNED, f FLOAT, d DOUBLE, t VARCHAR(50), b BLOB, j JSON, e ENUM('a')) ENGINE=InnoDB", []).await?;
    for (id, flag) in [(1, 0), (2, 1), (3, 2)] {
        super::run(
            &database,
            "INSERT INTO m1_values(id,flag) VALUES (?,?)",
            [SqlValue::Integer(id), SqlValue::Integer(flag)],
        )
        .await?;
    }
    let flags = super::run(&database, "SELECT flag FROM m1_values ORDER BY id", []).await?;
    assert!(!flags.rows[0].try_get_at::<bool>(0)?);
    assert!(flags.rows[1].try_get_at::<bool>(0)?);
    assert_eq!(flags.rows[2].cells()[0].1, SqlValue::Integer(2));
    assert!(flags.rows[2].try_get_at::<bool>(0).is_err());
    super::run(&database, "INSERT INTO m1_values(id,u,f,d,t,b,j,e) VALUES (4,9223372036854775807,1.5,2.5,?,?,CAST(? AS JSON),'a')", [SqlValue::Text("O'Reilly ☃".into()),SqlValue::Blob(vec![0,255,128]),SqlValue::Text("null".into())]).await?;
    let row = super::run(
        &database,
        "SELECT u,f,d,t,b,j FROM m1_values WHERE id=4",
        [],
    )
    .await?
    .rows
    .remove(0);
    assert_eq!(row.try_get_at::<i64>(0)?, i64::MAX);
    assert_eq!(row.cells()[1].1, SqlValue::Real(1.5));
    assert_eq!(row.cells()[2].1, SqlValue::Real(2.5));
    assert_eq!(row.try_get_at::<String>(3)?, "O'Reilly ☃");
    assert_eq!(row.cells()[4].1, SqlValue::Blob(vec![0, 255, 128]));
    assert_eq!(row.cells()[5].1, SqlValue::Text("null".into()));
    for sql in [
        "SELECT CAST(? AS SIGNED)",
        "SELECT CAST(? AS CHAR)",
        "SELECT CAST(? AS BINARY)",
        "SELECT CAST(? AS JSON)",
    ] {
        assert_eq!(
            super::run(&database, sql, [SqlValue::Null]).await?.rows[0].cells()[0].1,
            SqlValue::Null
        );
    }
    for sql in [
        "SELECT CAST(1 AS DECIMAL(10,2))",
        "SELECT CAST(NULL AS DECIMAL(10,2))",
        "SELECT CAST('2026-10-27' AS DATE)",
        "SELECT CAST('12:00:00' AS TIME)",
        "SELECT CAST('2026-10-27 00:00:00' AS DATETIME)",
        "SELECT e FROM m1_values WHERE id=4",
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
    assert!(
        database
            .run(query("SELECT 1e308 * 1e308", []), QueryLimits::default())
            .await
            .is_err()
    );
    assert!(
        database
            .run(
                query(
                    "INSERT INTO m1_values(id,t) VALUES (5,?)",
                    [SqlValue::Blob(vec![255])]
                ),
                QueryLimits::default()
            )
            .await
            .is_err()
    );

    // Invalid input must be rejected by unchanged core validation before native effects.
    assert!(SqlValue::utc_text("2026-02-30T00:00:00Z").is_err());
    let utc = SqlValue::utc_text("2026-10-27T00:00:00Z")?;
    assert_eq!(
        super::run(&database, "SELECT CAST(? AS CHAR)", [utc.clone()])
            .await?
            .rows[0]
            .cells()[0]
            .1,
        utc
    );
    assert!(
        Sql::literal(DIALECT, "SELECT ? + 0e0")
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
        .run(
            query("SELECT CAST(? AS SIGNED)", []),
            QueryLimits::default(),
        )
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
    let literal = super::run(
        &database,
        "SELECT '?', CAST(? AS SIGNED)",
        [SqlValue::Integer(7)],
    )
    .await?;
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
            query("SELECT REPEAT('x', 10000)", []),
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
        "SELECT CAST(? AS JSON) AS document",
        [SqlValue::Text(
            "{\"z\": [1, null], \"a\": {\"b\": true}}".into(),
        )],
    )
    .await?;
    assert_eq!(
        json.rows[0].try_get_at::<String>(0)?,
        "{\"a\":{\"b\":true},\"z\":[1,null]}"
    );
    let nulls = super::run(
        &database,
        "SELECT CAST(NULL AS JSON), CAST('null' AS JSON)",
        [],
    )
    .await?;
    assert_eq!(nulls.rows[0].cells()[0].1, SqlValue::Null);
    assert_eq!(nulls.rows[0].cells()[1].1, SqlValue::Text("null".into()));
    assert_eq!(
        database
            .run(
                query(
                    "SELECT CAST(? AS JSON) AS document",
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
                    "SELECT CAST(? AS JSON) AS document",
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
