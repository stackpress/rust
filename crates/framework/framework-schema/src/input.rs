//! Public/trusted input validation and bounded literal preparation, separate from stored decoding.

use crate::*;
use serde_json::Value;

/// Runtime input policy, emitted with only approved literal defaults and no private source data.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputSchema {
    /// Checked finite value types and input requirements.
    pub record: RecordSchema,
    /// Whether omission preserves existing fields and the update must be nonempty.
    pub patch: bool,
}
impl InputSchema {
    /// Validate input without generating defaults or acquiring resources.
    /// Client-only validation skips explicitly host-only rules; it is never host authorization.
    pub fn validate(
        &self,
        value: &Value,
        context: &ValueContext<'_>,
        client_only: bool,
    ) -> SchemaResult<ValidationErrors> {
        check_json(value)?;
        to_bounded_value(&self.record)?;
        let mut record = self.record.clone();
        if client_only {
            for model in std::iter::once(&mut record.model).chain(&mut record.fieldsets) {
                for field in &mut model.fields {
                    field.rules.retain(|rule| rule.client_checkable);
                }
            }
        }
        let mut issues = ValidationErrors::default();
        if self.patch && value.as_object().is_some_and(|value| value.is_empty()) {
            issues.push(&[], "empty");
        }
        issues.issues.extend(
            record
                .validate_input(value, context, self.patch)?
                .issues
                .into_iter()
                .take(64 - issues.issues.len()),
        );
        Ok(issues)
    }
    /// Apply only approved literal defaults, preserving explicit null, then validate.
    /// Host-generated values remain omitted for later explicit host preparation.
    pub fn prepare(&self, mut value: Value, context: &ValueContext<'_>) -> SchemaResult<Value> {
        check_json(&value)?;
        let mut budget = crate::values::JsonBudget::default();
        budget.charge(&value)?;
        self.defaults(&self.record.model, &mut value, 0, &mut 10_000, &mut budget)?;
        self.validate(&value, context, false)?.into_result()?;
        Ok(value)
    }
    /// Fill the finite embedding graph; validation still rejects unknown fields and wrong shapes.
    fn defaults(
        &self,
        model: &Model,
        value: &mut Value,
        depth: usize,
        remaining: &mut usize,
        budget: &mut crate::values::JsonBudget,
    ) -> SchemaResult<()> {
        if depth > 32 || *remaining == 0 {
            return Err(SchemaError::new(
                SchemaErrorKind::Limit,
                "input default traversal budget exceeded",
            ));
        }
        *remaining -= 1;
        let Some(values) = value.as_object_mut() else {
            return Ok(());
        };
        for field in &model.fields {
            if !self.patch
                && !values.contains_key(&field.name)
                && let Some(default) = &field.default
            {
                budget.charge_bytes(field.name.len())?;
                budget.charge(default)?;
                values.insert(field.name.clone(), default.clone());
            }
            if let (ValueType::Fieldset(name), Some(value)) =
                (&field.value_type, values.get_mut(&field.name))
            {
                let model = self
                    .record
                    .fieldsets
                    .iter()
                    .find(|model| model.name == *name)
                    .ok_or_else(|| {
                        SchemaError::new(SchemaErrorKind::Invalid, "missing input fieldset")
                    })?;
                if field.multiple {
                    if let Some(values) = value.as_array_mut() {
                        for value in values {
                            self.defaults(model, value, depth + 1, remaining, budget)?;
                        }
                    }
                } else {
                    self.defaults(model, value, depth + 1, remaining, budget)?;
                }
            }
        }
        Ok(())
    }
}
