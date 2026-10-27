//! Root-owned input state with exact missing/null/literal distinctions.

use std::collections::BTreeMap;

use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{
    Input, PublicDefault, Role, SchemaError, SchemaErrorKind, SchemaResult, UiField,
};

/// One generated model's explicitly approved controls, owned by its current Leptos root.
#[derive(Clone, Default)]
pub struct Draft {
    /// Only generated, approved field keys are admitted, independently of seed object keys.
    fields: BTreeMap<String, RwSignal<Input<Value>>>,
}

impl Draft {
    /// Create controls from public descriptors, dropping sensitive and host-only fields.
    ///
    /// Missing field values use approved literal defaults; explicit null is never defaulted.
    /// Filters start missing and do not apply creation defaults.
    /// # Errors
    /// Refuses nonobject, oversized or browser-inexact seed/default JSON and duplicate fields.
    pub fn new(fields: &[UiField], seed: &Value, role: Role) -> SchemaResult<Self> {
        stackpress_framework_schema::check_browser_json(seed)?;
        if !seed.is_object() {
            return Err(SchemaError::new(
                SchemaErrorKind::Invalid,
                "draft seed must be an object",
            ));
        }
        let mut result = Self::default();
        for field in fields.iter().filter(|field| admitted(field, role)) {
            let value = seed.get(&field.wire_name).cloned();
            // Defaults belong to creation inputs, never to query predicates or supplied null.
            let value = value.or_else(|| match (&field.default, role) {
                (PublicDefault::Literal(value), Role::Field | Role::Span) => Some(value.clone()),
                _ => None,
            });
            let input = match value {
                None => Input::Missing,
                Some(Value::Null) => Input::Null,
                Some(value) => {
                    stackpress_framework_schema::check_browser_json(&value)?;
                    Input::Value(value)
                }
            };
            if result.fields.contains_key(&field.wire_name) {
                return Err(SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "duplicate draft field",
                ));
            }
            result
                .fields
                .insert(field.wire_name.clone(), RwSignal::new(input));
        }
        Ok(result)
    }

    /// Borrow the signal for one generated, approved field; unknown fields remain absent.
    pub fn get(&self, field: &str) -> Option<RwSignal<Input<Value>>> {
        self.fields.get(field).copied()
    }

    /// Snapshot only admitted fields, retaining explicit null/false/zero/empty literals.
    /// # Errors
    /// Refuses a later edit that exceeds browser-safe JSON bounds.
    pub fn snapshot(&self) -> SchemaResult<Value> {
        let mut values = serde_json::Map::new();
        for (name, value) in &self.fields {
            match value.get_untracked() {
                Input::Missing => {}
                Input::Null => {
                    values.insert(name.clone(), Value::Null);
                }
                Input::Value(value) => {
                    values.insert(name.clone(), value);
                }
            }
        }
        let value = Value::Object(values);
        stackpress_framework_schema::check_browser_json(&value)?;
        Ok(value)
    }
}

/// Apply the same conservative presentation policy in generated and handwritten callers.
pub(crate) fn admitted(field: &UiField, role: Role) -> bool {
    !field.sensitive
        && match role {
            Role::Field | Role::Span => field.writable,
            Role::Filter => field.readable && field.filterable,
            Role::List | Role::View => field.readable,
        }
}

/// Uniform, data-only component interface selected at compile time by the app catalog.
#[derive(Clone)]
pub struct RoleContext {
    /// Unique instance/placement/field identity from the generated wrapper caller.
    pub id: String,
    /// Public descriptor for exactly this approved field.
    pub field: UiField,
    /// Independently selected placement; span retains input semantics.
    pub role: Role,
    /// Root-owned draft; display contexts are seeded only from generated Public DTOs.
    pub value: RwSignal<Input<Value>>,
}

impl RoleContext {
    /// Creation/span requiredness never turns an optional query filter into a required input.
    pub fn input_required(&self) -> bool {
        matches!(self.role, Role::Field | Role::Span) && self.field.input_required
    }

    /// Read one checked public string prop; absent values retain the component default.
    pub fn text_prop(&self, name: &str, fallback: &str) -> String {
        self.binding()
            .and_then(|binding| binding.props.get(name))
            .and_then(Value::as_str)
            .unwrap_or(fallback)
            .to_owned()
    }

    /// Read an exact Boolean prop, preserving explicit false.
    pub fn bool_prop(&self, name: &str, fallback: bool) -> bool {
        self.binding()
            .and_then(|binding| binding.props.get(name))
            .and_then(Value::as_bool)
            .unwrap_or(fallback)
    }

    /// Locate only this independently selected placement's normalized props.
    fn binding(&self) -> Option<&stackpress_framework_schema::RoleBinding> {
        self.field
            .roles
            .iter()
            .find(|binding| binding.role == self.role)
    }

    /// Produce plain escaped display text without truthiness or expression interpretation.
    pub fn text(&self) -> String {
        match self.value.get() {
            Input::Missing => String::new(),
            Input::Null => self.text_prop("null_label", "—"),
            Input::Value(Value::String(value)) => value,
            Input::Value(value) => value.to_string(),
        }
    }
}
