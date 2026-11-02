//! SQL-owned source rendering and lexical atoms, separate from generic template semantics.
//!
//! Contexts carry encoded literals and checked schema names. They never carry a
//! preassembled function body. Render-data headroom covers JSON escaping while
//! the engine and managed writer retain their existing one-MiB output ceilings.

use serde_json::{Value, json};
use stackpress_framework_schema::{Field, Model, Schema, ValueType};
use stackpress_idea_transformer::{Result, TransformError, TransformErrorKind};
use stackpress_lib::template::{
    TemplateEngine, TemplateError, TemplateErrorKind, TemplateLimits, TemplateOptions,
};

/// One generation's immutable renderer, with no global cache or caller-owned state.
pub(super) struct Templates {
    /// The parser owns source-language-independent delimiter/work policy.
    engine: TemplateEngine,
}

impl Templates {
    /// Reserve bounded JSON-encoding headroom without widening generated output.
    pub(super) fn new() -> Result<Self> {
        let options = TemplateOptions {
            delimiters: ["[%".into(), "%]".into()],
            limits: TemplateLimits {
                max_data_bytes: 4 * 1024 * 1024,
                ..Default::default()
            },
        };
        // Retain the concrete engine cause, even for an invalid future configuration.
        let engine = TemplateEngine::new(options)
            .map_err(|cause| template_error("SQL template configuration failed", cause))?;
        Ok(Self { engine })
    }

    /// Render one actual structural asset; errors disclose its identity, never its data.
    pub(super) fn render(&self, name: &'static str, source: &str, data: &Value) -> Result<String> {
        self.engine
            .render(source, data)
            .map_err(|cause| template_error(name, cause))
    }

    /// Append only successfully rendered source, leaving writer ownership/budgets unchanged.
    pub(super) fn append(
        &self,
        output: &mut String,
        name: &'static str,
        source: &str,
        data: &Value,
    ) -> Result<()> {
        // A failing render cannot append partial source to the accumulated file.
        output.push_str(&self.render(name, source, data)?);
        Ok(())
    }
}

/// Preserve a typed template cause behind bounded static callback context.
fn template_error(name: &'static str, cause: TemplateError) -> TransformError {
    // Resource refusals must retain the writer's public Limit classification,
    // including a per-asset output refusal before OutputPlan can inspect the file.
    let kind = if cause.kind() == TemplateErrorKind::Limit {
        TransformErrorKind::Limit
    } else {
        TransformErrorKind::Callback
    };
    TransformError::with_source(kind, name, cause).unwrap_or_else(|error| error)
}

/// Encode one Rust string literal using the emitter's original Debug escaping.
pub(super) fn literal(value: &str) -> String {
    format!("{value:?}")
}

/// Encode ordered literal atoms; array syntax remains in the structural template.
pub(super) fn literals<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    values.into_iter().map(literal).collect()
}

/// Keep serialized native policy inert and preserve the original Rust literal encoding.
pub(super) fn policy_literal(policy: &impl serde::Serialize) -> Result<String> {
    let value = serde_json::to_string(policy)
        .map_err(|cause| TransformError::from(std::io::Error::other(cause)))?;
    Ok(literal(&value))
}

/// One column's metadata, shared without trimming/reusing a rendered initializer.
pub(super) fn column(schema: &Schema, field: &Field) -> Value {
    json!({
        "wire_literal": literal(&field.name),
        "column_literal": literal(&field.column),
        "codec": super::codec(schema, field),
        "required": field.required,
        "nonempty": field.nonempty,
    })
}

/// Ordered identity atoms distinguish Copy scalars from owned keys in the templates.
pub(super) fn key_records(model: &Model) -> Vec<Value> {
    model.fields.iter().filter(|field| field.key).map(|field| json!({
        "member": field.rust_name,
        "variant": super::variant(field),
        "clone": !matches!(field.value_type, ValueType::Integer | ValueType::Boolean | ValueType::Number),
    })).collect()
}
