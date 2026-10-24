//! Bounded explicit JSON layers and package-owned typed sections, without process reads.

use crate::builder::{BuildError, BuildResult, StartupStage};
use crate::services::Registry;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use stackpress_lib::{Error, Nest, ParseLimits, Result, SourceLocation};
use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

/// A validated input layer retains its actual external registration site.
#[derive(Clone)]
struct Layer {
    /// Printable input identity, never a config-document dump.
    label: String,
    /// Owned bounded object input.
    value: Value,
    /// Synchronous caller before build/decoding starts.
    registered: SourceLocation,
}

/// Ordered named object layers supplied by application code or a CLI data-file adapter.
#[derive(Default)]
pub struct ConfigLayers {
    /// Retained ordered inputs cannot select uncompiled code.
    layers: Vec<Layer>,
    /// Aggregate encoded input size prevents growth through many small layers.
    bytes: usize,
    /// Last accepted merge is replaced atomically after every validation succeeds.
    merged: Map<String, Value>,
}

impl ConfigLayers {
    /// Start with no external configuration or hidden environment/file reads.
    pub fn new() -> Self {
        // Applications explicitly supply every non-default input.
        Self::default()
    }
    /// Parse and retain one named JSON object under finite input/depth/node budgets.
    #[track_caller]
    pub fn json(&mut self, label: &str, text: &str) -> Result<&mut Self> {
        crate::plugin::identity(label)?;
        // Bound raw input before parsing; malformed text is not included in the diagnostic.
        if text.len() > ParseLimits::default().max_bytes {
            return Err(Error::limit("configuration input exceeds byte limit").at_path(label));
        }
        let value = serde_json::from_str(text)
            .map_err(|_| Error::invalid("invalid configuration JSON").at_path(label))?;
        self.value(label, value)
    }
    /// Validate and atomically append a data-only object layer in supplied order.
    #[track_caller]
    pub fn value(&mut self, label: &str, value: Value) -> Result<&mut Self> {
        crate::plugin::identity(label)?;
        // Foundation validation bounds depth and nodes before recursive merge walks the value.
        let value = checked_object(value).map_err(|e| e.at_path(label))?;
        let bytes = self
            .bytes
            .checked_add(serde_json::to_vec(&value)?.len())
            .ok_or_else(|| Error::limit("configuration input total overflow"))?;
        if bytes > ParseLimits::default().max_bytes || self.layers.len() >= 1024 {
            return Err(Error::limit("aggregate configuration layers exceed limits").at_path(label));
        }
        let mut candidate = Value::Object(self.merged.clone());
        merge(&mut candidate, value.clone());
        validate(&candidate).map_err(|e| e.at_path(label))?;
        // Every fallible check precedes publication, leaving prior accepted layers untouched.
        self.merged = candidate
            .as_object()
            .cloned()
            .ok_or_else(|| Error::invalid("config root"))?;
        self.layers.push(Layer {
            label: label.into(),
            value,
            registered: SourceLocation::caller(),
        });
        self.bytes = bytes;
        Ok(self)
    }
    /// Borrow ordered inputs without serializing or logging their contents.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &Value)> {
        // Callers can inspect provenance without cloning the whole accumulator.
        self.layers
            .iter()
            .map(|layer| (layer.label.as_str(), &layer.value))
    }
    /// Stage composition without changing the receiver on an aggregate-budget failure.
    pub(crate) fn combined(&self, other: Self) -> Result<Self> {
        let mut candidate = Self {
            layers: self.layers.clone(),
            bytes: self.bytes,
            merged: self.merged.clone(),
        };
        // The candidate owns every tentative append until all budgets have passed.
        candidate.append(other)?;
        Ok(candidate)
    }

    /// Move another explicit input sequence after existing inputs, checking all budgets first.
    pub(crate) fn append(&mut self, other: Self) -> Result<()> {
        // Stage the complete append so a late invalid aggregate leaves this owner unchanged.
        let mut candidate = Self {
            layers: self.layers.clone(),
            bytes: self.bytes,
            merged: self.merged.clone(),
        };
        for layer in other.layers {
            candidate.value(&layer.label, layer.value)?;
            // Composition preserves the original external site rather than this append helper.
            candidate
                .layers
                .last_mut()
                .expect("just appended layer")
                .registered = layer.registered;
        }
        *self = candidate;
        Ok(())
    }
}

