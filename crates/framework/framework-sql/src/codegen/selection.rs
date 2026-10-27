//! Default selected projections have a distinct checked type and never widen field exposure.
use super::*;
/// Compile approved default selection and its exact required relation prefixes before output publication.
pub(super) fn paths(
    schema: &Schema,
    model: &Model,
) -> stackpress_idea_transformer::Result<(Vec<String>, Vec<String>)> {
    let ui = schema
        .project_ui()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    let selected = ui
        .models
        .iter()
        .find(|item| item.id == model.name)
        .map(|item| item.default_selection.clone())
        .unwrap_or_default();
    let mut includes = std::collections::BTreeSet::new();
    for path in &selected {
        let mut fields = model.fields.as_slice();
        let mut prefix = Vec::new();
        for part in path.split('.') {
            let field = fields
                .iter()
                .find(|field| field.name == part)
                .ok_or_else(|| TransformError::new("selected field missing"))?;
            prefix.push(part);
            fields = match &field.value_type {
                ValueType::Model(name) => {
                    includes.insert(prefix.join("."));
                    schema
                        .models()
                        .iter()
                        .find(|model| model.name == *name)
                        .ok_or_else(|| TransformError::new("selected relation target missing"))?
                        .fields
                        .as_slice()
                }
                ValueType::Fieldset(name) => schema
                    .fieldsets()
                    .iter()
                    .find(|model| model.name == *name)
                    .ok_or_else(|| TransformError::new("selected fieldset missing"))?
                    .fields
                    .as_slice(),
                _ => &[],
            };
        }
    }
    let includes = includes
        .iter()
        .filter(|path| {
            !includes
                .iter()
                .any(|other| other.len() > path.len() && other.starts_with(&format!("{path}.")))
        })
        .cloned()
        .collect::<Vec<_>>();
    if includes.len() > 16 || includes.iter().any(|path| path.split('.').count() > 3) {
        return Err(TransformError::new(
            "default detail includes exceed the supported sixteen-path/depth-three budget",
        ));
    }
    Ok((selected, includes))
}
/// Emit one model-specific opaque projection type; only a checked public view can construct it.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let name = &model.rust_name;
    let (selected, includes) = paths(schema, model)?;
    text.push_str(&format!(
        r#"
/// Owned app-default field selection, distinct from full rows and full eager views.
/// The private payload can only be built from a recursively projected checked View3.
#[derive(Clone,Debug,PartialEq,serde::Serialize)]
#[serde(transparent)]
pub struct Selected(serde_json::Value);
impl Selected{{
/// Borrow the already validated selected object; no loading or SQL occurs.
pub fn value(&self)->&serde_json::Value{{&self.0}}
}}
impl View3{{
/// Apply approved @query paths after recursive public projection, never as an exposure grant.
pub fn select_default(self)->DataResult<Selected>{{
let public=self.project()?;
Ok(Selected(stackpress_framework_sql::projection::select(&public,&{selected:?},{fallback})?))
}}
}}
impl {name}s {{
/// Build the exact fixed detail query for both pure preflight and executing public search.
fn detail_query(key:&Key)->Query{{
let mut query=Query{{take:1,includes:vec![{includes}],..Default::default()}};
"#,
        fallback = model.query.is_empty(),
        includes = includes
            .iter()
            .map(|path| format!("{path:?}.into()"))
            .collect::<Vec<_>>()
            .join(",")
    ));
    for field in model.fields.iter().filter(|field| field.key) {
        text.push_str(&format!(
            "query.filters.push(Filter::{}(stackpress_framework_schema::Predicate::Eq({})));\n",
            variant(field),
            copy_key(field, &format!("key.{}", field.rust_name))
        ));
    }
    text.push_str("query\n}\n/// Codegen support: check the exact public detail query and executor budgets before BEGIN.\n#[doc(hidden)]\npub fn preflight_detail(engine:&stackpress_sql::Engine,key:&Key)->DataResult<()> {\nlet query=Self::detail_query(key);\nstackpress_framework_sql::stores::preflight_public_search(&super::schema::catalog(),&SPEC,&ReadQuery::from_typed(&query)?,engine.dialect(),engine.limits())\n}\n/// Load the fixed app-default includes/projection using a complete identity.\n/// Explicit relation defaults require the caller's transaction, like ordinary eager search.\npub async fn detail(db:&mut DataSession<'_>,key:&Key)->DataResult<Option<Selected>> {\nlet query=Self::detail_query(key);\nlet page=stackpress_framework_sql::stores::search(db,&super::schema::catalog(),&SPEC,ReadQuery::from_typed(&query)?,Exposure::Public,Visibility::Active,RelationLimits::default()).await?;\nlet row=page.rows.into_iter().next().map(|row|serde_json::from_value::<View3>(row).map_err(stackpress_framework_sql::codec_error)).transpose()?;\nrow.map(|row|row.select_default()).transpose()\n}\n}\n");
    Ok(())
}
