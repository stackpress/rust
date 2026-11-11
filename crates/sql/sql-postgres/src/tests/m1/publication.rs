//! Exact output budgets and loss of an owned native connection, under parent CREATE/DROP.
use super::*;

/// Native metadata names and values are charged in bytes; NULL retains only its column name.
async fn retained_output(database: &PostgresDatabase) -> TestResult<()> {
    for (bytes, accepted) in [(5, true), (4, false)] {
        let result = database
            .run(
                query(r#"SELECT $1::TEXT AS "é""#, [SqlValue::Text("☃".into())]),
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

/// Terminate only this statement's own server connection, then prove the pool acquires a new one.
async fn disconnected(database: &PostgresDatabase) -> TestResult<()> {
    let before = integer(database, "SELECT pg_backend_pid()").await?;
    let error = database
        .run(
            query("SELECT pg_terminate_backend(pg_backend_pid())", []),
            QueryLimits::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert_ne!(integer(database, "SELECT pg_backend_pid()").await?, before);
    assert_eq!(integer(database, "SELECT 23").await?, 23);
    Ok(())
}

/// Exercise byte accounting and actual network loss only inside the parent's owned database.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;
    retained_output(&database).await?;
    disconnected(&database).await?;
    database.close().await?;
    Ok(())
}
