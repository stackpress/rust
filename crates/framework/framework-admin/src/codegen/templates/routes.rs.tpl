[%#?:private%]/// Mount the selected mappings during the existing Route phase.
fn mount_routes([%|%]/// Register fixed routes beneath a previously validated application mount.
pub fn routes([%/?:private%][%#?:has_models%]builder[%|%]_builder[%/?:has_models%]: &mut [%#?:private%]StartupContext[%|%]stackpress_server::ServerBuilder[%/?:private%], [%#?:has_models%]mount[%|%]_mount[%/?:has_models%]: &str) -> stackpress_lib::Result<()> {
[%#@:models model%][%#@:model.page_actions action%]    builder.get(&format!("{mount}[%model.module%]/[%action%]"), crate::host::page)?;
[%/@:model.page_actions%]    builder.get(&format!("{mount}[%model.module%]/export"), crate::host::transfer::export)?;
[%#@:model.import_actions action%]    builder.post(&format!("{mount}api/[%model.module%]/[%action%]"), crate::host::transfer::import)?;
[%/@:model.import_actions%][%#@:model.api_actions action%]    builder.post(&format!("{mount}api/[%model.module%]/[%action%]"), crate::host::api)?;
[%/@:model.api_actions%][%/@:models%]    Ok(())
}
