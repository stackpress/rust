//! Typed, explicitly Arc-owned slots frozen before runtime publication.

use stackpress_lib::{Error, Result, SourceLocation};
use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::Arc;

/// One contribution retains both its original and deliberate replacement provenance.
struct Slot {
    /// Sharing is explicit; the registry never clones the payload itself.
    value: Arc<dyn Any + Send + Sync>,
    /// First registration is retained across replacement.
    registered: SourceLocation,
    /// Last explicit replacement, if any.
    replaced: Option<SourceLocation>,
}

/// Owned registry with separate singleton and deliberately named namespaces.
#[derive(Default)]
pub(crate) struct Registry {
    /// Concrete type is the singleton authority.
    values: HashMap<TypeId, Slot>,
    /// A name identifies one checked type; a wrong type never aliases another slot.
    named: HashMap<String, Slot>,
}

impl Registry {
    /// Insert a singleton without silently replacing an earlier contribution.
    #[track_caller]
    pub(crate) fn insert<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Check before mutation; earlier ownership survives a duplicate error.
        if self.values.contains_key(&TypeId::of::<T>()) {
            return Err(Error::invalid(format!(
                "duplicate typed value: {}",
                type_name::<T>()
            )));
        }
        self.values.insert(TypeId::of::<T>(), Slot::new(value));
        Ok(())
    }

    /// Insert one named instance, rejecting name conflicts even across types.
    #[track_caller]
    pub(crate) fn insert_named<T: Any + Send + Sync>(
        &mut self,
        name: &str,
        value: Arc<T>,
    ) -> Result<()> {
        // Names appear in diagnostics, so reject unbounded/control-bearing input.
        crate::plugin::identity(name)?;
        if self.named.contains_key(name) {
            return Err(Error::invalid(format!("duplicate service name: {name}")));
        }
        self.named.insert(name.into(), Slot::new(value));
        Ok(())
    }

    /// Replace an existing singleton deliberately while preserving its original site.
    #[track_caller]
    pub(crate) fn replace<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Missing slots cannot be created accidentally through the customization API.
        let slot = self
            .values
            .get_mut(&TypeId::of::<T>())
            .ok_or_else(|| missing::<T>(None))?;
        slot.value = value;
        slot.replaced = Some(SourceLocation::caller());
        Ok(())
    }

    /// Borrow one singleton without acquiring any lock.
    pub(crate) fn get<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Checked downcasting guards the private insertion invariant as well as missing types.
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|s| s.value.downcast_ref())
            .ok_or_else(|| missing::<T>(None))
    }

    /// Borrow a named service only if the name's concrete type also agrees.
    pub(crate) fn named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T> {
        // Wrong types are lookup failures rather than casts or alternate named allocations.
        self.named
            .get(name)
            .and_then(|s| s.value.downcast_ref())
            .ok_or_else(|| missing::<T>(Some(name)))
    }

    /// Retain the exact singleton allocation through an explicitly returned Arc.
    pub(crate) fn shared<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        // Arc downcast preserves allocation identity and never invokes T::clone.
        self.values
            .get(&TypeId::of::<T>())
            .map(|s| Arc::clone(&s.value))
            .and_then(|v| v.downcast().ok())
            .ok_or_else(|| missing::<T>(None))
    }

    /// Inspect singleton contribution sites without exposing payloads or mutation.
    pub(crate) fn sites<T: Any + Send + Sync>(
        &self,
    ) -> Result<(&SourceLocation, Option<&SourceLocation>)> {
        // Both sites refer to stable immutable registry entries after Config.
        let slot = self
            .values
            .get(&TypeId::of::<T>())
            .ok_or_else(|| missing::<T>(None))?;
        Ok((&slot.registered, slot.replaced.as_ref()))
    }
}

impl Slot {
    /// Record the synchronous caller before erasing its explicitly shared value.
    #[track_caller]
    fn new<T: Any + Send + Sync>(value: Arc<T>) -> Self {
        // No callback or runtime owner enters the slot implicitly.
        Self {
            value,
            registered: SourceLocation::caller(),
            replaced: None,
        }
    }
}

/// Produce a typed lookup error without formatting any service contents.
fn missing<T: 'static>(name: Option<&str>) -> Error {
    // Names and types identify authority; registry payloads remain private.
    Error::new(format!(
        "missing or wrong-typed value: {}, name {name:?}",
        type_name::<T>()
    ))
}
