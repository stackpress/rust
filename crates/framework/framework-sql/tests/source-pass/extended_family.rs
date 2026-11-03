//! The application extension remains in the generated complete descriptor and record.
#[allow(dead_code)]
#[path = "../source-ui/native/extended.rs"]
mod generated;
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_schema::source::{Name, Record, SourceField};
use stackpress_framework_sql::source::{ApiModels, SourceModel};
fn family<M: ApiModels>() {}
fn main() {
    family::<generated::Family>();
    assert!(
        generated::Profile::describe()
            .fields
            .iter()
            .any(|field| field.name == "nickname")
    );
    let mut record = Record::new();
    record.insert("nickname".into(), "kept".into());
    <generated::ProfileSchema as SourceField<Name>>::write(&mut record, "Ada".into());
    assert_eq!(record["nickname"], "kept");
    assert_eq!(
        record.keys().map(String::as_str).collect::<Vec<_>>(),
        ["nickname", "name"]
    );
}
