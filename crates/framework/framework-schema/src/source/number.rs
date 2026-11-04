//! Original ColumnNumber-derived shape bounds/step, independent of source coercion.
use crate::{Field, ValueType};

/// Apply the source's implicit numeric checks before its ordered explicit refinements.
pub(super) fn assert(field: &Field, number: f64) -> Option<String> {
    if field.value_type == ValueType::Integer {
        if number.fract() != 0.0 {
            return Some("Invalid input: expected int, received number".into());
        }
        if number > 9_007_199_254_740_991.0 {
            return Some("Too big: expected int to be <=9007199254740991".into());
        }
        if number < -9_007_199_254_740_991.0 {
            return Some("Too small: expected int to be >=-9007199254740991".into());
        }
    }
    let min = bound(field, "min", &["eq", "gt", "ge"], true);
    let max = bound(field, "max", &["eq", "lt", "le"], false);
    if number < min {
        return Some(format!(
            "Too small: expected number to be >={}",
            display(min)
        ));
    }
    if max > min && number > max {
        return Some(format!("Too big: expected number to be <={}", display(max)));
    }
    let step = step(field);
    // Match Zod's decimal scaling instead of ordinary floating-point remainder (0.3/0.1).
    let scale = 10_f64.powi(decimals(number).max(decimals(step)) as i32);
    let divisor = (step * scale).round();
    if divisor == 0.0 || (number * scale).round() % divisor != 0.0 {
        return Some(format!(
            "Invalid number: must be a multiple of {}",
            display(step)
        ));
    }
    None
}

/// Explicit min/max wins. G51 keeps zero-valued assertion bounds so eq(0)/gt(0) work as named.
fn bound(field: &Field, name: &str, rules: &[&str], minimum: bool) -> f64 {
    if let Some(value) = attribute(field, name) {
        return value;
    }
    let selected: Vec<_> = field
        .rules
        .iter()
        .filter(|rule| rules.contains(&rule.code.as_str()))
        .collect();
    if selected.is_empty() {
        return 0.0;
    }
    let values = selected
        .iter()
        .filter_map(|rule| rule.args.first().and_then(serde_json::Value::as_f64));
    if minimum {
        values.fold(f64::INFINITY, f64::min)
    } else {
        values.fold(f64::NEG_INFINITY, f64::max)
    }
}

/// Original numeric attributes are methods with their number as the first argument.
fn attribute(field: &Field, name: &str) -> Option<f64> {
    field
        .attributes
        .iter()
        .find(|attribute| attribute.name == name)?
        .value
        .as_array()?
        .first()?
        .as_f64()
}

/// Number.toString switches to scientific notation outside the source decimal spelling range.
fn decimals(number: f64) -> usize {
    if !number.is_finite() || number != 0.0 && (number.abs() < 1e-6 || number.abs() >= 1e21) {
        return 0;
    }
    number
        .to_string()
        .split_once('.')
        .map(|(_, fraction)| fraction.len())
        .unwrap_or(0)
}

/// Source/Zod display uses capitalized infinities in implicit-bound diagnostics.
fn display(value: f64) -> String {
    if value == f64::INFINITY {
        "Infinity".into()
    } else if value == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        value.to_string()
    }
}

/// Original ColumnNumber step inference, shared with source SQL numeric family selection.
pub fn step(field: &Field) -> f64 {
    attribute(field, "step").unwrap_or_else(|| {
        let digits = field
            .rules
            .iter()
            .filter(|rule| {
                matches!(
                    rule.code.as_str(),
                    "eq" | "lt" | "le" | "gt" | "ge" | "max" | "min"
                )
            })
            .filter_map(|rule| rule.args.first().and_then(serde_json::Value::as_f64))
            .map(decimals)
            .max()
            .unwrap_or(0);
        10_f64.powi(-(digits as i32))
    })
}