/// One section's decoding operation with no runtime callback or emitter reference.
type Decode = Box<dyn FnOnce(Value, &mut Registry) -> Result<()> + Send>;

/// Registered section metadata retained until a single startup decoding pass.
pub(crate) struct Section {
    /// Literal top-level name, independent from the concrete type.
    key: String,
    /// One value per concrete type; newtypes distinguish roles.
    kind: TypeId,
    /// Human-readable type for diagnostics, never the section payload.
    type_name: &'static str,
    /// Lowest-precedence serialized defaults.
    defaults: Value,
    /// Owning selected plugin, if any.
    plugin: Option<String>,
    /// Real section contribution site.
    registered: SourceLocation,
    /// Consuming decoder and validator runs exactly once.
    decode: Decode,
}

/// Construction-only typed configuration authority.
#[derive(Default)]
pub(crate) struct Sections(Vec<Section>);

impl Sections {
    /// Claim one key and type without publishing conflicting or over-budget defaults.
    #[track_caller]
    pub(crate) fn register<T>(
        &mut self,
        key: &str,
        defaults: T,
        validator: fn(&T) -> Result<()>,
        plugin: Option<String>,
    ) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
    {
        crate::plugin::identity(key)?;
        if self
            .0
            .iter()
            .any(|s| s.key == key || s.kind == TypeId::of::<T>())
        {
            return Err(Error::invalid(format!(
                "duplicate configuration section/key/type: {key}"
            )));
        }
        // Validate serialized defaults as a section object wrapper before retaining them.
        let value = serde_json::to_value(defaults).map_err(|_| {
            Error::invalid("configuration defaults serialization failed").at_path(key)
        })?;
        let mut wrapper = Map::new();
        wrapper.insert(key.to_owned(), value);
        // Pass ownership to lib's iterative checks before any host-side deep clone.
        let mut wrapper = checked_object(Value::Object(wrapper))?;
        let value = wrapper
            .as_object_mut()
            .expect("checked defaults wrapper")
            .remove(key)
            .expect("inserted default section");
        self.0.push(Section {
            key: key.into(),
            kind: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            defaults: value,
            plugin,
            registered: SourceLocation::caller(),
            decode: Box::new(move |value, registry| {
                // Deserializer details are private causes; the public diagnostic never dumps data.
                let typed: T = serde_json::from_value(value).map_err(|cause| {
                    Error::invalid("configuration section decoding failed").with_cause(cause)
                })?;
                validator(&typed)?;
                registry.insert(Arc::new(typed))
            }),
        });
        Ok(())
    }

    /// Check cross-builder singleton claims before moving any contributions.
    pub(crate) fn compatible(&self, other: &Self) -> Result<()> {
        // Type and literal name are independent singleton authorities.
        if other.0.iter().any(|new| {
            self.0
                .iter()
                .any(|old| old.key == new.key || old.kind == new.kind)
        }) {
            return Err(Error::invalid(
                "duplicate composed configuration section/type",
            ));
        }
        Ok(())
    }

    /// Move preflighted sections with their original owner and source provenance.
    pub(crate) fn extend(&mut self, other: Self) {
        // The builder checks compatibility before this infallible ownership transfer.
        self.0.extend(other.0);
    }

