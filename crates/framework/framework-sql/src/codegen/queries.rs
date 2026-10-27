//! Typed native selectors and executing read functions. Public DTOs remain schema-owned.
use super::*;

/// Emit native selectors and bounded read/maintenance/atomic functions over the runtime catalog.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let name = &model.rust_name;
    text.push_str("/// Native scalar/list filters; direct store calls are trusted domain APIs.\n#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]\n#[serde(tag=\"field\",content=\"predicate\",deny_unknown_fields)]\npub enum Filter {\n");
    for field in stored.iter().filter(|field| is_scalar(field)) {
        text.push_str(&format!("/// Typed {:?} predicate.\n#[serde(rename={:?})]\n{}(stackpress_framework_schema::Predicate<{}>),\n",field.name,field.name,variant(field),scalar(schema,field)?));
    }
    text.push_str("}\n/// Native declared ordering fields; every page appends complete primary-key ties.\n#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]\npub enum SortField {\n");
    for field in stored
        .iter()
        .filter(|field| is_scalar(field) && !field.multiple && (field.key || field.sortable))
    {
        text.push_str(&format!(
            "/// Order by {:?}.\n#[serde(rename={:?})]\n{},\n",
            field.name,
            field.name,
            variant(field)
        ));
    }
    text.push_str("}\n/// Finite native query with the same page/budget semantics as the pure public contract.\npub type Query=stackpress_framework_sql::stores::Query<Filter,SortField>;\n");
    let (key_type, key_value) = key_argument(model);
    if key_type != "&Key" {
        let key = model
            .fields
            .iter()
            .find(|field| field.key)
            .ok_or_else(|| TransformError::new("legacy scalar key missing"))?;
        text.push_str(&format!("impl {name}Store {{\n/// Preserve the original detached simple-key lookup API, without capturing an executor.\npub fn select(key:{key_type})->Select {{let query=Select::new(COLUMNS.iter().map(|column|column.column)).from({:?}).where_eq({:?},key);\n",model.module,key.column));
        if let Some(active) = model.fields.iter().find(|field| field.active) {
            text.push_str(&format!(
                "let query=query.where_eq({:?},true);\n",
                active.column
            ));
        }
        text.push_str("query.limit(1)\n}\n}\n");
    }
    text.push_str(&format!(r#"
impl {name}Store {{
/// Build a complete-identity ordinary read, including every compound key member.
pub fn select_key(key:&Key, visibility:Visibility)->DataResult<Select>{{SPEC.select_key(key,visibility)}}
/// Build a checked deterministic page without database work.
pub fn search_plan(query:&Query,dialect:stackpress_sql::DialectId)->DataResult<Select>{{stackpress_framework_sql::stores::select(&SPEC,&ReadQuery::from_typed(query)?,dialect,Exposure::Trusted,Visibility::Active)}}
}}
/// Stateless executing functions; callers always select the exact borrowed executor.
pub struct {name}s;
impl {name}s {{
/// Ordinary base lookup; absence is distinct from storage/validation failure.
pub async fn get(db:&mut DataSession<'_>,key:{key_type})->DataResult<Option<_Stored>>{{Self::get_key(db,{key_value}).await}}
/// Ordinary lookup using the full typed identity, including non-text and compound keys.
pub async fn get_key(db:&mut DataSession<'_>,key:&Key)->DataResult<Option<_Stored>>{{
let mut rows=db.fetch(&{name}Store::select_key(key,Visibility::Active)?).await?;
rows.pop().map({name}Store::decode).transpose()
}}
/// Explicit maintenance lookup; inactive/all selection never comes from public request data.
pub async fn get_maintenance(db:&mut DataSession<'_>,key:&Key,visibility:Visibility)->DataResult<Option<_Stored>>{{
let mut rows=db.fetch(&{name}Store::select_key(key,visibility)?).await?;
rows.pop().map({name}Store::decode).transpose()
}}
/// Load a bounded native page, with explicit eager paths and optional exact base count.
pub async fn search(db:&mut DataSession<'_>,query:Query)->DataResult<Page<View3>>{{Self::search_with_limits(db,query,Visibility::Active,RelationLimits::default()).await}}
/// Trusted maintenance/limited search; budgets can only be lowered, never widened.
pub async fn search_with_limits(db:&mut DataSession<'_>,query:Query,visibility:Visibility,limits:RelationLimits)->DataResult<Page<View3>>{{
let page=stackpress_framework_sql::stores::search(db,&super::schema::catalog(),&SPEC,ReadQuery::from_typed(&query)?,Exposure::Trusted,visibility,limits).await?;
let rows=page.rows.into_iter().map(|row|serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error)).collect::<DataResult<Vec<View3>>>()?;
Ok(Page{{rows,total:page.total,skip:page.skip,take:page.take}})
}}
/// Open one owned SQLite read transaction for counts/includes, committing exactly once.
pub async fn search_atomic(engine:&stackpress_sql::Engine,query:Query)->DataResult<Page<View3>>{{
let mut transaction=engine.begin().await?;
let outcome=Self::search(&mut DataSession::transaction(&mut transaction),query).await;
stackpress_framework_sql::finish(transaction,outcome).await
}}
/// Public query allowlist plus recursive public projection; authorization stays app-owned.
pub async fn search_public(db:&mut DataSession<'_>,query:crate::generated::contracts::public::{name}Query)->DataResult<Page<PublicView3>>{{
query.validate().map_err(stackpress_framework_sql::stores::validation)?;
let page=stackpress_framework_sql::stores::search(db,&super::schema::catalog(),&SPEC,ReadQuery::from_typed(&query)?,Exposure::Public,Visibility::Active,RelationLimits::default()).await?;
let rows=page.rows.into_iter().map(|row|{{let row:View3=serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error)?;row.project()}}).collect::<DataResult<Vec<_>>>()?;
Ok(Page{{rows,total:page.total,skip:page.skip,take:page.take}})
}}
/// First deterministic result, retaining explicit include state.
pub async fn find(db:&mut DataSession<'_>,mut query:Query)->DataResult<Option<View3>>{{query.take=1;Ok(Self::search(db,query).await?.rows.into_iter().next())}}
/// The complete bounded page's rows; this is never an unbounded full-table fetch.
pub async fn find_all(db:&mut DataSession<'_>,query:Query)->DataResult<Vec<View3>>{{Ok(Self::search(db,query).await?.rows)}}
/// Count matched base identities without relation fanout or paging.
pub async fn count(db:&mut DataSession<'_>,query:Query)->DataResult<u64>{{stackpress_framework_sql::stores::count(db,&SPEC,&ReadQuery::from_typed(&query)?,Exposure::Trusted,Visibility::Active).await}}
}}
"#));
    Ok(())
}
