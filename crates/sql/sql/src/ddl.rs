//! Detached CREATE/ALTER descriptions and wholly validated ordered native DDL plans.
//!
//! Plans carry no executor. Engine explicitly executes supported plans on one transaction;
//! inspecting or rendering a plan never installs, introspects or migrates a database.

use crate::expr::{RenderContext, join_sql};
use crate::identifier::{invalid, quote, unsupported};
use crate::schema::{column_list, find_field, normalize_indexes};
use crate::{
    DefaultValue, DialectId, Field, ForeignKey, Index, Query, QueryLimits, Result, SqlValue,
};

/// A builder-generated batch; every statement validates before any caller can execute it.
#[derive(Clone, Debug)]
pub struct QueryPlan {
    /// Validated statements in required execution order, without arbitrary batch text.
    queries: Vec<Query>,
}

impl QueryPlan {
    /// Inspect statement order and each separate bind list.
    pub fn queries(&self) -> &[Query] {
        &self.queries
    }

    /// Validate every statement against executor-selected budgets before effects.
    pub fn validate(&self, dialect: DialectId, limits: QueryLimits) -> Result<()> {
        if self.queries.is_empty() {
            return Err(invalid("empty SQL DDL plan"));
        }
        for query in &self.queries {
            if query.dialect() != dialect {
                return Err(unsupported("SQL plan dialect mismatch"));
            }
            query.validate(limits)?;
        }
        Ok(())
    }

    /// Freeze an entirely rendered builder plan under the default budgets.
    pub(crate) fn new(queries: Vec<Query>) -> Result<Self> {
        let plan = Self { queries };
        let dialect = plan
            .queries
            .first()
            .ok_or_else(|| invalid("empty SQL DDL plan"))?
            .dialect();
        plan.validate(dialect, QueryLimits::default())?;
        Ok(plan)
    }
}

/// Inspectable CREATE data, with normalized unique/index precedence after build.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateData {
    /// One literal table name; dots are not qualification syntax.
    pub table: String,
    /// Ordered field definitions.
    pub fields: Vec<Field>,
    /// Ordered primary columns; empty means no explicit primary key.
    pub primary: Vec<String>,
    /// Ordered named index/unique definitions.
    pub indexes: Vec<Index>,
    /// Ordered named compound foreign keys.
    pub foreign_keys: Vec<ForeignKey>,
    /// Apply IF NOT EXISTS to both table and indexes; this never verifies existing shape.
    pub if_not_exists: bool,
}

/// Owned table creation builder; fields/keys are ordinary detached data.
#[derive(Clone, Debug, PartialEq)]
pub struct Create {
    data: CreateData,
}

