[%#?:bounded%]impl [%model%]Store {
    /// Enforce host storage declarations, including redacted private fields.
    fn storage_bounds(value: &_Stored) -> DataResult<()> {
[%#@:fields field%][%#?:field.required%]        let text = &value.[%field.member%];
        if text.chars().count() > [%field.length%]usize {
            return Err(DataError::new("declared storage length exceeded"));
        }
[%|%]        if value.[%field.member%].as_ref().is_some_and(|text| text.chars().count() > [%field.length%]usize) {
            return Err(DataError::new("declared storage length exceeded"));
        }
[%/?:field.required%][%/@:fields%]        Ok(())
    }
[%#?:patch_bounded%]    /// Check supplied encoded patch values before any write is submitted.
    fn patch_bounds(values: &[(String, stackpress_sql::SqlValue)]) -> DataResult<()> {
        for (column, value) in values {
            match column.as_str() {
[%#@:patch_fields field%]                [%field.column_literal%] => {
                    if let stackpress_sql::SqlValue::Text(text) = value
                        && text.chars().count() > [%field.length%]usize
                    {
                        return Err(DataError::new("declared storage length exceeded"));
                    }
                },
[%/@:patch_fields%]                _ => {}
            }
        }
        Ok(())
    }
[%/?:patch_bounded%]}
[%/?:bounded%]