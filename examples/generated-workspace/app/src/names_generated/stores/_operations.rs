//! Explicit generated Operations store; every operation borrows its executor.
use crate::generated::contracts::operations::{CreateOperations as _Create, Operations as _Stored};
use crate::generated::contracts::public::{OperationsKey as Key, PatchOperations as Patch};
use stackpress_framework_sql::stores::{
    Exposure, Loaded, Page, ReadQuery, RelationLimits, StoreSpec, Visibility,
};
use stackpress_framework_sql::{
    DataError, DataErrorKind, DataResult, DataSession, StorageCodec, StorageColumn,
};
use stackpress_sql::{Insert, Row, Select, TableDefinition};
const _: () = assert!(stackpress_framework_sql::CONTRACT_REVISION == 1);
/// Shared normalized schema identity.
pub const FINGERPRINT: &str = "dbc3f50dcd8942354ff56d52ef513a78f18d1d2e58287306362e6b7c83d46969";
/// Exact ordered stored codecs.
pub const COLUMNS: &[StorageColumn] = &[
    StorageColumn {
        wire: "id",
        column: "id",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "value",
        column: "value",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "schemaId",
        column: "schema_id",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
];
/// Pure inspectable table, query and codec functions.
pub struct OperationsStore;
impl OperationsStore {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new("operations");
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("id", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("value", stackpress_sql::SqlType::Text).not_null();
        table = table.add_field(field);
        // Preserve declared nullability in storage.
        let field = stackpress_sql::Field::new("schema_id", stackpress_sql::SqlType::Text);
        table = table.add_field(field);
        table = table.primary_key(["id"]);
        // Explicit owning FK; reverse collections have no column.
        let mut foreign = stackpress_sql::ForeignKey::new(
            "operations_schema_id_fk",
            ["schema_id"],
            "schema",
            ["id"],
        );
        foreign.on_delete = stackpress_sql::ReferentialAction::Restrict;
        table = table.add_foreign_key(foreign);
        Ok(table.definition()?)
    }
}
/// Exact immutable runtime metadata; it grants no transport authorization.
pub static SPEC: StoreSpec = StoreSpec {
    table: "operations",
    columns: COLUMNS,
    key: &["id"],
    active: None,
    fields: &[
        stackpress_framework_sql::stores::QueryField {
            storage: StorageColumn {
                wire: "id",
                column: "id",
                codec: StorageCodec::Text,
                required: true,
                nonempty: false,
            },
            multiple: false,
            scalar: StorageCodec::Text,
            choices: "[]",
            searchable: false,
            sortable: true,
            public_filter: true,
            public_sort: false,
            public_search: false,
        },
        stackpress_framework_sql::stores::QueryField {
            storage: StorageColumn {
                wire: "value",
                column: "value",
                codec: StorageCodec::Text,
                required: true,
                nonempty: false,
            },
            multiple: false,
            scalar: StorageCodec::Text,
            choices: "[]",
            searchable: false,
            sortable: false,
            public_filter: false,
            public_sort: false,
            public_search: false,
        },
        stackpress_framework_sql::stores::QueryField {
            storage: StorageColumn {
                wire: "schemaId",
                column: "schema_id",
                codec: StorageCodec::Text,
                required: false,
                nonempty: false,
            },
            multiple: false,
            scalar: StorageCodec::Text,
            choices: "[]",
            searchable: false,
            sortable: false,
            public_filter: false,
            public_sort: false,
            public_search: false,
        },
    ],
    relations: &[stackpress_framework_sql::stores::RelationSpec {
        name: "row",
        target: "schema",
        local: "schemaId",
        foreign: "id",
        many: false,
        required: false,
        public: true,
    }],
    decode: decode_value,
    project: project_value,
};
/// Validate through the actual owned model before bounded JSON enters the include arena.
fn decode_value(row: Row) -> DataResult<serde_json::Value> {
    stackpress_framework_schema::to_bounded_value(&OperationsStore::decode(row)?)
        .map_err(stackpress_framework_sql::stores::validation)
}
/// Project the actual model through the compiled app policy; no stored field is echoed implicitly.
fn project_value(value: serde_json::Value) -> DataResult<serde_json::Value> {
    let row: _Stored =
        serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)?;
    stackpress_framework_schema::to_bounded_value(
        &row.project()
            .map_err(stackpress_framework_sql::stores::projection_error)?,
    )
    .map_err(stackpress_framework_sql::stores::validation)
}
impl OperationsStore {
    /// Strict owned decode, including checked nested fields and stored validation.
    pub fn decode(row: Row) -> DataResult<_Stored> {
        let value: _Stored = stackpress_framework_sql::decode(row, COLUMNS)?;
        value.validate().map_err(|cause| {
            DataError::new("stored value validation failed")
                .with_kind(DataErrorKind::Decode)
                .with_cause(cause)
        })?;
        Ok(value)
    }
}
/// Finite depth-0 trusted include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View0 {
    /// Fully owned stored row.
    pub row: _Stored,
}
/// Finite depth-0 projected include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicView0 {
    /// Fully owned approved public row.
    pub row: crate::generated::contracts::public::PublicOperations,
}
impl View0 {
    /// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.
    pub fn project(self) -> DataResult<PublicView0> {
        Ok(PublicView0 {
            row: self
                .row
                .project()
                .map_err(stackpress_framework_sql::stores::projection_error)?,
        })
    }
}
/// Finite depth-1 trusted include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View1 {
    /// Fully owned stored row.
    pub row: _Stored,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::View0>>,
}
/// Finite depth-1 projected include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicView1 {
    /// Fully owned approved public row.
    pub row: crate::generated::contracts::public::PublicOperations,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::PublicView0>>,
}
impl View1 {
    /// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.
    pub fn project(self) -> DataResult<PublicView1> {
        Ok(PublicView1 {
            row: self
                .row
                .project()
                .map_err(stackpress_framework_sql::stores::projection_error)?,
            _relation_row: self
                ._relation_row
                .try_map(|values| values.map(|value| value.project()).transpose())?,
        })
    }
}
/// Finite depth-2 trusted include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View2 {
    /// Fully owned stored row.
    pub row: _Stored,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::View1>>,
}
/// Finite depth-2 projected include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicView2 {
    /// Fully owned approved public row.
    pub row: crate::generated::contracts::public::PublicOperations,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::PublicView1>>,
}
impl View2 {
    /// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.
    pub fn project(self) -> DataResult<PublicView2> {
        Ok(PublicView2 {
            row: self
                .row
                .project()
                .map_err(stackpress_framework_sql::stores::projection_error)?,
            _relation_row: self
                ._relation_row
                .try_map(|values| values.map(|value| value.project()).transpose())?,
        })
    }
}
/// Finite depth-3 trusted include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View3 {
    /// Fully owned stored row.
    pub row: _Stored,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::View2>>,
}
/// Finite depth-3 projected include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicView3 {
    /// Fully owned approved public row.
    pub row: crate::generated::contracts::public::PublicOperations,
    /// Explicit eager "row" relation.
    #[serde(rename = "__row", default)]
    pub _relation_row: Loaded<Option<super::_schema::PublicView2>>,
}
impl View3 {
    /// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.
    pub fn project(self) -> DataResult<PublicView3> {
        Ok(PublicView3 {
            row: self
                .row
                .project()
                .map_err(stackpress_framework_sql::stores::projection_error)?,
            _relation_row: self
                ._relation_row
                .try_map(|values| values.map(|value| value.project()).transpose())?,
        })
    }
}
/// Native scalar/list filters; direct store calls are trusted domain APIs.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum Filter {
    /// Typed "id" predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
    /// Typed "value" predicate.
    #[serde(rename = "value")]
    Value(stackpress_framework_schema::Predicate<String>),
    /// Typed "schemaId" predicate.
    #[serde(rename = "schemaId")]
    SchemaId(stackpress_framework_schema::Predicate<String>),
}
/// Native declared ordering fields; every page appends complete primary-key ties.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SortField {
    /// Order by "id".
    #[serde(rename = "id")]
    Id,
}
/// Finite native query with the same page/budget semantics as the pure public contract.
pub type Query = stackpress_framework_sql::stores::Query<Filter, SortField>;
impl OperationsStore {
    /// Preserve the original detached simple-key lookup API, without capturing an executor.
    pub fn select(key: &str) -> Select {
        let query = Select::new(COLUMNS.iter().map(|column| column.column))
            .from("operations")
            .where_eq("id", key);
        query.limit(1)
    }
}

