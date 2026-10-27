//! Deterministic generated store emitters, composed into the app's single tool registry.
//!
//! M0 emits create/get and explicit table definitions. The broader CRUD/operation surface
//! belongs to later milestones; missing methods cannot silently imply support.

use stackpress_framework_schema::{Field, Schema, ValueType};
use stackpress_idea_transformer::{OutputPlan, TransformError};

/// Emit stores consuming the exact generated contracts and borrowed runtime executor.
pub fn emit_stores(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let fingerprint = schema
        .fingerprint()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    for model in schema.models() {
        let stored: Vec<_> = model
            .fields
            .iter()
            .filter(|field| !matches!(field.value_type, ValueType::Model(_)))
            .collect();
        let key = stored
            .iter()
            .find(|field| field.key)
            .ok_or_else(|| TransformError::new("M0 generated store requires a key"))?;
        // M1 freezes schema/key DTOs; broader executable CRUD is explicitly gated to M2.
        if stored.iter().filter(|field| field.key).count() != 1
            || !matches!(key.value_type, ValueType::String | ValueType::Integer)
        {
            return Err(TransformError::new(
                "compound or non-text/integer executable store keys require M2",
            ));
        }
        let key_type = if key.value_type == ValueType::Integer {
            "i64"
        } else {
            "&str"
        };
        let mut text = format!(
            "//! Explicit generated {} store; no captured Engine or identity map.\nuse crate::generated::contracts::{}::{{{}, Create{}}};\nuse stackpress_framework_sql::{{DataError, DataErrorKind, DataResult, DataSession, StorageColumn, StorageCodec}};\nuse stackpress_sql::{{Identifier, Insert, Row, Select, TableDefinition}};\n\nconst _: () = assert!(stackpress_framework_sql::CONTRACT_REVISION == {});\n/// Same normalized input identity as the schema transform.\npub const FINGERPRINT: &str = {:?};\n/// Immutable field codecs, chosen by schema semantics.\npub const COLUMNS: &[StorageColumn] = &[\n",
            model.rust_name,
            model.module,
            model.rust_name,
            model.rust_name,
            crate::CONTRACT_REVISION,
            fingerprint
        );
        for field in &stored {
            text.push_str(&format!("StorageColumn {{ wire: {:?}, column: {:?}, codec: StorageCodec::{}, required: {}, nonempty: {} }},\n", field.name, field.column, codec(schema,field), field.required, field.nonempty));
        }
        text.push_str(&format!("] ;\n/// Pure inspectable table, query and codec functions.\npub struct {}Store;\nimpl {}Store {{\n/// Build the checked storage definition; installation is always explicit.\npub fn table() -> DataResult<TableDefinition> {{\nlet mut table = stackpress_sql::Create::new({:?});\n", model.rust_name, model.rust_name, model.module));
        for field in &stored {
            let sql_type = match codec(schema, field) {
                "Integer" => "Integer",
                "Boolean" => "Boolean",
                "Real" => "Real",
                _ => "Text",
            };
            text.push_str(&format!("// Preserve declared nullability in storage.\nlet field = stackpress_sql::Field::new({:?}, stackpress_sql::SqlType::{}){};\ntable = table.add_field(field);\n", field.column, sql_type, if field.required { ".not_null()" } else { "" }));
        }
        text.push_str(&format!("table = table.primary_key([{:?}]);\n", key.column));
        for field in &stored {
            if field.unique || field.searchable || field.sortable {
                let name = stackpress_framework_schema::codegen::portable_name(&format!(
                    "{}_{}_index",
                    model.module, field.column
                ))
                .map_err(stackpress_framework_schema::codegen::transform_error)?;
                text.push_str(&format!("// Storage constraints/indexes are authoritative; UI hints cannot replace them.\ntable = table.add_index(stackpress_sql::Index::new({:?}, [{:?}]){});\n",name,field.column,if field.unique{".unique()"}else{""}));
            }
        }
        for field in &model.fields {
            if let (ValueType::Model(target), Some(relation)) = (&field.value_type, &field.relation)
            {
                let target = schema
                    .models()
                    .iter()
                    .find(|model| &model.name == target)
                    .ok_or_else(|| TransformError::new("relation target missing"))?;
                let local = model
                    .fields
                    .iter()
                    .find(|field| field.name == relation.local)
                    .ok_or_else(|| TransformError::new("relation source missing"))?;
                let foreign = target
                    .fields
                    .iter()
                    .find(|field| field.name == relation.foreign)
                    .ok_or_else(|| TransformError::new("relation target field missing"))?;
                let name = format!("{}_{}_fk", model.module, local.column);
                text.push_str(&format!("// Explicit owning FK; reverse collections have no column.\nlet mut foreign = stackpress_sql::ForeignKey::new({:?}, [{:?}], {:?}, [{:?}]);\nforeign.on_delete = stackpress_sql::ReferentialAction::Restrict;\ntable = table.add_foreign_key(foreign);\n", name, local.column, target.module, foreign.column));
            }
        }
        text.push_str("Ok(table.definition()?)\n}\n");
        text.push_str(&format!("/// Decode strict stored values, returning a fully owned contract.\npub fn decode(row: Row) -> DataResult<{}> {{
let value: {} = stackpress_framework_sql::decode(row, COLUMNS)?;
value.validate().map_err(|cause| DataError::new(\"stored value validation failed\").with_kind(DataErrorKind::Decode).with_cause(cause))?;
Ok(value)
}}\n", model.rust_name, model.rust_name));
        text.push_str(&format!("/// Build one key-scoped ordinary read; no IO occurs here.\npub fn select(key: {}) -> Select {{\nlet query = Select::new(COLUMNS.iter().map(|column| Identifier::qualified([{:?}, column.column])))\n.from({:?}).where_eq({:?}, key);\n", key_type, model.module, model.module, key.column));
        if let Some(active) = model.fields.iter().find(|field| field.active) {
            // Ordinary reads honor the selected soft-removal policy at the SQL boundary.
            text.push_str(&format!("// Ordinary reads exclude inactive records, including key lookups.\nlet query = query.where_eq(Identifier::qualified([{:?}, {:?}]), true);\n", model.module, active.column));
        }
        text.push_str("query.limit(1)\n}\n}\n");
        text.push_str(&format!("/// Executing functions borrowing the caller's selected executor.\npub struct {}s;\nimpl {}s {{\n/// Execute one key lookup; absence is distinct from decode/storage failure.\npub async fn get(db: &mut DataSession<'_>, key: {}) -> DataResult<Option<{}>> {{\nlet mut rows = db.fetch(&{}Store::select(key)).await?;\nrows.pop().map({}Store::decode).transpose()\n}}\n", model.rust_name, model.rust_name, key_type, model.rust_name, model.rust_name, model.rust_name));
        text.push_str(&format!("/// Validate/prepare once and insert with authoritative statement-local RETURNING.\npub async fn create(db: &mut DataSession<'_>, input: Create{}) -> DataResult<{}> {{\nif !db.dialect().capabilities().returning {{ return Err(DataError::new(\"M0 create requires RETURNING\").with_kind(DataErrorKind::Unsupported)); }}\n// Literal defaults are applied once, before the first database effect.\nlet value = input.prepare().map_err(|cause| DataError::new(\"create preparation failed\").with_kind(DataErrorKind::Validation).with_cause(cause))?;\nlet values = stackpress_framework_sql::encode(&value, COLUMNS)?;\nlet query = Insert::new({:?}).columns(COLUMNS.iter().map(|column| column.column))\n.values(values).returning(COLUMNS.iter().map(|column| column.column));\nlet mut rows = db.fetch(&query).await?;\nif rows.len() != 1 {{ return Err(DataError::new(\"create returned an invalid row count\").with_kind(DataErrorKind::Decode)); }}\n{}Store::decode(rows.remove(0))\n}}\n}}\n", model.rust_name, model.rust_name, model.module, model.rust_name));
        output.write(format!("stores/{}.rs", model.module), text)?;
        output.contribute_module("stores", &model.module)?;
    }
    let mut tables = String::from(
        "//! Checked generated storage definitions; importing never installs them.\n/// Return all managed table definitions in source order.\npub fn definition() -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {\nOk(vec![\n",
    );
    for model in schema.models() {
        tables.push_str(&format!(
            "super::{}::{}Store::table()?,\n",
            model.module, model.rust_name
        ));
    }
    tables.push_str("])\n}\n");
    output.write("stores/schema.rs", tables)?;
    output.contribute_module("stores", "schema")
}

/// Choose an explicit strict storage codec; lists/fieldsets are one JSON column.
fn codec(schema: &Schema, field: &Field) -> &'static str {
    if field.multiple {
        return "Json";
    }
    match &field.value_type {
        ValueType::String | ValueType::Date | ValueType::Time | ValueType::Datetime => "Text",
        ValueType::Enum(name) => match schema
            .enums()
            .iter()
            .find(|item| item.name == *name)
            .and_then(|item| item.variants.first())
            .map(|(_, value)| value)
        {
            Some(value) if value.is_boolean() => "Boolean",
            Some(value) if value.is_i64() => "Integer",
            _ => "Text",
        },
        ValueType::Integer => "Integer",
        ValueType::Boolean => "Boolean",
        ValueType::Number => "Real",
        ValueType::Object | ValueType::Fieldset(_) => "Json",
        ValueType::Model(_) => "Json",
    }
}
