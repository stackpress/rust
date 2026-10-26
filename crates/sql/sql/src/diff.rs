//! Pure same-table structural differences. No live introspection or rename guessing.

use crate::identifier::invalid;
use crate::{
    Alter, AlterOperation, Create, CreateData, DialectId, Field, ForeignKey, Index, QueryPlan,
    Result,
};

/// A validated storage definition, separate from create-if-not-exists execution policy.
#[derive(Clone, Debug, PartialEq)]
pub struct TableDefinition {
    /// One logical literal table name.
    pub name: String,
    /// Ordered field definitions with every typed property retained.
    pub fields: Vec<Field>,
    /// Ordered compound primary-key columns.
    pub primary: Vec<String>,
    /// Named regular/unique indexes with ordered compound columns.
    pub indexes: Vec<Index>,
    /// Named compound foreign constraints and actions.
    pub foreign_keys: Vec<ForeignKey>,
}

impl TableDefinition {
    /// Rebuild a detached CREATE description without idempotence or installation effects.
    pub fn create(&self) -> Create {
        let mut create = Create::new(&self.name).primary_key(self.primary.clone());
        for field in &self.fields {
            create = create.add_field(field.clone());
        }
        for index in &self.indexes {
            create = create.add_index(index.clone());
        }
        for key in &self.foreign_keys {
            create = create.add_foreign_key(key.clone());
        }
        create
    }

    /// Validate names/defaults/references and return normalized unique-index precedence.
    pub fn normalized(&self) -> Result<Self> {
        Ok(self.create().build()?.into())
    }

    /// Compare this definition to an explicit target without inferring any field rename.
    pub fn diff(&self, target: &Self) -> Result<SchemaDiff> {
        SchemaDiff::between(self, target)
    }
}

impl From<CreateData> for TableDefinition {
    /// Drop creation policy while preserving all storage metadata and order.
    fn from(data: CreateData) -> Self {
        Self {
            name: data.table,
            fields: data.fields,
            primary: data.primary,
            indexes: data.indexes,
            foreign_keys: data.foreign_keys,
        }
    }
}

impl Create {
    /// Inspect a normalized storage definition without rendering or opening a database.
    pub fn definition(&self) -> Result<TableDefinition> {
        Ok(self.build()?.into())
    }
}

/// A typed no-op or an inspectable alteration; absence never becomes empty executable SQL.
#[derive(Clone, Debug, PartialEq)]
pub enum SchemaDiff {
    /// Equal storage semantics; no plan must be executed.
    NoChanges,
    /// An explicit difference with original field metadata for safe rename reconciliation.
    Changes {
        /// The complete known source definition.
        source: Box<TableDefinition>,
        /// The requested target definition.
        target: Box<TableDefinition>,
        /// Ordered removes/fields/adds, with no hidden native inspection.
        alteration: Box<Alter>,
    },
}

