//! Explicit native compiled admin registration.
use stackpress_framework_sql::operations::{Operation, Registration, Result, StartupContext};
/// Select the actual generated operation handlers; no handwritten operation substitute.
pub fn operations(start: &mut StartupContext, marker: &mut Registration) -> Result<()> {
    crate::generated::stores::operations::note::register(
        start,
        marker,
        "note",
        &[
            Operation::Get,
            Operation::Detail,
            Operation::Search,
            Operation::Create,
            Operation::Update,
            Operation::Batch,
        ],
    )?;
    Ok(())
}
/// Register fixed routes beneath a previously validated application mount.
pub fn routes(
    builder: &mut stackpress_server::ServerBuilder,
    mount: &str,
) -> stackpress_lib::Result<()> {
    builder.get(&format!("{mount}note/search"), crate::host::page)?;
    builder.get(&format!("{mount}note/detail"), crate::host::page)?;
    builder.get(&format!("{mount}note/create"), crate::host::page)?;
    builder.get(&format!("{mount}note/update"), crate::host::page)?;
    builder.get(&format!("{mount}note/copy"), crate::host::page)?;
    builder.get(&format!("{mount}note/import"), crate::host::page)?;
    builder.get(
        &format!("{mount}note/export"),
        crate::host::transfer::export,
    )?;
    builder.post(
        &format!("{mount}api/note/preview"),
        crate::host::transfer::import,
    )?;
    builder.post(
        &format!("{mount}api/note/confirm"),
        crate::host::transfer::import,
    )?;
    builder.post(&format!("{mount}api/note/search"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/get"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/detail"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/create"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/update"), crate::host::api)?;
    Ok(())
}

/// Validated application-owned mount for this explicit nonempty generated page selection.
pub struct Mount(String);

impl Mount {
    /// Validate the same mount supplied to the application's page/asset host configuration.
    ///
    /// # Errors
    /// Preserves invalid mount errors; no route or service is registered on failure.
    pub fn new(value: &str) -> Result<Self> {
        stackpress_framework_admin::pages::mount(value)
            .map(Self)
            .map_err(|cause| {
                stackpress_lib::Error::invalid("invalid admin mount").with_cause(cause)
            })
    }

    /// Borrow the normalized mount; the application uses this same value for its host config.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Config-only claim preventing repeated selection under different plugin identities.
struct Service;

/// Register this generated host after explicit native SQL integration.
///
/// Supply Mount and application handler configuration in Config. SQL's plugin must precede
/// this registrar and verify its app-owned Engine and Policy. This singleton contributes the
/// selected generated operations, then mounts only their existing page/API mappings in Route.
/// Applications combining operation families supply their own SQL Contributions and call the
/// direct operations/routes helpers instead. No database setup or listener runs at startup.
/// Developers own route inventories, priorities and cooperative abort behavior; each protected
/// handler enforces its own policy. Shared admin and Leptos imports register nothing.
///
/// # Errors
/// Build fails on missing/wrong-order SQL integration or Mount, duplicate singleton/services,
/// conflicting operation selections, or invalid routes. No partial Server is published.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    use stackpress_lib::{Flow, queue::task};
    use std::sync::Arc;

    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            // Underlying Engine/Policy alone cannot substitute for explicit SQL integration.
            start.service::<stackpress_framework_sql::plugin::Service>()?;
            start.service::<Mount>()?;
            start.register(Arc::new(Service))?;
            start.register(Arc::new(stackpress_framework_sql::plugin::Contributions {
                operations: &[operations],
            }))?;
            Ok(Flow::Continue)
        })
    }))?;

    app.on_route(task(|start: &mut StartupContext| {
        Box::pin(async move {
            // Release the immutable service borrow before contributing the fixed mappings.
            let mount = start.service::<Mount>()?.as_str().to_owned();
            mount_routes(start, &mount)?;
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}

/// Mount the selected mappings during the existing Route phase.
fn mount_routes(builder: &mut StartupContext, mount: &str) -> stackpress_lib::Result<()> {
    builder.get(&format!("{mount}note/search"), crate::host::page)?;
    builder.get(&format!("{mount}note/detail"), crate::host::page)?;
    builder.get(&format!("{mount}note/create"), crate::host::page)?;
    builder.get(&format!("{mount}note/update"), crate::host::page)?;
    builder.get(&format!("{mount}note/copy"), crate::host::page)?;
    builder.get(&format!("{mount}note/import"), crate::host::page)?;
    builder.get(
        &format!("{mount}note/export"),
        crate::host::transfer::export,
    )?;
    builder.post(
        &format!("{mount}api/note/preview"),
        crate::host::transfer::import,
    )?;
    builder.post(
        &format!("{mount}api/note/confirm"),
        crate::host::transfer::import,
    )?;
    builder.post(&format!("{mount}api/note/search"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/get"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/detail"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/create"), crate::host::api)?;
    builder.post(&format!("{mount}api/note/update"), crate::host::api)?;
    Ok(())
}
/// Authorized import preview only: existence, never a private row projection.
pub async fn import_exists(
    module: &str,
    db: &mut stackpress_framework_sql::DataSession<'_>,
    value: serde_json::Value,
) -> stackpress_framework_sql::DataResult<bool> {
    match module {
        "note" => {
            let key =
                serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)?;
            Ok(crate::generated::stores::note::Notes::get_maintenance(
                db,
                &key,
                stackpress_framework_sql::stores::Visibility::All,
            )
            .await?
            .is_some())
        }
        _ => Err(stackpress_framework_sql::DataError::new(
            "admin import unavailable",
        )),
    }
}