impl Create {
    /// Name a table without creating or opening anything.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            data: CreateData {
                table: table.into(),
                fields: Vec::new(),
                primary: Vec::new(),
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                if_not_exists: false,
            },
        }
    }

    /// Append a typed field in storage order.
    pub fn add_field(mut self, field: Field) -> Self {
        self.data.fields.push(field);
        self
    }

    /// Replace the explicitly ordered primary-key columns.
    pub fn primary_key(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.primary = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Append a regular/unique index; same-name incompatible definitions error in build.
    pub fn add_index(mut self, index: Index) -> Self {
        self.data.indexes.push(index);
        self
    }

    /// Append a named compound foreign key.
    pub fn add_foreign_key(mut self, foreign_key: ForeignKey) -> Self {
        self.data.foreign_keys.push(foreign_key);
        self
    }

    /// Explicitly request idempotent native table and index creation.
    pub fn if_not_exists(mut self) -> Self {
        self.data.if_not_exists = true;
        self
    }

    /// Return a normalized independent snapshot with validated identifiers/relationships.
    ///
    /// # Errors
    /// Rejects empty/duplicate fields, invalid defaults, absent key fields and conflicting indexes.
    pub fn build(&self) -> Result<CreateData> {
        quote(&self.data.table)?;
        validate_fields(&self.data.fields)?;
        let mut data = self.data.clone();
        data.indexes = normalize_indexes(&data.indexes)?;
        if !data.primary.is_empty() {
            validate_local_columns(&data.primary, &data.fields)?;
        }
        for index in &data.indexes {
            validate_local_columns(&index.columns, &data.fields)?;
        }
        if data.foreign_keys.len() > 1000 {
            return Err(invalid("too many SQL foreign keys"));
        }
        let mut names = std::collections::HashSet::new();
        for foreign_key in &data.foreign_keys {
            if !names.insert(foreign_key.name.to_ascii_lowercase()) {
                return Err(invalid("duplicate SQL foreign constraint"));
            }
            // A declared primary key carries NOT NULL semantics independently of the
            // field's default nullable flag; SET NULL cannot fulfill that definition.
            if [foreign_key.on_delete, foreign_key.on_update]
                .contains(&crate::ReferentialAction::SetNull)
                && foreign_key.columns.iter().any(|column| {
                    data.primary
                        .iter()
                        .any(|primary| primary.eq_ignore_ascii_case(column))
                })
            {
                return Err(invalid("SET NULL cannot target primary-key columns"));
            }
            // This pure renderer validates local fields, arity and SET NULL/DEFAULT constraints.
            foreign_key.sqlite(&data.fields)?;
        }
        Ok(data)
    }

    /// Render a complete native table/index batch without executing effects.
    ///
    /// # Errors
    /// Rejects unsupported annotations/types or invalid auto-increment/key combinations.
    pub fn render(&self, dialect: DialectId) -> Result<QueryPlan> {
        RenderContext::new(dialect)?;
        let data = self.build()?;
        if dialect != DialectId::Sqlite {
            return crate::network_ddl::create(&data, dialect);
        }
        let primary =
            if !data.primary.is_empty() && !data.fields.iter().any(|field| field.auto_increment) {
                Some(Ok(format!("PRIMARY KEY ({})", column_list(&data.primary)?)))
            } else {
                None
            };
        // Join incrementally; many individually valid defaults cannot amplify retained SQL.
        let parts = join_sql(
            data.fields
                .iter()
                .map(|field| field.sqlite(&data.primary))
                .chain(primary)
                .chain(
                    data.foreign_keys
                        .iter()
                        .map(|foreign| foreign.sqlite(&data.fields)),
                ),
            ", ",
        )?;
        let table = format!(
            "CREATE TABLE {}{} ({})",
            if data.if_not_exists {
                "IF NOT EXISTS "
            } else {
                ""
            },
            quote(&data.table)?,
            parts
        );
        let mut queries = vec![ddl_query(table)?];
        for index in &data.indexes {
            queries.push(ddl_query(index.sqlite(&data.table, data.if_not_exists)?)?);
        }
        QueryPlan::new(queries)
    }
}

/// A structural alteration, kept visible even when SQLite cannot execute it natively.
#[derive(Clone, Debug, PartialEq)]
pub enum AlterOperation {
    /// Add a new field.
    AddField(Field),
    /// Change a field's semantics (unsupported in native SQLite ALTER).
    ChangeField(Field),
    /// Remove a literal field.
    RemoveField(String),
    /// Explicitly rename a known source to a distinct target.
    RenameField {
        /// Existing field name.
        from: String,
        /// New field name.
        to: String,
    },
    /// Create a regular or unique index.
    AddIndex(Index),
    /// Drop one literal native index.
    RemoveIndex(String),
    /// Add/replace an ordered primary key (unsupported in SQLite).
    AddPrimary(Vec<String>),
    /// Remove a primary key (unsupported in SQLite).
    RemovePrimary,
    /// Add a named foreign constraint (unsupported in SQLite).
    AddForeignKey(ForeignKey),
    /// Remove a named foreign constraint (unsupported in SQLite).
    RemoveForeignKey(String),
}

