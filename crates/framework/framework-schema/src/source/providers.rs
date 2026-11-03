//! Native defaults use the existing explicit provider seam, retaining Date as a native value.
use super::{Value, date};
use crate::{Field, SchemaResult, ValueProviders};

/// Resolve a source field's declared default without assuming a core field name.
///
/// `now()` receives the supplied clock and remains a native Date. All other provider
/// calls use the app-owned `ValueProviders`; literal false/null/empty values stay literal.
/// Construction/normalization never calls this function implicitly.
/// # Errors
/// Propagates the selected provider's original error and cause.
pub fn default_value(
    field: &Field,
    now_millis: i64,
    providers: &dyn ValueProviders,
) -> SchemaResult<Option<Value>> {
    if let Some(rule) = &field.value_rule {
        if rule.provider == "now" && rule.args.is_empty() {
            return Ok(Some(date(now_millis)));
        }
        return providers
            .generate(&rule.provider, &rule.args)
            .map(Value::from)
            .map(Some);
    }
    Ok(field.default.clone().map(Value::from))
}
