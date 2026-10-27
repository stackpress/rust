//! Typed explicit primary/single-field-unique conflict targets, independent of public request policy.
use super::*;
/// Emit every schema-supported unique target; nullable unique fields require a nonnull scalar key.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let name = &model.rust_name;
    text.push_str("/// Explicit native conflict target. Public upsert handlers continue to require the full primary key.\n#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]\n#[serde(tag=\"target\",content=\"key\",deny_unknown_fields)]\npub enum ConflictKey {\n/// Complete ordered primary identity.\n#[serde(rename=\"primary\")]\nPrimary(Key),\n");
    for field in model.fields.iter().filter(|field| field.unique) {
        text.push_str(&format!(
            "/// Explicit nonnull {:?} unique value.\n#[serde(rename={:?})]\nUnique{}({}),\n",
            field.name,
            format!("unique.{}", field.name),
            variant(field),
            scalar(schema, field)?
        ));
    }
    text.push_str(&format!("}}\nimpl {name}Store {{\n/// Resolve only declared unique/primary targets and verify the prepared insert agrees before IO.\nfn conflict_target(target:&ConflictKey,value:&_Stored)->DataResult<Vec<String>> {{\nmatch target {{\nConflictKey::Primary(key)=>{{if !SPEC.agrees_with_key(value,key)?{{return Err(DataError::new(\"upsert key disagrees with create identity\"));}}SPEC.key.iter().map(|name|SPEC.column(name).map(|column|column.column.into())).collect()}},\n"));
    for field in model.fields.iter().filter(|field| field.unique) {
        text.push_str(&format!("ConflictKey::Unique{}(key)=>{{let row=stackpress_framework_schema::to_bounded_value(value).map_err(stackpress_framework_sql::stores::validation)?;let key=stackpress_framework_schema::to_bounded_value(key).map_err(stackpress_framework_sql::stores::validation)?;if key.is_null()||row.get({:?})!=Some(&key){{return Err(DataError::new(\"unique upsert key disagrees with create value\"));}}Ok(vec![{:?}.into()])}},\n",variant(field),field.name,field.column));
    }
    text.push_str("}\n}\n/// Build one native explicit-target UPSERT while keeping all primary/generated values immutable.\npub fn upsert_on_plan(target:&ConflictKey,value:&_Stored,patch:&Patch,context:&stackpress_framework_schema::ValueContext<'_>)->DataResult<Insert>{\nlet target=Self::conflict_target(target,value)?;\nlet key=Key{\n");
    for field in model.fields.iter().filter(|field| field.key) {
        text.push_str(&format!(
            "{}:{},\n",
            field.rust_name,
            copy_key(field, &format!("value.{}", field.rust_name))
        ));
    }
    text.push_str("};\nlet update=Self::update_plan(&key,patch,context)?;\nOk(Self::insert(value)?.on_conflict(stackpress_sql::InsertConflict::DoUpdate{target,assignments:update.assignments().to_vec()}))\n}\n}\n");
    text.push_str(&format!("impl {name}s {{\n/// Native unique/primary UPSERT; RETURNING supplies the existing identity on a unique conflict.\npub async fn upsert_on(db:&mut DataSession<'_>,target:ConflictKey,input:_Create,patch:Patch,context:&stackpress_framework_schema::ValueContext<'_>)->DataResult<_Stored>{{\ndb.require_returning()?;\nlet value=input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;\nSelf::one_returned(db.write(&{name}Store::upsert_on_plan(&target,&value,&patch,context)?).await?)\n}}\n}}\n"));
    Ok(())
}