impl OperationsStore {
    /// Build a complete-identity ordinary read, including every compound key member.
    pub fn select_key(key: &Key, visibility: Visibility) -> DataResult<Select> {
        SPEC.select_key(key, visibility)
    }
    /// Build a checked deterministic page without database work.
    pub fn search_plan(query: &Query, dialect: stackpress_sql::DialectId) -> DataResult<Select> {
        stackpress_framework_sql::stores::select(
            &SPEC,
            &ReadQuery::from_typed(query)?,
            dialect,
            Exposure::Trusted,
            Visibility::Active,
        )
    }
}
/// Stateless executing functions; callers always select the exact borrowed executor.
pub struct Operationss;
impl Operationss {
    /// Ordinary base lookup; absence is distinct from storage/validation failure.
    pub async fn get(db: &mut DataSession<'_>, key: &str) -> DataResult<Option<_Stored>> {
        Self::get_key(db, &Key { id: key.into() }).await
    }
    /// Ordinary lookup using the full typed identity, including non-text and compound keys.
    pub async fn get_key(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<_Stored>> {
        let mut rows = db
            .fetch(&OperationsStore::select_key(key, Visibility::Active)?)
            .await?;
        rows.pop().map(OperationsStore::decode).transpose()
    }
    /// Explicit maintenance lookup; inactive/all selection never comes from public request data.
    pub async fn get_maintenance(
        db: &mut DataSession<'_>,
        key: &Key,
        visibility: Visibility,
    ) -> DataResult<Option<_Stored>> {
        let mut rows = db
            .fetch(&OperationsStore::select_key(key, visibility)?)
            .await?;
        rows.pop().map(OperationsStore::decode).transpose()
    }
    /// Load a bounded native page, with explicit eager paths and optional exact base count.
    pub async fn search(db: &mut DataSession<'_>, query: Query) -> DataResult<Page<View3>> {
        Self::search_with_limits(db, query, Visibility::Active, RelationLimits::default()).await
    }
    /// Trusted maintenance/limited search; budgets can only be lowered, never widened.
    pub async fn search_with_limits(
        db: &mut DataSession<'_>,
        query: Query,
        visibility: Visibility,
        limits: RelationLimits,
    ) -> DataResult<Page<View3>> {
        let page = stackpress_framework_sql::stores::search(
            db,
            &super::schema::catalog(),
            &SPEC,
            ReadQuery::from_typed(&query)?,
            Exposure::Trusted,
            visibility,
            limits,
        )
        .await?;
        let rows = page
            .rows
            .into_iter()
            .map(|row| serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error))
            .collect::<DataResult<Vec<View3>>>()?;
        Ok(Page {
            rows,
            total: page.total,
            skip: page.skip,
            take: page.take,
        })
    }
    /// Open one owned SQLite read transaction for counts/includes, committing exactly once.
    pub async fn search_atomic(
        engine: &stackpress_sql::Engine,
        query: Query,
    ) -> DataResult<Page<View3>> {
        let mut transaction = engine.begin().await?;
        let outcome = Self::search(&mut DataSession::transaction(&mut transaction), query).await;
        stackpress_framework_sql::finish(transaction, outcome).await
    }
    /// Public query allowlist plus recursive public projection; authorization stays app-owned.
    pub async fn search_public(
        db: &mut DataSession<'_>,
        query: crate::generated::contracts::public::OperationsQuery,
    ) -> DataResult<Page<PublicView3>> {
        query
            .validate()
            .map_err(stackpress_framework_sql::stores::validation)?;
        let page = stackpress_framework_sql::stores::search(
            db,
            &super::schema::catalog(),
            &SPEC,
            ReadQuery::from_typed(&query)?,
            Exposure::Public,
            Visibility::Active,
            RelationLimits::default(),
        )
        .await?;
        let rows = page
            .rows
            .into_iter()
            .map(|row| {
                let row: View3 =
                    serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error)?;
                row.project()
            })
            .collect::<DataResult<Vec<_>>>()?;
        Ok(Page {
            rows,
            total: page.total,
            skip: page.skip,
            take: page.take,
        })
    }
    /// First deterministic result, retaining explicit include state.
    pub async fn find(db: &mut DataSession<'_>, mut query: Query) -> DataResult<Option<View3>> {
        query.take = 1;
        Ok(Self::search(db, query).await?.rows.into_iter().next())
    }
    /// The complete bounded page's rows; this is never an unbounded full-table fetch.
    pub async fn find_all(db: &mut DataSession<'_>, query: Query) -> DataResult<Vec<View3>> {
        Ok(Self::search(db, query).await?.rows)
    }
    /// Count matched base identities without relation fanout or paging.
    pub async fn count(db: &mut DataSession<'_>, query: Query) -> DataResult<u64> {
        stackpress_framework_sql::stores::count(
            db,
            &SPEC,
            &ReadQuery::from_typed(&query)?,
            Exposure::Trusted,
            Visibility::Active,
        )
        .await
    }
}
/// Only non-key, non-generated members can be patched.
const MUTABLE_COLUMNS: &[StorageColumn] = &[
    StorageColumn {
        wire: "value",
        column: "value",
        codec: StorageCodec::Text,
        required: true,
        nonempty: false,
    },
    StorageColumn {
        wire: "schemaId",
        column: "schema_id",
        codec: StorageCodec::Text,
        required: false,
        nonempty: false,
    },
];
/// Refresh the generated timestamp from one explicit checked clock before write effects.
fn timestamp(
    context: &stackpress_framework_schema::ValueContext<'_>,
) -> DataResult<Vec<(String, stackpress_sql::SqlValue)>> {
    let _ = context;
    Ok(Vec::new())
}

