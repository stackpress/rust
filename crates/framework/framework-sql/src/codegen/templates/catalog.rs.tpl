/// Exact immutable runtime metadata; it grants no transport authorization.
pub static SPEC: StoreSpec = StoreSpec {
    table: [%table_literal%],
    columns: COLUMNS,
    key: &[[%#@:keys key%][%#!:@first%], [%/!:@first%][%key%][%/@:keys%]],
    active: [%#?:has_active%]Some([%active_literal%])[%|%]None[%/?:has_active%],
    fields: &[
[%#@:fields field%]        stackpress_framework_sql::stores::QueryField {
            storage: StorageColumn {
                wire: [%field.wire_literal%],
                column: [%field.column_literal%],
                codec: StorageCodec::[%field.codec%],
                required: [%field.required%],
                nonempty: [%field.nonempty%],
            },
            multiple: [%field.multiple%],
            scalar: StorageCodec::[%field.scalar_codec%],
            choices: [%field.choices_literal%],
            searchable: [%field.searchable%],
            sortable: [%field.sortable%],
            public_filter: [%field.public_filter%],
            public_sort: [%field.public_sort%],
            public_search: [%field.public_search%],
        },
[%/@:fields%]    ],
    relations: &[
[%#@:relations relation%]        stackpress_framework_sql::stores::RelationSpec {
            name: [%relation.name_literal%],
            target: [%relation.target_literal%],
            local: [%relation.local_literal%],
            foreign: [%relation.foreign_literal%],
            many: [%relation.many%],
            required: [%relation.required%],
            public: [%relation.public%],
        },
[%/@:relations%]    ],
    decode: decode_value,
    project: project_value,
};
/// Validate through the actual owned model before bounded JSON enters the include arena.
fn decode_value(row: Row) -> DataResult<serde_json::Value> {
    stackpress_framework_schema::to_bounded_value(&[%model%]Store::decode(row)?)
        .map_err(stackpress_framework_sql::stores::validation)
}
/// Project the actual model through the compiled app policy; no stored field is echoed implicitly.
fn project_value(value: serde_json::Value) -> DataResult<serde_json::Value> {
    let row: _Stored = serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)?;
    stackpress_framework_schema::to_bounded_value(&row.project().map_err(stackpress_framework_sql::stores::projection_error)?).map_err(stackpress_framework_sql::stores::validation)
}
impl [%model%]Store {
    /// Strict owned decode, including checked nested fields and stored validation.
    pub fn decode(row: Row) -> DataResult<_Stored> {
        let value: _Stored = stackpress_framework_sql::decode(row, COLUMNS)?;
[%#?:bounded%]        Self::storage_bounds(&value).map_err(|cause|
            DataError::new("stored storage length validation failed")
                .with_kind(DataErrorKind::Decode)
                .with_cause(cause)
        )?;
[%/?:bounded%]        value.validate().map_err(|cause|
            DataError::new("stored value validation failed")
                .with_kind(DataErrorKind::Decode)
                .with_cause(cause)
        )?;
        Ok(value)
    }
}
