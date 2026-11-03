//! Deterministic native store and explicit operation generation from the app-owned registry.
//! Contracts/metadata remain owned by the schema transform; all output here stays in stores.
//!
//! The `codegen` feature is a build-host capability. Checked metadata and encoded
//! literals populate SQL-owned source templates; rendering never executes them.
use serde_json::json;
use stackpress_framework_schema::{Field, Model, Schema, ValueType};
use stackpress_idea_transformer::{OutputPlan, TransformError};
use templates::{Templates, column, literal, literals};
mod bounds;
mod catalog;
mod conflicts;
mod operations;
mod queries;
mod selection;
mod templates;
mod writes;

#[cfg(test)]
#[path = "codegen/tests.rs"]
mod tests;

/// Register a build-tool store transform after the explicit schema transform.
///
/// The caller still supplies normalization, diagnostics and operation/model selection.
/// No Engine, runtime registration or output installation is performed by this helper.
///
/// # Errors
/// Preserves registry selection/ownership errors; planning verifies dependency order.
pub fn register(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()> {
    registry.add(
        stackpress_idea_transformer::TransformSpec::new("framework-sql")
            .owns(&["stores"])
            .requires(&["framework-schema"]),
        transform,
    )
}

/// Emit strict native stores and finite eager views without named host handlers.
///
/// # Errors
/// Returns schema/storage failures, concrete template failures and output-plan
/// ownership or resource refusals. Template resource failures retain `Limit`.
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
///
/// `with_operations` additionally emits host handlers for explicit app registration.
/// Schema diagnostics annotate model/projection failures; template failures retain
/// their static asset identity and concrete cause.
///
/// # Errors
/// Returns schema/storage/projection failures, concrete template failures and
/// output-plan ownership or resource refusals. A registry may wrap these as callbacks.
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
    let definitions = storage_definitions(schema)?;
    // Construct one renderer after existing schema validation, before emitting any file.
    let templates = Templates::new()?;
    for (model, definition) in schema.models().iter().zip(&definitions) {
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
        let columns: Vec<_> = stored.iter().map(|field| column(schema, field)).collect();
        let mut text = templates.render(
            "SQL store-header template failed",
            include_str!("codegen/templates/store-header.rs.tpl"),
            &json!({
                "model": model.rust_name, "module": model.module,
                "revision": crate::CONTRACT_REVISION,
                "fingerprint_literal": literal(&fingerprint), "columns": columns,
                "has_relations": model.fields.iter().any(|field| matches!(field.value_type, ValueType::Model(_))),
            }),
        )?;
        // Preserve the original phase order: later failures never publish earlier file claims.
        table(definition, model, &templates, &mut text)?;
        bounds::emit(model, &templates, &mut text)?;
        catalog::emit(schema, model, &stored, &public, &templates, &mut text)?;
        queries::emit(schema, model, &stored, &templates, &mut text)?;
        writes::emit(schema, model, &stored, &templates, &mut text)?;
        selection::emit(schema, model, &templates, &mut text).map_err(|cause| {
            store_error(
                model,
                "default selected storage projection is unsupported",
                Some(cause),
                diagnostics,
            )
        })?;
        conflicts::emit(schema, model, &templates, &mut text)?;
        output.write(format!("stores/{}.rs", store_module(model)), text)?;
        output.contribute_module("stores", &store_module(model))?;
    }
    // The aggregate owns actual source loops; this context contains checked model names only.
    let models: Vec<_> = schema
        .models()
        .iter()
        .map(|model| {
            json!({
                "store": store_module(model), "name": model.rust_name,
            })
        })
        .collect();
    let text = templates.render(
        "SQL schema template failed",
        include_str!("codegen/templates/schema.rs.tpl"),
        &json!({"models": models}),
    )?;
    output.write("stores/schema.rs", text)?;
    output.contribute_module("stores", "schema")?;
    if with_operations {
        operations::emit(schema, &templates, output)?;
    }
    Ok(())
}

/// Explicitly select host-only named handler output in addition to the native stores.
/// The consuming app must select framework-sql/server and register its chosen operations.
///
/// # Errors
/// Returns the same schema, template and output-plan failures as [`emit_stores`],
/// plus policy/fieldset failures from the selected host handler generation.
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

