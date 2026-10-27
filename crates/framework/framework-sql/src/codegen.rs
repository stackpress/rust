//! Deterministic native store and explicit operation generation from the app-owned registry.
//! Contracts/metadata remain owned by the schema transform; all output here stays in stores.
use stackpress_framework_schema::{Field, Model, Schema, ValueType};
use stackpress_idea_transformer::{OutputPlan, TransformError};
mod catalog;
mod conflicts;
mod operations;
mod queries;
mod selection;
mod writes;

/// Emit strict native stores, finite eager views and explicitly selected named handlers.
pub fn emit_stores(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    emit_stores_with_diagnostics(
        schema,
        output,
        &stackpress_framework_schema::codegen::DiagnosticMap {
            entries: Vec::new(),
        },
        false,
    )
}
/// Emit selected native capabilities with real winning Idea coordinates (or JSON structural paths).
pub fn emit_stores_with_diagnostics(
    schema: &Schema,
    output: &mut OutputPlan,
    diagnostics: &stackpress_framework_schema::codegen::DiagnosticMap,
    with_operations: bool,
) -> stackpress_idea_transformer::Result<()> {
    let fingerprint = schema
        .fingerprint()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    let public = schema
        .project_ui()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    for model in schema.models() {
        let stored: Vec<_> = model
            .fields
            .iter()
            .filter(|field| !matches!(field.value_type, ValueType::Model(_)))
            .collect();
        if !stored.iter().any(|field| field.key) {
            return Err(store_error(
                model,
                "generated store requires a complete primary key",
                None,
                diagnostics,
            ));
        }
        let mut text = format!(
            "//! Explicit generated {} store; every operation borrows its executor.\nuse crate::generated::contracts::{}::{{{} as _Stored,Create{} as _Create}};\nuse crate::generated::contracts::public::{{{}Key as Key,Patch{} as Patch}};\nuse stackpress_framework_sql::{{DataError,DataErrorKind,DataResult,DataSession,StorageColumn,StorageCodec}};\nuse stackpress_framework_sql::stores::{{Exposure,Loaded,Page,ReadQuery,RelationLimits,StoreSpec,Visibility}};\nuse stackpress_sql::{{Insert,Row,Select,TableDefinition}};\nconst _: ()=assert!(stackpress_framework_sql::CONTRACT_REVISION=={});\n/// Shared normalized schema identity.\npub const FINGERPRINT:&str={:?};\n/// Exact ordered stored codecs.\npub const COLUMNS:&[StorageColumn]=&[\n",
            model.rust_name,
            model.module,
            model.rust_name,
            model.rust_name,
            model.rust_name,
            model.rust_name,
            crate::CONTRACT_REVISION,
            fingerprint
        );
        for field in &stored {
            text.push_str(&storage(schema, field));
        }
        text.push_str("];\n");
        table(schema, model, &stored, &mut text)?;
        catalog::emit(schema, model, &stored, &public, &mut text)?;
        queries::emit(schema, model, &stored, &mut text)?;
        writes::emit(schema, model, &stored, &mut text)?;
        selection::emit(schema, model, &mut text).map_err(|cause| {
            store_error(
                model,
                "default selected storage projection is unsupported",
                Some(cause),
                diagnostics,
            )
        })?;
        conflicts::emit(schema, model, &mut text)?;
        if !model
            .fields
            .iter()
            .any(|field| matches!(field.value_type, ValueType::Model(_)))
        {
            text = text.replace("Exposure,Loaded,Page", "Exposure,Page");
        }
        output.write(format!("stores/{}.rs", store_module(model)), text)?;
        output.contribute_module("stores", &store_module(model))?;
    }
    let mut text = String::from(
        "//! Explicit generated table installation and immutable native store catalog.\n/// Build every declared table definition; importing never installs one.\npub fn definition()->stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> { Ok(vec![\n",
    );
    for model in schema.models() {
        text.push_str(&format!(
            "super::{}::{}Store::table()?,\n",
            store_module(model),
            model.rust_name
        ));
    }
    text.push_str("]) }\n/// Return the finite generated catalog; it owns no executor or mutable cache.\npub fn catalog()->Vec<&'static stackpress_framework_sql::stores::StoreSpec> { vec![\n");
    for model in schema.models() {
        text.push_str(&format!("&super::{}::SPEC,\n", store_module(model)));
    }
    text.push_str("] }\n");
    output.write("stores/schema.rs", text)?;
    output.contribute_module("stores", "schema")?;
    if with_operations {
        operations::emit(schema, output)?;
    }
    Ok(())
}

