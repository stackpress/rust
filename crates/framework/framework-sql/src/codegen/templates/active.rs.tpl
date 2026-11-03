[%#?:active%]
impl [%model%]Store {
    /// Explicit idempotent active-state transition; it is never a hard delete.
    pub fn active_plan(key: &Key, active: bool, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<stackpress_sql::Update> {
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str([%policy_literal%]).map_err(stackpress_framework_sql::codec_error)?;
        policy.validate(&serde_json::json!({[%wire_literal%]:active}), context, true, false)
            .map_err(stackpress_framework_sql::stores::preparation_error)?.into_result().map_err(stackpress_framework_sql::stores::preparation_error)?;
        let mut query = stackpress_sql::Update::new([%table_literal%]).and_where(SPEC.key_predicate(key)?).set([%column_literal%], active);
        for (column, value) in timestamp(context)? {
            query = query.set(column, value);
        }
        Ok(query.returning(COLUMNS.iter().map(|column| column.column)))
    }
}
impl [%model%]s {
    /// Idempotently soft-remove a keyed row, including one already inactive.
    pub async fn remove(db: &mut DataSession<'_>, key: &Key, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        Self::optional_returned(db.write(&[%model%]Store::active_plan(key, false, context)?).await?)
    }
    /// Idempotently restore a keyed row; ordinary reads will select it again.
    pub async fn restore(db: &mut DataSession<'_>, key: &Key, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        Self::optional_returned(db.write(&[%model%]Store::active_plan(key, true, context)?).await?)
    }
}
[%/?:active%]
