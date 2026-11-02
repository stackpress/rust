
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
            &[[%#@:selected path%][%#!:@first%], [%/!:@first%][%path%][%/@:selected%]],
            [%fallback%],
        )?))
    }
}
impl [%model%]s {
    /// Build the exact fixed detail query for both pure preflight and executing public search.
    fn detail_query(key: &Key) -> Query {
        let mut query = Query {
            take: 1,
            includes: vec![[%#@:includes path%][%#!:@first%], [%/!:@first%][%path%].into()[%/@:includes%]],
            ..Default::default()
        };
[%#@:keys key%]        query.filters.push(Filter::[%key.variant%](stackpress_framework_schema::Predicate::Eq(key.[%key.member%][%#?:key.clone%].clone()[%/?:key.clone%])));
[%/@:keys%]        query
    }
    /// Codegen support: check the exact public detail query and executor budgets before BEGIN.
    #[doc(hidden)]
    pub fn preflight_detail(engine: &stackpress_sql::Engine, key: &Key) -> DataResult<()> {
        let query = Self::detail_query(key);
        stackpress_framework_sql::stores::preflight_public_search(
            &super::schema::catalog(), &SPEC, &ReadQuery::from_typed(&query)?, engine.dialect(), engine.limits()
        )
    }
    /// Load the fixed app-default includes/projection using a complete identity.
    /// Explicit relation defaults require the caller's transaction, like ordinary eager search.
    pub async fn detail(db: &mut DataSession<'_>, key: &Key) -> DataResult<Option<Selected>> {
        let query = Self::detail_query(key);
        let page = stackpress_framework_sql::stores::search(
            db, &super::schema::catalog(), &SPEC, ReadQuery::from_typed(&query)?,
            Exposure::Public, Visibility::Active, RelationLimits::default()
        ).await?;
        let row = page.rows.into_iter().next().map(|row|
            serde_json::from_value::<View3>(row).map_err(stackpress_framework_sql::codec_error)
        ).transpose()?;
        row.map(|row| row.select_default()).transpose()
    }
}
