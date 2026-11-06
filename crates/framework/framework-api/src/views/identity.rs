//! Source-admin path identities follow the projected graph's declared key order.
use serde_json::Value;
use stackpress_framework_admin::pages::PageSpec;

/// Interpolate every declared key, just as the original generated URL templates do.
/// The request takes precedence over stored data on edit/detail pages. Missing members
/// retain the source's `undefined` spelling instead of guessing a literal `id` field.
pub(super) fn path(spec: &PageSpec, data: &Value, fallback: &Value) -> Option<String> {
    (!spec.model.key.is_empty()).then(|| {
        spec.model
            .key
            .iter()
            .map(|name| match data.get(name).or_else(|| fallback.get(name)) {
                Some(Value::String(value)) => value.clone(),
                Some(Value::Number(value)) => value
                    .as_f64()
                    .map(number_text)
                    .unwrap_or_else(|| value.to_string()),
                Some(value) => value.to_string(),
                None => "undefined".into(),
            })
            .collect::<Vec<_>>()
            .join("/")
    })
}

/// Match source template interpolation, including zero and JavaScript's exponent range.
fn number_text(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let scientific = format!("{value:e}");
        if let Some((mantissa, exponent)) = scientific.split_once('e')
            && !exponent.starts_with('-')
        {
            return format!("{mantissa}e+{exponent}");
        }
        return scientific;
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Numeric IDs use the same spelling as the already accepted source table-link adapter.
    #[test]
    fn numeric_identity_interpolation() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (12.0, "12"),
            (1e21, "1e+21"),
            (1e-7, "1e-7"),
            (1e-6, "0.000001"),
        ] {
            assert_eq!(number_text(value), expected);
        }
    }
}
