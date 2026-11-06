//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{
    Model, SchemaResult,
    source::{Column, Kind, Record, SourceColumn, SourceSchema as SourceSchemaContract, Value},
};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str("{\"name\":\"Lists\",\"rust_name\":\"Lists\",\"module\":\"lists\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"id\",\"value\":true}]},{\"name\":\"whole\",\"rust_name\":\"whole\",\"column\":\"whole\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"fractional\",\"rust_name\":\"fractional\",\"column\":\"fractional\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"step\",\"value\":[0.01]}]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"floats\",\"rust_name\":\"floats\",\"column\":\"floats\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"integers\",\"rust_name\":\"integers\",\"column\":\"integers\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flags\",\"rust_name\":\"flags\",\"column\":\"flags\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"moods\",\"rust_name\":\"moods\",\"column\":\"moods\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Mood\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"addresses\",\"rust_name\":\"addresses\",\"column\":\"addresses\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}").expect("generator emitted valid source metadata")
}

/// Generate storage columns in declaration order; relation fields remain metadata only.
pub fn columns() -> Vec<SourceColumn> {
    let model = describe();
    let catalog = std::sync::Arc::new(super::_catalog::describe());
    let codecs = vec![
        (
            "id",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "whole",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "fractional",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "dates",
            Column {
                kind: Kind::Date,
                nullable: false,
            },
        ),
        (
            "numbers",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "floats",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "integers",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "flags",
            Column {
                kind: Kind::Boolean,
                nullable: false,
            },
        ),
        (
            "objects",
            Column {
                kind: Kind::Object,
                nullable: false,
            },
        ),
        (
            "moods",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "addresses",
            Column {
                kind: Kind::Object,
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
pub struct ListsSchema(stackpress_framework_schema::source::SourceRecordSchema);

impl ListsSchema {
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

impl SourceSchemaContract for ListsSchema {
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

impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Id>
    for ListsSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("id")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "id".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl stackpress_framework_schema::source::SourceDeclaration<stackpress_framework_schema::source::Id>
    for ListsSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
