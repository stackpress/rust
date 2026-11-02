/// Native scalar/list filters; direct store calls are trusted domain APIs.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum Filter {
[%#@:filters field%]    /// Typed [%field.name_literal%] predicate.
    #[serde(rename = [%field.name_literal%])]
    [%field.variant%](stackpress_framework_schema::Predicate<[%field.ty%]>),
[%/@:filters%]}
/// Native declared ordering fields; every page appends complete primary-key ties.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SortField {
[%#@:sort_fields field%]    /// Order by [%field.name_literal%].
    #[serde(rename = [%field.name_literal%])]
    [%field.variant%],
[%/@:sort_fields%]}
/// Finite native query with the same page/budget semantics as the pure public contract.
pub type Query = stackpress_framework_sql::stores::Query<Filter, SortField>;
[%#?:legacy_key%]impl [%model%]Store {
    /// Preserve the original detached simple-key lookup API, without capturing an executor.
    pub fn select(key: [%key_type%]) -> Select {
        let query = Select::new(COLUMNS.iter().map(|column| column.column))
            .from([%table_literal%]).where_eq([%key_column_literal%], key);
[%#?:has_active%]        let query = query.where_eq([%active_column_literal%], true);
[%/?:has_active%]        query.limit(1)
    }
}
[%/?:legacy_key%]
impl [%model%]Store {
    /// Build a complete-identity ordinary read, including every compound key member.
    pub fn select_key(key: &Key, visibility: Visibility) -> DataResult<Select> {
        SPEC.select_key(key, visibility)
    }
    /// Build a checked deterministic page without database work.
    pub fn search_plan(query: &Query, dialect: stackpress_sql::DialectId) -> DataResult<Select> {
        stackpress_framework_sql::stores::select(&SPEC, &ReadQuery::from_typed(query)?, dialect, Exposure::Trusted, Visibility::Active)
    }
}
/// Stateless executing functions; callers always select the exact borrowed executor.
pub struct [%model%]s;
impl [%model%]s {
    /// Ordinary base lookup; absence is distinct from storage/validation failure.
    pub async fn get(db: &mut DataSession<'_>, key: [%key_type%]) -> DataResult<Option<_Stored>> {
        Self::get_key(db, [%key_value%]).await
    }
    /// Ordinary lookup using the full typed identity, including non-text and compound keys.
    pub async fn get_key(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<_Stored>> {
        let mut rows = db.fetch(&[%model%]Store::select_key(key, Visibility::Active)?).await?;
        rows.pop().map([%model%]Store::decode).transpose()
    }
    /// Explicit maintenance lookup; inactive/all selection never comes from public request data.
    pub async fn get_maintenance(db: &mut DataSession<'_>, key: &Key, visibility: Visibility) -> DataResult<Option<_Stored>> {
        let mut rows = db.fetch(&[%model%]Store::select_key(key, visibility)?).await?;
        rows.pop().map([%model%]Store::decode).transpose()
    }
    /// Load a bounded native page, with explicit eager paths and optional exact base count.
    pub async fn search(db: &mut DataSession<'_>, query: Query) -> DataResult<Page<View3>> {
        Self::search_with_limits(db, query, Visibility::Active, RelationLimits::default()).await
    }
    /// Trusted maintenance/limited search; budgets can only be lowered, never widened.
    pub async fn search_with_limits(db: &mut DataSession<'_>, query: Query, visibility: Visibility, limits: RelationLimits) -> DataResult<Page<View3>> {
        let page = stackpress_framework_sql::stores::search(
            db, &super::schema::catalog(), &SPEC, ReadQuery::from_typed(&query)?, Exposure::Trusted, visibility, limits
        ).await?;
        let rows = page.rows.into_iter().map(|row| serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error)).collect::<DataResult<Vec<View3>>>()?;
        Ok(Page { rows, total: page.total, skip: page.skip, take: page.take })
    }
    /// Own one read transaction; snapshot-dependent shapes remain explicitly dialect-gated.
    pub async fn search_atomic(engine: &stackpress_sql::Engine, query: Query) -> DataResult<Page<View3>> {
        // A known dialect refusal must precede BEGIN, even when native acquisition would fail.
        stackpress_framework_sql::stores::preflight_atomic_search(&super::schema::catalog(), &SPEC, &ReadQuery::from_typed(&query)?, engine.dialect())?;
        let mut transaction = engine.begin().await?;
        let outcome = Self::search(&mut DataSession::transaction(&mut transaction), query).await;
        stackpress_framework_sql::finish(transaction, outcome).await
    }
    /// Codegen support: validate the public query, dialect and selected executor limits before BEGIN.
    #[doc(hidden)]
    pub fn preflight_search_public(engine: &stackpress_sql::Engine, query: &crate::generated::contracts::public::[%model%]Query) -> DataResult<()> {
        // Keep the public DTO checks as well as the runtime public-exposure catalog checks.
        query.validate().map_err(stackpress_framework_sql::stores::validation)?;
        stackpress_framework_sql::stores::preflight_public_search(&super::schema::catalog(), &SPEC, &ReadQuery::from_typed(query)?, engine.dialect(), engine.limits())
    }
    /// Public query allowlist plus recursive public projection; authorization stays app-owned.
    pub async fn search_public(db: &mut DataSession<'_>, query: crate::generated::contracts::public::[%model%]Query) -> DataResult<Page<PublicView3>> {
        query.validate().map_err(stackpress_framework_sql::stores::validation)?;
        let page = stackpress_framework_sql::stores::search(
            db, &super::schema::catalog(), &SPEC, ReadQuery::from_typed(&query)?, Exposure::Public, Visibility::Active, RelationLimits::default()
        ).await?;
        let rows = page.rows.into_iter().map(|row| {
            let row: View3 = serde_json::from_value(row).map_err(stackpress_framework_sql::codec_error)?;
            row.project()
        }).collect::<DataResult<Vec<_>>>()?;
        Ok(Page { rows, total: page.total, skip: page.skip, take: page.take })
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
        stackpress_framework_sql::stores::count(db, &SPEC, &ReadQuery::from_typed(&query)?, Exposure::Trusted, Visibility::Active).await
    }
}
