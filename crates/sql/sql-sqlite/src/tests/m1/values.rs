//! Native binding and retained-output boundaries, independent of future builder syntax.

use stackpress_sql::{
    Database, DialectId, Engine, ErrorKind, QueryLimits, Render, Sql, SqlValue, TrustedSql,
};
use tempfile::TempDir;

use crate::SqliteDatabase;
use crate::tests::{options, sql};

/// Rejected byte/row reservations must not consume remaining space or wrap arithmetic.
#[test]
fn rejected_reservations_preserve_remaining_budget() {
    let mut budget = crate::collect::OutputBudget::new(QueryLimits {
        rows: 1,
        bytes: 5,
        ..QueryLimits::default()
    });
    budget.start_row().unwrap();
    assert_eq!(budget.start_row().unwrap_err().kind(), ErrorKind::Limit);
    budget.charge(3).unwrap();
    assert_eq!(budget.charge(3).unwrap_err().kind(), ErrorKind::Limit);
    budget.charge(2).unwrap();
    assert_eq!(
        budget.charge(usize::MAX).unwrap_err().kind(),
        ErrorKind::Limit
    );
    budget.charge(0).unwrap();
    let mut empty = crate::collect::OutputBudget::new(QueryLimits {
        rows: 0,
        bytes: 0,
        ..QueryLimits::default()
    });
    assert_eq!(empty.start_row().unwrap_err().kind(), ErrorKind::Limit);
    assert_eq!(empty.charge(1).unwrap_err().kind(), ErrorKind::Limit);
    empty.charge(0).unwrap();
}

/// Construct explicitly audited fixture SQL; values always remain separate binds.
fn trusted(text: String) -> Sql {
    Sql::trusted(DialectId::Sqlite, TrustedSql::audited(text))
}