/// Inspectable normalized alteration data, including explicit source field metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct AlterData {
    /// One literal table name.
    pub table: String,
    /// Known original field definitions, required to reconcile rename semantics.
    pub source_fields: Vec<Field>,
    /// Ordered operations after explicit rename reconciliation and index normalization.
    pub operations: Vec<AlterOperation>,
}

/// Owned ALTER builder; SQLite reconstruction and automatic schema diff are outside M1.
#[derive(Clone, Debug, PartialEq)]
pub struct Alter {
    data: AlterData,
}

impl Alter {
    /// Start an empty alteration, which must gain an operation before build/render.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            data: AlterData {
                table: table.into(),
                source_fields: Vec::new(),
                operations: Vec::new(),
            },
        }
    }

    /// Supply original field metadata explicitly; this never introspects a live database.
    pub fn source_field(mut self, field: Field) -> Self {
        self.data.source_fields.push(field);
        self
    }

    /// Append a typed operation without performing effects.
    pub fn operation(mut self, operation: AlterOperation) -> Self {
        self.data.operations.push(operation);
        self
    }

    /// Add one typed field.
    pub fn add_field(self, field: Field) -> Self {
        self.operation(AlterOperation::AddField(field))
    }

    /// Record a type/nullability/default change; SQLite rejects this at rendering.
    pub fn change_field(self, field: Field) -> Self {
        self.operation(AlterOperation::ChangeField(field))
    }

    /// Remove a literal column, subject to actual native constraints at execution.
    pub fn remove_field(self, name: impl Into<String>) -> Self {
        self.operation(AlterOperation::RemoveField(name.into()))
    }

    /// Rename a known source; pending remove/add halves must either both exist or neither.
    pub fn rename_field(self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.operation(AlterOperation::RenameField {
            from: from.into(),
            to: to.into(),
        })
    }

    /// Append a named index; unique precedence requires identical ordered fields.
    pub fn add_index(self, index: Index) -> Self {
        self.operation(AlterOperation::AddIndex(index))
    }

    /// Remove a literal index name (including unique indexes created by this builder).
    pub fn remove_index(self, name: impl Into<String>) -> Self {
        self.operation(AlterOperation::RemoveIndex(name.into()))
    }

    /// Record an ordered primary-key replacement; SQLite requires deferred reconstruction.
    pub fn primary_key(self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.operation(AlterOperation::AddPrimary(
            columns.into_iter().map(Into::into).collect(),
        ))
    }

    /// Record primary-key removal without claiming native SQLite support.
    pub fn remove_primary_key(self) -> Self {
        self.operation(AlterOperation::RemovePrimary)
    }

    /// Record a named foreign constraint; SQLite requires deferred reconstruction.
    pub fn add_foreign_key(self, foreign: ForeignKey) -> Self {
        self.operation(AlterOperation::AddForeignKey(foreign))
    }

    /// Record named foreign-key removal without discarding requested semantics.
    pub fn remove_foreign_key(self, name: impl Into<String>) -> Self {
        self.operation(AlterOperation::RemoveForeignKey(name.into()))
    }

    /// Return an independent normalized plan, preserving changed semantics after a rename.
    pub fn build(&self) -> Result<AlterData> {
        quote(&self.data.table)?;
        if self.data.operations.is_empty() || self.data.operations.len() > 1000 {
            return Err(invalid("ALTER requires a bounded nonempty operation list"));
        }
        if !self.data.source_fields.is_empty() {
            validate_fields(&self.data.source_fields)?;
        }
        let operations = reconcile(&self.data)?;
        let mut data = self.data.clone();
        data.operations = operations;
        Ok(data)
    }

    /// Validate the entire operation set and return ordered native SQLite statements.
    ///
    /// # Errors
    /// On SQLite, type/nullability/primary/FK changes return Unsupported before a plan is exposed.
    pub fn render(&self, dialect: DialectId) -> Result<QueryPlan> {
        RenderContext::new(dialect)?;
        let data = self.build()?;
        if dialect != DialectId::Sqlite {
            return crate::network_ddl::alter(&data, dialect);
        }
        let table = quote(&data.table)?;
        let mut queries = Vec::new();
        for operation in &data.operations {
            let sql = match operation {
                AlterOperation::AddField(field) => {
                    // Native ADD cannot add primary/auto-increment, timestamp or expression defaults.
                    if field.auto_increment
                        || matches!(
                            field.default,
                            Some(DefaultValue::CurrentTimestamp | DefaultValue::Trusted(_, _))
                        )
                    {
                        return Err(unsupported("SQLite cannot add this field natively"));
                    }
                    if !field.nullable
                        && matches!(
                            field.default,
                            None | Some(DefaultValue::Value(SqlValue::Null))
                        )
                    {
                        return Err(unsupported(
                            "SQLite ADD NOT NULL requires a non-NULL default",
                        ));
                    }
                    format!("ALTER TABLE {table} ADD COLUMN {}", field.sqlite(&[])?)
                }
                AlterOperation::RemoveField(name) => {
                    format!("ALTER TABLE {table} DROP COLUMN {}", quote(name)?)
                }
                AlterOperation::RenameField { from, to } => format!(
                    "ALTER TABLE {table} RENAME COLUMN {} TO {}",
                    quote(from)?,
                    quote(to)?
                ),
                AlterOperation::AddIndex(index) => index.sqlite(&data.table, false)?,
                AlterOperation::RemoveIndex(name) => format!("DROP INDEX {}", quote(name)?),
                AlterOperation::ChangeField(_)
                | AlterOperation::AddPrimary(_)
                | AlterOperation::RemovePrimary
                | AlterOperation::AddForeignKey(_)
                | AlterOperation::RemoveForeignKey(_) => {
                    return Err(unsupported(
                        "SQLite alteration requires deferred table reconstruction",
                    ));
                }
            };
            queries.push(ddl_query(sql)?);
        }
        QueryPlan::new(queries)
    }
}

