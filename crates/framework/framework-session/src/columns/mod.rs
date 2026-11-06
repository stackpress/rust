//! Native source columns; shared mechanics live at the schema owner.
pub use stackpress_framework_schema::source::SourceRecordSchema as RecordSchema;
#[cfg(feature = "sql")]
pub(crate) use stackpress_framework_schema::source::now;
pub use stackpress_framework_schema::source::{
    Kind, Record, SourceColumn as Column, SourceSchema, Value, as_date, date, date_millis,
    date_text, date_with_display, to_json,
};
// SQL/auth flows use these private compatibility paths; native schemas use the lower owner.
#[cfg(feature = "sql")]
pub(crate) use stackpress_framework_schema::source::{string, truthy};
