{
    let raw=[%raw%];
    stackpress_framework_schema::check_json(&raw)?;
    if raw.is_null() {
[%#?:required%]        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid,"required default is null"));
[%|%]        None
[%/?:required%]    } else {
        let value:[%ty%]=serde_json::from_value(raw).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"default value decoding failed").with_cause(cause))?;
        [%#!:required%]Some([%/!:required%][%convert%][%#!:required%])[%/!:required%]
    }
}
