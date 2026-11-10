//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{
    Model, SchemaResult,
    source::{Column, Kind, Record, SourceColumn, SourceSchema as SourceSchemaContract, Value},
};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str("{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"ID\",\"description\":\"Unique generated identifier.\",\"examples\":[\"pjfdz7tg8bcf7e2lig3iuej3\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.overflow\",\"props\":{\"hellip\":true,\"length\":10},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"ID\"]},{\"name\":\"id\",\"value\":true},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.clip\",\"value\":[{\"length\":10,\"hellip\":true}]},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Unique generated identifier.\"]},{\"name\":\"examples\",\"value\":[\"pjfdz7tg8bcf7e2lig3iuej3\"]}]},{\"name\":\"image\",\"rust_name\":\"image\",\"column\":\"image\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Image\",\"description\":\"Profile image URL.\",\"examples\":[\"https://example.com/image.jpg\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Image\"]},{\"name\":\"field.url\",\"value\":true},{\"name\":\"list.image\",\"value\":[{\"width\":20,\"height\":20}]},{\"name\":\"view.image\",\"value\":[{\"width\":100,\"height\":100}]},{\"name\":\"description\",\"value\":[\"Profile image URL.\"]},{\"name\":\"examples\",\"value\":[\"https://example.com/image.jpg\"]}]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Name is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Name\",\"description\":\"Full name (first middle last).\",\"examples\":[\"John Doe\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Name\"]},{\"name\":\"searchable\",\"value\":true},{\"name\":\"field.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"is.required\",\"value\":[\"Name is required\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Full name (first middle last).\"]},{\"name\":\"examples\",\"value\":[\"John Doe\"]}]},{\"name\":\"type\",\"rust_name\":\"type_\",\"column\":\"type\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"person\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Type is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Type\",\"description\":\"Type of profile (ie. person, company, etc.).\",\"examples\":[\"person\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"filter\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.lowercase\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.lowercase\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Type\"]},{\"name\":\"default\",\"value\":[\"person\"]},{\"name\":\"field.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"is.required\",\"value\":[\"Type is required\"]},{\"name\":\"filter.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"list.lowercase\",\"value\":true},{\"name\":\"view.lowercase\",\"value\":true},{\"name\":\"description\",\"value\":[\"Type of profile (ie. person, company, etc.).\"]},{\"name\":\"examples\",\"value\":[\"person\"]}]},{\"name\":\"roles\",\"rust_name\":\"roles\",\"column\":\"roles\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[\"GUEST\"],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Roles\",\"description\":\"Roles held by the profile. This is used with sessions to determine access levels of the app.\",\"examples\":[[\"admin\",\"user\",\"guest\"]],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.stringlist\",\"props\":{\"add\":\"Add Role\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Roles\"]},{\"name\":\"default\",\"value\":[[\"GUEST\"]]},{\"name\":\"field.stringlist\",\"value\":[{\"add\":\"Add Role\"}]},{\"name\":\"list.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"view.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"description\",\"value\":[\"Roles held by the profile. This is used with sessions to determine access levels of the app.\"]},{\"name\":\"examples\",\"value\":[[\"admin\",\"user\",\"guest\"]]}]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Tags\",\"description\":\"Abritrary tags for general use.\",\"examples\":[[\"top buyer\",\"verified\",\"moderator\"]],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.tags\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Tags\"]},{\"name\":\"default\",\"value\":[[]]},{\"name\":\"field.taglist\",\"value\":true},{\"name\":\"list.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"view.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"description\",\"value\":[\"Abritrary tags for general use.\"]},{\"name\":\"examples\",\"value\":[[\"top buyer\",\"verified\",\"moderator\"]]}]},{\"name\":\"references\",\"rust_name\":\"references\",\"column\":\"references\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":{},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"References\",\"description\":\"Arbitrary key/value references for general use.\",\"examples\":[{\"fbid\":\"abc123\"}],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.metadata\",\"props\":{\"add\":\"Add Reference\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.metadata\",\"props\":{\"className\":\"frui-pt-md frui-pr-md frui-pb-md frui-pl-md\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"References\"]},{\"name\":\"default\",\"value\":[{}]},{\"name\":\"field.metadata\",\"value\":[{\"add\":\"Add Reference\"}]},{\"name\":\"view.metadata\",\"value\":[{\"className\":\"frui-pt-md frui-pr-md frui-pb-md frui-pl-md\"}]},{\"name\":\"description\",\"value\":[\"Arbitrary key/value references for general use.\"]},{\"name\":\"examples\",\"value\":[{\"fbid\":\"abc123\"}]}]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Active\",\"description\":\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"filter\",\"component\":\"stackpress.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Active\"]},{\"name\":\"default\",\"value\":[true]},{\"name\":\"active\",\"value\":true},{\"name\":\"filter.switch\",\"value\":true},{\"name\":\"view.yesno\",\"value\":true},{\"name\":\"description\",\"value\":[\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\"]},{\"name\":\"examples\",\"value\":[true]}]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Created\",\"description\":\"Generated timestamp when row was created.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"span\",\"component\":\"stackpress.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Created\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"sortable\",\"value\":true},{\"name\":\"span.datetime\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp when row was created.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Updated\",\"description\":\"Generated timestamp that is updated whenever the row has changed.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"span\",\"component\":\"stackpress.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Updated\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"timestamp\",\"value\":true},{\"name\":\"sortable\",\"value\":true},{\"name\":\"span.datetime\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp that is updated whenever the row has changed.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"auth\",\"rust_name\":\"auth\",\"column\":\"auth\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Auth\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Authentications\",\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Authentications\"]}]},{\"name\":\"applications\",\"rust_name\":\"applications\",\"column\":\"applications\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Application\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Applications\",\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Applications\"]}]},{\"name\":\"sessions\",\"rust_name\":\"sessions\",\"column\":\"sessions\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Session\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Sessions\",\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Sessions\"]}]},{\"name\":\"nickname\",\"rust_name\":\"nickname\",\"column\":\"nickname\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":\"application\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"unique\",\"args\":[],\"client_checkable\":false,\"message\":\"Already exists.\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"default\",\"value\":[\"application\"]},{\"name\":\"is.unique\",\"value\":true}]},{\"name\":\"mood\",\"rust_name\":\"mood\",\"column\":\"mood\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Mood\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"on\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"default\",\"value\":[\"on\"]}]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":\"{{name}}\",\"icon\":\"user\",\"labels\":[\"Profile\",\"Profiles\"]},\"query\":[],\"attributes\":[{\"name\":\"labels\",\"value\":[\"Profile\",\"Profiles\"]},{\"name\":\"display\",\"value\":[\"{{name}}\"]},{\"name\":\"icon\",\"value\":[\"user\"]}]}").expect("generator emitted valid source metadata")
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
            "image",
            Column {
                kind: Kind::String,
                nullable: true,
            },
        ),
        (
            "name",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "type",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "roles",
            Column {
                kind: Kind::Strings,
                nullable: false,
            },
        ),
        (
            "tags",
            Column {
                kind: Kind::Strings,
                nullable: false,
            },
        ),
        (
            "references",
            Column {
                kind: Kind::Object,
                nullable: true,
            },
        ),
        (
            "active",
            Column {
                kind: Kind::Boolean,
                nullable: false,
            },
        ),
        (
            "created",
            Column {
                kind: Kind::Date,
                nullable: false,
            },
        ),
        (
            "updated",
            Column {
                kind: Kind::Date,
                nullable: false,
            },
        ),
        (
            "nickname",
            Column {
                kind: Kind::String,
                nullable: true,
            },
        ),
        (
            "mood",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "address",
            Column {
                kind: Kind::Object,
                nullable: true,
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
pub struct ProfileSchema(stackpress_framework_schema::source::SourceRecordSchema);

impl ProfileSchema {
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

impl SourceSchemaContract for ProfileSchema {
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
    for ProfileSchema
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
    for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Image>
    for ProfileSchema
{
    type FieldValue = Option<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("image")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "image".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Image,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::Nullable<
        stackpress_framework_schema::source::StringType,
    >;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Name>
    for ProfileSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("name")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "name".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Name,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Type>
    for ProfileSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("type")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "type".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Type,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Roles>
    for ProfileSchema
{
    type FieldValue = Vec<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("roles")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "roles".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Roles,
    > for ProfileSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::StringType>;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Tags>
    for ProfileSchema
{
    type FieldValue = Vec<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("tags")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "tags".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Tags,
    > for ProfileSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::StringType>;
}
impl
    stackpress_framework_schema::source::SourceField<
        stackpress_framework_schema::source::References,
    > for ProfileSchema
{
    type FieldValue = Option<stackpress_framework_schema::source::Record>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("references")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "references".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::References,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::Nullable<
        stackpress_framework_schema::source::ObjectType,
    >;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Active>
    for ProfileSchema
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
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::BooleanType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Created>
    for ProfileSchema
{
    type FieldValue = stackpress_framework_schema::source::Date;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("created")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "created".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Created,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Updated>
    for ProfileSchema
{
    type FieldValue = stackpress_framework_schema::source::Date;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("updated")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "updated".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Updated,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Auth>
    for ProfileSchema
{
    type FieldValue = Vec<stackpress_framework_schema::source::Record>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("auth")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "auth".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Auth,
    > for ProfileSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::AuthType>;
}
impl
    stackpress_framework_schema::source::SourceField<
        stackpress_framework_schema::source::Applications,
    > for ProfileSchema
{
    type FieldValue = Vec<stackpress_framework_schema::source::Record>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("applications")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "applications".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Applications,
    > for ProfileSchema
{
    type Declaration = stackpress_framework_schema::source::List<
        stackpress_framework_schema::source::ApplicationType,
    >;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Sessions>
    for ProfileSchema
{
    type FieldValue = Vec<stackpress_framework_schema::source::Record>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("sessions")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "sessions".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Sessions,
    > for ProfileSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::SessionType>;
}
