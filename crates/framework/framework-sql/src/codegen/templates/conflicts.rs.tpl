/// Explicit native conflict target. Public upsert handlers continue to require the full primary key.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "target", content = "key", deny_unknown_fields)]
pub enum ConflictKey {
    /// Complete ordered primary identity.
    #[serde(rename = "primary")]
    Primary(Key),
[%#@:unique_fields field%]    /// Explicit nonnull [%field.name_literal%] unique value.
    #[serde(rename = [%field.tag_literal%])]
    Unique[%field.variant%]([%field.ty%]),
[%/@:unique_fields%]}
impl [%model%]Store {
    /// Resolve only declared unique/primary targets and verify the prepared insert agrees before IO.
    fn conflict_target(target: &ConflictKey, value: &_Stored) -> DataResult<Vec<String>> {
        match target {
            ConflictKey::Primary(key) => {
                if !SPEC.agrees_with_key(value, key)? {
                    return Err(DataError::new("upsert key disagrees with create identity"));
                }
                SPEC.key.iter().map(|name| SPEC.column(name).map(|column| column.column.into())).collect()
            },
[%#@:unique_fields field%]            ConflictKey::Unique[%field.variant%](key) => {
                let row = stackpress_framework_schema::to_bounded_value(value).map_err(stackpress_framework_sql::stores::validation)?;
                let key = stackpress_framework_schema::to_bounded_value(key).map_err(stackpress_framework_sql::stores::validation)?;
                if key.is_null() || row.get([%field.name_literal%]) != Some(&key) {
                    return Err(DataError::new("unique upsert key disagrees with create value"));
                }
                Ok(vec![[%field.column_literal%].into()])
            },
[%/@:unique_fields%]        }
    }
    /// Build one native explicit-target UPSERT while keeping all primary/generated values immutable.
    pub fn upsert_on_plan(target: &ConflictKey, value: &_Stored, patch: &Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Insert> {
        let target = Self::conflict_target(target, value)?;
        let key = Key {
[%#@:keys key%]            [%key.member%]: value.[%key.member%][%#?:key.clone%].clone()[%/?:key.clone%],
[%/@:keys%]        };
        let update = Self::update_plan(&key, patch, context)?;
        Ok(Self::insert(value)?.on_conflict(stackpress_sql::InsertConflict::DoUpdate {
            target, assignments: update.assignments().to_vec()
        }))
    }
}
impl [%model%]s {
    /// Native unique/primary UPSERT; RETURNING supplies the existing identity on a unique conflict.
    pub async fn upsert_on(db: &mut DataSession<'_>, target: ConflictKey, input: _Create, patch: Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(db.write(&[%model%]Store::upsert_on_plan(&target, &value, &patch, context)?).await?)
    }
}
