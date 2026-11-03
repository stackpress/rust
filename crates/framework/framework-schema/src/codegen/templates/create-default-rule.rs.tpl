serde_json::from_str::<stackpress_framework_schema::ValueRule>([%rule%])
    .map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"value rule decoding failed").with_cause(cause))?
    .resolve_for(&serde_json::from_str::<stackpress_framework_schema::ValueType>([%kind%])
        .map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"provider type decoding failed").with_cause(cause))?,context)?
