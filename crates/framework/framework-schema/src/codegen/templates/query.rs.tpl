/// Approved field predicates; variants cannot name private paths.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
#[serde(tag="field",content="predicate",deny_unknown_fields)]
pub enum [%name%]Filter {
[%#@:filters field%]/// Approved typed predicate.
#[serde(rename=[%field.wire%])]
[%field.variant%](stackpress_framework_schema::Predicate<[%field.ty%]>),
[%/@:filters%]}
impl [%name%]Filter {
/// Check operator applicability and value bounds before future IO.
pub fn validate(&self)->stackpress_framework_schema::SchemaResult<()> {
    match [%#!:has_filters%]*[%/!:has_filters%]self {
[%#@:filters field%]        Self::[%field.variant%](predicate)=>predicate.validate(
            &serde_json::from_str::<stackpress_framework_schema::ValueType>([%field.kind%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query type decoding failed").with_cause(cause))?,
            [%field.multiple%],[%field.nullable%]),
[%/@:filters%]    }
}
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
pub enum [%name%]Sort {
[%#@:sorts field%]/// Approved scalar ordering.
#[serde(rename=[%field.wire%])]
[%field.variant%],
[%/@:sorts%]}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct [%name%]Query {
/// Literal text search on approved searchable fields.
pub q:Option<String>,
/// Finite conjunctive typed predicates.
pub filters:Vec<[%name%]Filter>,
/// Explicit allowlisted directions and null placement.
pub sort:Vec<stackpress_framework_schema::Sort<[%name%]Sort>>,
/// At most sixteen public relation paths, depth three.
pub includes:Vec<String>,
/// Offset, at most 100,000.
pub skip:u64,
/// Page size, 1 through 100 (default 20).
pub take:u16,
/// Request a matched-base-row total in future execution.
pub total:bool,
}
impl Default for [%name%]Query {
    fn default()->Self {
        Self{q:None,filters:Vec::new(),sort:Vec::new(),includes:Vec::new(),skip:0,take:20,total:false}
    }
}
impl [%name%]Query {
/// Validate the complete allowlist/budget contract before any future query compilation.
pub fn validate(&self)->stackpress_framework_schema::SchemaResult<()> {
    stackpress_framework_schema::query_budget(self.q.as_deref(),self.filters.len(),self.sort.len(),&self.includes,self.skip,self.take)?;
[%#!:searchable%]    if self.q.as_ref().is_some_and(|value|!value.is_empty()) {
        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,"public text search is unavailable"));
    }
[%/!:searchable%]    for filter in &self.filters {filter.validate()?;}
    if !self.includes.is_empty() {
        let schema:stackpress_framework_schema::UiSchema=serde_json::from_str([%ui%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
        for path in &self.includes {stackpress_framework_schema::check_include(&schema,[%model%],path)?;}
    }
    Ok(())
}
}
