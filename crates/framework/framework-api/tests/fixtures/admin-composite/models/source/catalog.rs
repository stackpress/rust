//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{
    Model, SchemaResult,
    source::{Column, Kind, Record, SourceColumn, SourceSchema as SourceSchemaContract, Value},
};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str("{\"name\":\"Catalog\",\"rust_name\":\"Catalog\",\"module\":\"catalog\",\"fields\":[{\"name\":\"categoryId\",\"rust_name\":\"category_id\",\"column\":\"category_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"id\",\"value\":true},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true}]},{\"name\":\"articleId\",\"rust_name\":\"article_id\",\"column\":\"article_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"id\",\"value\":true},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true}]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"field.input\",\"value\":true},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true}]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"default\",\"value\":[true]},{\"name\":\"active\",\"value\":true}]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}").expect("generator emitted valid source metadata")
}

/// Generate storage columns in declaration order; relation fields remain metadata only.
pub fn columns() -> Vec<SourceColumn> {
    let model = describe();
    let catalog = std::sync::Arc::new(super::_catalog::describe());
    let codecs = vec![
        (
            "categoryId",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "articleId",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "title",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "active",
            Column {
                kind: Kind::Boolean,
                nullable: false,
            },
        ),
    ];
    // Pair by exact identity instead of dropping or shifting a virtual relation slot.
    codecs
        .into_iter()
        .map(|(name, codec)| {
            let field = model
                .fields
                .iter()
                .find(|field| field.name == name)
                .expect("generated column has source field")
                .clone();
            SourceColumn::new(codec, field).with_catalog(catalog.clone())
        })
        .collect()
}

/// Owned source schema state; construction performs no defaults, database or plugin IO.
#[derive(Clone)]
pub struct CatalogSchema(stackpress_framework_schema::source::SourceRecordSchema);

impl CatalogSchema {
    /// Use an explicit application provider; seed and clock offset remain immutable.
    pub fn with_providers(
        mut self,
        providers: std::sync::Arc<dyn stackpress_framework_schema::ValueProviders>,
    ) -> Self {
        self.0 = self.0.with_providers(providers);
        self
    }
    /// Borrow the generated storage inventory and its complete field metadata.
    pub fn columns(&self) -> &[SourceColumn] {
        self.0.columns()
    }
    /// Borrow one declared storage column by its source field name.
    pub fn column(&self, name: &str) -> Option<&SourceColumn> {
        self.0.column(name)
    }
}

impl SourceSchemaContract for CatalogSchema {
    type Value = Value;
    type Record = Record;
    fn new(seed: String, offset_minutes: i32) -> Self {
        Self(
            stackpress_framework_schema::source::SourceRecordSchema::new(
                columns(),
                seed,
                offset_minutes,
            ),
        )
    }
    fn describe() -> Model {
        describe()
    }
    fn catalog() -> stackpress_framework_schema::SourceCatalog {
        super::_catalog::describe()
    }
    fn seed(&self) -> &str {
        self.0.seed()
    }
    fn offset_minutes(&self) -> i32 {
        self.0.offset_minutes()
    }
    fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
        self.0.defaults_at(now)
    }
    fn defaults(&self) -> SchemaResult<Record> {
        self.0.defaults()
    }
    fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> {
        self.0.filter(input, populate)
    }
    fn populate(&self, input: &Record) -> SchemaResult<Record> {
        self.0.populate(input)
    }
    fn assert(&self, input: &Record, required: bool) -> Option<Record> {
        self.0.assert(input, required)
    }
    fn serialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.0.serialize(input, storage)
    }
    fn unserialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
        self.0.unserialize(input, storage)
    }
}

impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Active>
    for CatalogSchema
{
    type FieldValue = bool;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("active")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "active".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Active,
    > for CatalogSchema
{
    type Declaration = stackpress_framework_schema::source::BooleanType;
}