/// Emit the same checked DDL used by revision recording, keeping generated bytes stable.
fn table(
    definition: &stackpress_sql::TableDefinition,
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Validate type/length in the original field order before supplying language atoms.
    let fields = definition
        .fields
        .iter()
        .map(|field| {
            let kind = match field.data_type {
                stackpress_sql::SqlType::Integer => "Integer",
                stackpress_sql::SqlType::Boolean => "Boolean",
                stackpress_sql::SqlType::Real => "Real",
                stackpress_sql::SqlType::Varchar => "Varchar",
                stackpress_sql::SqlType::Text => "Text",
                _ => return Err(TransformError::new("generated storage type is unsupported")),
            };
            let length = match field.length {
                Some(stackpress_sql::Length::Size(n)) => Some(n),
                None => None,
                _ => {
                    return Err(TransformError::new(
                        "generated storage length is unsupported",
                    ));
                }
            };
            Ok(json!({
                "name_literal": literal(&field.name), "kind": kind,
                "not_null": !field.nullable, "has_length": length.is_some(), "length": length,
            }))
        })
        .collect::<stackpress_idea_transformer::Result<Vec<_>>>()?;
    let indexes: Vec<_> = definition
        .indexes
        .iter()
        .map(|index| {
            json!({
                "name_literal": literal(&index.name), "unique": index.unique,
                "columns": literals(index.columns.iter().map(String::as_str)),
            })
        })
        .collect();
    let foreign_keys: Vec<_> = definition
        .foreign_keys
        .iter()
        .map(|key| {
            json!({
                "name_literal": literal(&key.name), "table_literal": literal(&key.table),
                "columns": literals(key.columns.iter().map(String::as_str)),
                "foreign_columns": literals(key.foreign_columns.iter().map(String::as_str)),
            })
        })
        .collect();
    templates.append(text, "SQL table template failed", include_str!("codegen/templates/table.rs.tpl"), &json!({
        "model": model.rust_name, "table_literal": literal(&model.module),
        "fields": fields, "primary": literals(definition.primary.iter().map(String::as_str)),
        "indexes": indexes, "foreign_keys": foreign_keys, "mysql_target": definition.mysql_storage.is_some(),
    }))
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

/// Scalar Rust type independent of list wrapping, using exact checked schema contract types.
fn scalar(
    schema: &Schema,
    field: &Field,
    templates: &Templates,
) -> stackpress_idea_transformer::Result<String> {
    // Qualified enum paths are source structure; fixed primitive names are checked atoms.
    let data = match &field.value_type {
        ValueType::String => json!({"qualified": false, "symbol": "String"}),
        ValueType::Integer => json!({"qualified": false, "symbol": "i64"}),
        ValueType::Boolean => json!({"qualified": false, "symbol": "bool"}),
        ValueType::Number => {
            json!({"qualified": false, "symbol": "stackpress_framework_schema::FiniteNumber"})
        }
        ValueType::Date => {
            json!({"qualified": false, "symbol": "stackpress_framework_schema::DateValue"})
        }
        ValueType::Time => {
            json!({"qualified": false, "symbol": "stackpress_framework_schema::TimeValue"})
        }
        ValueType::Datetime => {
            json!({"qualified": false, "symbol": "stackpress_framework_schema::DatetimeValue"})
        }
        ValueType::Enum(name) => {
            let enumeration = schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .ok_or_else(|| TransformError::new("enum missing from checked schema"))?;
            json!({"qualified": true, "module": enumeration.module, "symbol": enumeration.rust_name})
        }
        _ => return Err(TransformError::new("query scalar is unsupported")),
    };
    templates.render(
        "SQL scalar-type template failed",
        include_str!("codegen/templates/scalar-type.rs.tpl"),
        &data,
    )
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
fn key_argument(
    model: &Model,
    templates: &Templates,
) -> stackpress_idea_transformer::Result<(String, String)> {
    let keys: Vec<_> = model.fields.iter().filter(|field| field.key).collect();
    // Only the original String/Integer singleton convenience signatures are retained.
    let simple = keys
        .first()
        .copied()
        .filter(|_| keys.len() == 1)
        .and_then(|field| {
            let ty = match field.value_type {
                ValueType::String => "&str",
                ValueType::Integer => "i64",
                _ => return None,
            };
            Some((field, ty))
        });
    let (ty, data) = if let Some((field, ty)) = simple {
        (
            ty,
            json!({"simple": true, "member": field.rust_name, "string_key": field.value_type == ValueType::String}),
        )
    } else {
        ("&Key", json!({"simple": false}))
    };
    let expression = templates.render(
        "SQL key-argument template failed",
        include_str!("codegen/templates/key-argument.rs.tpl"),
        &data,
    )?;
    Ok((ty.into(), expression))
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

/// Build the exact generated storage definitions without importing generated app source.
///
/// This is tool-only metadata; it never installs tables, writes revisions or opens a database.
///
/// # Errors
/// Returns missing key/relation, invalid storage-length or portable-name errors,
/// SQL definition failures and incompatible foreign-key storage widths.
pub fn storage_definitions(
    schema: &Schema,
) -> stackpress_idea_transformer::Result<Vec<stackpress_sql::TableDefinition>> {
    let tables = schema
        .models()
        .iter()
        .map(|model| {
            let stored = model
                .fields
                .iter()
                .filter(|field| !matches!(field.value_type, ValueType::Model(_)))
                .collect::<Vec<_>>();
            storage_definition(schema, model, &stored)
        })
        .collect::<stackpress_idea_transformer::Result<Vec<_>>>()?;
    crate::ddl_policy::check_widths(&tables).map_err(|cause| {
        TransformError::with_source(
            stackpress_idea_transformer::TransformErrorKind::Callback,
            "generated FK storage widths disagree",
            cause,
        )
        .unwrap_or_else(|error| error)
    })?;
    Ok(tables)
}
/// Normalize one model's storage once for both emitted Rust and durable revision snapshots.
fn storage_definition(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
) -> stackpress_idea_transformer::Result<stackpress_sql::TableDefinition> {
    if !stored.iter().any(|field| field.key) {
        return Err(TransformError::new(
            "generated store requires a complete primary key",
        ));
    }
    let mut table = stackpress_sql::Create::new(&model.module);
    for field in stored {
        let kind = match codec(schema, field) {
            "Integer" => stackpress_sql::SqlType::Integer,
            "Boolean" => stackpress_sql::SqlType::Boolean,
            "Real" => stackpress_sql::SqlType::Real,
            _ if field.storage_length.is_some() => stackpress_sql::SqlType::Varchar,
            _ => stackpress_sql::SqlType::Text,
        };
        let mut column = stackpress_sql::Field::new(&field.column, kind);
        column.nullable = !field.required;
        if let Some(length) = field.storage_length {
            let length = i32::try_from(length)
                .ok()
                .filter(|length| *length > 0)
                .ok_or_else(|| TransformError::new("invalid normalized storage length"))?;
            column.length = Some(stackpress_sql::Length::Size(length));
            table = table.require_mysql_target();
        }
        table = table.add_field(column);
        if field.unique || field.searchable || field.sortable {
            let mut index = stackpress_sql::Index::new(
                constraint_name(&model.module, &field.column, "index")?,
                [&field.column],
            );
            index.unique = field.unique;
            table = table.add_index(index);
        }
    }
    table = table.primary_key(stored.iter().filter(|f| f.key).map(|f| f.column.as_str()));
    for field in &model.fields {
        if let (ValueType::Model(target), Some(relation)) = (&field.value_type, &field.relation) {
            let target = schema
                .models()
                .iter()
                .find(|m| &m.name == target)
                .ok_or_else(|| TransformError::new("relation target missing"))?;
            let local = model
                .fields
                .iter()
                .find(|f| f.name == relation.local)
                .ok_or_else(|| TransformError::new("relation source missing"))?;
            let foreign = target
                .fields
                .iter()
                .find(|f| f.name == relation.foreign)
                .ok_or_else(|| TransformError::new("relation target field missing"))?;
            let shared = model
                .fields
                .iter()
                .filter_map(|f| f.relation.as_ref())
                .filter(|r| r.local == relation.local)
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
            let mut key = stackpress_sql::ForeignKey::new(
                name,
                [&local.column],
                &target.module,
                [&foreign.column],
            );
            key.on_delete = stackpress_sql::ReferentialAction::Restrict;
            table = table.add_foreign_key(key);
        }
    }

    table.definition().map_err(|cause| {
        TransformError::with_source(
            stackpress_idea_transformer::TransformErrorKind::Callback,
            "framework storage definition failed",
            cause,
        )
        .unwrap_or_else(|error| error)
    })
}

/// Build generated MySQL DDL with explicit app-owned target facts and catalog validation.
/// The returned definitions remain render-only; importing them performs no IO.
///
/// # Errors
/// Returns [`storage_definitions`] failures and unsupported MySQL target facts,
/// storage widths or catalog/index constraints, retaining the DDL policy cause.
pub fn mysql_storage_definitions(
    schema: &Schema,
    collation: &str,
    page_bytes: u32,
) -> stackpress_idea_transformer::Result<Vec<stackpress_sql::TableDefinition>> {
    crate::ddl_policy::mysql_definitions(storage_definitions(schema)?, collation, page_bytes)
        .map_err(|cause| {
            TransformError::with_source(
                stackpress_idea_transformer::TransformErrorKind::Callback,
                "generated MySQL target validation failed",
                cause,
            )
            .unwrap_or_else(|error| error)
        })
}
