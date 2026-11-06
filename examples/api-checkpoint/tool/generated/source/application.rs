//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{
    Model, SchemaResult,
    source::{Column, Kind, Record, SourceColumn, SourceSchema as SourceSchemaContract, Value},
};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str("{\"name\":\"Application\",\"rust_name\":\"Application\",\"module\":\"application\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"ID\",\"description\":\"Unique generated identifier.\",\"examples\":[\"tg8bcf7e2lig3iuej3dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"ID\"]},{\"name\":\"id\",\"value\":true},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Unique generated identifier.\"]},{\"name\":\"examples\",\"value\":[\"tg8bcf7e2lig3iuej3dz7pjf\"]}]},{\"name\":\"profileId\",\"rust_name\":\"profile_id\",\"column\":\"profile_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Profile\",\"description\":\"The profile author of this application.\",\"examples\":[\"pjfdz7tg8bcf7e2lig3iuej3\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"filter\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Profile\"]},{\"name\":\"field.relation\",\"value\":[{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"}]},{\"name\":\"is.required\",\"value\":[\"Profile is required\"]},{\"name\":\"filter.relation\",\"value\":[{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"}]},{\"name\":\"list.template\",\"value\":[{\"template\":\"{{profile.name}}\"}]},{\"name\":\"view.template\",\"value\":[{\"template\":\"{{profile.name}}\"}]},{\"name\":\"description\",\"value\":[\"The profile author of this application.\"]},{\"name\":\"examples\",\"value\":[\"pjfdz7tg8bcf7e2lig3iuej3\"]}]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Name is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Name\",\"description\":\"Arbitrary name of the application. Used in 3-legged OAuth flows.\",\"examples\":[\"My Awesome App\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Name\"]},{\"name\":\"searchable\",\"value\":true},{\"name\":\"field.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"is.required\",\"value\":[\"Name is required\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Arbitrary name of the application. Used in 3-legged OAuth flows.\"]},{\"name\":\"examples\",\"value\":[\"My Awesome App\"]}]},{\"name\":\"logo\",\"rust_name\":\"logo\",\"column\":\"logo\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Image\",\"description\":\"Logo image URL of the application. Used in 3-legged OAuth flows.\",\"examples\":[\"https://example.com/logo.png\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Image\"]},{\"name\":\"field.url\",\"value\":true},{\"name\":\"list.image\",\"value\":[20,20]},{\"name\":\"view.image\",\"value\":[100,100]},{\"name\":\"description\",\"value\":[\"Logo image URL of the application. Used in 3-legged OAuth flows.\"]},{\"name\":\"examples\",\"value\":[\"https://example.com/logo.png\"]}]},{\"name\":\"website\",\"rust_name\":\"website\",\"column\":\"website\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Website\",\"description\":\"Arbitrary website URL of the application. Used in 3-legged OAuth flows.\",\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Website\"]},{\"name\":\"searchable\",\"value\":true},{\"name\":\"field.url\",\"value\":true},{\"name\":\"list.link\",\"value\":[{\"target\":\"blank\"}]},{\"name\":\"view.link\",\"value\":[{\"target\":\"blank\"}]},{\"name\":\"description\",\"value\":[\"Arbitrary website URL of the application. Used in 3-legged OAuth flows.\"]}]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"Secret\",\"description\":\"Generated unique hash needed to perform write operations (POST, PUT, DELETE, etc.)\",\"examples\":[\"bcf7e2lig3iuej3tg8dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Secret\"]},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Generated unique hash needed to perform write operations (POST, PUT, DELETE, etc.)\"]},{\"name\":\"examples\",\"value\":[\"bcf7e2lig3iuej3tg8dz7pjf\"]}]},{\"name\":\"scopes\",\"rust_name\":\"scopes\",\"column\":\"scopes\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Scopes\",\"description\":\"API scopes that this application can request. This is mapped with endpoints.\",\"examples\":[[\"commerce\",\"location\",\"account\"]],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.tags\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Scopes\"]},{\"name\":\"default\",\"value\":[[]]},{\"name\":\"field.taglist\",\"value\":true},{\"name\":\"list.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"view.taglist\",\"value\":[{\"warning\":true,\"pill\":true,\"className\":\"frui-mr-md\"}]},{\"name\":\"description\",\"value\":[\"API scopes that this application can request. This is mapped with endpoints.\"]},{\"name\":\"examples\",\"value\":[[\"commerce\",\"location\",\"account\"]]}]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Active\",\"description\":\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"filter\",\"component\":\"stackpress.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Active\"]},{\"name\":\"default\",\"value\":[true]},{\"name\":\"active\",\"value\":true},{\"name\":\"filter.switch\",\"value\":true},{\"name\":\"view.yesno\",\"value\":true},{\"name\":\"description\",\"value\":[\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\"]},{\"name\":\"examples\",\"value\":[true]}]},{\"name\":\"expires\",\"rust_name\":\"expires\",\"column\":\"expires\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Expires\",\"description\":\"Expiration date of the application. After this date, the application will not be able to perform any operations.\",\"examples\":[\"2025-12-31T23:59:59Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Expires\"]},{\"name\":\"sortable\",\"value\":true},{\"name\":\"field.datetime\",\"value\":true},{\"name\":\"is.required\",\"value\":[\"Expires is required\"]},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Expiration date of the application. After this date, the application will not be able to perform any operations.\"]},{\"name\":\"examples\",\"value\":[\"2025-12-31T23:59:59Z\"]}]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Created\",\"description\":\"Generated timestamp when row was created.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Created\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"sortable\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp when row was created.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Updated\",\"description\":\"Generated timestamp that is updated whenever the row has changed.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Updated\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"timestamp\",\"value\":true},{\"name\":\"sortable\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp that is updated whenever the row has changed.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"session\",\"rust_name\":\"session\",\"column\":\"session\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Session\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Sessions\",\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Sessions\"]}]},{\"name\":\"profile\",\"rust_name\":\"profile\",\"column\":\"profile\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"relation\",\"value\":[{\"local\":\"profileId\",\"foreign\":\"id\"}]}]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":\"{{name}}\",\"icon\":\"laptop\",\"labels\":[\"App\",\"Apps\"]},\"query\":[\"*\",\"profile.*\"],\"attributes\":[{\"name\":\"labels\",\"value\":[\"App\",\"Apps\"]},{\"name\":\"display\",\"value\":[\"{{name}}\"]},{\"name\":\"icon\",\"value\":[\"laptop\"]},{\"name\":\"query\",\"value\":[\"*\",\"profile.*\"]}]}").expect("generator emitted valid source metadata")
}