    /// Decode and validate all owned sections before any asynchronous startup callback.
    pub(crate) fn decode(self, layers: ConfigLayers, registry: &mut Registry) -> BuildResult<()> {
        let mut merged = Value::Object(Map::new());
        let mut last_layers = HashMap::new();
        // Defaults are one lowest layer, including sections selected by plugin registration.
        for section in &self.0 {
            merged
                .as_object_mut()
                .expect("object accumulator")
                .insert(section.key.clone(), section.defaults.clone());
            // Aggregate default budgets can fail even when each section fits independently.
            validate(&merged).map_err(|error| section.failure(error, "defaults"))?;
        }
        for layer in layers.layers {
            let Layer {
                label,
                value,
                registered,
            } = layer;
            let layer_failure = |error: Error| BuildError {
                stage: StartupStage::Config,
                plugin: None,
                label: format!("configuration layer {label}"),
                registered: registered.clone(),
                error,
            };
            // Reject unclaimed roots with their actual input layer before any validator runs.
            for key in value.as_object().expect("validated layer").keys() {
                // Unclaimed input keys are diagnostic identities too; never format raw controls.
                crate::plugin::identity(key).map_err(&layer_failure)?;
                if !self.0.iter().any(|section| section.key == *key) {
                    return Err(layer_failure(Error::invalid(format!(
                        "unclaimed configuration section: {key}, layer {label}"
                    ))));
                }
                last_layers.insert(key.clone(), label.clone());
            }
            merge(&mut merged, value);
            validate(&merged).map_err(layer_failure)?;
        }
        for section in self.0 {
            let value = merged
                .as_object_mut()
                .expect("object accumulator")
                .remove(&section.key)
                .expect("registered default section");
            // All typed values are private until every decoder and validator succeeds.
            let last_layer = last_layers
                .get(&section.key)
                .map_or("defaults", String::as_str);
            (section.decode)(value, registry).map_err(|error| BuildError {
                stage: StartupStage::Config,
                plugin: section.plugin,
                label: format!(
                    "configuration {} ({}) from {last_layer}",
                    section.key, section.type_name
                ),
                registered: section.registered,
                error: error.at_path(section.key),
            })?;
        }
        Ok(())
    }
}

impl Section {
    /// Attribute aggregate defaults validation to the section that exceeded the budget.
    fn failure(&self, error: Error, layer: &str) -> BuildError {
        // A real contributor/site exists for defaults; no executing task is invented.
        BuildError {
            stage: StartupStage::Config,
            plugin: self.plugin.clone(),
            label: format!(
                "configuration {} ({}) from {layer}",
                self.key, self.type_name
            ),
            registered: self.registered.clone(),
            error: error.at_path(&self.key),
        }
    }
}

/// Consume external/default input through lib's iterative budgets before recursive cloning.
fn checked_object(value: Value) -> Result<Value> {
    // The foundation checks raw JSON by borrow before recursively building bounded rich values.
    let checked = Nest::from_object(value)?.to_json()?;
    if serde_json::to_vec(&checked)?.len() > ParseLimits::default().max_bytes {
        return Err(Error::limit(
            "configuration encoded output exceeds byte limit",
        ));
    }
    Ok(checked)
}

/// Validate JSON object size, nodes and depth using the existing foundation policy.
fn validate(value: &Value) -> Result<()> {
    // Only already-checked trees reach merge validation; a combined clone is still depth-bounded.
    // Encoded punctuation contributes to the output budget as well as string payloads.
    Nest::from_object(value.clone())?;
    if serde_json::to_vec(value)?.len() > ParseLimits::default().max_bytes {
        return Err(Error::limit("merged configuration exceeds byte limit"));
    }
    Ok(())
}

/// Objects recursively merge; arrays, scalars and null replace the previous value.
fn merge(target: &mut Value, incoming: Value) {
    // Both inputs were depth-checked before this bounded recursive walk.
    if let (Value::Object(old), Value::Object(new)) = (&mut *target, &incoming) {
        for (key, value) in new {
            match old.get_mut(key) {
                Some(existing) => merge(existing, value.clone()),
                None => {
                    old.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        // Null is a real replacement; it is never interpreted as deletion.
        *target = incoming;
    }
}
