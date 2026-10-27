//! Native immutable catalogs and finite include-view types, separate from frozen pure DTOs.
use super::*;

/// Resolve the exact owning/inverse scalar mapping already checked by schema normalization.
fn mapping<'a>(
    schema: &'a Schema,
    model: &'a Model,
    field: &'a Field,
) -> stackpress_idea_transformer::Result<(&'a Model, &'a Field, &'a Field)> {
    let ValueType::Model(name) = &field.value_type else {
        return Err(TransformError::new("relation field expected"));
    };
    let target = schema
        .models()
        .iter()
        .find(|model| model.name == *name)
        .ok_or_else(|| TransformError::new("relation target missing"))?;
    let (local, foreign) = if let Some(relation) = &field.relation {
        (relation.local.as_str(), relation.foreign.as_str())
    } else {
        let relation = target
            .fields
            .iter()
            .filter(|other| other.value_type == ValueType::Model(model.name.clone()))
            .filter_map(|other| other.relation.as_ref())
            .find(|relation| relation.inverse.as_deref() == Some(&field.name))
            .ok_or_else(|| TransformError::new("checked inverse relation missing"))?;
        (relation.foreign.as_str(), relation.local.as_str())
    };
    let local = model
        .fields
        .iter()
        .find(|field| field.name == local)
        .ok_or_else(|| TransformError::new("relation local scalar missing"))?;
    let foreign = target
        .fields
        .iter()
        .find(|field| field.name == foreign)
        .ok_or_else(|| TransformError::new("relation foreign scalar missing"))?;
    Ok((target, local, foreign))
}
/// Emit exact field access and normalized relations, without a mutable engine/identity cache.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    public: &stackpress_framework_schema::UiSchema,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let name = &model.rust_name;
    let public_model = public.models.iter().find(|item| item.id == model.name);
    text.push_str("/// Exact immutable runtime metadata; it grants no transport authorization.\npub static SPEC: StoreSpec=StoreSpec {\n");
    text.push_str(&format!(
        "table:{:?},columns:COLUMNS,key:&{:?},active:{:?},fields:&[\n",
        model.module,
        stored
            .iter()
            .filter(|field| field.key)
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        model
            .fields
            .iter()
            .find(|field| field.active)
            .map(|field| field.name.as_str())
    ));
    for field in stored.iter().filter(|field| is_scalar(field)) {
        let choices = if let ValueType::Enum(name) = &field.value_type {
            schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .map(|item| {
                    item.variants
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let choices = serde_json::to_string(&choices)
            .map_err(|cause| TransformError::from(std::io::Error::other(cause)))?;
        let public = public_model.and_then(|model| {
            model
                .fields
                .iter()
                .find(|item| item.wire_name == field.name)
        });
        let mut scalar_field = (*field).clone();
        scalar_field.multiple = false;
        text.push_str(&format!("stackpress_framework_sql::stores::QueryField{{ storage:{},multiple:{},scalar:StorageCodec::{},choices:{:?},searchable:{},sortable:{},public_filter:{},public_sort:{},public_search:{} }},\n",storage(schema,field).trim_end().trim_end_matches(','),field.multiple,codec(schema,&scalar_field),choices,field.searchable&&field.value_type==ValueType::String,field.sortable||field.key,public.is_some_and(|field|field.filterable),public.is_some_and(|field|field.sortable),public.is_some_and(|field|field.searchable)));
    }
    text.push_str("],relations:&[\n");
    for field in model
        .fields
        .iter()
        .filter(|field| matches!(field.value_type, ValueType::Model(_)))
    {
        let (target, local, foreign) = mapping(schema, model, field)?;
        text.push_str(&format!("stackpress_framework_sql::stores::RelationSpec{{name:{:?},target:{:?},local:{:?},foreign:{:?},many:{},required:{},public:{}}},\n",field.name,target.module,local.name,foreign.name,field.multiple,field.required,public_model.is_some_and(|model|model.fields.iter().any(|item|item.wire_name==field.name&&item.readable))));
    }
    text.push_str(&format!("],decode:decode_value,project:project_value}};\n/// Validate through the actual owned model before bounded JSON enters the include arena.\nfn decode_value(row:Row)->DataResult<serde_json::Value>{{stackpress_framework_schema::to_bounded_value(&{name}Store::decode(row)?).map_err(stackpress_framework_sql::stores::validation)}}\n/// Project the actual model through the compiled app policy; no stored field is echoed implicitly.\nfn project_value(value:serde_json::Value)->DataResult<serde_json::Value>{{let row:_Stored=serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)?;stackpress_framework_schema::to_bounded_value(&row.project().map_err(stackpress_framework_sql::stores::projection_error)?).map_err(stackpress_framework_sql::stores::validation)}}\nimpl {name}Store {{\n/// Strict owned decode, including checked nested fields and stored validation.\npub fn decode(row:Row)->DataResult<_Stored>{{let value:_Stored=stackpress_framework_sql::decode(row,COLUMNS)?;value.validate().map_err(|cause|DataError::new(\"stored value validation failed\").with_kind(DataErrorKind::Decode).with_cause(cause))?;Ok(value)}}\n}}\n"));
    // Four explicitly finite depth types encode the depth-three contract without recursive models.
    for depth in 0..=3 {
        for is_public in [false, true] {
            let prefix = if is_public { "Public" } else { "" };
            let row = if is_public {
                format!("crate::generated::contracts::public::Public{name}")
            } else {
                "_Stored".into()
            };
            text.push_str(&format!("/// Finite depth-{depth} {} include result; omission means Unloaded.\n#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct {prefix}View{depth} {{\n/// Fully owned {} row.\npub row:{row},\n",if is_public{"projected"}else{"trusted"},if is_public{"approved public"}else{"stored"}));
            if depth > 0 {
                for field in model.fields.iter().filter(|field| {
                    matches!(field.value_type, ValueType::Model(_))
                        && (!is_public || field.access.readable && !field.access.sensitive)
                }) {
                    let (target, _, _) = mapping(schema, model, field)?;
                    let target =
                        format!("super::{}::{prefix}View{}", store_module(target), depth - 1);
                    let target = if field.multiple {
                        format!("Vec<{target}>")
                    } else if !field.required {
                        format!("Option<{target}>")
                    } else {
                        target
                    };
                    text.push_str(&format!("/// Explicit eager {:?} relation.\n#[serde(rename={:?},default)]\npub {}:Loaded<{}>,\n",field.name,crate::stores::relation_member(&field.name),view_field(field),target));
                }
            }
            text.push_str("}\n");
        }
        text.push_str(&format!("impl View{depth} {{\n/// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.\npub fn project(self)->DataResult<PublicView{depth}> {{Ok(PublicView{depth} {{row:self.row.project().map_err(stackpress_framework_sql::stores::projection_error)?,\n"));
        if depth > 0 {
            for field in model.fields.iter().filter(|field| {
                matches!(field.value_type, ValueType::Model(_))
                    && field.access.readable
                    && !field.access.sensitive
            }) {
                let expression = if field.multiple {
                    "values.into_iter().map(|value|value.project()).collect::<DataResult<Vec<_>>>()"
                } else if !field.required {
                    "values.map(|value|value.project()).transpose()"
                } else {
                    "values.project()"
                };
                text.push_str(&format!(
                    "{}:self.{}.try_map(|values|{})?,\n",
                    view_field(field),
                    view_field(field),
                    expression
                ));
            }
        }
        text.push_str("})}\n}\n");
    }
    Ok(())
}

/// The base record owns `row`; an underscore-prefixed Rust member cannot collide with source names.
fn view_field(field: &Field) -> &str {
    if field.rust_name == "row" {
        "_relation_row"
    } else {
        &field.rust_name
    }
}
