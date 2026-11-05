//! Explicit generated maintenance and accepted G48 backend setup/purge semantics.
use super::{ActionError, NativeActions, SourceResult as Result};
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_sql::{Create, DialectId, Sql, TrustedSql};
impl<S: SourceSchema<Value = Value, Record = Record>> NativeActions<S> {
    /// Explicit table installation; MySQL accepts partial DDL progress under G48.
    pub async fn install(&self) -> Result<()> {
        crate::setup::execute_plan(
            &self.engine,
            &self.store.create().render(self.engine.dialect())?,
        )
        .await?;
        Ok(())
    }
    /// Explicit alteration; MySQL runs in order without atomic rollback (G48).
    pub async fn upgrade(&self, to: &Create) -> Result<()> {
        if let Some(plan) = self
            .store
            .create()
            .definition()?
            .diff(&to.definition()?)?
            .render(self.engine.dialect())?
        {
            crate::setup::execute_plan(&self.engine, &plan).await?;
        }
        Ok(())
    }
    /// Explicit source purge; SQLite's DELETE truncation already applies declared FK cascade.
    pub async fn purge(&self, cascade: bool) -> Result<()> {
        if self.engine.dialect() == stackpress_sql::DialectId::Mysql && cascade {
            return self.purge_mysql_dependents().await;
        }
        let command = stackpress_sql::TableCommand::Truncate {
            table: self.store.table().into(),
            cascade: cascade && self.engine.dialect() == stackpress_sql::DialectId::Postgres,
        };
        self.engine.execute(&command).await?;
        Ok(())
    }
    /// Explicit physical uninstall; no automatic related-table teardown is added.
    pub async fn uninstall(&self) -> Result<()> {
        self.engine
            .execute(&self.engine.drop_table(self.store.table()))
            .await?;
        Ok(())
    }

    /// G48 clears real foreign-key dependents before parents on MySQL; no connection-global changes.
    async fn purge_mysql_dependents(&self) -> Result<()> {
        let catalog = self
            .store
            .catalog
            .models
            .iter()
            .map(|model| self.store.table_of(model).to_owned())
            .collect::<Vec<_>>();
        // Read actual incoming edges once. Uninstalled catalog models have no edges;
        // outside children remain visible here but can never enter the deletion set.
        let rows = self.engine.fetch(&mysql_foreign_keys(&catalog)).await?;
        let edges = rows
            .iter()
            .map(|row| Ok((row.try_get("child")?, row.try_get("parent")?)))
            .collect::<stackpress_sql::Result<Vec<(String, String)>>>()?;
        let ordered = mysql_purge_order(self.store.table(), &catalog, &edges)?;
        if ordered.is_empty() {
            // No incoming FK needs the G48 DELETE fallback. This also preserves
            // MySQL's native 1146 when the requested target itself is missing.
            self.engine
                .execute(&stackpress_sql::TableCommand::Truncate {
                    table: self.store.table().into(),
                    cascade: false,
                })
                .await?;
            return Ok(());
        }
        // Execute the finite child-first sequence without suppressing driver errors.
        // An outside-catalog FK is deliberately left for MySQL to enforce unchanged.
        for table in ordered {
            self.engine
                .execute(&stackpress_sql::Delete::new(table).all_rows())
                .await?;
        }
        Ok(())
    }
}

/// Inspect current-database incoming FKs; table names are values, never SQL identifiers.
fn mysql_foreign_keys(catalog: &[String]) -> Sql {
    // Only placeholder count is assembled. Both ends stay in DATABASE(), and the
    // parent-name binds limit discovery to the selected compiled catalog.
    let placeholders = vec!["?"; catalog.len()].join(", ");
    let sql = format!(
        "SELECT DISTINCT TABLE_NAME AS child, REFERENCED_TABLE_NAME AS parent \
         FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = DATABASE() AND REFERENCED_TABLE_SCHEMA = DATABASE() \
         AND REFERENCED_TABLE_NAME IN ({placeholders}) \
         ORDER BY TABLE_NAME, REFERENCED_TABLE_NAME"
    );
    catalog.iter().fold(
        Sql::trusted(DialectId::Mysql, TrustedSql::audited(sql)),
        |query, table| query.bind(table.clone()),
    )
}