/// Explicitly select host-only named handler output in addition to the native stores.
/// The consuming app must select framework-sql/server and register its chosen operations.
pub fn emit_stores_with_operations(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    emit_stores_with_diagnostics(
        schema,
        output,
        &stackpress_framework_schema::codegen::DiagnosticMap {
            entries: Vec::new(),
        },
        true,
    )
}

/// Emit explicit checked DDL, reusing the core's identifiers and FK/index policies.
fn table(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    text.push_str(&format!("/// Pure inspectable table, query and codec functions.\npub struct {}Store;\nimpl {}Store {{\n/// Build the checked storage definition; installation is always explicit.\npub fn table() -> DataResult<TableDefinition> {{\nlet mut table = stackpress_sql::Create::new({:?});\n", model.rust_name, model.rust_name, model.module));
    for field in stored {
        let sql_type = match codec(schema, field) {
            "Integer" => "Integer",
            "Boolean" => "Boolean",
            "Real" => "Real",
            _ => "Text",
        };
        text.push_str(&format!("// Preserve declared nullability in storage.\nlet field = stackpress_sql::Field::new({:?}, stackpress_sql::SqlType::{}){};\ntable = table.add_field(field);\n", field.column, sql_type, if field.required { ".not_null()" } else { "" }));
    }
    text.push_str(&format!(
        "table = table.primary_key({:?});\n",
        stored
            .iter()
            .filter(|field| field.key)
            .map(|field| field.column.as_str())
            .collect::<Vec<_>>()
    ));
    for field in stored {
        if field.unique || field.searchable || field.sortable {
            let name = constraint_name(&model.module, &field.column, "index")?;
            text.push_str(&format!("// Storage constraints/indexes are authoritative; UI hints cannot replace them.\ntable = table.add_index(stackpress_sql::Index::new({:?}, [{:?}]){});\n",name,field.column,if field.unique{".unique()"}else{""}));
        }
    }
    for field in &model.fields {
        if let (ValueType::Model(target), Some(relation)) = (&field.value_type, &field.relation) {
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
            let shared = model
                .fields
                .iter()
                .filter_map(|field| field.relation.as_ref())
                .filter(|mapping| mapping.local == relation.local)
                .count()
                > 1;
            let name = constraint_name(
                &model.module,
                if shared {
                    &field.rust_name
                } else {
                    &local.column
                },
                "fk",
            )?;
            text.push_str(&format!("// Explicit owning FK; reverse collections have no column.\nlet mut foreign = stackpress_sql::ForeignKey::new({:?}, [{:?}], {:?}, [{:?}]);\nforeign.on_delete = stackpress_sql::ReferentialAction::Restrict;\ntable = table.add_foreign_key(foreign);\n", name, local.column, target.module, foreign.column));
        }
    }
    text.push_str("Ok(table.definition()?)\n}\n");
    text.push_str("}\n");
    Ok(())
}

