//! Native source columns; shared mechanics live at the schema owner.
mod model;
mod schema;
#[cfg(feature = "sql")]
pub(crate) use schema::now;
pub use schema::{Record, RecordSchema};
pub use stackpress_framework_schema::source::{
    Column, Kind, Value, as_date, date, date_millis, date_text, date_with_display, to_json,
};
// SQL/auth flows use these private compatibility paths; native schemas use the lower owner.
#[cfg(feature = "sql")]
pub(crate) use stackpress_framework_schema::source::{string, truthy};