/// Return child-first DELETE tables, or an empty sequence for plain TRUNCATE.
///
/// Edges are actual child → parent table names. Traversal and deletion stay inside
/// the catalog; an outside child is left to the backend's ordinary FK enforcement.
fn mysql_purge_order(
    target: &str,
    catalog: &[String],
    edges: &[(String, String)],
) -> Result<Vec<String>> {
    if !edges.iter().any(|(_, parent)| parent == target) {
        return Ok(Vec::new());
    }
    // Reach only installed dependents with actual edges, rather than assume that
    // every model declared in a closed generated catalog has been installed.
    let mut reachable = vec![target.to_owned()];
    let mut index = 0;
    while index < reachable.len() {
        for (child, parent) in edges {
            if parent == &reachable[index] && catalog.contains(child) && !reachable.contains(child)
            {
                reachable.push(child.clone());
            }
        }
        index += 1;
    }
    // Catalog order breaks ties between independent leaves. No deletion is issued
    // until the entire ordering succeeds, so cycles cannot cause partial purges.
    let mut remaining = catalog
        .iter()
        .filter(|table| reachable.contains(table))
        .cloned()
        .collect::<Vec<_>>();
    let mut ordered = Vec::new();
    while !remaining.is_empty() {
        let leaves = remaining
            .iter()
            .filter(|parent| {
                !edges
                    .iter()
                    .any(|(child, referenced)| referenced == *parent && remaining.contains(child))
            })
            .cloned()
            .collect::<Vec<_>>();
        if leaves.is_empty() {
            return Err(ActionError::new(
                500,
                "source purge relationship cycle",
                Record::new(),
            ));
        }
        remaining.retain(|table| !leaves.contains(table));
        ordered.extend(leaves);
    }
    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::{mysql_foreign_keys, mysql_purge_order};
    use stackpress_sql::{DialectId, Render, SqlValue};

    /// Keep graph fixtures small and explicit without depending on a live catalog.
    fn order(catalog: &[&str], edges: &[(&str, &str)]) -> super::Result<Vec<String>> {
        mysql_purge_order(
            "parent",
            &catalog
                .iter()
                .map(|table| (*table).into())
                .collect::<Vec<_>>(),
            &edges
                .iter()
                .map(|(child, parent)| ((*child).into(), (*parent).into()))
                .collect::<Vec<_>>(),
        )
    }

    /// Unreferenced targets retain TRUNCATE rather than acquire DELETE semantics.
    #[test]
    fn mysql_purge_without_dependents_truncates() {
        assert!(order(&["parent"], &[]).unwrap().is_empty());
    }

    /// A transitive chain deletes deepest children first.
    #[test]
    fn mysql_purge_chain_is_child_first() {
        assert_eq!(
            order(
                &["parent", "child", "grandchild"],
                &[("child", "parent"), ("grandchild", "child")]
            )
            .unwrap(),
            ["grandchild", "child", "parent"]
        );
    }

    /// A shared descendant is deleted once; independent ties retain catalog order.
    #[test]
    fn mysql_purge_diamond_deduplicates() {
        assert_eq!(
            order(
                &["parent", "right", "left", "leaf"],
                &[
                    ("left", "parent"),
                    ("right", "parent"),
                    ("leaf", "left"),
                    ("leaf", "right")
                ]
            )
            .unwrap(),
            ["leaf", "right", "left", "parent"]
        );
    }

    /// Cycles fail before any DELETE can be issued.
    #[test]
    fn mysql_purge_cycle_retains_source_error() {
        let error = order(
            &["parent", "child"],
            &[("child", "parent"), ("parent", "child")],
        )
        .unwrap_err();
        assert_eq!(error.code, 500);
        assert_eq!(error.error, "source purge relationship cycle");
        assert!(error.errors.is_empty());
    }

    /// Declaring an uninstalled child cannot invent an incoming database edge.
    #[test]
    fn mysql_purge_uninstalled_declared_child_is_skipped() {
        assert!(order(&["parent", "uninstalled"], &[]).unwrap().is_empty());
        assert_eq!(
            order(&["parent", "child", "uninstalled"], &[("child", "parent")]).unwrap(),
            ["child", "parent"]
        );
    }

    /// Outside children, and catalog descendants reached only through them, stay untouched.
    #[test]
    fn mysql_purge_outside_catalog_child_is_excluded() {
        assert_eq!(
            order(
                &["parent", "isolated"],
                &[("outside", "parent"), ("isolated", "outside")]
            )
            .unwrap(),
            ["parent"]
        );
    }

    /// One read-only query uses DATABASE() scopes and bound catalog names, including hostile text.
    #[test]
    fn mysql_purge_discovery_binds_catalog_names() {
        let names = vec!["parent".into(), "hostile') OR 1=1 --".into()];
        let query = mysql_foreign_keys(&names).render(DialectId::Mysql).unwrap();
        assert!(query.sql().starts_with("SELECT DISTINCT "));
        assert_eq!(query.sql().matches("DATABASE()").count(), 2);
        assert!(query.sql().contains("REFERENCED_TABLE_NAME IN (?, ?)"));
        assert!(!query.sql().contains(&names[1]));
        assert_eq!(
            query.binds(),
            &[
                SqlValue::Text(names[0].clone()),
                SqlValue::Text(names[1].clone())
            ]
        );
    }
}
