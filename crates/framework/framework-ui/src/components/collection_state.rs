//! Typed collection operations preserve stable identities, empty/null/missing and ordering.
use super::ControlError;

/// Presence is explicit; a missing field is different from null or a concrete empty value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue<T> {
    /// Field omitted by its owner.
    Missing,
    /// Explicit nullable value.
    Null,
    /// Concrete typed value, including false/zero/empty.
    Value(T),
}
/// Stable row identity is independent of position or the user's editable value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionRow<T> {
    /// Never reused inside this collection's lifetime.
    pub id: u64,
    /// Application-owned typed row value.
    pub value: T,
}
/// A bounded ordered collection. Operations never partially mutate on failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedList<T> {
    rows: Vec<CollectionRow<T>>,
    limit: usize,
    next: u64,
}
impl<T: CollectionValue> BoundedList<T> {
    /// Admit <=100 rows and unique nonzero identities, retaining document order.
    /// # Errors
    /// Rejects excessive limits/rows, duplicate IDs and exhausted identity space.
    pub fn new(rows: Vec<CollectionRow<T>>, limit: usize) -> Result<Self, ControlError> {
        // Validate the complete initial snapshot before committing identities or values.
        let mut keys = std::collections::BTreeSet::new();
        if !(1..=100).contains(&limit)
            || rows.len() > limit
            || rows.iter().any(|row| {
                row.id == 0 || !keys.insert(row.id) || !row.value.valid_collection_value()
            })
        {
            return Err(ControlError);
        }
        // Reserve above the largest admitted key; removal never makes a key reusable.
        let next = rows
            .iter()
            .map(|row| row.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ControlError)?;
        Ok(Self { rows, limit, next })
    }
    /// Exact ordered rows; callers cannot bypass identity/bounds checks through mutation.
    pub fn rows(&self) -> &[CollectionRow<T>] {
        &self.rows
    }
    /// Whether one more row can be admitted without reusing an identity.
    pub fn can_add(&self) -> bool {
        self.rows.len() < self.limit && self.next < u64::MAX
    }
    /// Append one typed value and return its stable key.
    /// # Errors
    /// Rejects the length/identity boundary without changing state.
    pub fn add(&mut self, value: T) -> Result<u64, ControlError> {
        if !self.can_add() || !value.valid_collection_value() {
            return Err(ControlError);
        }
        // Capacity and value were checked together; this publication cannot partially fail.
        let id = self.next;
        self.next += 1;
        self.rows.push(CollectionRow { id, value });
        Ok(id)
    }
    /// Replace only an existing key, never an index that may have moved.
    /// # Errors
    /// Unknown identities leave all rows unchanged.
    pub fn edit(&mut self, id: u64, value: T) -> Result<(), ControlError> {
        if !value.valid_collection_value() {
            return Err(ControlError);
        }
        let row = self
            .rows
            .iter_mut()
            .find(|row| row.id == id)
            .ok_or(ControlError)?;
        row.value = value;
        Ok(())
    }
    /// Remove an existing identity; its key is never reused by a later add.
    /// # Errors
    /// Unknown identities leave the collection unchanged.
    pub fn remove(&mut self, id: u64) -> Result<T, ControlError> {
        let index = self
            .rows
            .iter()
            .position(|row| row.id == id)
            .ok_or(ControlError)?;
        Ok(self.rows.remove(index).value)
    }
    /// Move an existing identity to a valid position, preserving every other row's order.
    /// # Errors
    /// Rejects unknown keys or positions outside the current collection.
    pub fn reorder(&mut self, id: u64, index: usize) -> Result<(), ControlError> {
        let current = self
            .rows
            .iter()
            .position(|row| row.id == id)
            .ok_or(ControlError)?;
        if index >= self.rows.len() {
            return Err(ControlError);
        }
        // Move the existing row itself, preserving its identity and every neighboring value.
        let row = self.rows.remove(current);
        self.rows.insert(index, row);
        Ok(())
    }
}
/// Editable metadata row; duplicate labels remain separate editable rows until app validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataEntry {
    /// Escaped editable key.
    pub key: String,
    /// Exact editable value, including empty/zero text.
    pub value: String,
}

/// Byte/domain admission for collection values before a mutation is committed.
/// Implement this for application row types, including validation of nested fields.
pub trait CollectionValue {
    /// Whether this owned value fits the application's bounded field contract.
    fn valid_collection_value(&self) -> bool;
}
impl CollectionValue for String {
    /// Text rows have a 4096-byte ceiling; empty and zero text remain valid.
    fn valid_collection_value(&self) -> bool {
        self.len() <= 4096
    }
}
impl CollectionValue for bool {
    /// False is a concrete admitted value.
    fn valid_collection_value(&self) -> bool {
        true
    }
}
// Fixed-size primitive values need no string-byte budget or missing-to-zero coercion.
macro_rules! integer_collection_values {
    ($($number:ty),+ $(,)?) => {$(
        impl CollectionValue for $number {
            /// Every concrete fixed-size integer, including zero, is admitted.
            fn valid_collection_value(&self) -> bool { true }
        }
    )+};
}
integer_collection_values!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl CollectionValue for f64 {
    /// Finite float values preserve zero/sign; NaN and infinity are not field values.
    fn valid_collection_value(&self) -> bool {
        self.is_finite()
    }
}
impl CollectionValue for f32 {
    /// Finite float values preserve zero/sign; NaN and infinity are not field values.
    fn valid_collection_value(&self) -> bool {
        self.is_finite()
    }
}
impl<T: CollectionValue> CollectionValue for FieldValue<T> {
    /// Missing/null stay explicit; concrete values retain their own admission rule.
    fn valid_collection_value(&self) -> bool {
        match self {
            Self::Missing | Self::Null => true,
            Self::Value(value) => value.valid_collection_value(),
        }
    }
}
impl CollectionValue for MetadataEntry {
    /// Metadata labels and values have separate bounded footprints.
    fn valid_collection_value(&self) -> bool {
        self.key.len() <= 1024 && self.value.len() <= 4096
    }
}
