//! Emit finite per-model query selectors and values; execution remains a separate milestone.

use super::{projection, rust_type, transform_error, type_name};
use crate::{Model, Schema, ValueType};
use stackpress_idea_transformer::Result;

/// Emit an exact typed filter enum plus the explicitly projected ordering/paging envelope.
pub(super) fn emit(schema: &Schema, model: &Model) -> Result<String> {
    let mut text = format!(
        "/// Approved field predicates; variants cannot name private paths.\n#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]\n#[serde(tag=\"field\",content=\"predicate\",deny_unknown_fields)]\npub enum {}Filter {{\n",
        model.rust_name
    );
    let filters: Vec<_> = model
        .fields
        .iter()
        .filter(|field| projection::filterable(field))
        .collect();
    for field in &filters {
        let mut scalar = (*field).clone();
        scalar.required = true;
        scalar.multiple = false;
        let ty = rust_type(schema, &scalar).map_err(transform_error)?;
        let variant = type_name(&field.name).map_err(transform_error)?;
        text.push_str(&format!("/// Approved typed predicate.\n#[serde(rename={:?})]\n{variant}(stackpress_framework_schema::Predicate<{ty}>),\n",field.name));
    }
    text.push_str("}\n");
    text.push_str(&format!("impl {}Filter {{\n/// Check operator applicability and value bounds before future IO.\npub fn validate(&self)->stackpress_framework_schema::SchemaResult<()> {{",model.rust_name));
    // Empty selector enums are uninhabited; match the value directly without a redundant outer arm.
    text.push_str(if filters.is_empty() {
        "match *self {\n"
    } else {
        "match self {\n"
    });
    for field in &filters {
        let variant = type_name(&field.name).map_err(transform_error)?;
        let mut kind = field.value_type.clone();
        if let ValueType::Enum(name) = &kind {
            kind = schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .and_then(|item| item.variants.first())
                .map(|(_, value)| {
                    if value.is_string() {
                        ValueType::String
                    } else if value.is_boolean() {
                        ValueType::Boolean
                    } else {
                        ValueType::Integer
                    }
                })
                .expect("checked enum");
        }
        text.push_str(&format!("Self::{variant}(predicate)=>predicate.validate(&serde_json::from_str::<stackpress_framework_schema::ValueType>({:?}).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,\"query type decoding failed\").with_cause(cause))?,{},{}),\n",serde_json::to_string(&kind).expect("scalar kind serialization"),field.multiple,!field.required));
    }
    text.push_str("} } }\n");
    text.push_str(&format!("/// Approved ordering field, independent of presentation roles.\n#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]\npub enum {}Sort {{\n",model.rust_name));
    for field in model
        .fields
        .iter()
        .filter(|field| field.access.readable && !field.access.sensitive && field.sortable)
    {
        text.push_str(&format!(
            "/// Approved scalar ordering.\n#[serde(rename={:?})]\n{},\n",
            field.name,
            type_name(&field.name).map_err(transform_error)?
        ));
    }
    text.push_str("}\n");
    text.push_str(&format!("/// Public typed query; it performs no IO and registers no operation.\n#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]\n#[serde(default,deny_unknown_fields)]\npub struct {}Query {{\n/// Literal text search on approved searchable fields.\npub q:Option<String>,\n/// Finite conjunctive typed predicates.\npub filters:Vec<{}Filter>,\n/// Explicit allowlisted directions and null placement.\npub sort:Vec<stackpress_framework_schema::Sort<{}Sort>>,\n/// At most sixteen public relation paths, depth three.\npub includes:Vec<String>,\n/// Offset, at most 100,000.\npub skip:u64,\n/// Page size, 1 through 100 (default 20).\npub take:u16,\n/// Request a matched-base-row total in future execution.\npub total:bool,\n}}\nimpl Default for {}Query {{fn default()->Self {{Self{{q:None,filters:Vec::new(),sort:Vec::new(),includes:Vec::new(),skip:0,take:20,total:false}}}}}}\nimpl {}Query {{\n/// Validate the complete allowlist/budget contract before any future query compilation.\npub fn validate(&self)->stackpress_framework_schema::SchemaResult<()> {{stackpress_framework_schema::query_budget(self.q.as_deref(),self.filters.len(),self.sort.len(),&self.includes,self.skip,self.take)?;\n",model.rust_name,model.rust_name,model.rust_name,model.rust_name,model.rust_name));
    let searchable = model.fields.iter().any(|field| {
        field.access.readable
            && !field.access.sensitive
            && field.searchable
            && field.value_type == ValueType::String
            && !field.multiple
    });
    if !searchable {
        text.push_str("if self.q.as_ref().is_some_and(|value|!value.is_empty()){return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,\"public text search is unavailable\"));}\n");
    }
    let ui = serde_json::to_string(&schema.project_ui().map_err(transform_error)?)
        .expect("checked UI schema serialization");
    text.push_str(&format!("for filter in &self.filters {{filter.validate()?;}}\nif !self.includes.is_empty() {{let schema:stackpress_framework_schema::UiSchema=serde_json::from_str({ui:?}).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,\"query projection decoding failed\").with_cause(cause))?;for path in &self.includes {{stackpress_framework_schema::check_include(&schema,{:?},path)?;}}}}Ok(())}}}}\n",model.name));
    Ok(text)
}
