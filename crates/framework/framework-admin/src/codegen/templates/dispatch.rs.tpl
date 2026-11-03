/// Explicit compiled page modules in navigation order.
pub const MODELS: &[&str] = &[[%#@:models model%][%#!:@first%], [%/!:@first%][%model.module_literal%][%/@:models%]];
[%#?:has_models%]/// Resolve only compiler-owned static model identities.
pub fn identity(module: &str) -> Option<(&'static str, &'static str)> {
    match module {
[%#@:models model%]        [%model.module_literal%] => Some(([%model.module_literal%], [%model.name_literal%])),
[%/@:models%]        _ => None,
    }
}
[%|%]/// No page model is selected.
pub fn identity(_: &str) -> Option<(&'static str, &'static str)> {
    None
}
[%/?:has_models%][%#@:adapters adapter%][%#?:@root.has_models%]/// Dispatch only to an explicitly generated typed adapter.
pub fn [%adapter.name%]([%#@:adapter.parameters parameter%][%#!:@first%],[%/!:@first%][%parameter.name%]:[%parameter.ty%][%/@:adapter.parameters%]) -> SchemaResult<[%adapter.result%]> {
    match module {
[%#@:models model%]        [%model.module_literal%] => [%model.module%]::[%adapter.name%]([%#@:adapter.arguments argument%][%#!:@first%],[%/!:@first%][%argument%][%/@:adapter.arguments%]),
[%/@:models%]        _ => Err(unavailable()),
    }
}
[%|%]/// This adapter is unavailable without a selected model.
pub fn [%adapter.name%]([%#@:adapter.parameters parameter%][%#!:@first%],[%/!:@first%]_: [%parameter.ty%][%/@:adapter.parameters%]) -> SchemaResult<[%adapter.result%]> {
    Err(unavailable())
}
[%/?:@root.has_models%][%/@:adapters%]/// Missing model/role is unavailable; never substitute another operation.
fn unavailable() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "admin model or role unavailable")
}
