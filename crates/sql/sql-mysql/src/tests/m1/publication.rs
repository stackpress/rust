//! Exact output budgets and loss of an owned native connection, under parent CREATE/DROP.
use super::*;

/// Native metadata names and values are charged in bytes; NULL retains only its column name.
async fn retained_output(database: &MysqlDatabase) -> TestResult<()> {
    for (bytes, accepted) in [(5, true), (4, false)] {
        let result = database
            .run(
                query(
                    r#"SELECT CAST(? AS CHAR) AS `é`"#,
                    [SqlValue::Text("☃".into())],
                ),
                QueryLimits {
                    bytes,
                    ..Default::default()
                },
            )
            .await;
        if accepted {
            let rows = result?.rows;
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].cells(), &[("é".into(), SqlValue::Text("☃".into()))]);
        } else {
            assert_eq!(result.unwrap_err().kind(), ErrorKind::Limit);
        }
    }
    let result = database
        .run(
            query("SELECT NULL AS n", []),
            QueryLimits {
                bytes: 1,
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(result.rows[0].cells(), &[("n".into(), SqlValue::Null)]);
    assert_eq!(integer(database, "SELECT 19").await?, 19);
    Ok(())
}

/// Native integer widths preserve signedness and the exact shared i64 boundary.
async fn integer_widths(database: &MysqlDatabase) -> TestResult<()> {
    super::run(database,"CREATE TABLE release_widths (a TINYINT,b SMALLINT,c MEDIUMINT,d INT,e TINYINT UNSIGNED,f SMALLINT UNSIGNED,g MEDIUMINT UNSIGNED,h INT UNSIGNED,i BIGINT UNSIGNED,raw VARBINARY(8),textual VARCHAR(8) COLLATE utf8mb4_bin)",[]).await?;
    super::run(database,"INSERT INTO release_widths VALUES (-128,-32768,-8388608,-2147483648,255,65535,16777215,4294967295,9223372036854775807,?,?)",[SqlValue::Blob(vec![0,255]),SqlValue::Text("é".into())]).await?;
    let row = super::run(
        database,
        "SELECT a,b,c,d,e,f,g,h,i,raw,textual FROM release_widths",
        [],
    )
    .await?
    .rows
    .remove(0);
    for (index, expected) in [
        -128,
        -32768,
        -8388608,
        -2147483648,
        255,
        65535,
        16777215,
        4294967295,
        i64::MAX,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(row.try_get_at::<i64>(index)?, expected);
    }
    assert_eq!(row.cells()[9].1, SqlValue::Blob(vec![0, 255]));
    assert_eq!(row.cells()[10].1, SqlValue::Text("é".into()));
    // This native value cannot fit the shared value contract; rejection returns no partial row.
    super::run(
        database,
        "UPDATE release_widths SET i=18446744073709551615",
        [],
    )
    .await?;
    assert_eq!(
        database
            .run(
                query("SELECT i FROM release_widths", []),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    Ok(())
}

/// Kill exactly the retained owned transaction socket; no other session can be selected.
async fn disconnected(database: &MysqlDatabase, name: &str) -> TestResult<()> {
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    let id = transaction
        .run(query("SELECT CONNECTION_ID()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at::<i64>(0)?;
    assert!(id > 0);
    // Administration connects to the same newly created database and uses only its retained ID.
    let options = fixture_options()?.database(name);
    let mut admin = MySqlConnection::connect_with(&options).await?;
    admin
        .execute(sqlx::AssertSqlSafe(format!("KILL CONNECTION {id}")).into_sql_str())
        .await?;
    admin.close().await?;
    assert_eq!(
        transaction
            .run(query("SELECT 1", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Driver
    );
    assert_eq!(
        transaction.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    assert_ne!(integer(database, "SELECT CONNECTION_ID()").await?, id);
    assert_eq!(integer(database, "SELECT 23").await?, 23);
    Ok(())
}

/// Exercise native types, byte accounting and network loss under the existing fixture owner.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;
    retained_output(&database).await?;
    integer_widths(&database).await?;
    disconnected(&database, name).await?;
    database.close().await?;
    Ok(())
}
