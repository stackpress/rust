[%#?:published%]serde_json::from_str::<serde_json::Value>([%literal%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"literal default decoding failed").with_cause(cause))?
[%|%]context.providers.ok_or_else(||stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Unsupported,"explicit host defaults are required"))?.generate([%provider%],&[])?
[%/?:published%]
