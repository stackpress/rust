impl stackpress_framework_schema::source::SourceField<stackpress_framework_schema::source::[%marker%]> for [%schema_type%] {
    type FieldValue = [%field_type%];
    fn read(record: &stackpress_framework_schema::source::Record) -> Option<Self::FieldValue> {
        <Self::FieldValue as stackpress_framework_schema::source::NativeField>::read(record.get([%key%])?)
    }
    fn write(record: &mut stackpress_framework_schema::source::Record, value: Self::FieldValue) {
        record.insert([%key%].into(), stackpress_framework_schema::source::NativeField::value(value));
    }
}
impl stackpress_framework_schema::source::SourceDeclaration<stackpress_framework_schema::source::[%marker%]> for [%schema_type%] { type Declaration = [%declaration%]; }
