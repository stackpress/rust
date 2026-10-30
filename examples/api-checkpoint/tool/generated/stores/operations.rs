//! Named generated handlers; registration and transport mapping are always explicit.

/// Explicit Application operation family; importing this module installs nothing.
pub mod application {
    use crate::generated::contracts::public::{
        ApplicationKey as Key, PatchApplication as Patch, PublicCreateApplication as CreateInput,
        PublicPatchApplication as PatchInput,
    };
    use crate::generated::stores::application::Applications;
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
        serde_json::from_str("{\"model\":{\"name\":\"Application\",\"rust_name\":\"Application\",\"module\":\"application\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"ID\",\"description\":\"Unique generated identifier.\",\"examples\":[\"tg8bcf7e2lig3iuej3dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"profileId\",\"rust_name\":\"profile_id\",\"column\":\"profile_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Profile\",\"description\":\"The profile author of this application.\",\"examples\":[\"pjfdz7tg8bcf7e2lig3iuej3\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Name is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Name\",\"description\":\"Arbitrary name of the application. Used in 3-legged OAuth flows.\",\"examples\":[\"My Awesome App\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"logo\",\"rust_name\":\"logo\",\"column\":\"logo\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Image\",\"description\":\"Logo image URL of the application. Used in 3-legged OAuth flows.\",\"examples\":[\"https://example.com/logo.png\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"website\",\"rust_name\":\"website\",\"column\":\"website\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Website\",\"description\":\"Arbitrary website URL of the application. Used in 3-legged OAuth flows.\",\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"Secret\",\"description\":\"Generated unique hash needed to perform write operations (POST, PUT, DELETE, etc.)\",\"examples\":[\"bcf7e2lig3iuej3tg8dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"scopes\",\"rust_name\":\"scopes\",\"column\":\"scopes\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Scopes\",\"description\":\"API scopes that this application can request. This is mapped with endpoints.\",\"examples\":[[\"commerce\",\"location\",\"account\"]],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Active\",\"description\":\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"filter\",\"component\":\"api.source.field.switch\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"expires\",\"rust_name\":\"expires\",\"column\":\"expires\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Expires\",\"description\":\"Expiration date of the application. After this date, the application will not be able to perform any operations.\",\"examples\":[\"2025-12-31T23:59:59Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Created\",\"description\":\"Generated timestamp when row was created.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Updated\",\"description\":\"Generated timestamp that is updated whenever the row has changed.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"session\",\"rust_name\":\"session\",\"column\":\"session\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Session\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Sessions\",\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":false,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"profile\",\"rust_name\":\"profile\",\"column\":\"profile\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":false,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":\"{{name}}\",\"icon\":\"laptop\",\"labels\":[\"App\",\"Apps\"]},\"query\":[\"id\",\"profileId\",\"name\",\"logo\",\"website\",\"secret\",\"scopes\",\"active\",\"expires\",\"created\",\"updated\",\"profile.id\",\"profile.image\",\"profile.name\",\"profile.type\",\"profile.roles\",\"profile.tags\",\"profile.references\",\"profile.active\",\"profile.created\",\"profile.updated\"],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::application::CreateApplication> {
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
        operations::authorize(policy, "Application", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Applications::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Application", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Applications::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Applications::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Application", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::ApplicationQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Applications::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Applications::search_public(&mut DataSession::transaction(&mut transaction), input)
                .await;
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
        operations::authorize(policy, "Application", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Applications::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Application", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Applications::update_key(
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
        operations::authorize(policy, "Application", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Upsert, req)
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
        let row = Applications::upsert_key(
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
        operations::authorize(policy, "Application", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Batch, req)
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
                    crate::generated::stores::application::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::application::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::application::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Applications::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::application::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::application::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::application::BatchOutcome::Upserted(row) => {
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
    /// Named authorized remove handler; prior completion skips every effect.
    pub async fn remove(
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
        operations::authorize(policy, "Application", Operation::Remove, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Remove, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Applications::remove(&mut DataSession::pooled(engine), &input.key, &context)
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
    /// Named authorized restore handler; prior completion skips every effect.
    pub async fn restore(
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
        operations::authorize(policy, "Application", Operation::Restore, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Application", Operation::Restore, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Applications::restore(&mut DataSession::pooled(engine), &input.key, &context)
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
        operations::authorize(policy, "Application", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Applications::delete(&mut DataSession::pooled(engine), &input.key)
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
                Operation::Remove => start.on(&event, remove)?,
                Operation::Restore => start.on(&event, restore)?,
                Operation::Delete => start.on(&event, delete)?,
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}

/// Explicit Session operation family; importing this module installs nothing.
pub mod session {
    use crate::generated::contracts::public::{
        PatchSession as Patch, PublicCreateSession as CreateInput,
        PublicPatchSession as PatchInput, SessionKey as Key,
    };
    use crate::generated::stores::session::Sessions;
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
        serde_json::from_str("{\"model\":{\"name\":\"Session\",\"rust_name\":\"Session\",\"module\":\"session\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"ID\",\"description\":\"Unique generated identifier.\",\"examples\":[\"7e2lig3iuej3bcftg8dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"applicationId\",\"rust_name\":\"application_id\",\"column\":\"application_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"App is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"App\",\"description\":\"Application this session belongs to.\",\"examples\":[\"tg8bcf7e2lig3iuej3dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/application/search?json\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/application/search?json\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{application.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{application.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"profileId\",\"rust_name\":\"profile_id\",\"column\":\"profile_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Profile\",\"description\":\"Profile this session belongs to.\",\"examples\":[\"pjfdz7tg8bcf7e2lig3iuej3\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":\"Secret\",\"description\":\"Generated unique hash needed to perform write operations (POST, PUT, DELETE, etc.)\",\"examples\":[\"lig3iuej37e2bcftg8dz7pjf\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"scopes\",\"rust_name\":\"scopes\",\"column\":\"scopes\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Scopes\",\"description\":\"API scopes that this session can request. This is mapped with endpoints.\",\"examples\":[[\"commerce\",\"location\",\"account\"]],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":\"Active\",\"description\":\"Special flag to indicate active rows. Inactive rows are not shown in the list view, but can be viewed in the detail view.\",\"examples\":[true],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"filter\",\"component\":\"api.source.field.switch\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"expires\",\"rust_name\":\"expires\",\"column\":\"expires\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":\"Expires\",\"description\":\"Expiration date of the session. After this date, the session will not be able to perform any operations.\",\"examples\":[\"2025-12-31T23:59:59Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Created\",\"description\":\"Generated timestamp when row was created.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":\"Updated\",\"description\":\"Generated timestamp that is updated whenever the row has changed.\",\"examples\":[\"2025-10-01T12:00:00Z\"],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"application\",\"rust_name\":\"application\",\"column\":\"application\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Application\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"applicationId\",\"foreign\":\"id\",\"inverse\":\"session\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":false,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"profile\",\"rust_name\":\"profile\",\"column\":\"profile\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":false,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":\"{{profile.name}}\",\"icon\":\"coffee\",\"labels\":[\"Session\",\"Sessions\"]},\"query\":[\"id\",\"applicationId\",\"profileId\",\"secret\",\"scopes\",\"active\",\"expires\",\"created\",\"updated\",\"profile.id\",\"profile.image\",\"profile.name\",\"profile.type\",\"profile.roles\",\"profile.tags\",\"profile.references\",\"profile.active\",\"profile.created\",\"profile.updated\",\"application.id\",\"application.profileId\",\"application.name\",\"application.logo\",\"application.website\",\"application.secret\",\"application.scopes\",\"application.active\",\"application.expires\",\"application.created\",\"application.updated\"],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::session::CreateSession> {
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
        operations::authorize(policy, "Session", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Sessions::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Session", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Sessions::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Sessions::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Session", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::SessionQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Sessions::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Sessions::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Session", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Sessions::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Session", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Sessions::update_key(
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
        operations::authorize(policy, "Session", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Upsert, req)
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
        let row = Sessions::upsert_key(
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
        operations::authorize(policy, "Session", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Batch, req)
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
                    crate::generated::stores::session::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::session::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::session::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Sessions::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::session::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::session::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::session::BatchOutcome::Upserted(row) => {
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
    /// Named authorized remove handler; prior completion skips every effect.
    pub async fn remove(
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
        operations::authorize(policy, "Session", Operation::Remove, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Remove, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Sessions::remove(&mut DataSession::pooled(engine), &input.key, &context)
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
    /// Named authorized restore handler; prior completion skips every effect.
    pub async fn restore(
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
        operations::authorize(policy, "Session", Operation::Restore, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Session", Operation::Restore, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Sessions::restore(&mut DataSession::pooled(engine), &input.key, &context)
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
        operations::authorize(policy, "Session", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Sessions::delete(&mut DataSession::pooled(engine), &input.key)
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
                Operation::Remove => start.on(&event, remove)?,
                Operation::Restore => start.on(&event, restore)?,
                Operation::Delete => start.on(&event, delete)?,
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}

/// Explicit Profile operation family; importing this module installs nothing.
pub mod profile {
    use crate::generated::contracts::public::{
        PatchProfile as Patch, ProfileKey as Key, PublicCreateProfile as CreateInput,
        PublicPatchProfile as PatchInput,
    };
    use crate::generated::stores::profile::Profiles;
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
        serde_json::from_str("{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"image\",\"rust_name\":\"image\",\"column\":\"image\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"type\",\"rust_name\":\"type_\",\"column\":\"type_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"person\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"roles\",\"rust_name\":\"roles\",\"column\":\"roles\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[\"GUEST\"],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"references\",\"rust_name\":\"references\",\"column\":\"references\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":{},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"},\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::profile::CreateProfile> {
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
        operations::authorize(policy, "Profile", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Profiles::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Profile", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Refuse the exact public detail plan before entering the selected executor.
        Profiles::preflight_detail(engine, &input.key)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Profiles::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Profile", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::ProfileQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
        Profiles::preflight_search_public(engine, &input)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Profiles::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Profile", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Profiles::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Profile", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Profiles::update_key(
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
        operations::authorize(policy, "Profile", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Upsert, req)
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
        let row = Profiles::upsert_key(
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
        operations::authorize(policy, "Profile", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Batch, req)
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
                    crate::generated::stores::profile::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::profile::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::profile::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Profiles::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::profile::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::profile::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::profile::BatchOutcome::Upserted(row) => {
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
    /// Named authorized remove handler; prior completion skips every effect.
    pub async fn remove(
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
        operations::authorize(policy, "Profile", Operation::Remove, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Remove, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Profiles::remove(&mut DataSession::pooled(engine), &input.key, &context)
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
    /// Named authorized restore handler; prior completion skips every effect.
    pub async fn restore(
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
        operations::authorize(policy, "Profile", Operation::Restore, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Profile", Operation::Restore, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Profiles::restore(&mut DataSession::pooled(engine), &input.key, &context)
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
        operations::authorize(policy, "Profile", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Profiles::delete(&mut DataSession::pooled(engine), &input.key)
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
                Operation::Remove => start.on(&event, remove)?,
                Operation::Restore => start.on(&event, restore)?,
                Operation::Delete => start.on(&event, delete)?,
            };
            handles.push(handle);
        }
        Ok(handles)
    }
}
