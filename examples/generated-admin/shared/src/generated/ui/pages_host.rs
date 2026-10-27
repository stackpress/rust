//! Explicit native compiled admin registration.
use stackpress_framework_sql::operations::{Operation, Registration, Result, StartupContext};
/// Select the actual generated operation handlers; no handwritten operation substitute.
pub fn operations(start: &mut StartupContext, marker: &mut Registration) -> Result<()> {
    crate::generated::stores::operations::profile::register(
        start,
        marker,
        "profile",
        &[
            Operation::Get,
            Operation::Detail,
            Operation::Search,
            Operation::Create,
            Operation::Update,
            Operation::Remove,
            Operation::Restore,
        ],
    )?;
    crate::generated::stores::operations::project::register(
        start,
        marker,
        "project",
        &[
            Operation::Get,
            Operation::Detail,
            Operation::Search,
            Operation::Create,
            Operation::Update,
        ],
    )?;
    crate::generated::stores::operations::task::register(
        start,
        marker,
        "task",
        &[
            Operation::Get,
            Operation::Detail,
            Operation::Search,
            Operation::Create,
            Operation::Update,
        ],
    )?;
    Ok(())
}
/// Register fixed routes beneath a previously validated application mount.
pub fn routes(
    builder: &mut stackpress_server::ServerBuilder,
    mount: &str,
) -> stackpress_lib::Result<()> {
    builder.get(&format!("{mount}profile/search"), crate::host::page)?;
    builder.get(&format!("{mount}profile/detail"), crate::host::page)?;
    builder.get(&format!("{mount}profile/create"), crate::host::page)?;
    builder.get(&format!("{mount}profile/update"), crate::host::page)?;
    builder.get(&format!("{mount}profile/copy"), crate::host::page)?;
    builder.get(&format!("{mount}profile/remove"), crate::host::page)?;
    builder.get(&format!("{mount}profile/restore"), crate::host::page)?;
    builder.post(&format!("{mount}api/profile/search"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/get"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/detail"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/create"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/update"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/remove"), crate::host::api)?;
    builder.post(&format!("{mount}api/profile/restore"), crate::host::api)?;
    builder.get(&format!("{mount}project/search"), crate::host::page)?;
    builder.get(&format!("{mount}project/detail"), crate::host::page)?;
    builder.get(&format!("{mount}project/create"), crate::host::page)?;
    builder.get(&format!("{mount}project/update"), crate::host::page)?;
    builder.get(&format!("{mount}project/copy"), crate::host::page)?;
    builder.post(&format!("{mount}api/project/search"), crate::host::api)?;
    builder.post(&format!("{mount}api/project/get"), crate::host::api)?;
    builder.post(&format!("{mount}api/project/detail"), crate::host::api)?;
    builder.post(&format!("{mount}api/project/create"), crate::host::api)?;
    builder.post(&format!("{mount}api/project/update"), crate::host::api)?;
    builder.get(&format!("{mount}task/search"), crate::host::page)?;
    builder.get(&format!("{mount}task/detail"), crate::host::page)?;
    builder.get(&format!("{mount}task/create"), crate::host::page)?;
    builder.get(&format!("{mount}task/update"), crate::host::page)?;
    builder.get(&format!("{mount}task/copy"), crate::host::page)?;
    builder.post(&format!("{mount}api/task/search"), crate::host::api)?;
    builder.post(&format!("{mount}api/task/get"), crate::host::api)?;
    builder.post(&format!("{mount}api/task/detail"), crate::host::api)?;
    builder.post(&format!("{mount}api/task/create"), crate::host::api)?;
    builder.post(&format!("{mount}api/task/update"), crate::host::api)?;
    Ok(())
}
