//! Checked app-owned definitions. Import mappings stay on this build-host side of the boundary.

use super::{invalid, require_flag, unsupported};
use crate::{FieldAccess, Role, RoleBinding, SchemaResult, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_idea_parser::AttributeValue;
use std::collections::{BTreeMap, BTreeSet};

/// Supported component argument families, with no arbitrary executable objects.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropType {
    /// Bounded UTF-8 text.
    String,
    /// An actual Boolean.
    Boolean,
    /// Finite JSON number.
    Number,
    /// Bounded literal JSON.
    Json,
}
/// A definition-ordered method argument.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropDefinition {
    /// Exact argument/object member identity.
    pub name: String,
    /// Expected literal family.
    pub value_type: PropType,
    /// Whether the method must supply this argument.
    pub required: bool,
    /// Whether this prop is approved to cross the public descriptor boundary.
    pub public: bool,
}
/// One compiled component definition, independent of Leptos implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentDefinition {
    /// Canonical field.* or view.* name; derived families share this definition.
    pub name: String,
    /// Stable component key, never its import path.
    pub key: String,
    /// Version selected before normalization.
    pub revision: String,
    /// Accepted positional/object arguments in definition order.
    pub props: Vec<PropDefinition>,
    /// Fixed values override supplied values before validation/public selection.
    pub fixed: BTreeMap<String, Value>,
    /// True only when the application compiled a concrete implementation.
    pub available: bool,
}
/// Tool-only Rust import mapping. This type is never embedded in UiSchema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentImport {
    /// Stable component key shared with a checked definition.
    pub component: String,
    /// Compiled Rust module path.
    pub module: String,
    /// Exported Rust component symbol.
    pub symbol: String,
    /// Whether the tool resolves a default export convention.
    pub default: bool,
}
/// Registered typed host default; algorithms remain application-owned.
#[derive(Clone, Debug)]
pub struct ProviderDefinition {
    /// Finite recognized generator name, without parentheses.
    pub name: String,
    /// Stable algorithm revision included in the field's value rule.
    pub revision: String,
    /// Declared scalar output type, rechecked against actual results.
    pub output: ValueType,
}
/// Explicit non-built-in host validator, never approximated by an unrelated regex.
#[derive(Clone, Debug)]
pub struct ValidatorDefinition {
    /// Canonical assertion name without is. prefix.
    pub name: String,
    /// Stable implementation revision.
    pub revision: String,
    /// Exact accepted scalar type families.
    pub types: Vec<ValueType>,
}
/// Frozen app-owned definition catalog; builder methods consume and return owned copies.
#[derive(Clone, Debug)]
pub struct DefinitionRegistry {
    /// Legacy selected assertion versions; no global or last-writer-wins registration.
    pub(super) definitions: Vec<(String, String)>,
    /// Complete canonical role vocabulary, with unavailable entries retained explicitly.
    components: Vec<ComponentDefinition>,
    /// Canonical definitions explicitly registered by this application.
    registered: BTreeSet<String>,
    /// Aggregate application-supplied component bytes, charged before retaining definitions.
    catalog_bytes: usize,
    /// Private executable import mappings.
    imports: Vec<ComponentImport>,
    /// Explicit host default algorithms.
    pub(super) providers: Vec<ProviderDefinition>,
    /// Explicit host validation extensions.
    pub(super) validators: Vec<ValidatorDefinition>,
    /// Exact model/field policy paths.
    policies: BTreeMap<(String, String), FieldAccess>,
    /// Typed source naming overrides, scoped by exact model/field identities.
    names: BTreeMap<(String, String), String>,
    /// Declaration module/table overrides preserve the exact wire identity and Rust type name.
    declaration_names: BTreeMap<String, String>,
}
impl DefinitionRegistry {
    /// Select all built-in rules and the complete metadata-only role vocabulary.
    pub fn standard() -> Self {
        let mut catalog = Self {
            definitions: vec![("is.ne".into(), "1".into())],
            components: Vec::new(),
            registered: BTreeSet::new(),
            catalog_bytes: 0,
            imports: Vec::new(),
            providers: Vec::new(),
            validators: Vec::new(),
            policies: BTreeMap::new(),
            names: BTreeMap::new(),
            declaration_names: BTreeMap::new(),
        };
        for (family, names) in [("field", FIELD_NAMES), ("view", VIEW_NAMES)] {
            for name in names.split_whitespace() {
                catalog.components.push(ComponentDefinition {
                    name: format!("{family}.{name}"),
                    key: format!("stackpress.{family}.{name}"),
                    revision: "1".into(),
                    props: Vec::new(),
                    fixed: BTreeMap::new(),
                    available: false,
                });
            }
        }
        catalog
    }
    /// Check selected assertion revisions without replacing built-in semantics.
    pub fn checked(definitions: Vec<(String, String)>) -> SchemaResult<Self> {
        let mut names = BTreeSet::new();
        if definitions.len() > 256
            || definitions
                .iter()
                .any(|(name, revision)| !token(name) || !token(revision) || !names.insert(name))
        {
            return Err(invalid("invalid definition registry"));
        }
        let mut catalog = Self::standard();
        catalog.definitions = definitions;
        Ok(catalog)
    }
    /// Register a provider before normalization. Duplicates and unknown generator names refuse.
    pub fn with_provider(mut self, value: ProviderDefinition) -> SchemaResult<Self> {
        if !matches!(value.name.as_str(), "cuid" | "nano" | "nanoid" | "random")
            || !token(&value.revision)
            || self.providers.iter().any(|v| v.name == value.name)
        {
            return Err(invalid("invalid provider definition"));
        }
        self.providers.push(value);
        Ok(self)
    }
    /// Register an exact validator extension with explicit applicability.
    pub fn with_validator(mut self, value: ValidatorDefinition) -> SchemaResult<Self> {
        if !valid_assertion_name(&value.name)
            || super::assertions::builtin(&value.name)
            || !token(&value.revision)
            || value.types.is_empty()
            || self.validators.iter().any(|v| v.name == value.name)
        {
            return Err(invalid("invalid validator definition"));
        }
        self.validators.push(value);
        Ok(self)
    }
    /// Replace one unavailable vocabulary entry with a checked app component definition.
    pub fn with_component(
        mut self,
        value: ComponentDefinition,
        import: Option<ComponentImport>,
    ) -> SchemaResult<Self> {
        // One fixed-prop catalog cannot multiply an unbounded payload across selected fields.
        crate::to_bounded_value(&value)?;
        let bytes = serde_json::to_vec(&value)
            .map_err(|cause| invalid("component definition encoding failed").with_cause(cause))?
            .len();
        self.catalog_bytes = self
            .catalog_bytes
            .checked_add(bytes)
            .ok_or_else(super::limit)?;
        if self.catalog_bytes > 1024 * 1024 {
            return Err(super::limit());
        }
        let (role, canonical) = canonical_role(&value.name)?;
        if !matches!(role, Role::Field | Role::View)
            || value.name != canonical
            || !token(&value.key)
            || !token(&value.revision)
            || value.props.len() > 32
        {
            return Err(invalid("invalid component definition"));
        }
        let mut names = BTreeSet::new();
        for prop in &value.props {
            if !token(&prop.name) || !names.insert(&prop.name) {
                return Err(invalid("invalid component prop definition"));
            }
        }
        for (name, literal) in &value.fixed {
            validate_prop(&value, name, literal)?;
        }
        if let Some(import) = import {
            if !value.available
                || import.component != value.key
                || !rust_path(&import.module)
                || !rust_path(&import.symbol)
                || self.imports.iter().any(|v| v.component == import.component)
            {
                return Err(invalid("invalid component import"));
            }
            self.imports.push(import);
        }
        if !self.registered.insert(canonical.clone()) {
            return Err(invalid("duplicate component definition"));
        }
        if let Some(slot) = self
            .components
            .iter_mut()
            .find(|value| value.name == canonical)
        {
            *slot = value;
        } else {
            if self.components.len() >= 256 {
                return Err(invalid("component catalog limit exceeded"));
            }
            self.components.push(value);
        }
        Ok(self)
    }
    /// Register explicit field publication/access, rejecting duplicate/conflicting policies.
    pub fn with_field_policy(
        mut self,
        model: &str,
        field: &str,
        policy: FieldAccess,
    ) -> SchemaResult<Self> {
        super::portable_name(model)?;
        super::portable_name(field)?;
        if policy.sensitive && (policy.readable || policy.publish_default)
            || self
                .policies
                .insert((model.into(), field.into()), policy)
                .is_some()
        {
            return Err(invalid("invalid field access policy"));
        }
        Ok(self)
    }
    /// Register an explicit portable SQL/member name override; source wire names stay exact.
    pub fn with_name(mut self, model: &str, field: &str, name: &str) -> SchemaResult<Self> {
        let name = super::portable_name(name)?;
        if self
            .names
            .insert((model.into(), field.into()), name)
            .is_some()
        {
            return Err(invalid("duplicate naming override"));
        }
        Ok(self)
    }
    /// Override one model/fieldset module/table name without inventing an Idea attribute.
    pub fn with_model_name(mut self, source: &str, name: &str) -> SchemaResult<Self> {
        super::portable_name(source)?;
        let name = super::portable_name(name)?;
        if self.declaration_names.insert(source.into(), name).is_some() {
            return Err(invalid("duplicate declaration naming override"));
        }
        Ok(self)
    }
    /// Borrow a compiled declaration naming override.
    pub(super) fn declaration_name(&self, source: &str) -> Option<&str> {
        self.declaration_names.get(source).map(String::as_str)
    }
    /// Borrow the private catalog for a UI emitter's Rust import resolution.
    pub fn imports(&self) -> &[ComponentImport] {
        &self.imports
    }
    /// Borrow the application policy; absent entries deny public access.
    pub(super) fn access(&self, model: &str, field: &str) -> FieldAccess {
        self.policies
            .get(&(model.into(), field.into()))
            .cloned()
            .unwrap_or_default()
    }
    /// Resolve an app override without changing the source/wire identity.
    pub(super) fn name(&self, model: &str, field: &str) -> Option<&str> {
        self.names
            .get(&(model.into(), field.into()))
            .map(String::as_str)
    }
    /// Normalize one exact role, its aliases, fixed props and public argument selection.
    pub(super) fn role(&self, name: &str, args: &AttributeValue) -> SchemaResult<RoleBinding> {
        let (role, canonical) = canonical_role(name)?;
        let definition = self
            .components
            .iter()
            .find(|v| v.name == canonical)
            .ok_or_else(|| unsupported("unknown selected component"))?;
        let mut values = BTreeMap::new();
        match args {
            AttributeValue::Boolean(_) => require_flag(args)?,
            AttributeValue::Array(args) => {
                if args.len() == 1 && args[0].is_object() {
                    for (key, value) in args[0].as_object().expect("object checked") {
                        values.insert(key.clone(), value.clone());
                    }
                } else {
                    if args.len() > definition.props.len() {
                        return Err(invalid("too many component arguments"));
                    }
                    for (prop, value) in definition.props.iter().zip(args) {
                        values.insert(prop.name.clone(), value.clone());
                    }
                }
            }
        }
        // Definition-fixed values have final precedence, then all retained props are checked.
        for (name, value) in &values {
            validate_prop(definition, name, value)?;
        }
        values.extend(definition.fixed.clone());
        for (name, value) in &values {
            validate_prop(definition, name, value)?;
        }
        for prop in &definition.props {
            if prop.required && !values.contains_key(&prop.name) {
                return Err(invalid("missing component argument"));
            }
        }
        values.retain(|name, _| {
            definition
                .props
                .iter()
                .any(|prop| prop.name == *name && prop.public)
        });
        Ok(RoleBinding {
            role,
            component: definition.key.clone(),
            props: values,
            definition_revision: definition.revision.clone(),
            available: definition.available,
        })
    }
}
/// Finite canonical field catalog; aliases are normalized separately.
const FIELD_NAMES: &str = "checkbox code color country currency date datetime email file image input integer json markdown mask metadata number password phone price radio rating select slider slug small suggest switch tags textarea editor time url datelist datetimelist filelist imagelist numberlist stringlist textlist timelist fieldset relation";
/// Finite canonical view catalog; list derives this complete family.
const VIEW_NAMES: &str = "capitalize chars comma country currency date email formula line lowercase number phone price relative text transform uppercase words yesno carousel code color film html image json markdown metadata rating tags list ol spread tabular ul overflow fieldset template link rel";
/// Normalize every upstream alias and derived family without synthesizing missing selections.
fn canonical_role(name: &str) -> SchemaResult<(Role, String)> {
    let (prefix, name) = name
        .split_once('.')
        .ok_or_else(|| unsupported("invalid role name"))?;
    let (role, family) = match prefix {
        "field" => (Role::Field, "field"),
        "filter" => (Role::Filter, "field"),
        "span" => (Role::Span, "field"),
        "view" => (Role::View, "view"),
        "list" => (Role::List, "view"),
        _ => return Err(unsupported("unknown role family")),
    };
    let name = if family == "field" {
        match name {
            "taglist" => "tags",
            "string" => "input",
            "text" => "textarea",
            "float" => "number",
            "boolean" => "switch",
            "object" | "hash" => "json",
            "strings" | "texts" => "stringlist",
            "dates" => "datelist",
            "datetimes" => "datetimelist",
            "times" => "timelist",
            "integerlist" | "integers" | "floatlist" | "floats" | "numbers" => "numberlist",
            name => name,
        }
    } else {
        match name {
            "table" => "tabular",
            "taglist" => "tags",
            "clip" => "overflow",
            "string" => "text",
            "float" | "integer" => "number",
            "boolean" => "yesno",
            "datetime" => "date",
            "object" | "hash" => "json",
            "strings" | "texts" | "dates" | "datetimes" | "times" | "integers" | "floats"
            | "numbers" | "stringlist" | "textlist" | "datelist" | "datetimelist" | "timelist"
            | "integerlist" | "floatlist" | "numberlist" => "list",
            name => name,
        }
    };
    Ok((role, format!("{family}.{name}")))
}
/// Bound definition identities without interpreting them as paths or code.
fn token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}
/// Check the complete literal before cloning it into public descriptor data.
fn validate_prop(definition: &ComponentDefinition, name: &str, value: &Value) -> SchemaResult<()> {
    let prop = definition
        .props
        .iter()
        .find(|v| v.name == name)
        .ok_or_else(|| invalid("unknown component prop"))?;
    crate::check_json(value)?;
    let valid = match prop.value_type {
        PropType::String => value.is_string(),
        PropType::Boolean => value.is_boolean(),
        PropType::Number => value.as_f64().is_some_and(f64::is_finite),
        PropType::Json => true,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("component prop has incompatible type"))
    }
}

/// Compiled imports are checked Rust identities, never arbitrary emitted fragments.
fn rust_path(value: &str) -> bool {
    value.len() <= 128
        && value.split("::").all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(index, byte)| {
                    byte.is_ascii_alphabetic() || byte == b'_' || index > 0 && byte.is_ascii_digit()
                })
        })
}

/// Plugin assertion identifiers are bounded metadata names; built-ins cannot be replaced.
fn valid_assertion_name(value: &str) -> bool {
    value.len() <= 128
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(index, byte)| {
                    byte.is_ascii_lowercase() || byte == b'_' || index > 0 && byte.is_ascii_digit()
                })
        })
}
