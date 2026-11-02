/// Named authorized [%operation.name%] handler; prior completion skips every effect.
pub async fn [%operation.name%](req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    // Respect an earlier listener before policy, decoding or database work.
    if operations::completed(res) {
        return Ok(Flow::Continue);
    }
    // App authorization is required independently of schema/UI exposure.
    let policy = invocation.service::<Policy>()?;
    operations::authorize(policy, [%model_literal%], Operation::[%operation.variant%], req)?;
    let schema = policy_schema().map_err(operations::host_error)?;
[%#?:operation.host%]    // Host inputs use one explicit owned clock/provider set for this invocation.
    let host = policy.0.values([%model_literal%], Operation::[%operation.variant%], req).map_err(operations::host_error)?;
    let context = host.context();
[%/?:operation.host%]    // Decode only this operation's strict public envelope.
    let input: [%#?:operation.search%]crate::generated::contracts::public::[%model%]Query[%|%][%operation.input%][%/?:operation.search%] = operations::decode(req)?;
    // The executing domain functions receive the exact registered executor.
    let engine = invocation.service::<stackpress_sql::Engine>()?;
[%#?:operation.get%]    let row = [%model%]s::get_key(&mut DataSession::pooled(engine), &input.key).await.map_err(|cause| operations::data_error(cause, &schema))?.ok_or_else(operations::not_found)?;
[%/?:operation.get%][%#?:operation.detail%]    // Refuse the exact public detail plan before entering the selected executor.
    [%model%]s::preflight_detail(engine, &input.key).map_err(|cause| operations::data_error(cause, &schema))?;
    let mut transaction = engine.begin().await.map_err(|cause| operations::data_error(cause.into(), &schema))?;
    let outcome = [%model%]s::detail(&mut DataSession::transaction(&mut transaction), &input.key).await;
    let result = stackpress_framework_sql::finish(transaction, outcome).await.map_err(|cause| operations::data_error(cause, &schema))?.ok_or_else(operations::not_found)?;
[%/?:operation.detail%][%#?:operation.search%]    // Preserve public DTO/catalog checks and validate selected limits before BEGIN.
    [%model%]s::preflight_search_public(engine, &input).map_err(|cause| operations::data_error(cause, &schema))?;
    let mut transaction = engine.begin().await.map_err(|cause| operations::data_error(cause.into(), &schema))?;
    let outcome = [%model%]s::search_public(&mut DataSession::transaction(&mut transaction), input).await;
    let result = stackpress_framework_sql::finish(transaction, outcome).await.map_err(|cause| operations::data_error(cause, &schema))?;
[%/?:operation.search%][%#?:operation.create%]    let input = create_input(input.input, &host.create, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?;
    let row = [%model%]s::create_with(&mut DataSession::pooled(engine), input, &context).await.map_err(|cause| operations::data_error(cause, &schema))?;
[%/?:operation.create%][%#?:operation.patch%]    let patch = patch_input(input.patch, &host.patch, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?;
[%#?:operation.upsert%]    let create = create_input(input.input, &host.create, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?;
    let row = [%model%]s::upsert_key(&mut DataSession::pooled(engine), &input.key, create, patch, &context).await.map_err(|cause| operations::data_error(cause, &schema))?;
[%|%]    let row = [%model%]s::update_key(&mut DataSession::pooled(engine), &input.key, patch, &context).await.map_err(|cause| operations::data_error(cause, &schema))?.ok_or_else(operations::not_found)?;
[%/?:operation.upsert%][%/?:operation.patch%][%#?:operation.batch%]    if input.items.is_empty() || input.items.len() > 100 {
        return Err(operations::data_error(DataError::new("batch item limit exceeded"), &schema));
    }
    let mut items = Vec::new();
    for item in input.items {
        items.push(match item {
            BatchItem::Create { input } => crate::generated::stores::[%store%]::BatchInput::Create {
                input: create_input(*input, &host.create, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?
            },
            BatchItem::Update { key, patch } => crate::generated::stores::[%store%]::BatchInput::Update {
                key, patch: patch_input(*patch, &host.patch, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?
            },
            BatchItem::Upsert { key, input, patch } => crate::generated::stores::[%store%]::BatchInput::Upsert {
                key,
                input: create_input(*input, &host.create, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?,
                patch: patch_input(*patch, &host.patch, &schema, &context).map_err(|cause| operations::data_error(cause, &schema))?
            }
        });
    }
    let rows = [%model%]s::batch_atomic(engine, items, &context).await.map_err(|cause| operations::data_error(cause, &schema))?;
    let result = rows.into_iter().map(|row| match row {
        crate::generated::stores::[%store%]::BatchOutcome::Created(row) => row.project().map(Some),
        crate::generated::stores::[%store%]::BatchOutcome::Updated(row) => row.map(|row| row.project()).transpose(),
        crate::generated::stores::[%store%]::BatchOutcome::Upserted(row) => row.project().map(Some)
    }).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>().map_err(|cause| operations::data_error(stackpress_framework_sql::stores::projection_error(cause), &schema))?;
[%/?:operation.batch%][%#?:operation.mutation%]    let row = [%model%]s::[%operation.name%](&mut DataSession::pooled(engine), &input.key[%#?:operation.context%], &context[%/?:operation.context%]).await.map_err(|cause| operations::data_error(cause, &schema))?.ok_or_else(operations::not_found)?;
[%/?:operation.mutation%][%#?:operation.project_row%]    let result = row.project().map_err(|cause| operations::data_error(stackpress_framework_sql::stores::projection_error(cause), &schema))?;
[%/?:operation.project_row%]    // Publish only the already projected owned result after data acknowledgement.
    res.results(&result)?;
    Ok(Flow::Continue)
}
