//! Generated source schema; regenerate from the composed Idea graph, never edit.
use stackpress_framework_schema::{
    Model, SchemaResult,
    source::{Column, Kind, Record, SourceColumn, SourceSchema as SourceSchemaContract, Value},
};

/// Complete source metadata, including relations, UI props and private attributes.
pub fn describe() -> Model {
    stackpress_framework_schema::source::__private::serde_json::from_str("{\"name\":\"Auth\",\"rust_name\":\"Auth\",\"module\":\"auth\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"ID\",\"description\":\"Unique generated identifier.\",\"examples\":[\"dz7tg8bcf7e2lig3iuej3pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.overflow\",\"props\":{\"hellip\":true,\"length\":10},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"ID\"]},{\"name\":\"id\",\"value\":true},{\"name\":\"default\",\"value\":[\"cuid()\"]},{\"name\":\"list.clip\",\"value\":[{\"length\":10,\"hellip\":true}]},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Unique generated identifier.\"]},{\"name\":\"examples\",\"value\":[\"dz7tg8bcf7e2lig3iuej3pjf\"]}]},{\"name\":\"profileId\",\"rust_name\":\"profile_id\",\"column\":\"profile_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Profile\",\"description\":\"Profile this auth belongs to.\",\"examples\":[\"pjfdz7tg8bcf7e2lig3iuej3\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"filter\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Profile\"]},{\"name\":\"field.relation\",\"value\":[{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"}]},{\"name\":\"is.required\",\"value\":[\"Profile is required\"]},{\"name\":\"filter.relation\",\"value\":[{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"}]},{\"name\":\"list.template\",\"value\":[{\"template\":\"{{profile.name}}\"}]},{\"name\":\"view.template\",\"value\":[{\"template\":\"{{profile.name}}\"}]},{\"name\":\"description\",\"value\":[\"Profile this auth belongs to.\"]},{\"name\":\"examples\",\"value\":[\"pjfdz7tg8bcf7e2lig3iuej3\"]}]},{\"name\":\"type\",\"rust_name\":\"type_\",\"column\":\"type\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"username\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Type is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Type\",\"description\":\"Type of authentication method (username, email, phone).\",\"examples\":[\"username\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"filter\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Type\"]},{\"name\":\"default\",\"value\":[\"username\"]},{\"name\":\"field.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"is.required\",\"value\":[\"Type is required\"]},{\"name\":\"filter.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"Type of authentication method (username, email, phone).\"]},{\"name\":\"examples\",\"value\":[\"username\"]}]},{\"name\":\"token\",\"rust_name\":\"token\",\"column\":\"token\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Token is required\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[5],\"client_checkable\":true,\"message\":\"Must be at least 5 characters\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Token\",\"description\":\"The actual username, email, or phone (depending on type)\",\"examples\":[\"john@doe.com\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Token\"]},{\"name\":\"unique\",\"value\":true},{\"name\":\"encrypted\",\"value\":true},{\"name\":\"field.input\",\"value\":[{\"type\":\"text\"}]},{\"name\":\"is.required\",\"value\":[\"Token is required\"]},{\"name\":\"is.cge\",\"value\":[5,\"Must be at least 5 characters\"]},{\"name\":\"list.string\",\"value\":true},{\"name\":\"view.string\",\"value\":true},{\"name\":\"description\",\"value\":[\"The actual username, email, or phone (depending on type)\"]},{\"name\":\"examples\",\"value\":[\"john@doe.com\"]}]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Secret is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Secret\",\"description\":\"Password used to verify.\",\"examples\":[\"s3cr3t P@ssw0rd\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.password\",\"props\":{\"type\":\"password\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":true,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Secret\"]},{\"name\":\"hashed\",\"value\":true},{\"name\":\"secret\",\"value\":true},{\"name\":\"field.password\",\"value\":true},{\"name\":\"is.required\",\"value\":[\"Secret is required\"]},{\"name\":\"description\",\"value\":[\"Password used to verify.\"]},{\"name\":\"examples\",\"value\":[\"s3cr3t P@ssw0rd\"]}]},{\"name\":\"verified\",\"rust_name\":\"verified\",\"column\":\"verified\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Verified is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Verified\",\"description\":\"Indicates if the type has been verified (email, phone, etc.).\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"filter\",\"component\":\"stackpress.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"list\",\"component\":\"stackpress.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Verified\"]},{\"name\":\"default\",\"value\":[false]},{\"name\":\"field.switch\",\"value\":true},{\"name\":\"is.required\",\"value\":[\"Verified is required\"]},{\"name\":\"filter.switch\",\"value\":true},{\"name\":\"list.yesno\",\"value\":true},{\"name\":\"view.yesno\",\"value\":true},{\"name\":\"description\",\"value\":[\"Indicates if the type has been verified (email, phone, etc.).\"]},{\"name\":\"examples\",\"value\":[true]}]},{\"name\":\"consumed\",\"rust_name\":\"consumed\",\"column\":\"consumed\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Last Used\",\"description\":\"Timestamp when this auth was last used.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Last Used\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"sortable\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Timestamp when this auth was last used.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Active\",\"description\":\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"filter\",\"component\":\"stackpress.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Active\"]},{\"name\":\"default\",\"value\":[true]},{\"name\":\"active\",\"value\":true},{\"name\":\"filter.switch\",\"value\":true},{\"name\":\"view.yesno\",\"value\":true},{\"name\":\"description\",\"value\":[\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\"]},{\"name\":\"examples\",\"value\":[true]}]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Created\",\"description\":\"Generated timestamp when row was created.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Created\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"sortable\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp when row was created.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Updated\",\"description\":\"Generated timestamp that is updated whenever the row has changed.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":false}],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"label\",\"value\":[\"Updated\"]},{\"name\":\"default\",\"value\":[\"now()\"]},{\"name\":\"timestamp\",\"value\":true},{\"name\":\"sortable\",\"value\":true},{\"name\":\"list.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"view.date\",\"value\":[\"m d, Y h:iA\"]},{\"name\":\"description\",\"value\":[\"Generated timestamp that is updated whenever the row has changed.\"]},{\"name\":\"examples\",\"value\":[\"2025-10-01T12:00:00Z\"]}]},{\"name\":\"profile\",\"rust_name\":\"profile\",\"column\":\"profile\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[{\"name\":\"relation\",\"value\":[{\"local\":\"profileId\",\"foreign\":\"id\"}]}]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":\"{{token}}\",\"icon\":\"lock\",\"labels\":[\"Auth\",\"Auth\"]},\"query\":[\"*\",\"profile.*\"],\"attributes\":[{\"name\":\"labels\",\"value\":[\"Auth\",\"Auth\"]},{\"name\":\"display\",\"value\":[\"{{token}}\"]},{\"name\":\"icon\",\"value\":[\"lock\"]},{\"name\":\"query\",\"value\":[\"*\",\"profile.*\"]}]}").expect("generator emitted valid source metadata")
}

