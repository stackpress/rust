/// Select named capabilities explicitly; services must exist before this Listen callback.
pub fn register(start: &mut StartupContext, registration: &mut Registration, prefix: &str, operations: &[Operation]) -> Result<Vec<ListenerId>> {
    start.service::<stackpress_sql::Engine>()?;
    start.service::<Policy>()?;
[%#!:mutable%]    if operations.iter().any(|operation| matches!(operation, Operation::Update | Operation::Upsert)) {
        return Err(operations::Error::invalid("model declares no mutable patch fields"));
    }
[%/!:mutable%][%#!:active%]    if operations.iter().any(|operation| matches!(operation, Operation::Remove | Operation::Restore)) {
        return Err(operations::Error::invalid("soft removal requires a Boolean active model"));
    }
[%/!:active%]    registration.claim(prefix, operations)?;
    let mut handles = Vec::new();
    for operation in operations {
        let event = format!("{prefix}.{}", operation.as_str());
        let handle = match operation {
[%#@:operations operation%]            Operation::[%operation.variant%] => start.on(&event, [%operation.name%])?,
[%/@:operations%][%#!:active%]            Operation::Remove | Operation::Restore => return Err(operations::Error::invalid("soft removal is unavailable")),
[%/!:active%]        };
        handles.push(handle);
    }
    Ok(handles)
}
}