/// Validate field identity/order and every dialect-independent field rule.
fn validate_fields(fields: &[Field]) -> Result<()> {
    if fields.is_empty() || fields.len() > 1000 {
        return Err(invalid("invalid SQL field list"));
    }
    let mut names = std::collections::HashSet::new();
    let mut retained_defaults = 0_usize;
    for field in fields {
        field.validate()?;
        if let Some(DefaultValue::Value(value)) = &field.default {
            retained_defaults = retained_defaults
                .checked_add(value.retained_bytes())
                .ok_or_else(|| invalid("SQL field default byte budget overflow"))?;
            if retained_defaults > QueryLimits::default().bytes {
                return Err(invalid("SQL field defaults exceed input byte budget"));
            }
        }
        if !names.insert(field.name.to_ascii_lowercase()) {
            return Err(invalid("duplicate SQL field"));
        }
    }
    Ok(())
}

/// Check ordered columns against the known local field set.
fn validate_local_columns(columns: &[String], fields: &[Field]) -> Result<()> {
    column_list(columns)?;
    for column in columns {
        find_field(fields, column)?;
    }
    Ok(())
}

/// Reconcile explicit renames and compatible same-name indexes in a pure build phase.
fn reconcile(data: &AlterData) -> Result<Vec<AlterOperation>> {
    let mut sources = std::collections::HashSet::new();
    let mut targets = std::collections::HashSet::new();
    let mut skipped = std::collections::HashSet::new();
    let mut changes = std::collections::HashMap::new();
    for (index, operation) in data.operations.iter().enumerate() {
        if let AlterOperation::RenameField { from, to } = operation {
            quote(from)?;
            quote(to)?;
            // Duplicate sources/targets or occupied targets would make ordered execution ambiguous.
            if from.eq_ignore_ascii_case(to)
                || !sources.insert(from.to_ascii_lowercase())
                || !targets.insert(to.to_ascii_lowercase())
                || data
                    .source_fields
                    .iter()
                    .any(|field| field.name.eq_ignore_ascii_case(to))
            {
                return Err(invalid("conflicting SQL field rename"));
            }
            let source = find_field(&data.source_fields, from)?;
            let removals = data.operations.iter().enumerate().filter(|(_, operation)| matches!(operation, AlterOperation::RemoveField(name) if name.eq_ignore_ascii_case(from))).map(|(index, _)| index).collect::<Vec<_>>();
            let additions = data
                .operations
                .iter()
                .enumerate()
                .filter_map(|(index, operation)| match operation {
                    AlterOperation::AddField(field) if field.name.eq_ignore_ascii_case(to) => {
                        Some((index, field))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            if removals.len() > 1
                || additions.len() > 1
                || removals.is_empty() != additions.is_empty()
            {
                return Err(invalid("incomplete SQL rename reconciliation"));
            }
            if let (Some(removal), Some((addition, field))) = (removals.first(), additions.first())
            {
                field.validate()?;
                skipped.insert(*removal);
                skipped.insert(*addition);
                let mut renamed = source.clone();
                renamed.name = field.name.clone();
                // A rename never hides a semantic change; preserve that separate operation.
                if renamed != **field {
                    changes.insert(index, (*field).clone());
                }
            }
        }
    }
    let indexes = data
        .operations
        .iter()
        .filter_map(|operation| match operation {
            AlterOperation::AddIndex(index) => Some(index.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let indexes = normalize_indexes(&indexes)?;
    let mut emitted_indexes = std::collections::HashSet::new();
    let mut field_operations = std::collections::HashSet::new();
    let mut operations = Vec::new();
    let mut current_fields = data.source_fields.clone();
    for (index, operation) in data.operations.iter().enumerate() {
        if skipped.contains(&index) {
            continue;
        }
        match operation {
            AlterOperation::AddIndex(index) => {
                let name = index.name.to_ascii_lowercase();
                if emitted_indexes.insert(name) {
                    let normalized = indexes
                        .iter()
                        .find(|normalized| normalized.name.eq_ignore_ascii_case(&index.name))
                        .ok_or_else(|| invalid("missing normalized SQL index"))?;
                    operations.push(AlterOperation::AddIndex(normalized.clone()));
                }
                continue;
            }
            AlterOperation::AddField(field) | AlterOperation::ChangeField(field) => {
                field.validate()?;
                if !field_operations.insert(field.name.to_ascii_lowercase()) {
                    return Err(invalid("conflicting SQL field operations"));
                }
                current_fields.retain(|previous| previous.name != field.name);
                current_fields.push(field.clone());
            }
            AlterOperation::RemoveField(name) => {
                quote(name)?;
                if !field_operations.insert(name.to_ascii_lowercase()) {
                    return Err(invalid("conflicting SQL field operations"));
                }
                current_fields.retain(|field| &field.name != name);
            }
            AlterOperation::RemoveIndex(name) | AlterOperation::RemoveForeignKey(name) => {
                quote(name)?;
            }
            AlterOperation::AddPrimary(columns) => {
                column_list(columns)?;
            }
            AlterOperation::AddForeignKey(foreign) => {
                foreign.sqlite(&current_fields)?;
            }
            AlterOperation::RenameField { from, to } => {
                if let Some(field) = current_fields.iter_mut().find(|field| &field.name == from) {
                    field.name = to.clone();
                }
            }
            _ => {}
        }
        operations.push(operation.clone());
        if let Some(field) = changes.remove(&index) {
            operations.push(AlterOperation::ChangeField(field));
        }
    }
    Ok(operations)
}

/// Freeze one AST-generated rowless statement without routing through trusted raw text.
fn ddl_query(sql: String) -> Result<Query> {
    Query::bound(DialectId::Sqlite, sql, Vec::new())
}

#[cfg(test)]
#[path = "tests/plan_validation.rs"]
mod tests;
