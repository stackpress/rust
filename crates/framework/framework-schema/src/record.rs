//! Shared typed-record validation. Generated code delegates shape/rule policy to this pure module.

use crate::*;
use serde_json::Value;

/// Check one scalar family without coercion or ambient defaults.
pub fn value_matches(kind: &ValueType, value: &Value) -> bool {
    match kind {
        ValueType::String => value.as_str().is_some_and(|s| s.len() <= 1024 * 1024),
        ValueType::Integer => value.as_i64().is_some(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::Number => FiniteNumber::from_json(value).is_ok(),
        ValueType::Object => value.is_object() && check_json(value).is_ok(),
        ValueType::Date => value.as_str().is_some_and(|s| DateValue::parse(s).is_ok()),
        ValueType::Time => value.as_str().is_some_and(|s| TimeValue::parse(s).is_ok()),
        ValueType::Datetime => value
            .as_str()
            .is_some_and(|s| DatetimeValue::parse(s).is_ok()),
        ValueType::Enum(_) => value.is_string() || value.as_i64().is_some() || value.is_boolean(),
        ValueType::Fieldset(_) => value.is_object(),
        ValueType::Model(_) => false,
    }
}
/// Bounded runtime record schema, emitted independently of tool/private UI metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordSchema {
    /// Root declaration identity.
    pub model: Model,
    /// Acyclic embedded definitions, with no database handles or parent pointers.
    pub fieldsets: Vec<Model>,
    /// Finite scalar choice definitions.
    pub enums: Vec<Enumeration>,
}
impl RecordSchema {
    /// Validate complete values or partial input with bounded logical issues.
    /// # Errors
    /// Returns unavailable explicit host context separately from business validation issues.
    pub fn validate(
        &self,
        value: &Value,
        context: &ValueContext<'_>,
        partial: bool,
        stored: bool,
    ) -> SchemaResult<ValidationErrors> {
        check_json(value)?;
        self.validate_mode(value, context, partial, stored, false)
    }
    /// Validate creation/patch omissions using the already-normalized input requirement flags.
    pub(crate) fn validate_input(
        &self,
        value: &Value,
        context: &ValueContext<'_>,
        partial: bool,
    ) -> SchemaResult<ValidationErrors> {
        self.validate_mode(value, context, partial, false, true)
    }
    /// Check supplied literal defaults using input omissions and without time-relative/host checks.
    #[cfg(feature = "codegen")]
    pub(crate) fn validate_defaults(
        &self,
        value: &Value,
        context: &ValueContext<'_>,
    ) -> SchemaResult<ValidationErrors> {
        self.validate_mode(value, context, true, true, true)
    }
    /// Own traversal state in one place; recursive calls receive no independent mode flags.
    fn validate_mode(
        &self,
        value: &Value,
        context: &ValueContext<'_>,
        partial: bool,
        stored: bool,
        input: bool,
    ) -> SchemaResult<ValidationErrors> {
        check_json(value)?;
        let mut state = ValidationState {
            context,
            stored,
            input,
            path: Vec::new(),
            issues: ValidationErrors::default(),
        };
        self.record(&self.model, value, partial, &mut state)?;
        Ok(state.issues)
    }
    /// Traverse the checked finite embedding graph, retaining field/index paths.
    fn record(
        &self,
        model: &Model,
        value: &Value,
        partial: bool,
        state: &mut ValidationState<'_, '_>,
    ) -> SchemaResult<()> {
        let Some(object) = value.as_object() else {
            state.issues.push(&state.path, "type");
            return Ok(());
        };
        for name in object.keys() {
            if !model
                .fields
                .iter()
                .any(|f| f.name == *name && !matches!(f.value_type, ValueType::Model(_)))
            {
                // Unknown input keys are not trusted schema paths and must never be echoed.
                state.issues.push(&state.path, "unknown");
            }
        }
        // Virtual relations have no stored/input member; each scalar field owns one path scope.
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            state.path.push(field.name.clone());
            // Generated members are absent from every create input, including nested literal inputs.
            if state.input && field.generated && object.contains_key(&field.name) {
                state.issues.push(&state.path, "generated");
                state.path.pop();
                continue;
            }
            let presence = field
                .rules
                .iter()
                .any(|rule| matches!(rule.code.as_str(), "required" | "ne"));
            match object.get(&field.name) {
                None => {
                    // Partial updates omit root fields; nested supplied objects still need their inputs.
                    if !partial
                        && if state.input {
                            field.input_required
                        } else {
                            field.required || presence
                        }
                    {
                        if let Some(rule) = field
                            .rules
                            .iter()
                            .find(|rule| matches!(rule.code.as_str(), "required" | "ne"))
                        {
                            state.issues.push_rule(&state.path, rule);
                        } else {
                            state.issues.push(&state.path, "required");
                        }
                    }
                }
                Some(Value::Null) => {
                    // Explicit null never consults a default and must satisfy value/presence policy.
                    if field.required || presence {
                        if let Some(rule) = field
                            .rules
                            .iter()
                            .find(|rule| matches!(rule.code.as_str(), "required" | "ne"))
                        {
                            state.issues.push_rule(&state.path, rule);
                        } else {
                            state.issues.push(&state.path, "null");
                        }
                    }
                }
                Some(value) => {
                    if field.multiple {
                        if let Some(values) = value.as_array() {
                            if values.len() > 1024 {
                                state.issues.push(&state.path, "limit");
                            } else {
                                for (index, value) in values.iter().enumerate() {
                                    // Balance the index scope even when an element produces business issues.
                                    state.path.push(index.to_string());
                                    self.scalar(field, value, state)?;
                                    state.path.pop();
                                }
                            }
                        } else {
                            state.issues.push(&state.path, "type");
                        }
                        // Presence rules apply to the list, rather than each element.
                        for rule in &field.rules {
                            if matches!(rule.code.as_str(), "required" | "ne")
                                && !check_rule(rule, value, state.context, state.stored)?
                            {
                                state.issues.push_rule(&state.path, rule);
                            }
                        }
                    } else {
                        self.scalar(field, value, state)?;
                    }
                }
            }
            state.path.pop();
        }
        Ok(())
    }
    /// Check a nonnullable element and every selected scalar assertion.
    fn scalar(
        &self,
        field: &Field,
        value: &Value,
        state: &mut ValidationState<'_, '_>,
    ) -> SchemaResult<()> {
        if !value_matches(&field.value_type, value) {
            state.issues.push(&state.path, "type");
            return Ok(());
        }
        match &field.value_type {
            ValueType::Enum(name) => {
                if !self
                    .enums
                    .iter()
                    .any(|e| e.name == *name && e.variants.iter().any(|(_, v)| v == value))
                {
                    state.issues.push(&state.path, "option");
                }
            }
            ValueType::Fieldset(name) => {
                let model = self
                    .fieldsets
                    .iter()
                    .find(|m| m.name == *name)
                    .ok_or_else(|| {
                        SchemaError::new(SchemaErrorKind::Invalid, "missing embedded descriptor")
                    })?;
                self.record(model, value, false, state)?;
            }
            _ => {}
        }
        // Existing stored data need not repeat remote/private input checks when projected or read.
        for rule in &field.rules {
            if state.stored && !rule.client_checkable {
                continue;
            }
            if field.multiple && matches!(rule.code.as_str(), "ne" | "required") {
                continue;
            }
            if !check_rule(rule, value, state.context, state.stored)? {
                state.issues.push_rule(&state.path, rule);
            }
        }
        Ok(())
    }
}
impl ValidationErrors {
    /// Convert bounded business issues into a typed private cause without repeating values.
    pub fn into_result(self) -> SchemaResult<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(
                SchemaError::new(SchemaErrorKind::Invalid, "record validation failed")
                    .with_cause(self),
            )
        }
    }
}
impl std::fmt::Display for ValidationErrors {
    /// Public Display contains neither rejected values nor issue-dependent text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("field validation failed")
    }
}
impl std::error::Error for ValidationErrors {}

/// Exclusive traversal state; no mutable path/issue owner is duplicated across recursion.
struct ValidationState<'a, 'context> {
    /// Explicit caller clock and host functions.
    context: &'a ValueContext<'context>,
    /// Stored decoding omits creation-time relative rules.
    stored: bool,
    /// Missing values follow create-input requirements instead of stored row requirements.
    input: bool,
    /// Current logical field/index location.
    path: Vec<String>,
    /// Bounded accumulated failures.
    issues: ValidationErrors,
}