impl SchemaDiff {
    /// Compare every field/key/FK property by value; compound-key order is significant.
    ///
    /// # Errors
    /// Rejects invalid definitions or different logical table names. Field additions/removals
    /// are explicit until [`Self::rename_field`] reconciles a selected matching pair.
    pub fn between(source: &TableDefinition, target: &TableDefinition) -> Result<Self> {
        let source = source.normalized()?;
        let target = target.normalized()?;
        if source.name != target.name {
            return Err(invalid("schema diff requires the same logical table"));
        }
        // Reordering existing columns needs a physical reconstruction design; it cannot
        // silently become an empty migration while the requested ordinals differ.
        let common_source = source
            .fields
            .iter()
            .filter(|field| {
                target
                    .fields
                    .iter()
                    .any(|candidate| candidate.name == field.name)
            })
            .map(|field| &field.name)
            .collect::<Vec<_>>();
        let common_target = target
            .fields
            .iter()
            .filter(|field| {
                source
                    .fields
                    .iter()
                    .any(|candidate| candidate.name == field.name)
            })
            .map(|field| &field.name)
            .collect::<Vec<_>>();
        if common_source != common_target {
            return Err(crate::identifier::unsupported(
                "physical field reordering requires a migration design",
            ));
        }
        let mut operations = Vec::new();
        // Remove dependent constraints before removing/changing fields; add them last.
        for key in &source.foreign_keys {
            if !target.foreign_keys.contains(key) {
                operations.push(AlterOperation::RemoveForeignKey(key.name.clone()));
            }
        }
        if source.primary != target.primary && !source.primary.is_empty() {
            operations.push(AlterOperation::RemovePrimary);
        }
        for index in &source.indexes {
            if !target.indexes.contains(index) {
                operations.push(AlterOperation::RemoveIndex(index.name.clone()));
            }
        }
        for field in &source.fields {
            match target
                .fields
                .iter()
                .find(|candidate| candidate.name == field.name)
            {
                None => operations.push(AlterOperation::RemoveField(field.name.clone())),
                Some(candidate) if candidate != field => {
                    operations.push(AlterOperation::ChangeField(candidate.clone()))
                }
                Some(_) => {}
            }
        }
        for field in &target.fields {
            if !source
                .fields
                .iter()
                .any(|candidate| candidate.name == field.name)
            {
                operations.push(AlterOperation::AddField(field.clone()));
            }
        }
        if source.primary != target.primary && !target.primary.is_empty() {
            operations.push(AlterOperation::AddPrimary(target.primary.clone()));
        }
        for index in &target.indexes {
            if !source.indexes.contains(index) {
                operations.push(AlterOperation::AddIndex(index.clone()));
            }
        }
        for key in &target.foreign_keys {
            if !source.foreign_keys.contains(key) {
                operations.push(AlterOperation::AddForeignKey(key.clone()));
            }
        }
        if operations.is_empty() {
            return Ok(Self::NoChanges);
        }
        let alteration = build_alter(&source, operations);
        alteration.build()?;
        Ok(Self::Changes {
            source: Box::new(source),
            target: Box::new(target),
            alteration: Box::new(alteration),
        })
    }

    /// Inspect the normalized ordered alteration, or None for a typed no-op.
    pub fn alteration(&self) -> Option<&Alter> {
        match self {
            Self::NoChanges => None,
            Self::Changes { alteration, .. } => Some(alteration),
        }
    }

    /// Reconcile exactly one pending source removal and target addition into a rename.
    /// Changed field properties remain a separate ChangeField rather than being discarded.
    pub fn rename_field(self, from: &str, to: &str) -> Result<Self> {
        let Self::Changes {
            source,
            target,
            alteration,
        } = self
        else {
            return Err(invalid("no pending schema changes to rename"));
        };
        let mut operations = alteration.build()?.operations;
        if !operations.iter().any(|operation| matches!(operation, AlterOperation::RemoveField(name) if name == from)) || !operations.iter().any(|operation| matches!(operation, AlterOperation::AddField(field) if field.name == to)) {
            return Err(invalid("schema rename requires both pending field changes"));
        }
        // Reconcile before adding dependent keys so native index/FK statements see the new name.
        let position = operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    AlterOperation::AddIndex(_)
                        | AlterOperation::AddPrimary(_)
                        | AlterOperation::AddForeignKey(_)
                )
            })
            .unwrap_or(operations.len());
        operations.insert(
            position,
            AlterOperation::RenameField {
                from: from.into(),
                to: to.into(),
            },
        );
        let alteration = build_alter(&source, operations);
        alteration.build()?;
        Ok(Self::Changes {
            source,
            target,
            alteration: Box::new(alteration),
        })
    }

    /// Render the whole difference, or None for NoChanges; never execute an empty statement.
    /// Unsupported native operations fail before any plan is returned.
    pub fn render(&self, dialect: DialectId) -> Result<Option<QueryPlan>> {
        self.alteration()
            .map(|alteration| alteration.render(dialect))
            .transpose()
    }
}

/// Carry explicit source metadata into the ordinary ALTER normalization boundary.
fn build_alter(source: &TableDefinition, operations: Vec<AlterOperation>) -> Alter {
    let mut alter = Alter::new(&source.name);
    for field in &source.fields {
        alter = alter.source_field(field.clone());
    }
    for operation in operations {
        alter = alter.operation(operation);
    }
    alter
}