impl OperationsStore {
    /// Encode one prepared owned value; no host provider/default is evaluated by this plan builder.
    pub fn insert(value: &_Stored) -> DataResult<Insert> {
        Ok(Insert::new("operations")
            .columns(COLUMNS.iter().map(|column| column.column))
            .values(Self::encode(value)?)
            .returning(COLUMNS.iter().map(|column| column.column)))
    }
    /// Encode the exact declared record shape after stored/business validation, without regenerating values.
    pub fn encode(value: &_Stored) -> DataResult<Vec<stackpress_sql::SqlValue>> {
        value
            .validate()
            .map_err(stackpress_framework_sql::stores::validation)?;
        stackpress_framework_sql::encode(value, COLUMNS)
    }
    /// Validate one nonempty patch, refresh its timestamp and scope it to the complete identity.
    pub fn update_plan(
        key: &Key,
        patch: &Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<stackpress_sql::Update> {
        if MUTABLE_COLUMNS.is_empty() {
            return Err(DataError::new("model declares no mutable patch fields")
                .with_kind(DataErrorKind::Unsupported));
        }
        patch
            .validate(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        let mut assignments = stackpress_framework_sql::encode_patch(patch, MUTABLE_COLUMNS)?;
        assignments.extend(timestamp(context)?);
        let mut query =
            stackpress_sql::Update::new("operations").and_where(SPEC.key_predicate(key)?);
        for (column, value) in assignments {
            query = query.set(column, value);
        }
        Ok(query.returning(COLUMNS.iter().map(|column| column.column)))
    }
    /// Native targeted UPSERT over a prepared create value and an independently validated patch.
    pub fn upsert_plan(
        key: &Key,
        value: &_Stored,
        patch: &Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Insert> {
        Self::upsert_on_plan(&ConflictKey::Primary(key.clone()), value, patch, context)
    }
    /// Hard deletion is a separately selected key-scoped operation; foreign keys remain RESTRICT.
    pub fn delete_plan(key: &Key) -> DataResult<stackpress_sql::Delete> {
        Ok(stackpress_sql::Delete::new("operations")
            .and_where(SPEC.key_predicate(key)?)
            .returning(COLUMNS.iter().map(|column| column.column)))
    }
}
impl Operationss {
    /// Prepare using the default context; explicit host values use create_with.
    pub async fn create(db: &mut DataSession<'_>, input: _Create) -> DataResult<_Stored> {
        Self::create_with(
            db,
            input,
            &stackpress_framework_schema::ValueContext::default(),
        )
        .await
    }
    /// Prepare defaults/providers once before submitting the write once.
    pub async fn create_with(
        db: &mut DataSession<'_>,
        input: _Create,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(db.write(&OperationsStore::insert(&value)?).await?)
    }
    /// Targeted no-change is an explicit AlreadyExists result, never a fabricated row.
    pub async fn create_if_absent(
        db: &mut DataSession<'_>,
        input: _Create,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<stackpress_framework_sql::InsertOutcome<_Stored>> {
        db.require_returning()?;
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        let query = OperationsStore::insert(&value)?.on_conflict(
            stackpress_sql::InsertConflict::DoNothing {
                target: SPEC
                    .key
                    .iter()
                    .map(|name| SPEC.column(name).map(|column| column.column.into()))
                    .collect::<DataResult<Vec<_>>>()?,
            },
        );
        let rows = db.write(&query).await?;
        if rows.is_empty() {
            return Ok(stackpress_framework_sql::InsertOutcome::AlreadyExists);
        }
        Ok(stackpress_framework_sql::InsertOutcome::Inserted(
            Self::one_returned(rows)?,
        ))
    }
    /// Last-writer-wins convenience for the original simple key shape.
    pub async fn update(
        db: &mut DataSession<'_>,
        key: &str,
        patch: Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Option<_Stored>> {
        Self::update_key(db, &Key { id: key.into() }, patch, context).await
    }
    /// Last-writer-wins complete-key update, with no optimistic-version or retry behavior.
    pub async fn update_key(
        db: &mut DataSession<'_>,
        key: &Key,
        patch: Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        let rows = db
            .write(&OperationsStore::update_plan(key, &patch, context)?)
            .await?;
        Self::optional_returned(rows)
    }
    /// Convenience targeted UPSERT using the original simple key shape.
    pub async fn upsert(
        db: &mut DataSession<'_>,
        key: &str,
        input: _Create,
        patch: Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<_Stored> {
        Self::upsert_key(db, &Key { id: key.into() }, input, patch, context).await
    }
    /// Atomic native targeted UPSERT, never a check-then-create sequence.
    pub async fn upsert_key(
        db: &mut DataSession<'_>,
        key: &Key,
        input: _Create,
        patch: Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(
            db.write(&OperationsStore::upsert_plan(key, &value, &patch, context)?)
                .await?,
        )
    }
    /// Explicit hard delete, independently selected from soft remove.
    pub async fn delete(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<_Stored>> {
        db.require_returning()?;
        Self::optional_returned(db.write(&OperationsStore::delete_plan(key)?).await?)
    }
    /// Decode exactly one authoritative acknowledged row; never infer a write branch from row counts.
    fn one_returned(mut rows: Vec<Row>) -> DataResult<_Stored> {
        if rows.len() != 1 {
            return Err(DataError::new("write returned an invalid row count")
                .with_kind(DataErrorKind::Decode));
        }
        OperationsStore::decode(rows.remove(0))
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
    Upsert {
        key: Key,
        input: _Create,
        patch: Patch,
    },
    /// Explicit native nonnull unique target, validated before any batch write.
    UpsertOn {
        target: ConflictKey,
        input: _Create,
        patch: Patch,
    },
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
impl Operationss {
    /// Validate/prepare the entire bounded batch before its first write, retaining input order.
    /// A transaction session is required; callers own success completion and rollback on error.
    pub async fn batch(
        db: &mut DataSession<'_>,
        inputs: Vec<BatchInput>,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Vec<BatchOutcome>> {
        db.require_transaction()?;
        db.require_returning()?;
        if inputs.is_empty() || inputs.len() > 100 {
            return Err(DataError::new("batch item limit exceeded"));
        }
        stackpress_framework_schema::to_bounded_value(&inputs)
            .map_err(stackpress_framework_sql::stores::validation)?;
        let mut plans = Vec::new();
        let mut bytes = 0usize;
        for input in inputs {
            let (mode, query) = match input {
                BatchInput::Create { input } => {
                    let value = input
                        .prepare_with(context)
                        .map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (
                        BatchMode::Create,
                        Prepared::Insert(OperationsStore::insert(&value)?),
                    )
                }
                BatchInput::Update { key, patch } => (
                    BatchMode::Update,
                    Prepared::Update(OperationsStore::update_plan(&key, &patch, context)?),
                ),
                BatchInput::Upsert { key, input, patch } => {
                    let value = input
                        .prepare_with(context)
                        .map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (
                        BatchMode::Upsert,
                        Prepared::Insert(OperationsStore::upsert_plan(
                            &key, &value, &patch, context,
                        )?),
                    )
                }
                BatchInput::UpsertOn {
                    target,
                    input,
                    patch,
                } => {
                    let value = input
                        .prepare_with(context)
                        .map_err(stackpress_framework_sql::stores::preparation_error)?;
                    (
                        BatchMode::Upsert,
                        Prepared::Insert(OperationsStore::upsert_on_plan(
                            &target, &value, &patch, context,
                        )?),
                    )
                }
            };
            // Pure rendering validates every later plan before an earlier write can take effect.
            let rendered = stackpress_sql::Render::render(&query, db.dialect())
                .map_err(stackpress_framework_sql::write_plan_error)?;
            rendered
                .validate(db.limits())
                .map_err(stackpress_framework_sql::write_plan_error)?;
            bytes = bytes.saturating_add(rendered.sql().len()).saturating_add(
                rendered
                    .binds()
                    .iter()
                    .map(stackpress_sql::SqlValue::retained_bytes)
                    .sum::<usize>(),
            );
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
    pub async fn batch_atomic(
        engine: &stackpress_sql::Engine,
        inputs: Vec<BatchInput>,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Vec<BatchOutcome>> {
        let mut transaction = engine.begin().await?;
        let outcome = Self::batch(
            &mut DataSession::transaction(&mut transaction),
            inputs,
            context,
        )
        .await;
        stackpress_framework_sql::finish(transaction, outcome).await
    }
}
/// Exhaustive acknowledgement interpretation; no numeric fallback may invent a result mode.
enum BatchMode {
    Create,
    Update,
    Upsert,
}
/// Finite internal plan family for whole-batch preflight, without an executor capture.
enum Prepared {
    Insert(Insert),
    Update(stackpress_sql::Update),
}
impl stackpress_sql::Render for Prepared {
    /// Delegate pure rendering without submitting any database work.
    fn render(
        &self,
        dialect: stackpress_sql::DialectId,
    ) -> stackpress_sql::Result<stackpress_sql::Query> {
        match self {
            Self::Insert(query) => stackpress_sql::Render::render(query, dialect),
            Self::Update(query) => stackpress_sql::Render::render(query, dialect),
        }
    }
}

/// Owned app-default field selection, distinct from full rows and full eager views.
/// The private payload can only be built from a recursively projected checked View3.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct Selected(serde_json::Value);
impl Selected {
    /// Borrow the already validated selected object; no loading or SQL occurs.
    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }
}
impl View3 {
    /// Apply approved @query paths after recursive public projection, never as an exposure grant.
    pub fn select_default(self) -> DataResult<Selected> {
        let public = self.project()?;
        Ok(Selected(stackpress_framework_sql::projection::select(
            &public,
            &["id", "row.value"],
            false,
        )?))
    }
}
impl Operationss {
    /// Load the fixed app-default includes/projection using a complete identity.
    /// Explicit relation defaults require the caller's transaction, like ordinary eager search.
    pub async fn detail(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<Selected>> {
        let mut query = Query {
            take: 1,
            includes: vec!["row".into()],
            ..Default::default()
        };
        query
            .filters
            .push(Filter::Id(stackpress_framework_schema::Predicate::Eq(
                key.id.clone(),
            )));
        let page = stackpress_framework_sql::stores::search(
            db,
            &super::schema::catalog(),
            &SPEC,
            ReadQuery::from_typed(&query)?,
            Exposure::Public,
            Visibility::Active,
            RelationLimits::default(),
        )
        .await?;
        let row = page
            .rows
            .into_iter()
            .next()
            .map(|row| {
                serde_json::from_value::<View3>(row).map_err(stackpress_framework_sql::codec_error)
            })
            .transpose()?;
        row.map(|row| row.select_default()).transpose()
    }
}
/// Explicit native conflict target. Public upsert handlers continue to require the full primary key.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "target", content = "key", deny_unknown_fields)]
pub enum ConflictKey {
    /// Complete ordered primary identity.
    #[serde(rename = "primary")]
    Primary(Key),
}
impl OperationsStore {
    /// Resolve only declared unique/primary targets and verify the prepared insert agrees before IO.
    fn conflict_target(target: &ConflictKey, value: &_Stored) -> DataResult<Vec<String>> {
        match target {
            ConflictKey::Primary(key) => {
                if !SPEC.agrees_with_key(value, key)? {
                    return Err(DataError::new("upsert key disagrees with create identity"));
                }
                SPEC.key
                    .iter()
                    .map(|name| SPEC.column(name).map(|column| column.column.into()))
                    .collect()
            }
        }
    }
    /// Build one native explicit-target UPSERT while keeping all primary/generated values immutable.
    pub fn upsert_on_plan(
        target: &ConflictKey,
        value: &_Stored,
        patch: &Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<Insert> {
        let target = Self::conflict_target(target, value)?;
        let key = Key {
            id: value.id.clone(),
        };
        let update = Self::update_plan(&key, patch, context)?;
        Ok(
            Self::insert(value)?.on_conflict(stackpress_sql::InsertConflict::DoUpdate {
                target,
                assignments: update.assignments().to_vec(),
            }),
        )
    }
}
impl Operationss {
    /// Native unique/primary UPSERT; RETURNING supplies the existing identity on a unique conflict.
    pub async fn upsert_on(
        db: &mut DataSession<'_>,
        target: ConflictKey,
        input: _Create,
        patch: Patch,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> DataResult<_Stored> {
        db.require_returning()?;
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        Self::one_returned(
            db.write(&OperationsStore::upsert_on_plan(
                &target, &value, &patch, context,
            )?)
            .await?,
        )
    }
}
