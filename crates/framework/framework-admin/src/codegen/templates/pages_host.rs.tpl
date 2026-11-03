//! Explicit native compiled admin registration.
use stackpress_framework_sql::operations::{[%#?:has_models%]Operation,[%/?:has_models%]Registration,StartupContext,Result};
/// Select the actual generated operation handlers; no handwritten operation substitute.
pub fn operations([%#?:has_models%]start[%|%]_start[%/?:has_models%]: &mut StartupContext, [%#?:has_models%]marker[%|%]_marker[%/?:has_models%]: &mut Registration) -> Result<()> {
[%#@:models model%]    crate::generated::stores::operations::[%model.native_module%]::register(start, marker, [%model.module_literal%], &[
        Operation::Get,
        Operation::Detail,
        Operation::Search,
        Operation::Create,
        Operation::Update,
        Operation::Batch[%#?:model.soft_remove%],
        Operation::Remove,
        Operation::Restore[%/?:model.soft_remove%]
    ])?;
[%/@:models%]    Ok(())
}
[%direct_routes%][%#?:has_models%][%registration%]
[%mounted_routes%][%/?:has_models%][%#?:has_models%]/// Authorized import preview only: existence, never a private row projection.
pub async fn import_exists(module: &str, db: &mut stackpress_framework_sql::DataSession<'_>, value: serde_json::Value) -> stackpress_framework_sql::DataResult<bool> {
    match module {
[%#@:models model%]        [%model.module_literal%] => {
            let key = serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)?;
            Ok(crate::generated::stores::[%model.native_module%]::[%model.store_type%]::get_maintenance(db, &key, stackpress_framework_sql::stores::Visibility::All).await?.is_some())
        },
[%/@:models%]        _ => Err(stackpress_framework_sql::DataError::new("admin import unavailable")),
    }
}
[%|%]/// No import model is selected.
pub async fn import_exists(_: &str, _: &mut stackpress_framework_sql::DataSession<'_>, _: serde_json::Value) -> stackpress_framework_sql::DataResult<bool> {
    Err(stackpress_framework_sql::DataError::new("admin import unavailable"))
}
[%/?:has_models%]
