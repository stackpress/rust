//! Pure same-table structural differences. No live introspection or rename guessing.

use crate::identifier::invalid;
use crate::{
    Alter, AlterOperation, Create, CreateData, DialectId, Field, ForeignKey, Index, QueryPlan,
    Result,
};

/// A validated storage definition, separate from create-if-not-exists execution policy.
///
/// Public fields allow editing a detached target; call [`Self::normalized`] to
/// validate after edits. Definitions do not assert that live storage matches.
/// Compound keys and every field property compare by value. Existing physical
/// column reordering is explicitly unsupported; renames require reconciliation.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
/// let source = Create::new("profile")
///     .add_field(Field::new("name", SqlType::Text))
///     .definition()?;
/// assert_eq!(source.name, "profile");
/// assert_eq!(source.fields.len(), 1);
/// assert!(
///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
/// );
/// let mut target = source.clone();
/// target.fields[0].name = "display_name".into();
/// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
/// assert!(diff.render(DialectId::Sqlite)?.is_some());
/// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TableDefinition {
    /// One logical literal table name.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SqlType};
    /// let value = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert_eq!(value.name, "profile");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub name: String,
    /// Ordered field definitions with every typed property retained.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SqlType};
    /// let value = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert_eq!(value.fields.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fields: Vec<Field>,
    /// Ordered compound primary-key columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SqlType};
    /// let value = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert!(value.primary.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub primary: Vec<String>,
    /// Named regular/unique indexes with ordered compound columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SqlType};
    /// let value = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert!(value.indexes.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub indexes: Vec<Index>,
    /// Named compound foreign constraints and actions.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SqlType};
    /// let value = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert!(value.foreign_keys.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub foreign_keys: Vec<ForeignKey>,
}

impl TableDefinition {
    /// Rebuild a detached CREATE description without idempotence or installation effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// assert_eq!(source.create().definition()?, source);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Errors
    ///
    /// Rejects invalid fields/defaults/key references and conflicting same-name indexes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// assert_eq!(source.normalized()?, source);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn normalized(&self) -> Result<Self> {
        Ok(self.create().build()?.into())
    }

    /// Compare this definition to an explicit target without inferring any field rename.
    ///
    /// # Errors
    ///
    /// Rejects invalid definitions, different table names or physical field reordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Errors
    ///
    /// Returns the same identifier, field, default and relationship errors as [`Self::build`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn definition(&self) -> Result<TableDefinition> {
        Ok(self.build()?.into())
    }
}