/// Generate storage columns in declaration order; relation fields remain metadata only.
pub fn columns() -> Vec<SourceColumn> {
    let model = describe();
    let catalog = std::sync::Arc::new(super::catalog::describe());
    let codecs = vec![
        (
            "id",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "profileId",
            Column {
                kind: Kind::String,
                nullable: false,
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
            "logo",
            Column {
                kind: Kind::String,
                nullable: true,
            },
        ),
        (
            "website",
            Column {
                kind: Kind::String,
                nullable: true,
            },
        ),
        (
            "secret",
            Column {
                kind: Kind::String,
                nullable: false,
            },
        ),
        (
            "scopes",
            Column {
                kind: Kind::Strings,
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
        (
            "expires",
            Column {
                kind: Kind::Date,
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
pub struct ApplicationSchema(stackpress_framework_schema::source::SourceRecordSchema);

impl ApplicationSchema {
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

impl SourceSchemaContract for ApplicationSchema {
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
        super::catalog::describe()
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
    for ApplicationSchema
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
    for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl
    stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::ProfileId>
    for ApplicationSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("profileId")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "profileId".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::ProfileId,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Name>
    for ApplicationSchema
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
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Logo>
    for ApplicationSchema
{
    type FieldValue = Option<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("logo")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "logo".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Logo,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::Nullable<
        stackpress_framework_schema::source::StringType,
    >;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Website>
    for ApplicationSchema
{
    type FieldValue = Option<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("website")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "website".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Website,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::Nullable<
        stackpress_framework_schema::source::StringType,
    >;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Secret>
    for ApplicationSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("secret")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "secret".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Secret,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Scopes>
    for ApplicationSchema
{
    type FieldValue = Vec<String>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("scopes")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "scopes".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Scopes,
    > for ApplicationSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::StringType>;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Active>
    for ApplicationSchema
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
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::BooleanType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Expires>
    for ApplicationSchema
{
    type FieldValue = stackpress_framework_schema::source::Date;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("expires")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "expires".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Expires,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Created>
    for ApplicationSchema
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
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Updated>
    for ApplicationSchema
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
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Session>
    for ApplicationSchema
{
    type FieldValue = Vec<stackpress_framework_schema::source::Record>;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("session")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "session".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Session,
    > for ApplicationSchema
{
    type Declaration =
        stackpress_framework_schema::source::List<stackpress_framework_schema::source::SessionType>;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Profile>
    for ApplicationSchema
{
    type FieldValue = stackpress_framework_schema::source::Record;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("profile")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "profile".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Profile,
    > for ApplicationSchema
{
    type Declaration = stackpress_framework_schema::source::ProfileType;
}
