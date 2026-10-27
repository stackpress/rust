//! Public-only display labels and validation wording; no rejected value is echoed.
use serde_json::Value;
use stackpress_framework_schema::{FieldIssue, UiField, UiModel};

/// Human result link from approved name/title fields, otherwise the ordered key values.
pub fn record_label(model: &UiModel, row: &Value) -> String {
    for name in ["name", "title", "label"] {
        if model
            .fields
            .iter()
            .any(|f| f.wire_name == name && f.readable && !f.sensitive)
            && let Some(text) = row
                .get(name)
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
        {
            return text.to_owned();
        }
    }
    let keys = model
        .key
        .iter()
        .filter_map(|name| row.get(name))
        .filter_map(|v| match v {
            Value::String(v) => Some(v.clone()),
            Value::Number(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if keys.is_empty() {
        "View record".into()
    } else {
        format!("View {}", keys.join(" · "))
    }
}

/// Use an actual public rule message first, then derive bounded specific wording from its code.
pub fn issue_message(field: &UiField, issue: &FieldIssue) -> String {
    let rule = field.validation.iter().find(|rule| rule.code == issue.code);
    let message = rule
        .map(|r| r.message.as_str())
        .filter(|m| !generic(m))
        .or_else(|| (!generic(&issue.message)).then_some(issue.message.as_str()));
    if let Some(message) = message {
        return message.into();
    }
    let label = &field.label;
    match issue.code.as_str() {
        "required" | "ne" => format!("{label} is required"),
        "null" => format!("{label} cannot be empty"),
        "cle" | "clt" | "cge" | "cgt" | "ceq" => {
            let count = rule.and_then(|r| r.args.first()).and_then(Value::as_u64);
            if let Some(mut count) = count {
                let phrase = match issue.code.as_str() {
                    "cle" => "at most",
                    "clt" => {
                        count = count.saturating_sub(1);
                        "at most"
                    }
                    "cge" => "at least",
                    "cgt" => {
                        count = count.saturating_add(1);
                        "at least"
                    }
                    _ => "exactly",
                };
                format!("{label} must contain {phrase} {count} characters")
            } else {
                format!("{label} has an invalid length")
            }
        }
        "limit" => format!("{label} exceeds the allowed size"),
        "integer" | "number" | "float" => format!("{label} must be a valid number"),
        "date" => format!("{label} must be a valid date"),
        _ => format!("Enter a valid value for {label}"),
    }
}
fn generic(message: &str) -> bool {
    message.is_empty() || matches!(message, "field validation failed" | "validation failed")
}

/// Enrich a public envelope's issues without consulting native causes or input values.
pub fn issues(model: &UiModel, mut issues: Vec<FieldIssue>) -> Vec<FieldIssue> {
    for issue in &mut issues {
        if let Some(field) = model
            .fields
            .iter()
            .find(|field| Some(&field.wire_name) == issue.path.first())
        {
            issue.message = issue_message(field, issue);
        }
    }
    issues
}

/// Recover specific public rule wording when a transport collapses its failure to `invalid`.
/// Only the submitted public snapshot and locally checkable rules participate; input is not echoed.
pub fn submitted_issues(
    model: &UiModel,
    mut issues: Vec<FieldIssue>,
    values: &Value,
) -> Vec<FieldIssue> {
    for issue in &mut issues {
        if issue.path.len() != 1 || issue.code != "invalid" || !generic(&issue.message) {
            continue;
        }
        let Some(field) = model
            .fields
            .iter()
            .find(|f| Some(&f.wire_name) == issue.path.first())
        else {
            continue;
        };
        match values.get(&field.wire_name) {
            None if field.input_required => issue.code = "required".into(),
            Some(Value::Null) if !field.nullable => issue.code = "null".into(),
            Some(value) => {
                // Reuse schema's canonical checker; unavailable host rules remain generic.
                if let Some(rule) = field.validation.iter().find(|rule| {
                    rule.client_checkable
                        && stackpress_framework_schema::check_rule(
                            rule,
                            value,
                            &Default::default(),
                            false,
                        )
                        .is_ok_and(|valid| !valid)
                }) {
                    issue.code = rule.code.clone();
                }
            }
            _ => {}
        }
    }
    self::issues(model, issues)
}
