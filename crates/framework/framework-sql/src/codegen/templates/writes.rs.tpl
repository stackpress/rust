
impl [%model%]Store {
    /// Encode one prepared owned value; no host provider/default is evaluated by this plan builder.
    pub fn insert(value: &_Stored) -> DataResult<Insert> {
        Ok(Insert::new([%table_literal%]).columns(COLUMNS.iter().map(|column| column.column))
            .values(Self::encode(value)?).returning(COLUMNS.iter().map(|column| column.column)))
    }
    /// Encode the exact declared record shape after stored/business validation, without regenerating values.
    pub fn encode(value: &_Stored) -> DataResult<Vec<stackpress_sql::SqlValue>> {
        value.validate().map_err(stackpress_framework_sql::stores::validation)?;
[%#?:bounded%]        Self::storage_bounds(value)?;
[%/?:bounded%]        stackpress_framework_sql::encode(value, COLUMNS)
    }
    /// Validate one nonempty patch, refresh its timestamp and scope it to the complete identity.
    pub fn update_plan(key: &Key, patch: &Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<stackpress_sql::Update> {
        if MUTABLE_COLUMNS.is_empty() {
            return Err(DataError::new("model declares no mutable patch fields").with_kind(DataErrorKind::Unsupported));
        }
        patch.validate(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
        let mut assignments = stackpress_framework_sql::encode_patch(patch, MUTABLE_COLUMNS)?;
        assignments.extend(timestamp(context)?);
[%#?:patch_bounded%]        Self::patch_bounds(&assignments)?;
[%|%]
[%/?:patch_bounded%]        let mut query = stackpress_sql::Update::new([%table_literal%]).and_where(SPEC.key_predicate(key)?);
        for (column, value) in assignments {
            query = query.set(column, value);
        }
        Ok(query.returning(COLUMNS.iter().map(|column| column.column)))
    }
    /// Native targeted UPSERT over a prepared create value and an independently validated patch.
    pub fn upsert_plan(key: &Key, value: &_Stored, patch: &Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Insert> {
        Self::upsert_on_plan(&ConflictKey::Primary(key.clone()), value, patch, context)
    }
    /// Hard deletion is a separately selected key-scoped operation; foreign keys remain RESTRICT.
    pub fn delete_plan(key: &Key) -> DataResult<stackpress_sql::Delete> {
        Ok(stackpress_sql::Delete::new([%table_literal%]).and_where(SPEC.key_predicate(key)?).returning(COLUMNS.iter().map(|column| column.column)))
    }
}
impl [%model%]s {
    /// Prepare using the default context; explicit host values use create_with.
    pub async fn create(db: &mut DataSession<'_>, input: _Create) -> DataResult<_Stored> {
        Self::create_with(db, input, &stackpress_framework_schema::ValueContext::default()).await
    }
    /// Prepare defaults/providers once before submitting the write once.
    pub async fn create_with(db: &mut DataSession<'_>, input: _Create, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(db.write(&[%model%]Store::insert(&value)?).await?)
    }
    /// Targeted no-change is an explicit AlreadyExists result, never a fabricated row.
    pub async fn create_if_absent(db: &mut DataSession<'_>, input: _Create, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<stackpress_framework_sql::InsertOutcome<_Stored>> {
        db.require_returning()?;
        let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
        let query = [%model%]Store::insert(&value)?.on_conflict(stackpress_sql::InsertConflict::DoNothing {
            target: SPEC.key.iter().map(|name| SPEC.column(name).map(|column| column.column.into())).collect::<DataResult<Vec<_>>>()?
        });
        let rows = db.write(&query).await?;
        if rows.is_empty() {
            return Ok(stackpress_framework_sql::InsertOutcome::AlreadyExists);
        }
        Ok(stackpress_framework_sql::InsertOutcome::Inserted(Self::one_returned(rows)?))
    }
    /// Last-writer-wins convenience for the original simple key shape.
    pub async fn update(db: &mut DataSession<'_>, key: [%key_type%], patch: Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Option<_Stored>> {
        Self::update_key(db, [%key_value%], patch, context).await
    }
    /// Last-writer-wins complete-key update, with no optimistic-version or retry behavior.
    pub async fn update_key(db: &mut DataSession<'_>, key: &Key, patch: Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        let rows = db.write(&[%model%]Store::update_plan(key, &patch, context)?).await?;
        Self::optional_returned(rows)
    }
    /// Convenience targeted UPSERT using the original simple key shape.
    pub async fn upsert(db: &mut DataSession<'_>, key: [%key_type%], input: _Create, patch: Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<_Stored> {
        Self::upsert_key(db, [%key_value%], input, patch, context).await
    }
    /// Atomic native targeted UPSERT, never a check-then-create sequence.
    pub async fn upsert_key(db: &mut DataSession<'_>, key: &Key, input: _Create, patch: Patch, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(db.write(&[%model%]Store::upsert_plan(key, &value, &patch, context)?).await?)
    }
    /// Explicit hard delete, independently selected from soft remove.
    pub async fn delete(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        Self::optional_returned(db.write(&[%model%]Store::delete_plan(key)?).await?)
    }
    /// Decode exactly one authoritative acknowledged row; never infer a write branch from row counts.
    fn one_returned(mut rows: Vec<Row>) -> DataResult<_Stored> {
        if rows.len() != 1 {
            return Err(DataError::new("write returned an invalid row count").with_kind(DataErrorKind::Decode));
        }
        [%model%]Store::decode(rows.remove(0))
    }
    /// An absent keyed update/delete remains distinct from decode or native failure.
    fn optional_returned(rows: Vec<Row>) -> DataResult<Option<_Stored>> {
        if rows.is_empty() {
            Ok(None)
        } else {
            Self::one_returned(rows).map(Some)
        }
    }
}
/// Explicit finite batch item; modes cannot smuggle arbitrary predicates or nested writes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum BatchInput {
    /// Prepare and insert one row.
    Create { input: _Create },
    /// Update one complete identity.
    Update { key: Key, patch: Patch },
    /// Native atomic UPSERT for one complete identity.
    Upsert { key: Key, input: _Create, patch: Patch },
    /// Explicit native nonnull unique target, validated before any batch write.
    UpsertOn { target: ConflictKey, input: _Create, patch: Patch },
}
/// Ordered per-item batch results, retaining an absent update explicitly.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "mode", content = "row", rename_all = "snake_case")]
pub enum BatchOutcome {
    /// Authoritative inserted row.
    Created(_Stored),
    /// Updated row, or a missing keyed row.
    Updated(Option<_Stored>),
    /// Authoritative inserted/updated row; no branch flag is fabricated.
    Upserted(_Stored),
}
impl [%model%]s {
    /// Validate/prepare the entire bounded batch before its first write, retaining input order.
    /// A transaction session is required; callers own success completion and rollback on error.
    pub async fn batch(db: &mut DataSession<'_>, inputs: Vec<BatchInput>, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Vec<BatchOutcome>> {
        db.require_transaction()?;
        db.require_returning()?;
        if inputs.is_empty() || inputs.len() > 100 {
            return Err(DataError::new("batch item limit exceeded"));
        }
        stackpress_framework_schema::to_bounded_value(&inputs).map_err(stackpress_framework_sql::stores::validation)?;
        let mut plans = Vec::new();
        let mut bytes = 0usize;
        for input in inputs {
            let (mode, query) = match input {
                BatchInput::Create { input } => {
                    let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (BatchMode::Create, Prepared::Insert([%model%]Store::insert(&value)?))
                },
                BatchInput::Update { key, patch } => (BatchMode::Update, Prepared::Update([%model%]Store::update_plan(&key, &patch, context)?)),
                BatchInput::Upsert { key, input, patch } => {
                    let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (BatchMode::Upsert, Prepared::Insert([%model%]Store::upsert_plan(&key, &value, &patch, context)?))
                },
                BatchInput::UpsertOn { target, input, patch } => {
                    let value = input.prepare_with(context).map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (BatchMode::Upsert, Prepared::Insert([%model%]Store::upsert_on_plan(&target, &value, &patch, context)?))
                },
            };
            // Pure rendering validates every later plan before an earlier write can take effect.
            let rendered = stackpress_sql::Render::render(&query, db.dialect()).map_err(stackpress_framework_sql::write_plan_error)?;
            rendered.validate(db.limits()).map_err(stackpress_framework_sql::write_plan_error)?;
            bytes = bytes.saturating_add(rendered.sql().len()).saturating_add(rendered.binds().iter().map(stackpress_sql::SqlValue::retained_bytes).sum::<usize>());
            if bytes > 16 * 1024 * 1024 {
                return Err(DataError::new("prepared batch byte limit exceeded"));
            }
            plans.push((mode, rendered));
        }
        let mut output = Vec::new();
        for (mode, query) in plans {
            let rows = db.write(&query).await?;
            output.push(match mode {
                BatchMode::Create => BatchOutcome::Created(Self::one_returned(rows)?),
                BatchMode::Update => BatchOutcome::Updated(Self::optional_returned(rows)?),
                BatchMode::Upsert => BatchOutcome::Upserted(Self::one_returned(rows)?),
            });
        }
        Ok(output)
    }
    /// Own one transaction around the exact same validated batch function; never commit per item.
    pub async fn batch_atomic(engine: &stackpress_sql::Engine, inputs: Vec<BatchInput>, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Vec<BatchOutcome>> {
        // Check the existing write capability before acquiring any transaction or preparing providers.
        DataSession::pooled(engine).require_returning()?;
        let mut transaction = engine.begin().await?;
        let outcome = Self::batch(&mut DataSession::transaction(&mut transaction), inputs, context).await;
        stackpress_framework_sql::finish(transaction, outcome).await
    }
}
/// Exhaustive acknowledgement interpretation; no numeric fallback may invent a result mode.
enum BatchMode { Create, Update, Upsert }
/// Finite internal plan family for whole-batch preflight, without an executor capture.
enum Prepared { Insert(Insert), Update(stackpress_sql::Update) }
impl stackpress_sql::Render for Prepared {
    /// Delegate pure rendering without submitting any database work.
    fn render(&self, dialect: stackpress_sql::DialectId) -> stackpress_sql::Result<stackpress_sql::Query> {
        match self {
            Self::Insert(query) => stackpress_sql::Render::render(query, dialect),
            Self::Update(query) => stackpress_sql::Render::render(query, dialect),
        }
    }
}
