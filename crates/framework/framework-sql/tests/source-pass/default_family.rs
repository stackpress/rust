//! All core fields compile from the full unmodified shipped graph.
#[allow(dead_code)]
#[path = "../source-ui/native/generated.rs"]
mod generated;
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_schema::source::{Created, Date, Record, Roles, SourceField, date};
use stackpress_framework_sql::source::{ApiModels, SourceModel};
fn family<M: ApiModels>() {}
fn main() {
    family::<generated::Family>();
    let mut record = Record::new();
    <generated::ProfileSchema as SourceField<Roles>>::write(&mut record, vec!["USER".into()]);
    <generated::ProfileSchema as SourceField<Created>>::write(
        &mut record,
        Date {
            millis: 42,
            display: None,
        },
    );
    assert_eq!(record["created"], date(42));
    assert_eq!(
        <generated::ProfileSchema as SourceField<Roles>>::read(&record),
        Some(vec!["USER".into()])
    );
    assert_eq!(generated::Profile::describe().fields.len(), 13);
}
