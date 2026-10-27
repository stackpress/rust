//! Named generated handlers; registration and transport mapping are always explicit.

/// Explicit Schema operation family; importing this module installs nothing.
pub mod _schema {
    use crate::generated::contracts::public::{
        PatchSchema as Patch, PublicCreateSchema as CreateInput, PublicPatchSchema as PatchInput,
        SchemaKey as Key,
    };
    use crate::generated::stores::_schema::Schemas;
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
        serde_json::from_str("{\"model\":{\"name\":\"Schema\",\"rust_name\":\"Schema\",\"module\":\"schema\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"safe\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"type\",\"rust_name\":\"type_\",\"column\":\"type_\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"eq\",\"args\":[true],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::schema::CreateSchema> {
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
        operations::authorize(policy, "Schema", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Schemas::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Schema", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Schemas::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Schema", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::SchemaQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        input.validate().map_err(|cause| {
            operations::data_error(stackpress_framework_sql::stores::validation(cause), &schema)
        })?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Schemas::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Schema", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Schemas::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Schema", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Schemas::update_key(
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
        operations::authorize(policy, "Schema", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Upsert, req)
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
        let row = Schemas::upsert_key(
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
        operations::authorize(policy, "Schema", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Batch, req)
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
                    crate::generated::stores::_schema::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::_schema::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::_schema::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Schemas::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::_schema::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::_schema::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::_schema::BatchOutcome::Upserted(row) => {
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
        operations::authorize(policy, "Schema", Operation::Remove, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Remove, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Schemas::remove(&mut DataSession::pooled(engine), &input.key, &context)
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
        operations::authorize(policy, "Schema", Operation::Restore, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Schema", Operation::Restore, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Schemas::restore(&mut DataSession::pooled(engine), &input.key, &context)
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
        operations::authorize(policy, "Schema", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Schemas::delete(&mut DataSession::pooled(engine), &input.key)
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

/// Explicit Operations operation family; importing this module installs nothing.
pub mod _operations {
    use crate::generated::contracts::public::{
        OperationsKey as Key, PatchOperations as Patch, PublicCreateOperations as CreateInput,
        PublicPatchOperations as PatchInput,
    };
    use crate::generated::stores::_operations::Operationss;
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
        serde_json::from_str("{\"model\":{\"name\":\"Operations\",\"rust_name\":\"Operations\",\"module\":\"operations\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"safe\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"schemaId\",\"rust_name\":\"schema_id\",\"column\":\"schema_id\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Schema\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"schemaId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[\"id\",\"row.value\"],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::operations::CreateOperations> {
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
        operations::authorize(policy, "Operations", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Operationss::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Operations", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Operationss::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Operations", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::OperationsQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        input.validate().map_err(|cause| {
            operations::data_error(stackpress_framework_sql::stores::validation(cause), &schema)
        })?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Operationss::search_public(&mut DataSession::transaction(&mut transaction), input)
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
        operations::authorize(policy, "Operations", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Operations", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Operationss::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Operations", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Operations", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Operationss::update_key(
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
        operations::authorize(policy, "Operations", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Operations", Operation::Upsert, req)
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
        let row = Operationss::upsert_key(
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
        operations::authorize(policy, "Operations", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Operations", Operation::Batch, req)
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
                    crate::generated::stores::_operations::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::_operations::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::_operations::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Operationss::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::_operations::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::_operations::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::_operations::BatchOutcome::Upserted(row) => {
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
        operations::authorize(policy, "Operations", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Operationss::delete(&mut DataSession::pooled(engine), &input.key)
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

/// Explicit Identity operation family; importing this module installs nothing.
pub mod identity {
    use crate::generated::contracts::public::{
        IdentityKey as Key, PatchIdentity as Patch, PublicCreateIdentity as CreateInput,
        PublicPatchIdentity as PatchInput,
    };
    use crate::generated::stores::identity::Identitys;
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
        serde_json::from_str("{\"model\":{\"name\":\"Identity\",\"rust_name\":\"Identity\",\"module\":\"identity\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::identity::CreateIdentity> {
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
        operations::authorize(policy, "Identity", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Identitys::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Identity", Operation::Detail, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Identitys::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Identity", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::IdentityQuery = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        input.validate().map_err(|cause| {
            operations::data_error(stackpress_framework_sql::stores::validation(cause), &schema)
        })?;
        let mut transaction = engine
            .begin()
            .await
            .map_err(|cause| operations::data_error(cause.into(), &schema))?;
        let outcome =
            Identitys::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Identity", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Identity", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Identitys::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Identity", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Identity", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Identitys::update_key(
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
        operations::authorize(policy, "Identity", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Identity", Operation::Upsert, req)
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
        let row = Identitys::upsert_key(
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
        operations::authorize(policy, "Identity", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Identity", Operation::Batch, req)
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
                    crate::generated::stores::identity::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::identity::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::identity::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Identitys::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::identity::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::identity::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::identity::BatchOutcome::Upserted(row) => {
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
        operations::authorize(policy, "Identity", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Identitys::delete(&mut DataSession::pooled(engine), &input.key)
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
            .any(|operation| matches!(operation, Operation::Update | Operation::Upsert))
        {
            return Err(operations::Error::invalid(
                "model declares no mutable patch fields",
            ));
        }
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
