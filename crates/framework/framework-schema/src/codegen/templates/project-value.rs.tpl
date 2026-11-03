[%#?:fieldset%][%#?:multiple%][%#?:required%]self.[%member%]
    .iter()
    .map(|value| value.project())
    .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?
[%|%]self.[%member%]
    .as_ref()
    .map(|values| values.iter().map(|value| value.project())
        .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>())
    .transpose()?
[%/?:required%][%|%][%#?:required%]self.[%member%].project()?
[%|%]self.[%member%].as_ref().map(|value| value.project()).transpose()?
[%/?:required%][%/?:multiple%][%|%][%#?:copy%]self.[%member%]
[%|%]self.[%member%].clone()
[%/?:copy%][%/?:fieldset%]