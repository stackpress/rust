//! Named generated handlers; registration and transport mapping are always explicit.

/// Explicit Matrix operation family; importing this module installs nothing.
pub mod matrix {
    use crate::generated::contracts::public::{
        MatrixKey as Key, PatchMatrix as Patch, PublicCreateMatrix as CreateInput,
        PublicPatchMatrix as PatchInput,
    };
    use crate::generated::stores::matrix::Matrixs;
    use stackpress_framework_sql::operations::{
        self, Flow, Invocation, ListenerId, Operation, Policy, Registration, Request, Response,
        Result, ServerInvocationExt, StartupContext,
    };
    use stackpress_framework_sql::{DataError, DataResult, DataSession};
    /// Strict key envelope; unknown request fields are invalid.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct KeyRequest {
        /// Complete declared identity.
        pub key: Key,
    }
    /// Strict approved creation envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CreateRequest {
        /// Approved creation members only.
        pub input: CreateInput,
    }
    /// Strict keyed update envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpdateRequest {
        /// Complete immutable identity.
        pub key: Key,
        /// Nonempty approved partial replacement.
        pub patch: PatchInput,
    }
    /// Strict keyed atomic UPSERT envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpsertRequest {
        /// Explicit primary conflict identity.
        pub key: Key,
        /// Candidate row with matching identity.
        pub input: CreateInput,
        /// Independently validated conflict patch.
        pub patch: PatchInput,
    }
    /// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
    /// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
    pub enum BatchItem {
        /// Insert one prepared candidate.
        Create {
            /// Approved creation members.
            input: Box<CreateInput>,
        },
        /// Patch one complete identity.
        Update {
            /// Immutable target identity.
            key: Key,
            /// Approved partial replacement.
            patch: Box<PatchInput>,
        },
        /// One atomic native primary-key UPSERT.
        Upsert {
            /// Explicit conflict identity.
            key: Key,
            /// Candidate with the same identity.
            input: Box<CreateInput>,
            /// Approved conflict replacement.
            patch: Box<PatchInput>,
        },
    }
    /// Strict finite batch envelope; native whole-batch validation precedes writes.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BatchRequest {
        /// Ordered finite atomic work, fully preflighted before the first write.
        pub items: Vec<BatchItem>,
    }
    /// Native-only record policy used for safe host assembly and field-error filtering.
    fn policy_schema() -> DataResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"Café\",\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0.25,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-02-29\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"12:00:00.12\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-03-01T00:00:00Z\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"nested\":[null,true]},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"city\":\"Default\"},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"token\",\"rust_name\":\"token\",\"column\":\"token\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"default:Matrix.secret\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"links\",\"rust_name\":\"links\",\"column\":\"links\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Link\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::matrix::CreateMatrix> {
        operations::assemble(&input, host, policy, false, context)
    }
    /// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
    fn patch_input(
        input: PatchInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Patch> {
        let mut value: serde_json::Value =
            operations::assemble(&input, host, policy, true, context)?;
        let mut budget = stackpress_framework_schema::PreparationBudget::default();
        let object = value
            .as_object_mut()
            .ok_or_else(|| DataError::new("patch assembly requires an object"))?;
        // Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.
        if let Some(raw) = object.get_mut("address").filter(|value| !value.is_null()) {
            let input: crate::generated::contracts::matrix_address::CreateMatrixAddress =
                serde_json::from_value(raw.take())
                    .map_err(stackpress_framework_sql::codec_error)?;
            *raw = stackpress_framework_schema::to_bounded_value(
                &input
                    .prepare_bounded(context, &mut budget)
                    .map_err(stackpress_framework_sql::stores::preparation_error)?,
            )
            .map_err(stackpress_framework_sql::stores::validation)?;
        }
        // Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.
        if let Some(raw) = object.get_mut("siblings").filter(|value| !value.is_null()) {
            let inputs: Vec<crate::generated::contracts::matrix_address::CreateMatrixAddress> =
                serde_json::from_value(raw.take())
                    .map_err(stackpress_framework_sql::codec_error)?;
            let values = inputs
                .into_iter()
                .map(|input| {
                    input
                        .prepare_bounded(context, &mut budget)
                        .map_err(stackpress_framework_sql::stores::preparation_error)
                })
                .collect::<DataResult<Vec<_>>>()?;
            *raw = stackpress_framework_schema::to_bounded_value(&values)
                .map_err(stackpress_framework_sql::stores::validation)?;
        }
        // Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.
        if let Some(raw) = object.get_mut("favorite").filter(|value| !value.is_null()) {
            let input: crate::generated::contracts::matrix_address::CreateMatrixAddress =
                serde_json::from_value(raw.take())
                    .map_err(stackpress_framework_sql::codec_error)?;
            *raw = stackpress_framework_schema::to_bounded_value(
                &input
                    .prepare_bounded(context, &mut budget)
                    .map_err(stackpress_framework_sql::stores::preparation_error)?,
            )
            .map_err(stackpress_framework_sql::stores::validation)?;
        }
        serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)
    }
    /// Named authorized get handler; prior completion skips every effect.
    pub async fn get(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Matrixs::get_key(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized detail handler; prior completion skips every effect.
    pub async fn detail(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Matrixs::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Matrixs::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized search handler; prior completion skips every effect.
    pub async fn search(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::MatrixQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Matrixs::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Matrixs::search_public(&mut DataSession::transaction(&mut transaction), input).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized create handler; prior completion skips every effect.
    pub async fn create(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Matrix", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Matrixs::create_with(&mut DataSession::pooled(engine), input, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized update handler; prior completion skips every effect.
    pub async fn update(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Matrix", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Matrixs::update_key(
            &mut DataSession::pooled(engine),
            &input.key,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?
        .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized upsert handler; prior completion skips every effect.
    pub async fn upsert(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Matrix", Operation::Upsert, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpsertRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let create = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Matrixs::upsert_key(
            &mut DataSession::pooled(engine),
            &input.key,
            create,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized batch handler; prior completion skips every effect.
    pub async fn batch(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Matrix", Operation::Batch, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: BatchRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        if input.items.is_empty() || input.items.len() > 100 {
            return Err(operations::data_error(
                DataError::new("batch item limit exceeded"),
                &schema,
            ));
        }
        let mut items = Vec::new();
        for item in input.items {
            items.push(match item {
                BatchItem::Create { input } => {
                    crate::generated::stores::matrix::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::matrix::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::matrix::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Matrixs::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::matrix::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::matrix::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::matrix::BatchOutcome::Upserted(row) => {
                    row.project().map(Some)
                }
            })
            .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()
            .map_err(|cause| {
                operations::data_error(
                    stackpress_framework_sql::stores::projection_error(cause),
                    &schema,
                )
            })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized delete handler; prior completion skips every effect.
    pub async fn delete(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Matrix", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Matrixs::delete(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Select named capabilities explicitly; services must exist before this Listen callback.
    pub fn register(
        start: &mut StartupContext,
        registration: &mut Registration,
        prefix: &str,
        operations: &[Operation],
    ) -> Result<Vec<ListenerId>> {
        start.service::<stackpress_sql::Engine>()?;
        start.service::<Policy>()?;
        if operations
            .iter()
            .any(|operation| matches!(operation, Operation::Remove | Operation::Restore))
        {
            return Err(operations::Error::invalid(
                "soft removal requires a Boolean active model",
            ));
        }
        registration.claim(prefix, operations)?;
        let mut handles = Vec::new();
        for operation in operations {
            let event = format!("{prefix}.{}", operation.as_str());
            let handle = match operation {
                Operation::Get => start.on(&event, get)?,
                Operation::Detail => start.on(&event, detail)?,
                Operation::Search => start.on(&event, search)?,
                Operation::Create => start.on(&event, create)?,
                Operation::Update => start.on(&event, update)?,
                Operation::Upsert => start.on(&event, upsert)?,
                Operation::Batch => start.on(&event, batch)?,
                Operation::Delete => start.on(&event, delete)?,
                Operation::Remove | Operation::Restore => {
                    return Err(operations::Error::invalid("soft removal is unavailable"));
                }
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}

/// Explicit Link operation family; importing this module installs nothing.
pub mod link {
    use crate::generated::contracts::public::{
        LinkKey as Key, PatchLink as Patch, PublicCreateLink as CreateInput,
        PublicPatchLink as PatchInput,
    };
    use crate::generated::stores::link::Links;
    use stackpress_framework_sql::operations::{
        self, Flow, Invocation, ListenerId, Operation, Policy, Registration, Request, Response,
        Result, ServerInvocationExt, StartupContext,
    };
    use stackpress_framework_sql::{DataError, DataResult, DataSession};
    /// Strict key envelope; unknown request fields are invalid.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct KeyRequest {
        /// Complete declared identity.
        pub key: Key,
    }
    /// Strict approved creation envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CreateRequest {
        /// Approved creation members only.
        pub input: CreateInput,
    }
    /// Strict keyed update envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpdateRequest {
        /// Complete immutable identity.
        pub key: Key,
        /// Nonempty approved partial replacement.
        pub patch: PatchInput,
    }
    /// Strict keyed atomic UPSERT envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpsertRequest {
        /// Explicit primary conflict identity.
        pub key: Key,
        /// Candidate row with matching identity.
        pub input: CreateInput,
        /// Independently validated conflict patch.
        pub patch: PatchInput,
    }
    /// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
    /// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
    pub enum BatchItem {
        /// Insert one prepared candidate.
        Create {
            /// Approved creation members.
            input: Box<CreateInput>,
        },
        /// Patch one complete identity.
        Update {
            /// Immutable target identity.
            key: Key,
            /// Approved partial replacement.
            patch: Box<PatchInput>,
        },
        /// One atomic native primary-key UPSERT.
        Upsert {
            /// Explicit conflict identity.
            key: Key,
            /// Candidate with the same identity.
            input: Box<CreateInput>,
            /// Approved conflict replacement.
            patch: Box<PatchInput>,
        },
    }
    /// Strict finite batch envelope; native whole-batch validation precedes writes.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BatchRequest {
        /// Ordered finite atomic work, fully preflighted before the first write.
        pub items: Vec<BatchItem>,
    }
    /// Native-only record policy used for safe host assembly and field-error filtering.
    fn policy_schema() -> DataResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"owner\",\"rust_name\":\"owner\",\"column\":\"owner\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Matrix\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"links\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[\"id\",\"owner.text\",\"owner.favorite.city\"],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::link::CreateLink> {
        operations::assemble(&input, host, policy, false, context)
    }
    /// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
    fn patch_input(
        input: PatchInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Patch> {
        let value: serde_json::Value = operations::assemble(&input, host, policy, true, context)?;
        let _ = context;
        serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)
    }
    /// Named authorized get handler; prior completion skips every effect.
    pub async fn get(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Links::get_key(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized detail handler; prior completion skips every effect.
    pub async fn detail(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Links::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Links::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized search handler; prior completion skips every effect.
    pub async fn search(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::LinkQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Links::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Links::search_public(&mut DataSession::transaction(&mut transaction), input).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized create handler; prior completion skips every effect.
    pub async fn create(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Link", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Links::create_with(&mut DataSession::pooled(engine), input, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized update handler; prior completion skips every effect.
    pub async fn update(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Link", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Links::update_key(
            &mut DataSession::pooled(engine),
            &input.key,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?
        .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized upsert handler; prior completion skips every effect.
    pub async fn upsert(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Link", Operation::Upsert, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpsertRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let create = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Links::upsert_key(
            &mut DataSession::pooled(engine),
            &input.key,
            create,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized batch handler; prior completion skips every effect.
    pub async fn batch(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Link", Operation::Batch, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: BatchRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        if input.items.is_empty() || input.items.len() > 100 {
            return Err(operations::data_error(
                DataError::new("batch item limit exceeded"),
                &schema,
            ));
        }
        let mut items = Vec::new();
        for item in input.items {
            items.push(match item {
                BatchItem::Create { input } => crate::generated::stores::link::BatchInput::Create {
                    input: create_input(*input, &host.create, &schema, &context)
                        .map_err(|cause| operations::data_error(cause, &schema))?,
                },
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::link::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::link::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Links::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::link::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::link::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::link::BatchOutcome::Upserted(row) => {
                    row.project().map(Some)
                }
            })
            .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()
            .map_err(|cause| {
                operations::data_error(
                    stackpress_framework_sql::stores::projection_error(cause),
                    &schema,
                )
            })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized delete handler; prior completion skips every effect.
    pub async fn delete(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Link", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Links::delete(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Select named capabilities explicitly; services must exist before this Listen callback.
    pub fn register(
        start: &mut StartupContext,
        registration: &mut Registration,
        prefix: &str,
        operations: &[Operation],
    ) -> Result<Vec<ListenerId>> {
        start.service::<stackpress_sql::Engine>()?;
        start.service::<Policy>()?;
        if operations
            .iter()
            .any(|operation| matches!(operation, Operation::Remove | Operation::Restore))
        {
            return Err(operations::Error::invalid(
                "soft removal requires a Boolean active model",
            ));
        }
        registration.claim(prefix, operations)?;
        let mut handles = Vec::new();
        for operation in operations {
            let event = format!("{prefix}.{}", operation.as_str());
            let handle = match operation {
                Operation::Get => start.on(&event, get)?,
                Operation::Detail => start.on(&event, detail)?,
                Operation::Search => start.on(&event, search)?,
                Operation::Create => start.on(&event, create)?,
                Operation::Update => start.on(&event, update)?,
                Operation::Upsert => start.on(&event, upsert)?,
                Operation::Batch => start.on(&event, batch)?,
                Operation::Delete => start.on(&event, delete)?,
                Operation::Remove | Operation::Restore => {
                    return Err(operations::Error::invalid("soft removal is unavailable"));
                }
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}

/// Explicit Compound operation family; importing this module installs nothing.
pub mod compound {
    use crate::generated::contracts::public::{
        CompoundKey as Key, PatchCompound as Patch, PublicCreateCompound as CreateInput,
        PublicPatchCompound as PatchInput,
    };
    use crate::generated::stores::compound::Compounds;
    use stackpress_framework_sql::operations::{
        self, Flow, Invocation, ListenerId, Operation, Policy, Registration, Request, Response,
        Result, ServerInvocationExt, StartupContext,
    };
    use stackpress_framework_sql::{DataError, DataResult, DataSession};
    /// Strict key envelope; unknown request fields are invalid.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct KeyRequest {
        /// Complete declared identity.
        pub key: Key,
    }
    /// Strict approved creation envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CreateRequest {
        /// Approved creation members only.
        pub input: CreateInput,
    }
    /// Strict keyed update envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpdateRequest {
        /// Complete immutable identity.
        pub key: Key,
        /// Nonempty approved partial replacement.
        pub patch: PatchInput,
    }
    /// Strict keyed atomic UPSERT envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpsertRequest {
        /// Explicit primary conflict identity.
        pub key: Key,
        /// Candidate row with matching identity.
        pub input: CreateInput,
        /// Independently validated conflict patch.
        pub patch: PatchInput,
    }
    /// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
    /// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
    pub enum BatchItem {
        /// Insert one prepared candidate.
        Create {
            /// Approved creation members.
            input: Box<CreateInput>,
        },
        /// Patch one complete identity.
        Update {
            /// Immutable target identity.
            key: Key,
            /// Approved partial replacement.
            patch: Box<PatchInput>,
        },
        /// One atomic native primary-key UPSERT.
        Upsert {
            /// Explicit conflict identity.
            key: Key,
            /// Candidate with the same identity.
            input: Box<CreateInput>,
            /// Approved conflict replacement.
            patch: Box<PatchInput>,
        },
    }
    /// Strict finite batch envelope; native whole-batch validation precedes writes.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BatchRequest {
        /// Ordered finite atomic work, fully preflighted before the first write.
        pub items: Vec<BatchItem>,
    }
    /// Native-only record policy used for safe host assembly and field-error filtering.
    fn policy_schema() -> DataResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"region\",\"rust_name\":\"region\",\"column\":\"region\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"code\",\"rust_name\":\"code\",\"column\":\"code\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"enabled\",\"rust_name\":\"enabled\",\"column\":\"enabled\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"clock\",\"rust_name\":\"clock\",\"column\":\"clock\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"instant\",\"rust_name\":\"instant\",\"column\":\"instant\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"ratio\",\"rust_name\":\"ratio\",\"column\":\"ratio\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::compound::CreateCompound> {
        operations::assemble(&input, host, policy, false, context)
    }
    /// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
    fn patch_input(
        input: PatchInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Patch> {
        let value: serde_json::Value = operations::assemble(&input, host, policy, true, context)?;
        let _ = context;
        serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)
    }
    /// Named authorized get handler; prior completion skips every effect.
    pub async fn get(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Compounds::get_key(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized detail handler; prior completion skips every effect.
    pub async fn detail(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Compounds::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Compounds::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized search handler; prior completion skips every effect.
    pub async fn search(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::CompoundQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Compounds::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Compounds::search_public(&mut DataSession::transaction(&mut transaction), input).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized create handler; prior completion skips every effect.
    pub async fn create(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Compound", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Compounds::create_with(&mut DataSession::pooled(engine), input, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized update handler; prior completion skips every effect.
    pub async fn update(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Compound", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Compounds::update_key(
            &mut DataSession::pooled(engine),
            &input.key,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?
        .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized upsert handler; prior completion skips every effect.
    pub async fn upsert(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Compound", Operation::Upsert, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpsertRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let create = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Compounds::upsert_key(
            &mut DataSession::pooled(engine),
            &input.key,
            create,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized batch handler; prior completion skips every effect.
    pub async fn batch(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Compound", Operation::Batch, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: BatchRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        if input.items.is_empty() || input.items.len() > 100 {
            return Err(operations::data_error(
                DataError::new("batch item limit exceeded"),
                &schema,
            ));
        }
        let mut items = Vec::new();
        for item in input.items {
            items.push(match item {
                BatchItem::Create { input } => {
                    crate::generated::stores::compound::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::compound::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::compound::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Compounds::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::compound::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::compound::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::compound::BatchOutcome::Upserted(row) => {
                    row.project().map(Some)
                }
            })
            .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()
            .map_err(|cause| {
                operations::data_error(
                    stackpress_framework_sql::stores::projection_error(cause),
                    &schema,
                )
            })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized delete handler; prior completion skips every effect.
    pub async fn delete(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Compound", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Compounds::delete(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Select named capabilities explicitly; services must exist before this Listen callback.
    pub fn register(
        start: &mut StartupContext,
        registration: &mut Registration,
        prefix: &str,
        operations: &[Operation],
    ) -> Result<Vec<ListenerId>> {
        start.service::<stackpress_sql::Engine>()?;
        start.service::<Policy>()?;
        if operations
            .iter()
            .any(|operation| matches!(operation, Operation::Remove | Operation::Restore))
        {
            return Err(operations::Error::invalid(
                "soft removal requires a Boolean active model",
            ));
        }
        registration.claim(prefix, operations)?;
        let mut handles = Vec::new();
        for operation in operations {
            let event = format!("{prefix}.{}", operation.as_str());
            let handle = match operation {
                Operation::Get => start.on(&event, get)?,
                Operation::Detail => start.on(&event, detail)?,
                Operation::Search => start.on(&event, search)?,
                Operation::Create => start.on(&event, create)?,
                Operation::Update => start.on(&event, update)?,
                Operation::Upsert => start.on(&event, upsert)?,
                Operation::Batch => start.on(&event, batch)?,
                Operation::Delete => start.on(&event, delete)?,
                Operation::Remove | Operation::Restore => {
                    return Err(operations::Error::invalid("soft removal is unavailable"));
                }
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}

/// Explicit Query operation family; importing this module installs nothing.
pub mod query {
    use crate::generated::contracts::public::{
        PatchQuery as Patch, PublicCreateQuery as CreateInput, PublicPatchQuery as PatchInput,
        QueryKey as Key,
    };
    use crate::generated::stores::query::Querys;
    use stackpress_framework_sql::operations::{
        self, Flow, Invocation, ListenerId, Operation, Policy, Registration, Request, Response,
        Result, ServerInvocationExt, StartupContext,
    };
    use stackpress_framework_sql::{DataError, DataResult, DataSession};
    /// Strict key envelope; unknown request fields are invalid.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct KeyRequest {
        /// Complete declared identity.
        pub key: Key,
    }
    /// Strict approved creation envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CreateRequest {
        /// Approved creation members only.
        pub input: CreateInput,
    }
    /// Strict keyed update envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpdateRequest {
        /// Complete immutable identity.
        pub key: Key,
        /// Nonempty approved partial replacement.
        pub patch: PatchInput,
    }
    /// Strict keyed atomic UPSERT envelope.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct UpsertRequest {
        /// Explicit primary conflict identity.
        pub key: Key,
        /// Candidate row with matching identity.
        pub input: CreateInput,
        /// Independently validated conflict patch.
        pub patch: PatchInput,
    }
    /// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
    /// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
    pub enum BatchItem {
        /// Insert one prepared candidate.
        Create {
            /// Approved creation members.
            input: Box<CreateInput>,
        },
        /// Patch one complete identity.
        Update {
            /// Immutable target identity.
            key: Key,
            /// Approved partial replacement.
            patch: Box<PatchInput>,
        },
        /// One atomic native primary-key UPSERT.
        Upsert {
            /// Explicit conflict identity.
            key: Key,
            /// Candidate with the same identity.
            input: Box<CreateInput>,
            /// Approved conflict replacement.
            patch: Box<PatchInput>,
        },
    }
    /// Strict finite batch envelope; native whole-batch validation precedes writes.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BatchRequest {
        /// Ordered finite atomic work, fully preflighted before the first write.
        pub items: Vec<BatchItem>,
    }
    /// Native-only record policy used for safe host assembly and field-error filtering.
    fn policy_schema() -> DataResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"safe\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[\"id\",\"payload.row\",\"payload.state\"],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::query::CreateQuery> {
        operations::assemble(&input, host, policy, false, context)
    }
    /// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
    fn patch_input(
        input: PatchInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Patch> {
        let mut value: serde_json::Value =
            operations::assemble(&input, host, policy, true, context)?;
        let mut budget = stackpress_framework_schema::PreparationBudget::default();
        let object = value
            .as_object_mut()
            .ok_or_else(|| DataError::new("patch assembly requires an object"))?;
        // Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.
        if let Some(raw) = object.get_mut("payload").filter(|value| !value.is_null()) {
            let input: crate::generated::contracts::payload::CreatePayload =
                serde_json::from_value(raw.take())
                    .map_err(stackpress_framework_sql::codec_error)?;
            *raw = stackpress_framework_schema::to_bounded_value(
                &input
                    .prepare_bounded(context, &mut budget)
                    .map_err(stackpress_framework_sql::stores::preparation_error)?,
            )
            .map_err(stackpress_framework_sql::stores::validation)?;
        }
        serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)
    }
    /// Named authorized get handler; prior completion skips every effect.
    pub async fn get(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Querys::get_key(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized detail handler; prior completion skips every effect.
    pub async fn detail(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Querys::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Querys::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized search handler; prior completion skips every effect.
    pub async fn search(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::QueryQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Querys::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Querys::search_public(&mut DataSession::transaction(&mut transaction), input).await;
        let result = stackpress_framework_sql::finish(transaction, outcome)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized create handler; prior completion skips every effect.
    pub async fn create(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Query", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Querys::create_with(&mut DataSession::pooled(engine), input, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized update handler; prior completion skips every effect.
    pub async fn update(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Query", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Querys::update_key(
            &mut DataSession::pooled(engine),
            &input.key,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?
        .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized upsert handler; prior completion skips every effect.
    pub async fn upsert(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Query", Operation::Upsert, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpsertRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let create = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Querys::upsert_key(
            &mut DataSession::pooled(engine),
            &input.key,
            create,
            patch,
            &context,
        )
        .await
        .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized batch handler; prior completion skips every effect.
    pub async fn batch(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Query", Operation::Batch, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: BatchRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        if input.items.is_empty() || input.items.len() > 100 {
            return Err(operations::data_error(
                DataError::new("batch item limit exceeded"),
                &schema,
            ));
        }
        let mut items = Vec::new();
        for item in input.items {
            items.push(match item {
                BatchItem::Create { input } => {
                    crate::generated::stores::query::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::query::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::query::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Querys::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::query::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::query::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::query::BatchOutcome::Upserted(row) => {
                    row.project().map(Some)
                }
            })
            .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()
            .map_err(|cause| {
                operations::data_error(
                    stackpress_framework_sql::stores::projection_error(cause),
                    &schema,
                )
            })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Named authorized delete handler; prior completion skips every effect.
    pub async fn delete(
        req: &mut Request,
        res: &mut Response,
        invocation: Invocation<'_>,
    ) -> Result<Flow> {
        // Respect an earlier listener before policy, decoding or database work.
        if operations::completed(res) {
            return Ok(Flow::Continue);
        }
        // App authorization is required independently of schema/UI exposure.
        let policy = invocation.service::<Policy>()?;
        operations::authorize(policy, "Query", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Querys::delete(&mut DataSession::pooled(engine), &input.key)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?
            .ok_or_else(operations::not_found)?;
        let result = row.project().map_err(|cause| {
            operations::data_error(
                stackpress_framework_sql::stores::projection_error(cause),
                &schema,
            )
        })?;
        // Publish only the already projected owned result after data acknowledgement.
        res.results(&result)?;
        Ok(Flow::Continue)
    }
    /// Select named capabilities explicitly; services must exist before this Listen callback.
    pub fn register(
        start: &mut StartupContext,
        registration: &mut Registration,
        prefix: &str,
        operations: &[Operation],
    ) -> Result<Vec<ListenerId>> {
        start.service::<stackpress_sql::Engine>()?;
        start.service::<Policy>()?;
        if operations
            .iter()
            .any(|operation| matches!(operation, Operation::Remove | Operation::Restore))
        {
            return Err(operations::Error::invalid(
                "soft removal requires a Boolean active model",
            ));
        }
        registration.claim(prefix, operations)?;
        let mut handles = Vec::new();
        for operation in operations {
            let event = format!("{prefix}.{}", operation.as_str());
            let handle = match operation {
                Operation::Get => start.on(&event, get)?,
                Operation::Detail => start.on(&event, detail)?,
                Operation::Search => start.on(&event, search)?,
                Operation::Create => start.on(&event, create)?,
                Operation::Update => start.on(&event, update)?,
                Operation::Upsert => start.on(&event, upsert)?,
                Operation::Batch => start.on(&event, batch)?,
                Operation::Delete => start.on(&event, delete)?,
                Operation::Remove | Operation::Restore => {
                    return Err(operations::Error::invalid("soft removal is unavailable"));
                }
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}