/// Generate storage columns in declaration order; relation fields remain metadata only.
pub fn columns() -> Vec<SourceColumn> {
    let model = describe();
    let catalog = std::sync::Arc::new(super::_catalog::describe());
    let codecs = vec![
        Column {
            name: "id",
            sql: "id",
            kind: Kind::String,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "profileId",
            sql: "profile_id",
            kind: Kind::String,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "type",
            sql: "type",
            kind: Kind::String,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "token",
            sql: "token",
            kind: Kind::Token,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "secret",
            sql: "secret",
            kind: Kind::Secret,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "verified",
            sql: "verified",
            kind: Kind::Boolean,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "consumed",
            sql: "consumed",
            kind: Kind::Date,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "active",
            sql: "active",
            kind: Kind::Boolean,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "created",
            sql: "created",
            kind: Kind::Date,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
        Column {
            name: "updated",
            sql: "updated",
            kind: Kind::Date,
            nullable: false,
            required_message: None,
            minimum: None,
            default: None,
        },
    ];
    // Pair by exact identity instead of dropping or shifting a virtual relation slot.
    codecs
        .into_iter()
        .map(|codec| {
            let field = model
                .fields
                .iter()
                .find(|field| field.name == codec.name)
                .expect("generated column has source field")
                .clone();
            SourceColumn::new(codec, field).with_catalog(catalog.clone())
        })
        .collect()
}

/// Owned source schema state; construction performs no defaults, database or plugin IO.
#[derive(Clone)]
pub struct AuthSchema(stackpress_framework_schema::source::SourceRecordSchema);

impl AuthSchema {
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
}

impl SourceSchemaContract for AuthSchema {
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
    for AuthSchema
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
    for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl
    stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::ProfileId>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Type>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Token>
    for AuthSchema
{
    type FieldValue = String;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("token")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "token".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Token,
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Secret>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::StringType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Verified>
    for AuthSchema
{
    type FieldValue = bool;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("verified")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "verified".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Verified,
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::BooleanType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Consumed>
    for AuthSchema
{
    type FieldValue = stackpress_framework_schema::source::Date;
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(
            record.get("consumed")?,
        )
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert(
            "consumed".into(),
            stackpress_framework_schema::source::NativeField::value(value),
        );
    }
}
impl
    stackpress_framework_schema::source::SourceDeclaration<
        stackpress_framework_schema::source::Consumed,
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Active>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::BooleanType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Created>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Updated>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::DatetimeType;
}
impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::Profile>
    for AuthSchema
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
    > for AuthSchema
{
    type Declaration = stackpress_framework_schema::source::ProfileType;
}