/// Every native scalar, hostile text, and exact positional arity survives without input mutation.
pub(super) async fn binding() {
    let root = TempDir::new().unwrap();
    let native = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let engine = Engine::new(native);
    for (input, expected) in [
        (SqlValue::Null, SqlValue::Null),
        (SqlValue::Bool(false), SqlValue::Integer(0)),
        (SqlValue::Bool(true), SqlValue::Integer(1)),
        (SqlValue::Integer(i64::MIN), SqlValue::Integer(i64::MIN)),
        (SqlValue::Integer(i64::MAX), SqlValue::Integer(i64::MAX)),
        (SqlValue::Real(f64::MAX), SqlValue::Real(f64::MAX)),
        (
            SqlValue::Real(f64::MIN_POSITIVE),
            SqlValue::Real(f64::MIN_POSITIVE),
        ),
        (SqlValue::Text(String::new()), SqlValue::Text(String::new())),
        (SqlValue::Blob(Vec::new()), SqlValue::Blob(Vec::new())),
        (
            SqlValue::Blob((0..=255).collect()),
            SqlValue::Blob((0..=255).collect()),
        ),
    ] {
        let statement = sql("SELECT ? AS value").bind(input);
        let snapshot = statement
            .render(DialectId::Sqlite)
            .unwrap()
            .binds()
            .to_vec();
        let row = engine.fetch(&statement).await.unwrap().remove(0);
        assert_eq!(row.cells()[0].1, expected);
        // Re-executing the same detached statement proves native binding did not mutate it.
        assert_eq!(
            statement.render(DialectId::Sqlite).unwrap().binds(),
            snapshot
        );
        assert_eq!(
            engine.fetch(&statement).await.unwrap()[0].cells()[0].1,
            expected
        );
    }
    engine
        .execute(&sql("CREATE TABLE profile(name TEXT NOT NULL)"))
        .await
        .unwrap();
    for value in [
        "' OR 1=1; DROP TABLE profile; --",
        "/* ? */ ?1 :x @x $x",
        "NUL\0line\nquote'",
        "λ🦀é",
        "[]",
        "{}",
        "null",
        "2026-10-26T00:00:00.000Z",
        "not-a-date",
    ] {
        let result = engine
            .execute(&sql("INSERT INTO profile(name) VALUES (?) RETURNING name").bind(value))
            .await
            .unwrap();
        assert_eq!(result.rows[0].try_get::<String>("name").unwrap(), value);
    }
    assert_eq!(
        engine
            .fetch(&sql("SELECT count(*) AS n FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        9
    );
    let quoted =
        sql("SELECT '?' AS literal, ? AS [q?], ? AS `back?`, ? AS \"double?\" /* ? */; -- ?\n")
            .bind("one")
            .bind("two")
            .bind("three");
    let rows = engine.fetch(&quoted).await.unwrap();
    for (column, expected) in [
        ("literal", "?"),
        ("q?", "one"),
        ("back?", "two"),
        ("double?", "three"),
    ] {
        assert_eq!(rows[0].try_get::<String>(column).unwrap(), expected);
    }
    for (statement, expected, actual) in [
        (sql("SELECT ? AS a, ? AS b").bind(1_i64), 2, 1),
        (sql("SELECT '?'").bind(1_i64), 0, 1),
        (sql("SELECT ? /* ? */").bind(1_i64).bind(2_i64), 1, 2),
    ] {
        assert_eq!(
            engine.execute(&statement).await.unwrap_err().kind(),
            ErrorKind::Arity { expected, actual }
        );
    }
    for text in [
        "INSERT INTO profile VALUES (?1)",
        "INSERT INTO profile VALUES (:name)",
        "INSERT INTO profile VALUES (@name)",
        "INSERT INTO profile VALUES ($name)",
        "INSERT INTO profile VALUES ('bad'); DELETE FROM profile",
    ] {
        assert!(
            engine
                .execute(&trusted(text.into()).bind("bad"))
                .await
                .is_err()
        );
    }
    assert_eq!(
        engine
            .fetch(&sql("SELECT count(*) AS n FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        9
    );
    engine.close().await.unwrap();
}

/// Exercise boundary and one-over limits for each payload kind before owning native values.
pub(super) async fn limits() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let default = QueryLimits::default();
    for (text, bytes) in [
        ("SELECT 'abc' AS x", 4),
        ("SELECT zeroblob(3) AS x", 4),
        ("SELECT 17 AS x", 9),
        ("SELECT 1.5 AS x", 9),
        ("SELECT NULL AS long_name", 9),
    ] {
        let query = trusted(text.into()).render(DialectId::Sqlite).unwrap();
        let exact = QueryLimits {
            rows: 1,
            bytes,
            ..default
        };
        assert_eq!(db.run(query.clone(), exact).await.unwrap().rows.len(), 1);
        assert_eq!(
            db.run(
                query,
                QueryLimits {
                    bytes: bytes - 1,
                    ..exact
                }
            )
            .await
            .unwrap_err()
            .kind(),
            ErrorKind::Limit
        );
    }
    let query = sql("SELECT 'a' AS x UNION ALL SELECT 'b' AS x")
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(
        db.run(
            query.clone(),
            QueryLimits {
                rows: 2,
                bytes: 4,
                ..default
            }
        )
        .await
        .unwrap()
        .rows
        .len(),
        2
    );
    for limits in [
        QueryLimits { rows: 1, ..default },
        QueryLimits {
            bytes: 3,
            ..default
        },
        QueryLimits { rows: 0, ..default },
    ] {
        assert_eq!(
            db.run(query.clone(), limits).await.unwrap_err().kind(),
            ErrorKind::Limit
        );
    }
    let empty = sql("SELECT 1 WHERE 0").render(DialectId::Sqlite).unwrap();
    assert!(
        db.run(
            empty,
            QueryLimits {
                rows: 0,
                bytes: 0,
                ..default
            }
        )
        .await
        .unwrap()
        .rows
        .is_empty()
    );
    let null = sql("SELECT NULL AS \"\"")
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(
        db.run(
            null,
            QueryLimits {
                rows: 1,
                bytes: 0,
                ..default
            }
        )
        .await
        .unwrap()
        .rows[0]
            .cells()[0]
            .1,
        SqlValue::Null
    );
    // Native allocation is outside the retained budget; the adapter must reject before
    // cloning this large borrowed blob into a second first-party allocation.
    let huge = sql("SELECT zeroblob(1048576) AS x")
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(
        db.run(
            huge,
            QueryLimits {
                bytes: 8,
                ..default
            }
        )
        .await
        .unwrap_err()
        .kind(),
        ErrorKind::Limit
    );

    let mut many = trusted(format!(
        "SELECT count(*) AS n FROM (VALUES {})",
        vec!["(?)"; 1000].join(",")
    ));
    for _ in 0..1000 {
        many = many.bind(1_i64);
    }
    let query = many.render(DialectId::Sqlite).unwrap();
    assert_eq!(
        db.run(query.clone(), default).await.unwrap().rows[0]
            .try_get::<i64>("n")
            .unwrap(),
        1000
    );
    assert_eq!(
        db.run(
            query,
            QueryLimits {
                binds: 999,
                ..default
            }
        )
        .await
        .unwrap_err()
        .kind(),
        ErrorKind::Limit
    );
    assert!(many.bind(1_i64).render(DialectId::Sqlite).is_err());
    let small = sql("SELECT ? AS x")
        .bind("é")
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(
        db.run(
            small.clone(),
            QueryLimits {
                sql_bytes: small.sql().len(),
                binds: 1,
                bytes: 3,
                ..default
            }
        )
        .await
        .unwrap()
        .rows[0]
            .try_get::<String>("x")
            .unwrap(),
        "é"
    );
    for limits in [
        QueryLimits {
            sql_bytes: small.sql().len() - 1,
            ..default
        },
        QueryLimits {
            bytes: 1,
            ..default
        },
        QueryLimits {
            binds: 0,
            ..default
        },
    ] {
        assert_eq!(
            db.run(small.clone(), limits).await.unwrap_err().kind(),
            ErrorKind::Limit
        );
    }
    // A limit on RETURNING must not become silent partial success; inside a transaction
    // the failure marks rollback-only and closure discards all uncommitted rows.
    db.close().await.unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap())
        .with_limits(QueryLimits { rows: 1, ..default });
    engine
        .execute(&sql("CREATE TABLE profile(id INTEGER)"))
        .await
        .unwrap();
    let mut tx = engine.begin().await.unwrap();
    let error = tx
        .execute(&sql("INSERT INTO profile VALUES (1), (2) RETURNING id"))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Limit);
    assert_eq!(
        tx.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    assert_eq!(
        engine
            .fetch(&sql("SELECT count(*) AS n FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        0
    );
    engine.close().await.unwrap();
}