/// Choose an explicit strict storage codec; lists/fieldsets are one JSON column.
fn codec(schema: &Schema, field: &Field) -> &'static str {
    if field.multiple {
        return "Json";
    }
    match &field.value_type {
        ValueType::String => "Text",
        ValueType::Date => "Date",
        ValueType::Time => "Time",
        ValueType::Datetime => "Datetime",
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

/// One generated static column literal, shared by storage, query and patch catalogs.
fn storage(schema: &Schema, field: &Field) -> String {
    format!(
        "StorageColumn {{wire:{:?},column:{:?},codec:StorageCodec::{},required:{},nonempty:{}}},\n",
        field.name,
        field.column,
        codec(schema, field),
        field.required,
        field.nonempty
    )
}
/// Scalar Rust type independent of list wrapping, using exact checked schema contract types.
fn scalar(schema: &Schema, field: &Field) -> stackpress_idea_transformer::Result<String> {
    Ok(match &field.value_type {
        ValueType::String => "String".into(),
        ValueType::Integer => "i64".into(),
        ValueType::Boolean => "bool".into(),
        ValueType::Number => "stackpress_framework_schema::FiniteNumber".into(),
        ValueType::Date => "stackpress_framework_schema::DateValue".into(),
        ValueType::Time => "stackpress_framework_schema::TimeValue".into(),
        ValueType::Datetime => "stackpress_framework_schema::DatetimeValue".into(),
        ValueType::Enum(name) => {
            let enumeration = schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .ok_or_else(|| TransformError::new("enum missing from checked schema"))?;
            format!(
                "crate::generated::contracts::{}::{}",
                enumeration.module, enumeration.rust_name
            )
        }
        _ => return Err(TransformError::new("query scalar is unsupported")),
    })
}
/// Native predicate families exclude objects, embedded records and model references.
fn is_scalar(field: &Field) -> bool {
    !matches!(
        field.value_type,
        ValueType::Object | ValueType::Fieldset(_) | ValueType::Model(_)
    )
}
/// Checked field identities have already been disambiguated by the schema compiler.
fn variant(field: &Field) -> String {
    let name: String = field
        .rust_name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect();
    if name == "Self" {
        "SelfValue".into()
    } else {
        name
    }
}
/// Preserve legacy convenient scalar keys while complete-key functions serve every identity shape.
fn key_argument(model: &Model) -> (String, String) {
    let keys: Vec<_> = model.fields.iter().filter(|field| field.key).collect();
    if keys.len() == 1 {
        let field = keys[0];
        let ty = match field.value_type {
            ValueType::String => Some("&str"),
            ValueType::Integer => Some("i64"),
            _ => None,
        };
        if let Some(ty) = ty {
            return (
                ty.into(),
                format!(
                    "&Key{{{}:{}}}",
                    field.rust_name,
                    if field.value_type == ValueType::Integer {
                        "key"
                    } else {
                        "key.into()"
                    }
                ),
            );
        }
    }
    ("&Key".into(), "key".into())
}

/// Escape only aggregate filenames; normalized source modules cannot start with underscore.
fn store_module(model: &Model) -> String {
    if matches!(model.module.as_str(), "schema" | "operations") {
        format!("_{}", model.module)
    } else {
        model.module.clone()
    }
}
/// Retain ordinary constraint names while making keyword-suffixed and long names portable.
fn constraint_name(
    table: &str,
    column: &str,
    suffix: &str,
) -> stackpress_idea_transformer::Result<String> {
    let candidate = format!("{table}_{column}_{suffix}");
    let candidate = if candidate.contains("__") {
        format!(
            "N{}X{}N{}X{}X{}",
            table.len(),
            table,
            column.len(),
            column,
            suffix
        )
    } else {
        candidate
    };
    stackpress_framework_schema::codegen::portable_name(&candidate)
        .map_err(stackpress_framework_schema::codegen::transform_error)
}
/// Avoid cloning Copy scalars while retaining exact owned string/enum/canonical temporal identities.
fn copy_key(field: &Field, expression: &str) -> String {
    if matches!(
        field.value_type,
        ValueType::Integer | ValueType::Boolean | ValueType::Number
    ) {
        expression.into()
    } else {
        format!("{expression}.clone()")
    }
}
/// Attach the model's winning source position without inventing coordinates for normalized JSON.
fn store_error(
    model: &Model,
    context: &'static str,
    cause: Option<TransformError>,
    diagnostics: &stackpress_framework_schema::codegen::DiagnosticMap,
) -> TransformError {
    let error = stackpress_framework_schema::SchemaError::new(
        stackpress_framework_schema::SchemaErrorKind::Unsupported,
        context,
    )
    .at(vec!["model".into(), model.name.clone()]);
    let error = if let Some(cause) = cause {
        error.with_cause(cause)
    } else {
        error
    };
    stackpress_framework_schema::codegen::transform_error(error.with_diagnostics(diagnostics))
}
