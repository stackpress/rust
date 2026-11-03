{
[%#?:nested%]    budget.container([%wire%])?;
[%#?:multiple%]    if value.len()>1024 {
        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Limit,"list element budget exceeded"));
    }
    value.into_iter().map(|input|input.prepare_bounded(context,budget)).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?
[%|%]    value.prepare_bounded(context,budget)?
[%/?:multiple%][%|%]    budget.charge([%wire%],&value)?;
    value
[%/?:nested%]}
