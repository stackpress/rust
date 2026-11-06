//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{Model, SchemaResult, source::{Column, Kind, Record, Value, SourceColumn, SourceSchema as SourceSchemaContract}};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str([%model%]).expect("generator emitted valid source metadata")
}

/// Generate storage columns in declaration order; relation fields remain metadata only.
pub fn columns() -> Vec<SourceColumn> {
    let model = describe();
    let catalog=std::sync::Arc::new(super::[%catalog_module%]::describe());
    let codecs = vec![
[%#@:columns column%]
        ([%column.name%], Column { kind: Kind::[%column.kind%], nullable: [%column.nullable%] }),
[%/@:columns%]
    ];
    // Pair by exact identity instead of dropping or shifting a virtual relation slot.
    codecs.into_iter().map(|(name, codec)| {
        let field = model.fields.iter().find(|field| field.name == name).expect("generated column has source field").clone();
        SourceColumn::new(codec, field).with_catalog(catalog.clone())
    }).collect()
}

/// Owned source schema state; construction performs no defaults, database or plugin IO.
#[derive(Clone)]
pub struct [%name%]Schema(stackpress_framework_schema::source::SourceRecordSchema);

impl [%name%]Schema {
    /// Use an explicit application provider; seed and clock offset remain immutable.
    pub fn with_providers(mut self, providers: std::sync::Arc<dyn stackpress_framework_schema::ValueProviders>) -> Self {
        self.0 = self.0.with_providers(providers); self
    }
    /// Borrow the generated storage inventory and its complete field metadata.
    pub fn columns(&self) -> &[SourceColumn] { self.0.columns() }
    /// Borrow one declared storage column by its source field name.
    pub fn column(&self, name: &str) -> Option<&SourceColumn> { self.0.column(name) }
}

impl SourceSchemaContract for [%name%]Schema {
    type Value = Value;
    type Record = Record;
    fn new(seed: String, offset_minutes: i32) -> Self { Self(stackpress_framework_schema::source::SourceRecordSchema::new(columns(), seed, offset_minutes)) }
    fn describe() -> Model { describe() }
    fn catalog() -> stackpress_framework_schema::SourceCatalog { super::[%catalog_module%]::describe() }
    fn seed(&self) -> &str { self.0.seed() }
    fn offset_minutes(&self) -> i32 { self.0.offset_minutes() }
    fn defaults_at(&self, now: i64) -> SchemaResult<Record> { self.0.defaults_at(now) }
    fn defaults(&self) -> SchemaResult<Record> { self.0.defaults() }
    fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> { self.0.filter(input, populate) }
    fn populate(&self, input: &Record) -> SchemaResult<Record> { self.0.populate(input) }
    fn assert(&self, input: &Record, required: bool) -> Option<Record> { self.0.assert(input, required) }
    fn serialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> { self.0.serialize(input, storage) }
    fn unserialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> { self.0.unserialize(input, storage) }
}

[%witnesses%]
