//! Explicit generated maintenance and accepted G48 backend setup/purge semantics.
use super::{ActionError, NativeActions, SourceResult as Result};
use stackpress_framework_schema::{
    ValueType,
    source::{Record, SourceSchema, Value},
};
use stackpress_sql::Create;
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
        let mut pending = vec![self.store.model.name.clone()];
        let mut tables = Vec::new();
        while let Some(name) = pending.pop() {
            if tables.contains(&name) {
                continue;
            }
            tables.push(name.clone());
            for model in &self.store.catalog.models {
                if model
                    .fields
                    .iter()
                    .any(|f| f.relation.is_some() && f.value_type == ValueType::Model(name.clone()))
                {
                    pending.push(model.name.clone())
                }
            }
        }
        // Choose child-first leaves from the same generated owning relationships.
        let definitions = self
            .store
            .catalog
            .models
            .iter()
            .filter(|m| tables.contains(&m.name))
            .collect::<Vec<_>>();
        let mut remaining = definitions;
        let mut ordered = Vec::new();
        while !remaining.is_empty() {
            let before = remaining.len();
            let leaves = remaining
                .iter()
                .filter(|m| {
                    !remaining.iter().any(|other| {
                        other.fields.iter().any(|f| {
                            f.relation.is_some() && f.value_type == ValueType::Model(m.name.clone())
                        })
                    })
                })
                .map(|m| m.name.clone())
                .collect::<Vec<_>>();
            for name in leaves {
                let i = remaining
                    .iter()
                    .position(|m| m.name == name)
                    .expect("selected leaf");
                ordered.push(remaining.remove(i));
            }
            if remaining.len() == before {
                return Err(ActionError::new(
                    500,
                    "source purge relationship cycle",
                    Record::new(),
                ));
            }
        }
        for model in ordered {
            self.engine
                .execute(&stackpress_sql::Delete::new(self.store.table_of(model)).all_rows())
                .await?;
        }
        Ok(())
    }
}