/// A typed no-op or an inspectable alteration; absence never becomes empty executable SQL.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
/// let source = Create::new("profile")
///     .add_field(Field::new("name", SqlType::Text))
///     .definition()?;
/// assert_eq!(source.name, "profile");
/// assert_eq!(source.fields.len(), 1);
/// assert!(
///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
/// );
/// let mut target = source.clone();
/// target.fields[0].name = "display_name".into();
/// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
/// assert!(diff.render(DialectId::Sqlite)?.is_some());
/// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum SchemaDiff {
    /// Equal storage semantics; no plan must be executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::SchemaDiff;
    /// assert!(SchemaDiff::NoChanges.alteration().is_none());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    NoChanges,
    /// An explicit difference with original field metadata for safe rename reconciliation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Field, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// let mut target = source.clone();
    /// target.fields.push(Field::new("name", SqlType::Text));
    /// let diff = source.diff(&target)?;
    /// if let SchemaDiff::Changes {
    ///     source,
    ///     target,
    ///     alteration,
    /// } = diff
    /// {
    ///     assert_eq!(source.fields.len(), 1);
    ///     assert_eq!(target.fields.len(), 2);
    ///     assert_eq!(alteration.build()?.operations.len(), 1);
    /// } else {
    ///     panic!("expected explicit changes");
    /// }
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Changes {
        /// The complete known source definition.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{Create, Field, SchemaDiff, SqlType};
        /// let source = Create::new("profile")
        ///     .add_field(Field::new("id", SqlType::Integer))
        ///     .definition()?;
        /// let mut target = source.clone();
        /// target.fields.push(Field::new("name", SqlType::Text));
        /// let diff = source.diff(&target)?;
        /// if let SchemaDiff::Changes {
        ///     source,
        ///     target,
        ///     alteration,
        /// } = diff
        /// {
        ///     assert_eq!(source.fields.len(), 1);
        ///     assert_eq!(target.fields.len(), 2);
        ///     assert_eq!(alteration.build()?.operations.len(), 1);
        /// } else {
        ///     panic!("expected explicit changes");
        /// }
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        source: Box<TableDefinition>,
        /// The requested target definition.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{Create, Field, SchemaDiff, SqlType};
        /// let source = Create::new("profile")
        ///     .add_field(Field::new("id", SqlType::Integer))
        ///     .definition()?;
        /// let mut target = source.clone();
        /// target.fields.push(Field::new("name", SqlType::Text));
        /// let diff = source.diff(&target)?;
        /// if let SchemaDiff::Changes {
        ///     source,
        ///     target,
        ///     alteration,
        /// } = diff
        /// {
        ///     assert_eq!(source.fields.len(), 1);
        ///     assert_eq!(target.fields.len(), 2);
        ///     assert_eq!(alteration.build()?.operations.len(), 1);
        /// } else {
        ///     panic!("expected explicit changes");
        /// }
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        target: Box<TableDefinition>,
        /// Ordered removes/fields/adds, with no hidden native inspection.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{Create, Field, SchemaDiff, SqlType};
        /// let source = Create::new("profile")
        ///     .add_field(Field::new("id", SqlType::Integer))
        ///     .definition()?;
        /// let mut target = source.clone();
        /// target.fields.push(Field::new("name", SqlType::Text));
        /// let diff = source.diff(&target)?;
        /// if let SchemaDiff::Changes {
        ///     source,
        ///     target,
        ///     alteration,
        /// } = diff
        /// {
        ///     assert_eq!(source.fields.len(), 1);
        ///     assert_eq!(target.fields.len(), 2);
        ///     assert_eq!(alteration.build()?.operations.len(), 1);
        /// } else {
        ///     panic!("expected explicit changes");
        /// }
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        alteration: Box<Alter>,
    },
}

impl SchemaDiff {
    /// Compare every field/key/FK property by value; compound-key order is significant.
    ///
    /// # Errors
    /// Rejects invalid definitions or different logical table names. Field additions/removals
    /// are explicit until [`Self::rename_field`] reconciles a selected matching pair.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// assert_eq!(
    ///     SchemaDiff::between(&source, &source)?,
    ///     SchemaDiff::NoChanges
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// assert!(diff.alteration().is_some());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn alteration(&self) -> Option<&Alter> {
        match self {
            Self::NoChanges => None,
            Self::Changes { alteration, .. } => Some(alteration),
        }
    }

    /// Reconcile exactly one pending source removal and target addition into a rename.
    /// Changed field properties remain a separate ChangeField rather than being discarded.
    ///
    /// # Errors
    ///
    /// Rejects no-op differences, absent remove/add halves, invalid source metadata or conflicting renames.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Errors
    ///
    /// Returns definition, limit or native-capability errors before exposing any partial plan.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, DialectId, Field, Render, SchemaDiff, SqlType};
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("name", SqlType::Text))
    ///     .definition()?;
    /// assert_eq!(source.name, "profile");
    /// assert_eq!(source.fields.len(), 1);
    /// assert!(
    ///     source.primary.is_empty() && source.indexes.is_empty() && source.foreign_keys.is_empty()
    /// );
    /// let mut target = source.clone();
    /// target.fields[0].name = "display_name".into();
    /// let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    /// assert!(diff.render(DialectId::Sqlite)?.is_some());
    /// assert_eq!(source.diff(&source)?, SchemaDiff::NoChanges);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
