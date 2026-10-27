//! Named generated handlers; registration and transport mapping are always explicit.

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
        serde_json::from_str("{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"projects\",\"rust_name\":\"projects\",\"column\":\"projects\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Project\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
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
        input.validate().map_err(|cause| {
            operations::data_error(stackpress_framework_sql::stores::validation(cause), &schema)
        })?;
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

/// Explicit Project operation family; importing this module installs nothing.
pub mod project {
    use crate::generated::contracts::public::{
        PatchProject as Patch, ProjectKey as Key, PublicCreateProject as CreateInput,
        PublicPatchProject as PatchInput,
    };
    use crate::generated::stores::project::Projects;
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
        serde_json::from_str("{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"owner\",\"rust_name\":\"owner\",\"column\":\"owner\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"tasks\",\"rust_name\":\"tasks\",\"column\":\"tasks\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Task\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::project::CreateProject> {
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
        operations::authorize(policy, "Project", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Projects::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Project", Operation::Detail, req)?;
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
            Projects::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Project", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::ProjectQuery = operations::decode(req)?;
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
            Projects::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Project", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Project", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Projects::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Project", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Project", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Projects::update_key(
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
        operations::authorize(policy, "Project", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Project", Operation::Upsert, req)
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
        let row = Projects::upsert_key(
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
        operations::authorize(policy, "Project", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Project", Operation::Batch, req)
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
                    crate::generated::stores::project::BatchInput::Create {
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::project::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::project::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Projects::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::project::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::project::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::project::BatchOutcome::Upserted(row) => {
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
        operations::authorize(policy, "Project", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Projects::delete(&mut DataSession::pooled(engine), &input.key)
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

/// Explicit Task operation family; importing this module installs nothing.
pub mod task {
    use crate::generated::contracts::public::{
        PatchTask as Patch, PublicCreateTask as CreateInput, PublicPatchTask as PatchInput,
        TaskKey as Key,
    };
    use crate::generated::stores::task::Tasks;
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
        serde_json::from_str("{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"project\",\"rust_name\":\"project\",\"column\":\"project\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Project\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":true,\"writable\":true,\"sensitive\":false,\"publish_default\":true},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(stackpress_framework_sql::codec_error)
    }
    /// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
    fn create_input(
        input: CreateInput,
        host: &serde_json::Value,
        policy: &stackpress_framework_schema::RecordSchema,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<crate::generated::contracts::task::CreateTask> {
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
        operations::authorize(policy, "Task", Operation::Get, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Tasks::get_key(&mut DataSession::pooled(engine), &input.key)
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
        operations::authorize(policy, "Task", Operation::Detail, req)?;
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
            Tasks::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
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
        operations::authorize(policy, "Task", Operation::Search, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: crate::generated::contracts::public::TaskQuery = operations::decode(req)?;
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
            Tasks::search_public(&mut DataSession::transaction(&mut transaction), input).await;
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
        operations::authorize(policy, "Task", Operation::Create, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Task", Operation::Create, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: CreateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let input = create_input(input.input, &host.create, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Tasks::create_with(&mut DataSession::pooled(engine), input, &context)
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
        operations::authorize(policy, "Task", Operation::Update, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Task", Operation::Update, req)
            .map_err(operations::host_error)?;
        let context = host.context();
        // Decode only this operation's strict public envelope.
        let input: UpdateRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let patch = patch_input(input.patch, &host.patch, &schema, &context)
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let row = Tasks::update_key(
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
        operations::authorize(policy, "Task", Operation::Upsert, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Task", Operation::Upsert, req)
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
        let row = Tasks::upsert_key(
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
        operations::authorize(policy, "Task", Operation::Batch, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Host inputs use one explicit owned clock/provider set for this invocation.
        let host = policy
            .0
            .values("Task", Operation::Batch, req)
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
                BatchItem::Create { input } => crate::generated::stores::task::BatchInput::Create {
                    input: create_input(*input, &host.create, &schema, &context)
                        .map_err(|cause| operations::data_error(cause, &schema))?,
                },
                BatchItem::Update { key, patch } => {
                    crate::generated::stores::task::BatchInput::Update {
                        key,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
                BatchItem::Upsert { key, input, patch } => {
                    crate::generated::stores::task::BatchInput::Upsert {
                        key,
                        input: create_input(*input, &host.create, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                        patch: patch_input(*patch, &host.patch, &schema, &context)
                            .map_err(|cause| operations::data_error(cause, &schema))?,
                    }
                }
            });
        }
        let rows = Tasks::batch_atomic(engine, items, &context)
            .await
            .map_err(|cause| operations::data_error(cause, &schema))?;
        let result = rows
            .into_iter()
            .map(|row| match row {
                crate::generated::stores::task::BatchOutcome::Created(row) => {
                    row.project().map(Some)
                }
                crate::generated::stores::task::BatchOutcome::Updated(row) => {
                    row.map(|row| row.project()).transpose()
                }
                crate::generated::stores::task::BatchOutcome::Upserted(row) => {
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
        operations::authorize(policy, "Task", Operation::Delete, req)?;
        let schema = policy_schema().map_err(operations::host_error)?;
        // Decode only this operation's strict public envelope.
        let input: KeyRequest = operations::decode(req)?;
        // The executing domain functions receive the exact registered executor.
        let engine = invocation.service::<stackpress_sql::Engine>()?;
        let row = Tasks::delete(&mut DataSession::pooled(engine), &input.key)
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
