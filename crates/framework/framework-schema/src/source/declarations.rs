//! Declaration witnesses distinguish native payloads that deliberately share a representation.
use super::SourceSchema;
use std::marker::PhantomData;

/// Generated declaration identity for a core field, independent of its transferable payload.
/// A relation target is not inferred from its Record shape; the emitter uses the actual source.
pub trait SourceDeclaration<Name>: SourceSchema {
    /// Exact declaration family, including nullable/list and target identity.
    type Declaration: Send + Sync + 'static;
}
/// Source Unknown identity, never repaired into a core concrete field type.
pub enum UnknownType {}
/// Source text (String/Text have the same plugin-visible type).
pub enum StringType {}
/// Source integer.
pub enum IntegerType {}
/// Source number.
pub enum NumberType {}
/// Source Boolean.
pub enum BooleanType {}
/// Source object (Hash/Object/Json).
pub enum ObjectType {}
/// A declared enum is distinct from an unrestricted core String declaration.
pub enum EnumType {}
/// An embedded fieldset is distinct from an unrestricted core Hash declaration.
pub enum FieldsetType {}
/// Source calendar Date, distinct from Datetime even though both use native Dates.
pub enum DateType {}
/// Source Time.
pub enum TimeType {}
/// Source Datetime.
pub enum DatetimeType {}
/// Relation to the generated Profile role.
pub enum ProfileType {}
/// Relation to the generated Auth role.
pub enum AuthType {}
/// Relation to the generated Application role.
pub enum ApplicationType {}
/// Relation to the generated API Session role.
pub enum SessionType {}
/// A different declared model cannot masquerade as a framework core relation target.
pub enum OtherModelType {}
/// Source list declaration; no runtime container or model is synthesized.
pub struct List<T>(PhantomData<T>);
/// Source nullable declaration, independent of a missing unprepared record member.
pub struct Nullable<T>(PhantomData<T>);
