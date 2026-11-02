/// Refresh the generated timestamp from one explicit checked clock before write effects.
fn timestamp(context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Vec<(String, stackpress_sql::SqlValue)>> {
[%#?:timestamped%]    let now = context.now.ok_or_else(|| DataError::new("timestamp refresh requires an explicit clock").with_kind(DataErrorKind::Unsupported))?;
    let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str([%policy_literal%]).map_err(stackpress_framework_sql::codec_error)?;
    policy.validate(&serde_json::json!({[%wire_literal%]:now}), context, true, false)
        .map_err(stackpress_framework_sql::stores::validation)?.into_result().map_err(stackpress_framework_sql::stores::validation)?;
    Ok(vec![([%column_literal%].into(), now.as_str().into())])
[%|%]    let _ = context;
    Ok(Vec::new())
[%/?:timestamped%]}
