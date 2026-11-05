//! Open native q carrier; resolution/interpolation live in the SQL source runtime.
pub use stackpress_framework_sql::source::StoreQuery;
/// Retained legacy static selector carrier; generic emitted stores expose dynamic source names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreSelector {
    /// Original expanded expression.
    pub expression: String,
    /// Native output alias.
    pub alias: String,
    /// Source field name.
    pub field: String,
    /// SQL column identity.
    pub column: String,
    /// Source table alias.
    pub table: String,
    /// Nested native record path.
    pub path: Vec<String>,
    /// JSON predicate path.
    pub json: Vec<String>,
    /// Existing finite API/default-model source identity.
    pub model: &'static str,
}
impl StoreSelector {
    pub(super) fn from_native(
        value: stackpress_framework_sql::source::StoreSelector,
        base: &'static str,
    ) -> Self {
        let model = match value.model.as_str() {
            "Profile" => "Profile",
            "Application" => "Application",
            "Session" => "Session",
            _ => base,
        };
        Self {
            model,
            expression: value.expression,
            alias: value.alias,
            field: value.field,
            column: value.column,
            table: value.table,
            path: value.path,
            json: value.json,
        }
    }
}
