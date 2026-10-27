//! Frozen format-2 codecs and explicit format-3 dispatch. Never rehash v2 through new DTOs.
use super::*;

/// Original field codec. New storage policy members cannot enter format 2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldV2 {
    name: String,
    kind: String,
    nullable: bool,
    length: Option<(i32, Option<i32>)>,
    default: Option<LiteralV2>,
    automatic: bool,
    unsigned: bool,
    comment: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexV2 {
    name: String,
    columns: Vec<String>,
    unique: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ForeignV2 {
    name: String,
    columns: Vec<String>,
    table: String,
    foreign_columns: Vec<String>,
    delete: String,
    update: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableV2 {
    name: String,
    fields: Vec<FieldV2>,
    primary: Vec<String>,
    indexes: Vec<IndexV2>,
    foreign_keys: Vec<ForeignV2>,
}
/// Original scalar spelling, including tagged real/blob literals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum LiteralV2 {
    Null,
    Bool(bool),
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}
/// Frozen mapping authority; future convenience fields cannot silently alter old hashes.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RenamesV2 {
    fields: Vec<FieldRenameV2>,
    tables: Vec<TableRenameV2>,
    drops: Vec<(String, String)>,
    backfills: Vec<BackfillV2>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldRenameV2 {
    table: String,
    from: String,
    to: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableRenameV2 {
    from: String,
    to: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackfillV2 {
    table: String,
    field: String,
    value: LiteralV2,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedV2 {
    table: String,
    values: BTreeMap<String, LiteralV2>,
    upsert: bool,
}
/// Original operation vocabulary. New operator modes belong only in later codecs.
#[derive(Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum KindV2 {
    Schema,
    Purge,
    Uninstall,
    Populate { seed: String, rows: Vec<SeedV2> },
}
/// Frozen original envelope, retaining its original member order for canonical hashes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotV2 {
    format: u32,
    app: String,
    revision: String,
    parent: Option<String>,
    source_fingerprint: String,
    generator_revision: u32,
    definition_revision: String,
    tables: Vec<TableV2>,
}
/// Current envelope. Format 3 tables carry target facts independently of ledger format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotV3 {
    format: u32,
    app: String,
    revision: String,
    parent: Option<String>,
    source_fingerprint: String,
    generator_revision: u32,
    definition_revision: String,
    tables: Vec<TableDto>,
}

impl SnapshotV2 {
    /// Convert only data representable by the frozen codec; added members refuse.
    fn capture(body: &SnapshotBody) -> DataResult<Self> {
        Ok(Self {
            format: body.format,
            app: body.app.clone(),
            revision: body.revision.clone(),
            parent: body.parent.clone(),
            source_fingerprint: body.source_fingerprint.clone(),
            generator_revision: body.generator_revision,
            definition_revision: body.definition_revision.clone(),
            tables: convert(&body.tables)?,
        })
    }
    /// Upgrade in-memory checked metadata without changing the durable format/hash codec.
    fn body(self) -> DataResult<SnapshotBody> {
        if self.format != 2 {
            return Err(unsupported("unknown legacy revision format"));
        }
        Ok(SnapshotBody {
            format: self.format,
            app: self.app,
            revision: self.revision,
            parent: self.parent,
            source_fingerprint: self.source_fingerprint,
            generator_revision: self.generator_revision,
            definition_revision: self.definition_revision,
            tables: convert(&self.tables)?,
        })
    }
}
impl Serialize for SnapshotBody {
    /// Select the codec before serialization, preserving original v2 bytes and member order.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.format == 2 {
            return SnapshotV2::capture(self)
                .map_err(|_| serde::ser::Error::custom("invalid format-2 revision metadata"))?
                .serialize(serializer);
        }
        SnapshotV3 {
            format: self.format,
            app: self.app.clone(),
            revision: self.revision.clone(),
            parent: self.parent.clone(),
            source_fingerprint: self.source_fingerprint.clone(),
            generator_revision: self.generator_revision,
            definition_revision: self.definition_revision.clone(),
            tables: self.tables.clone(),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SnapshotBody {
    /// Decode the version discriminator before accepting any current storage members.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(decoder)?;
        match value.get("format").and_then(serde_json::Value::as_u64) {
            Some(2) => serde_json::from_value::<SnapshotV2>(value)
                .map_err(|_| serde::de::Error::custom("invalid format-2 revision"))?
                .body()
                .map_err(|_| serde::de::Error::custom("invalid format-2 revision metadata")),
            Some(3) => {
                let body: SnapshotV3 = serde_json::from_value(value)
                    .map_err(|_| serde::de::Error::custom("invalid format-3 revision"))?;
                Ok(Self {
                    format: body.format,
                    app: body.app,
                    revision: body.revision,
                    parent: body.parent,
                    source_fingerprint: body.source_fingerprint,
                    generator_revision: body.generator_revision,
                    definition_revision: body.definition_revision,
                    tables: body.tables,
                })
            }
            _ => Err(serde::de::Error::custom("unknown revision format")),
        }
    }
}

/// Original recipe shape/order, excluding new typed preconditions.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeV2 {
    format: u32,
    dialect: String,
    from: SnapshotV2,
    to: SnapshotV2,
    renames: RenamesV2,
    kind: KindV2,
    reverse: bool,
    source_order: BTreeMap<String, Vec<String>>,
}
impl RecipeV2 {
    /// Copy original authority only when all new capabilities are absent.
    fn capture(recipe: &PlanRecipe) -> DataResult<Self> {
        if !recipe.length_checks.is_empty() || recipe.from.format != 2 || recipe.to.format != 2 {
            return Err(unsupported("format-2 plan cannot carry new authority"));
        }
        Ok(Self {
            format: recipe.format,
            dialect: recipe.dialect.clone(),
            from: SnapshotV2::capture(&recipe.from)?,
            to: SnapshotV2::capture(&recipe.to)?,
            renames: convert(&recipe.renames)?,
            kind: convert(&recipe.kind)?,
            reverse: recipe.reverse,
            source_order: recipe.source_order.clone(),
        })
    }
}
/// Current recipe wire representation, explicitly including even an empty check list.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeV3 {
    format: u32,
    dialect: String,
    from: SnapshotBody,
    to: SnapshotBody,
    renames: RenameMap,
    kind: PlanKind,
    reverse: bool,
    source_order: BTreeMap<String, Vec<String>>,
    length_checks: Vec<LengthCheck>,
}
impl Serialize for PlanRecipe {
    /// Keep legacy hashing/artifact serialization on its frozen codec.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.format == 2 {
            return RecipeV2::capture(self)
                .map_err(|_| serde::ser::Error::custom("invalid format-2 plan metadata"))?
                .serialize(serializer);
        }
        RecipeV3 {
            format: self.format,
            dialect: self.dialect.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            renames: self.renames.clone(),
            kind: self.kind.clone(),
            reverse: self.reverse,
            source_order: self.source_order.clone(),
            length_checks: self.length_checks.clone(),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PlanRecipe {
    /// Unknown versions and format-2 extensions refuse before planning or database IO.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(decoder)?;
        match value.get("format").and_then(serde_json::Value::as_u64) {
            Some(2) => {
                let recipe: RecipeV2 = serde_json::from_value(value)
                    .map_err(|_| serde::de::Error::custom("invalid format-2 plan"))?;
                Ok(Self {
                    format: 2,
                    dialect: recipe.dialect,
                    from: recipe
                        .from
                        .body()
                        .map_err(|_| serde::de::Error::custom("invalid legacy predecessor"))?,
                    to: recipe
                        .to
                        .body()
                        .map_err(|_| serde::de::Error::custom("invalid legacy target"))?,
                    renames: convert(&recipe.renames)
                        .map_err(|_| serde::de::Error::custom("invalid legacy mappings"))?,
                    kind: convert(&recipe.kind)
                        .map_err(|_| serde::de::Error::custom("invalid legacy operation"))?,
                    reverse: recipe.reverse,
                    source_order: recipe.source_order,
                    length_checks: vec![],
                })
            }
            Some(3) => {
                let recipe: RecipeV3 = serde_json::from_value(value)
                    .map_err(|_| serde::de::Error::custom("invalid format-3 plan"))?;
                Ok(Self {
                    format: 3,
                    dialect: recipe.dialect,
                    from: recipe.from,
                    to: recipe.to,
                    renames: recipe.renames,
                    kind: recipe.kind,
                    reverse: recipe.reverse,
                    source_order: recipe.source_order,
                    length_checks: recipe.length_checks,
                })
            }
            _ => Err(serde::de::Error::custom("unknown migration plan format")),
        }
    }
}

/// Transcode between explicitly frozen/current DTO shapes; errors keep static contextual causes.
fn convert<T: Serialize, U: serde::de::DeserializeOwned>(value: &T) -> DataResult<U> {
    let value = serde_json::to_value(value).map_err(|cause| {
        DataError::new("migration codec conversion encoding failed")
            .with_kind(DataErrorKind::Decode)
            .with_cause(cause)
    })?;
    serde_json::from_value(value).map_err(|cause| {
        DataError::new("migration codec conversion decoding failed")
            .with_kind(DataErrorKind::Decode)
            .with_cause(cause)
    })
}
