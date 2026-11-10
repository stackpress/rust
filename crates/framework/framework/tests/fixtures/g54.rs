//! Shared SQLite/live G54 cases over real generated DDL and actions, without a fixture schema.
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_framework_sql::source::{NativeActions, StoreQuery};
use stackpress_sql::{JsonScalar, JsonValue, SqlValue, WriteValue};

/// Run only in a fresh owned database (or after its Profile teardown); the caller owns closure.
pub async fn prove<S: SourceSchema<Value = Value, Record = Record>>(
    actions: NativeActions<S>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    actions.install().await?;
    let strings = |items: &[&str]| {
        JsonValue::Array(
            items
                .iter()
                .map(|item| JsonValue::Scalar(JsonScalar::String((*item).into())))
                .collect(),
        )
    };
    for (id, tags, references) in [
        (
            "red",
            Some(vec!["red"]),
            WriteValue::Json(strings(&["red"])),
        ),
        (
            "blue",
            Some(vec!["blue"]),
            WriteValue::Json(strings(&["blue"])),
        ),
        (
            "both",
            Some(vec!["red", "blue"]),
            WriteValue::Json(strings(&["red", "blue"])),
        ),
        ("empty", Some(vec![]), WriteValue::Json(strings(&[]))),
        ("sql-null", Some(vec![]), WriteValue::Value(SqlValue::Null)),
        ("missing", None, WriteValue::Json(JsonValue::Object(vec![]))),
        (
            "json-null",
            Some(vec![]),
            WriteValue::Json(JsonValue::Scalar(JsonScalar::Null)),
        ),
    ] {
        let mut row = Record::from_iter([("id".into(), id.into()), ("name".into(), id.into())]);
        if let Some(tags) = tags {
            row.insert(
                "tags".into(),
                Value::Array(tags.into_iter().map(Value::from).collect()),
            );
        }
        actions.create(&row).await?;
        // references is the real nullable JSON column. Write list/NULL boundary
        // values explicitly so normal source Hash/list coercion cannot hide them.
        actions
            .engine
            .execute(
                &actions
                    .engine
                    .update("profile")
                    .set("references", references)
                    .where_eq("id", id),
            )
            .await?;
    }
    let query = |column: &str, value: Value| StoreQuery {
        columns: vec!["id".into()],
        hasnt: Record::from_iter([(column.into(), value)]),
        sort: Record::from_iter([("id".into(), "ASC".into())]),
        ..Default::default()
    };
    // Both the ordinary list and nullable JSON list storage use none-of semantics.
    for column in ["tags", "references"] {
        let single = actions.find_all(&query(column, "red".into())).await?;
        assert_eq!(
            ids(&single),
            ["blue", "empty", "json-null", "missing", "sql-null"]
        );
        let several = query(column, Value::Array(vec!["red".into(), "blue".into()]));
        assert_eq!(
            ids(&actions.find_all(&several).await?),
            ["empty", "json-null", "missing", "sql-null"]
        );
        assert_eq!(actions.count(&several).await?, 4);
    }
    // A missing JSON path and a SQL NULL column both mean no requested member.
    assert_eq!(
        actions
            .count(&query("references:absent", "red".into()))
            .await?,
        7
    );
    // A physically nonexistent SQL column still reaches the backend and fails;
    // absence handling must not silently discard an invalid column expression.
    let missing = actions
        .count(&query("profile.not_installed", "red".into()))
        .await
        .unwrap_err();
    let mut cause = Some(&missing as &(dyn std::error::Error + 'static));
    let mut driver = false;
    let mut diagnostic = String::new();
    while let Some(error) = cause {
        if let Some(error) = error.downcast_ref::<stackpress_sql::SqlError>() {
            driver |= error.kind() == stackpress_sql::ErrorKind::Driver;
        }
        diagnostic.push_str(&error.to_string());
        cause = error.source();
    }
    assert!(
        driver && diagnostic.contains("not_installed"),
        "driver={driver}; {diagnostic}"
    );
    println!(
        "G54 {:?}: none-of, SQL NULL, JSON null, omitted input and missing JSON path passed; nonexistent SQL column rejected",
        actions.engine.dialect()
    );
    actions.uninstall().await?;
    Ok(())
}

/// Observe row identities without decoding nullable JSON into an unrelated field type.
fn ids(rows: &[Record]) -> Vec<&str> {
    rows.iter().map(|row| row["id"].as_str().unwrap()).collect()
}
